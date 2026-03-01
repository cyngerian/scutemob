//! Activated and triggered ability handling (CR 602-603).
//!
//! ## Activated abilities (CR 602)
//!
//! Activated abilities are written as "Cost: Effect." They are NOT mana abilities
//! (those are handled in `rules/mana.rs`). Activating puts a `StackObject` on
//! the stack. The active player receives priority afterward.
//!
//! ## Triggered abilities (CR 603)
//!
//! Triggered abilities begin with "when", "whenever", or "at". When a trigger
//! condition is met:
//! 1. The ability goes into `GameState::pending_triggers`.
//! 2. The next time a player would receive priority, pending triggers are flushed
//!    to the stack in APNAP order (CR 603.3).
//!
//! **Intervening-if (CR 603.4)**: If the ability reads "... if [condition] ...",
//! the condition is checked at trigger time (ability only queues if true) AND at
//! resolution time (ability has no effect if condition became false).

use im::OrdSet;

use crate::cards::card_definition::AbilityDefinition;
use crate::state::error::GameStateError;
use crate::state::game_object::{InterveningIf, ManaCost, ObjectId, TriggerEvent};
use crate::state::player::{CardId, PlayerId};
use crate::state::stack::{StackObject, StackObjectKind};
use crate::state::stubs::{PendingTrigger, TriggerDoubler, TriggerDoublerFilter};
use crate::state::targeting::{SpellTarget, Target};
use crate::state::types::{CardType, CounterType, KeywordAbility};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::casting;
use super::events::{CombatDamageTarget, GameEvent};

// ---------------------------------------------------------------------------
// Activated ability handler
// ---------------------------------------------------------------------------

/// Handle an ActivateAbility command: validate, pay cost, push onto the stack.
///
/// CR 602.2: To activate an ability, the controller announces it, pays all costs
/// in full, and the ability is placed on the stack. Unlike mana abilities, activated
/// abilities DO use the stack and must be responded to before resolving.
///
/// After activation, the active player receives priority (CR 116.3b).
pub fn handle_activate_ability(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    targets: Vec<Target>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // CR 602.2: Activating requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // CR 702.61a: If a spell with split second is on the stack, no non-mana
    // abilities can be activated. (Mana abilities are handled in mana.rs and
    // are exempt from this check per CR 702.61b.)
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; non-mana abilities cannot be activated (CR 702.61a)".into(),
        ));
    }

    // Source must be on the battlefield.
    {
        let obj = state.object(source)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(source));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: source,
            });
        }
        // Validate the ability index exists.
        if obj
            .characteristics
            .activated_abilities
            .get(ability_index)
            .is_none()
        {
            return Err(GameStateError::InvalidAbilityIndex {
                object_id: source,
                index: ability_index,
            });
        }
    }

    // CR 602.5d: Check sorcery-speed restriction before paying any costs.
    {
        let obj = state.object(source)?;
        let ab = &obj.characteristics.activated_abilities[ability_index];
        if ab.sorcery_speed {
            // Must be active player's main phase with empty stack.
            if state.turn.active_player != player {
                return Err(GameStateError::InvalidCommand(
                    "sorcery-speed ability can only be activated during your own turn".into(),
                ));
            }
            if !matches!(
                state.turn.step,
                crate::state::turn::Step::PreCombatMain | crate::state::turn::Step::PostCombatMain
            ) {
                return Err(GameStateError::NotMainPhase);
            }
            if !state.stack_objects.is_empty() {
                return Err(GameStateError::StackNotEmpty);
            }
        }
    }

    // Clone the cost and capture effect before mutating state.
    // Effect must be captured now in case sacrifice-as-cost removes the source object.
    let (ability_cost, embedded_effect) = {
        let obj = state.object(source)?;
        let ab = &obj.characteristics.activated_abilities[ability_index];
        (ab.cost.clone(), ab.effect.clone())
    };

    // CR 702.6a / CR 601.2c: Equip abilities can only target "a creature you control."
    // Validate target type and controller BEFORE spending any costs, so that mana is
    // not wasted when the activation is illegal.
    //
    // This is a special-case check for AttachEquipment effects. The general activated-
    // ability framework does not (yet) have a TargetRequirement field; this check
    // bridges that gap for Equip specifically.
    if matches!(
        &embedded_effect,
        Some(crate::cards::card_definition::Effect::AttachEquipment { .. })
    ) {
        if let Some(Target::Object(target_id)) = targets.first() {
            let target_id = *target_id;
            // Check: target must be a creature on the battlefield controlled by the
            // activating player. Use layer-computed characteristics for correctness
            // under continuous effects (e.g. animated artifacts).
            let on_battlefield_and_controlled = state
                .objects
                .get(&target_id)
                .map(|obj| {
                    obj.zone == crate::state::zone::ZoneId::Battlefield && obj.controller == player
                })
                .unwrap_or(false);
            let is_creature = {
                let layer_chars = crate::rules::layers::calculate_characteristics(state, target_id)
                    .or_else(|| {
                        state
                            .objects
                            .get(&target_id)
                            .map(|o| o.characteristics.clone())
                    });
                layer_chars
                    .map(|chars| {
                        chars
                            .card_types
                            .contains(&crate::state::types::CardType::Creature)
                    })
                    .unwrap_or(false)
            };
            if !on_battlefield_and_controlled {
                return Err(GameStateError::InvalidTarget(
                    "equip target must be a creature you control on the battlefield".into(),
                ));
            }
            if !is_creature {
                return Err(GameStateError::InvalidTarget(
                    "equip target must be a creature".into(),
                ));
            }
        }
    }

    let mut events = Vec::new();

    // Pay tap cost if required (CR 602.2b).
    if ability_cost.requires_tap {
        let obj = state.object(source)?;
        if obj.status.tapped {
            return Err(GameStateError::PermanentAlreadyTapped(source));
        }
        // CR 302.6 / CR 702.10: Summoning sickness prevents using {T} abilities
        // on creatures unless they have haste.
        let is_creature = obj
            .characteristics
            .card_types
            .contains(&crate::state::types::CardType::Creature);
        if is_creature && obj.has_summoning_sickness {
            let has_haste = obj
                .characteristics
                .keywords
                .contains(&crate::state::types::KeywordAbility::Haste);
            if !has_haste {
                return Err(GameStateError::InvalidCommand(format!(
                    "object {:?} has summoning sickness and cannot use abilities with {{T}}",
                    source
                )));
            }
        }
        if let Some(obj) = state.objects.get_mut(&source) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: source,
        });
    }

    // Pay mana cost if required (CR 602.2a).
    if let Some(ref mana_cost) = ability_cost.mana_cost {
        if mana_cost.mana_value() > 0 {
            let player_state = state.player_mut(player)?;
            if !casting::can_pay_cost(&player_state.mana_pool, mana_cost) {
                return Err(GameStateError::InsufficientMana);
            }
            casting::pay_cost(&mut player_state.mana_pool, mana_cost);
            events.push(GameEvent::ManaCostPaid {
                player,
                cost: mana_cost.clone(),
            });
        }
    }

    // Pay sacrifice cost (CR 602.2c). Move source to graveyard before pushing to stack.
    if ability_cost.sacrifice_self {
        let (is_creature, owner, pre_death_controller, pre_death_counters) = {
            let obj = state.object(source)?;
            (
                obj.characteristics
                    .card_types
                    .contains(&crate::state::types::CardType::Creature),
                obj.owner,
                // CR 603.3a: capture controller before move_object_to_zone resets it to owner.
                obj.controller,
                // CR 702.79a: capture counters before move_object_to_zone resets them.
                obj.counters.clone(),
            )
        };
        let (new_id, _) = state.move_object_to_zone(source, ZoneId::Graveyard(owner))?;
        if is_creature {
            events.push(GameEvent::CreatureDied {
                object_id: source,
                new_grave_id: new_id,
                controller: pre_death_controller,
                pre_death_counters,
            });
        } else {
            events.push(GameEvent::PermanentDestroyed {
                object_id: source,
                new_grave_id: new_id,
            });
        }
    }

    // CR 602.2c: Validate targets for existence, hexproof, shroud, and protection.
    // Fetch source characteristics once for protection-from checks (CR 702.16b).
    let source_chars =
        crate::rules::layers::calculate_characteristics(state, source).or_else(|| {
            state
                .objects
                .get(&source)
                .map(|o| o.characteristics.clone())
        });
    for t in &targets {
        if let Target::Object(id) = t {
            // MR-M3-04: Non-existent object must be rejected, not silently skipped.
            let obj = state
                .objects
                .get(id)
                .ok_or(GameStateError::ObjectNotFound(*id))?;
            // CR 702.11a / CR 702.18a / CR 702.16b: Hexproof, shroud, and protection.
            super::validate_target_protection(
                &obj.characteristics.keywords,
                obj.controller,
                player,
                source_chars.as_ref(),
            )?;
        }
    }

    // Snapshot targets (zone recorded at activation time for fizzle check at resolution).
    let spell_targets: Vec<SpellTarget> = targets
        .iter()
        .map(|t| match t {
            Target::Player(id) => SpellTarget {
                target: Target::Player(*id),
                zone_at_cast: None,
            },
            Target::Object(id) => {
                let zone = state.objects.get(id).map(|o| o.zone);
                SpellTarget {
                    target: Target::Object(*id),
                    zone_at_cast: zone,
                }
            }
        })
        .collect();

    // Push the activated ability onto the stack.
    let stack_id = state.next_object_id();

    // CR 702.21a: Collect battlefield object targets before moving spell_targets into
    // the stack object. These are used to emit PermanentTargeted events for Ward.
    let battlefield_targets: Vec<ObjectId> = spell_targets
        .iter()
        .filter_map(|st| {
            if let Target::Object(id) = st.target {
                if matches!(st.zone_at_cast, Some(ZoneId::Battlefield)) {
                    return Some(id);
                }
            }
            None
        })
        .collect();

    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::ActivatedAbility {
            source_object: source,
            ability_index,
            embedded_effect: embedded_effect.map(Box::new),
        },
        targets: spell_targets,
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
    };
    state.stack_objects.push_back(stack_obj);

    // CR 602.2e: After activating, the active player receives priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: source,
        stack_object_id: stack_id,
    });

    // CR 702.21a: Emit PermanentTargeted for each battlefield permanent that this
    // activated ability targets. These events drive Ward trigger checks in check_triggers.
    // `targeting_stack_id` is the stack entry's own ObjectId so the ward CounterSpell
    // effect can locate it via direct stack ID match (so.id == id).
    for target_id in battlefield_targets {
        events.push(GameEvent::PermanentTargeted {
            target_id,
            targeting_stack_id: stack_id,
            targeting_controller: player,
        });
    }

    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}

// ---------------------------------------------------------------------------
// Cycling handler
// ---------------------------------------------------------------------------

/// Handle a CycleCard command: validate, pay mana cost, discard self, push draw onto stack.
///
/// CR 702.29a: Cycling is an activated ability from hand. "[Cost], Discard this card: Draw a card."
/// The discard is part of the cost (happens immediately before ability goes on stack).
/// The draw uses the stack and can be responded to (e.g., Stifle).
///
/// CR 702.29b: The keyword exists in all zones, but activation is only legal from hand.
pub fn handle_cycle_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // CR 702.61a: Cycling is an activated ability, not a mana ability.
    // It cannot be activated while a spell with split second is on the stack.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; cycling cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 2. Zone check (CR 702.29a): card must be in Hand(player).
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "CycleCard: card {:?} is not in Hand({:?}); cycling can only be activated from hand (CR 702.29a)",
                card, player
            )));
        }
    }

    // 3. Keyword check (CR 702.29a): card must have KeywordAbility::Cycling.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Cycling)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "CycleCard: card {:?} does not have the Cycling keyword (CR 702.29a)",
                card
            )));
        }
    }

    // 4. Look up cycling cost from CardRegistry (CR 702.29a).
    let card_id_opt = state.object(card)?.card_id.clone();
    let cycling_cost = get_cycling_cost(&card_id_opt, &state.card_registry.clone());

    // 5. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if let Some(ref cost) = cycling_cost {
        if cost.mana_value() > 0 {
            let player_state = state.player_mut(player)?;
            if !casting::can_pay_cost(&player_state.mana_pool, cost) {
                return Err(GameStateError::InsufficientMana);
            }
            casting::pay_cost(&mut player_state.mana_pool, cost);
            events.push(GameEvent::ManaCostPaid {
                player,
                cost: cost.clone(),
            });
        }
    }

    // 6. Discard self as cost (CR 702.29a): move card from hand to graveyard (or exile if madness).
    // This happens BEFORE the ability goes on the stack.
    // Capture owner before zone move (move_object_to_zone resets controller to owner).
    let owner = state.object(card)?.owner;

    // CR 702.35a: If the card has madness, exile instead of graveyard.
    let cycle_card_id_opt = state.object(card)?.card_id.clone();
    let has_madness = state
        .object(card)?
        .characteristics
        .keywords
        .contains(&KeywordAbility::Madness);

    let discard_destination = if has_madness {
        ZoneId::Exile
    } else {
        ZoneId::Graveyard(owner)
    };
    let (new_grave_id, _) = state.move_object_to_zone(card, discard_destination)?;

    // Emit CardDiscarded (CR 701.8 — discard is always announced, even when going to exile).
    events.push(GameEvent::CardDiscarded {
        player,
        object_id: card,
        new_id: new_grave_id,
    });
    // Emit CardCycled (CR 702.29a — distinct event for "when you cycle" trigger matching).
    events.push(GameEvent::CardCycled {
        player,
        object_id: card,
        new_id: new_grave_id,
    });

    // CR 702.35a: If madness applied, queue the madness trigger via pending_triggers
    // so it goes through flush_pending_triggers and properly signals priority granting.
    if has_madness {
        let madness_cost = cycle_card_id_opt.as_ref().and_then(|cid| {
            state.card_registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Madness { cost } = a {
                        Some(cost.clone())
                    } else {
                        None
                    }
                })
            })
        });
        state.pending_triggers.push_back(PendingTrigger {
            source: new_grave_id,
            ability_index: 0,
            controller: player,
            triggering_event: None,
            entering_object_id: None,
            targeting_stack_id: None,
            triggering_player: None,
            exalted_attacker_id: None,
            defending_player_id: None,
            is_evoke_sacrifice: false,
            is_madness_trigger: true,
            madness_exiled_card: Some(new_grave_id),
            madness_cost,
            is_miracle_trigger: false,
            miracle_revealed_card: None,
            miracle_cost: None,
            is_unearth_trigger: false,
            is_exploit_trigger: false,
            is_modular_trigger: false,
            modular_counter_count: None,
            is_evolve_trigger: false,
            evolve_entering_creature: None,
            is_myriad_trigger: false,
            is_suspend_counter_trigger: false,
            is_suspend_cast_trigger: false,
            suspend_card_id: None,
            is_hideaway_trigger: false,
            hideaway_count: None,
            is_partner_with_trigger: false,
            partner_with_name: None,
        });
    }

    // 7. Push cycling ability onto stack as ActivatedAbility with embedded DrawCards effect.
    // CR 602.2c: The source object (card) is now in the graveyard; source_object records
    // the retired ObjectId for reference. ability_index 0 is a placeholder.
    let stack_id = state.next_object_id();
    let draw_effect = crate::cards::card_definition::Effect::DrawCards {
        player: crate::cards::card_definition::PlayerTarget::Controller,
        count: crate::cards::card_definition::EffectAmount::Fixed(1),
    };
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::ActivatedAbility {
            source_object: card,
            ability_index: 0,
            embedded_effect: Some(Box::new(draw_effect)),
        },
        targets: Vec::new(),
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
    };
    state.stack_objects.push_back(stack_obj);

    // 8. Reset priority (CR 602.2e): active player gets priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}

/// CR 702.29a: Look up the cycling cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Cycling { cost }`, or `None`
/// if the card has no definition or no cycling ability defined. When `None` is returned,
/// no mana payment is required (free cycling, e.g., Street Wraith).
fn get_cycling_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Cycling { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Unearth (CR 702.84)
// ---------------------------------------------------------------------------

/// Handle an UnearthCard command: validate, pay cost, push unearth ability onto stack.
///
/// CR 702.84a: Unearth is an activated ability from the graveyard.
/// "[Cost]: Return this card from your graveyard to the battlefield. It gains haste.
/// Exile it at the beginning of the next end step. If it would leave the battlefield,
/// exile it instead. Activate only as a sorcery."
pub fn handle_unearth_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // 2. Split second check (CR 702.61a): activated abilities cannot be used when
    //    a spell with split second is on the stack.
    if casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; unearth cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 3. Zone check (CR 702.84a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "UnearthCard: card {:?} is not in Graveyard({:?}); unearth can only be activated from your graveyard (CR 702.84a)",
                card, player
            )));
        }
    }

    // 4. Keyword check (CR 702.84a): card must have KeywordAbility::Unearth.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Unearth)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "UnearthCard: card {:?} does not have the Unearth keyword (CR 702.84a)",
                card
            )));
        }
    }

    // 5. Sorcery speed check (CR 702.84a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "UnearthCard: unearth can only be activated during your own turn (CR 702.84a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "UnearthCard: unearth can only be activated during a main phase (CR 702.84a)"
                    .into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "UnearthCard: unearth can only be activated with an empty stack (CR 702.84a)"
                    .into(),
            ));
        }
    }

    // 6. Look up unearth cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let unearth_cost_opt = get_unearth_cost(&card_id_opt, &state.card_registry.clone());
    let unearth_cost = match unearth_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "UnearthCard: no unearth cost found in card definition (CR 702.84a)".into(),
            ));
        }
    };

    // 7. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if unearth_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &unearth_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &unearth_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: unearth_cost.clone(),
        });
    }

    // 8. Push the unearth ability onto the stack as UnearthAbility.
    //    The card stays in the graveyard until the ability resolves (unlike cycling
    //    where the card is discarded as a cost).
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::UnearthAbility {
            source_object: card,
        },
        targets: Vec::new(),
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
    };
    state.stack_objects.push_back(stack_obj);

    // 9. Reset priority (CR 602.2e): active player gets priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}

/// CR 702.84a: Look up the unearth cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Unearth { cost }`, or `None`
/// if the card has no definition or no unearth ability defined.
fn get_unearth_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Unearth { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Trigger checking
// ---------------------------------------------------------------------------

/// Scan all permanents for triggered abilities that fire in response to `events`.
///
/// Called after any batch of events. Returns `PendingTrigger` entries for each
/// ability that triggered. Does NOT modify state — caller pushes results into
/// `state.pending_triggers`.
///
/// CR 603.2: A triggered ability triggers whenever the trigger event occurs
/// and the trigger condition is met.
/// CR 603.4: If an intervening-if clause is present, the condition is checked
/// at trigger time; the ability only queues if the condition is true.
pub fn check_triggers(state: &GameState, events: &[GameEvent]) -> Vec<PendingTrigger> {
    let mut triggers = Vec::new();

    for event in events {
        match event {
            GameEvent::PermanentEnteredBattlefield { object_id, .. } => {
                // SelfEntersBattlefield: fires on the entering permanent itself.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::SelfEntersBattlefield,
                    Some(*object_id), // Only check this specific object
                    Some(*object_id), // entering_object_id: the permanent itself
                );

                // AnyPermanentEntersBattlefield: fires on ALL permanents (including the entering one).
                // Pass the entering object so TriggerDoublerFilter::ArtifactOrCreatureETB can
                // verify the entering object's card types (CR 603.2d).
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::AnyPermanentEntersBattlefield,
                    None,             // Check all battlefield permanents
                    Some(*object_id), // entering_object_id: the permanent that entered
                );

                // CR 702.74a: If the permanent was evoked, generate the evoke sacrifice trigger.
                // "When this permanent enters, if its evoke cost was paid, its controller
                // sacrifices it." This goes on the stack as a separate triggered ability,
                // allowing the controller to order it relative to other ETB triggers
                // (e.g., Mulldrifter can resolve draw before sacrifice).
                if let Some(obj) = state.objects.get(object_id) {
                    if obj.was_evoked {
                        let evoke_trigger = PendingTrigger {
                            source: *object_id,
                            ability_index: 0, // unused for evoke sacrifice
                            controller: obj.controller,
                            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                            entering_object_id: Some(*object_id),
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            is_evoke_sacrifice: true,
                            is_madness_trigger: false,
                            madness_exiled_card: None,
                            madness_cost: None,
                            is_miracle_trigger: false,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            is_unearth_trigger: false,
                            is_exploit_trigger: false,
                            is_modular_trigger: false,
                            modular_counter_count: None,
                            is_evolve_trigger: false,
                            evolve_entering_creature: None,
                            is_myriad_trigger: false,
                            is_suspend_counter_trigger: false,
                            is_suspend_cast_trigger: false,
                            suspend_card_id: None,
                            is_hideaway_trigger: false,
                            hideaway_count: None,
                            is_partner_with_trigger: false,
                            partner_with_name: None,
                        };
                        triggers.push(evoke_trigger);
                    }
                }

                // CR 702.110a: If the permanent has Exploit, generate the exploit trigger.
                // "When this creature enters, you may sacrifice a creature."
                // Each instance of Exploit in the card definition triggers separately.
                if let Some(obj) = state.objects.get(object_id) {
                    if obj
                        .characteristics
                        .keywords
                        .contains(&KeywordAbility::Exploit)
                    {
                        // Count exploit instances from card definition for multiple instances.
                        // OrdSet deduplicates, so check the card definition for exact count.
                        let exploit_count = obj
                            .card_id
                            .as_ref()
                            .and_then(|cid| state.card_registry.get(cid.clone()))
                            .map(|def| {
                                def.abilities
                                    .iter()
                                    .filter(|a| {
                                        matches!(
                                            a,
                                            crate::cards::card_definition::AbilityDefinition::Keyword(
                                                KeywordAbility::Exploit
                                            )
                                        )
                                    })
                                    .count()
                            })
                            .unwrap_or(1)
                            .max(1);

                        let controller = obj.controller;
                        for _ in 0..exploit_count {
                            triggers.push(PendingTrigger {
                                source: *object_id,
                                ability_index: 0, // unused for exploit triggers
                                controller,
                                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                                entering_object_id: Some(*object_id),
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                is_evoke_sacrifice: false,
                                is_madness_trigger: false,
                                madness_exiled_card: None,
                                madness_cost: None,
                                is_miracle_trigger: false,
                                miracle_revealed_card: None,
                                miracle_cost: None,
                                is_unearth_trigger: false,
                                is_exploit_trigger: true,
                                is_modular_trigger: false,
                                modular_counter_count: None,
                                is_evolve_trigger: false,
                                evolve_entering_creature: None,
                                is_myriad_trigger: false,
                                is_suspend_counter_trigger: false,
                                is_suspend_cast_trigger: false,
                                suspend_card_id: None,
                                is_hideaway_trigger: false,
                                hideaway_count: None,
                                is_partner_with_trigger: false,
                                partner_with_name: None,
                            });
                        }
                    }
                }

                // CR 702.75a: Hideaway(N) — "When this permanent enters, look at
                // the top N cards of your library. Exile one of them face down
                // and put the rest on the bottom of your library in a random order."
                //
                // Each Hideaway(N) keyword on the permanent generates one trigger.
                // Multiple instances trigger separately (CR 603.2: each keyword instance
                // is a separate triggered ability).
                if let Some(obj) = state.objects.get(object_id) {
                    let controller = obj.controller;
                    let hideaway_keywords: Vec<u32> = obj
                        .characteristics
                        .keywords
                        .iter()
                        .filter_map(|kw| {
                            if let KeywordAbility::Hideaway(n) = kw {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .collect();
                    for n in hideaway_keywords {
                        triggers.push(PendingTrigger {
                            source: *object_id,
                            ability_index: 0, // unused for hideaway triggers
                            controller,
                            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                            entering_object_id: Some(*object_id),
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            is_evoke_sacrifice: false,
                            is_madness_trigger: false,
                            madness_exiled_card: None,
                            madness_cost: None,
                            is_miracle_trigger: false,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            is_unearth_trigger: false,
                            is_exploit_trigger: false,
                            is_modular_trigger: false,
                            modular_counter_count: None,
                            is_evolve_trigger: false,
                            evolve_entering_creature: None,
                            is_myriad_trigger: false,
                            is_suspend_counter_trigger: false,
                            is_suspend_cast_trigger: false,
                            suspend_card_id: None,
                            is_hideaway_trigger: true,
                            hideaway_count: Some(n),
                            is_partner_with_trigger: false,
                            partner_with_name: None,
                        });
                    }
                }

                // CR 702.124j: Partner With ETB trigger —
                // "When this permanent enters, target player may search their
                // library for a card named [name], reveal it, put it into their
                // hand, then shuffle."
                //
                // CR 603.3: The trigger goes on the stack (can be countered).
                // Target player: deterministic fallback = the entering permanent's
                // controller (the player most likely to have the partner in their
                // library in a Commander game).
                {
                    if let Some(obj) = state.objects.get(object_id) {
                        let controller = obj.controller;
                        let partner_with_names: Vec<String> = obj
                            .characteristics
                            .keywords
                            .iter()
                            .filter_map(|kw| {
                                if let KeywordAbility::PartnerWith(name) = kw {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        for name in partner_with_names {
                            triggers.push(PendingTrigger {
                                source: *object_id,
                                ability_index: 0, // unused for partner-with triggers
                                controller,
                                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                                entering_object_id: Some(*object_id),
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                is_evoke_sacrifice: false,
                                is_madness_trigger: false,
                                madness_exiled_card: None,
                                madness_cost: None,
                                is_miracle_trigger: false,
                                miracle_revealed_card: None,
                                miracle_cost: None,
                                is_unearth_trigger: false,
                                is_exploit_trigger: false,
                                is_modular_trigger: false,
                                modular_counter_count: None,
                                is_evolve_trigger: false,
                                evolve_entering_creature: None,
                                is_myriad_trigger: false,
                                is_suspend_counter_trigger: false,
                                is_suspend_cast_trigger: false,
                                suspend_card_id: None,
                                is_hideaway_trigger: false,
                                hideaway_count: None,
                                is_partner_with_trigger: true,
                                partner_with_name: Some(name),
                            });
                        }
                    }
                }

                // CR 702.100a: Evolve — "Whenever a creature you control enters,
                // if that creature's power is greater than this creature's power
                // and/or that creature's toughness is greater than this creature's
                // toughness, put a +1/+1 counter on this creature."
                //
                // CR 702.100c: Noncreature permanents cannot trigger evolve.
                // CR 702.100d: Multiple instances of evolve each trigger separately.
                // CR 603.4: Intervening-if — P/T comparison is checked at trigger time.
                {
                    // First verify the entering permanent is a creature (CR 702.100c).
                    let entering_is_creature =
                        crate::rules::layers::calculate_characteristics(state, *object_id)
                            .or_else(|| {
                                state
                                    .objects
                                    .get(object_id)
                                    .map(|o| o.characteristics.clone())
                            })
                            .map(|chars| chars.card_types.contains(&CardType::Creature))
                            .unwrap_or(false);

                    if entering_is_creature {
                        let entering_controller =
                            state.objects.get(object_id).map(|o| o.controller);

                        if let Some(controller) = entering_controller {
                            // Get the entering creature's P/T (layer-aware).
                            let entering_chars =
                                crate::rules::layers::calculate_characteristics(state, *object_id)
                                    .or_else(|| {
                                        state
                                            .objects
                                            .get(object_id)
                                            .map(|o| o.characteristics.clone())
                                    });

                            let (entering_power, entering_toughness) = entering_chars
                                .as_ref()
                                .map(|c| (c.power.unwrap_or(0), c.toughness.unwrap_or(0)))
                                .unwrap_or((0, 0));

                            // Collect all creatures with evolve controlled by the same player.
                            // Exclude the entering creature itself (cannot evolve from itself).
                            let evolve_sources: Vec<ObjectId> = state
                                .objects
                                .values()
                                .filter(|obj| {
                                    obj.zone == ZoneId::Battlefield
                                        && obj.controller == controller
                                        && obj.id != *object_id
                                        && obj
                                            .characteristics
                                            .keywords
                                            .contains(&KeywordAbility::Evolve)
                                })
                                .map(|obj| obj.id)
                                .collect();

                            for evolve_id in evolve_sources {
                                // CR 603.4: Intervening-if check at trigger time.
                                // Get the evolve creature's current P/T (layer-aware).
                                let evolve_chars = crate::rules::layers::calculate_characteristics(
                                    state, evolve_id,
                                )
                                .or_else(|| {
                                    state
                                        .objects
                                        .get(&evolve_id)
                                        .map(|o| o.characteristics.clone())
                                });

                                let (evolve_power, evolve_toughness) = evolve_chars
                                    .as_ref()
                                    .map(|c| (c.power.unwrap_or(0), c.toughness.unwrap_or(0)))
                                    .unwrap_or((0, 0));

                                // CR 702.100a: trigger fires if entering P > evolve P
                                // OR entering T > evolve T (inclusive or).
                                if entering_power > evolve_power
                                    || entering_toughness > evolve_toughness
                                {
                                    let evolve_controller = state
                                        .objects
                                        .get(&evolve_id)
                                        .map(|o| o.controller)
                                        .unwrap_or(controller);

                                    // CR 702.100d: Count evolve instances from card
                                    // definition — OrdSet deduplicates, so check the
                                    // card definition for the exact count.
                                    let evolve_count = state
                                        .objects
                                        .get(&evolve_id)
                                        .and_then(|obj| obj.card_id.as_ref())
                                        .and_then(|cid| state.card_registry.get(cid.clone()))
                                        .map(|def| {
                                            def.abilities
                                                .iter()
                                                .filter(|a| {
                                                    matches!(
                                                        a,
                                                        AbilityDefinition::Keyword(
                                                            KeywordAbility::Evolve
                                                        )
                                                    )
                                                })
                                                .count()
                                        })
                                        .unwrap_or(1)
                                        .max(1);

                                    for _ in 0..evolve_count {
                                        triggers.push(PendingTrigger {
                                            source: evolve_id,
                                            ability_index: 0, // unused for evolve triggers
                                            controller: evolve_controller,
                                            triggering_event: Some(
                                                TriggerEvent::AnyPermanentEntersBattlefield,
                                            ),
                                            entering_object_id: Some(*object_id),
                                            targeting_stack_id: None,
                                            triggering_player: None,
                                            exalted_attacker_id: None,
                                            defending_player_id: None,
                                            is_evoke_sacrifice: false,
                                            is_madness_trigger: false,
                                            madness_exiled_card: None,
                                            madness_cost: None,
                                            is_miracle_trigger: false,
                                            miracle_revealed_card: None,
                                            miracle_cost: None,
                                            is_unearth_trigger: false,
                                            is_exploit_trigger: false,
                                            is_modular_trigger: false,
                                            modular_counter_count: None,
                                            is_evolve_trigger: true,
                                            evolve_entering_creature: Some(*object_id),
                                            is_myriad_trigger: false,
                                            is_suspend_counter_trigger: false,
                                            is_suspend_cast_trigger: false,
                                            suspend_card_id: None,
                                            is_hideaway_trigger: false,
                                            hideaway_count: None,
                                            is_partner_with_trigger: false,
                                            partner_with_name: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            GameEvent::SpellCast {
                player,
                source_object_id,
                ..
            } => {
                // AnySpellCast: fires on all permanents that watch for spell casts.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::AnySpellCast,
                    None,
                    None,
                );

                // CR 702.108a: Prowess — "Whenever you cast a noncreature spell."
                // Check if the cast spell is noncreature by inspecting the source object's
                // card types. Only fire if the spell lacks CardType::Creature.
                let is_noncreature = state
                    .objects
                    .get(source_object_id)
                    .map(|obj| {
                        !obj.characteristics
                            .card_types
                            .contains(&crate::state::types::CardType::Creature)
                    })
                    .unwrap_or(false);

                if is_noncreature {
                    // Collect triggers only for permanents controlled by the caster.
                    // Prowess says "whenever YOU cast" -- only the controller's creatures trigger.
                    let prowess_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == *player)
                        .map(|obj| obj.id)
                        .collect();

                    for obj_id in prowess_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::ControllerCastsNoncreatureSpell,
                            Some(obj_id),
                            None,
                        );
                    }
                }

                // CR 702.101a: Extort — "Whenever you cast a spell."
                // Collect triggers only for permanents controlled by the caster.
                // No type restriction (unlike Prowess which requires noncreature).
                // Each extort instance triggers separately (CR 702.101b).
                {
                    let controller_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == *player)
                        .map(|obj| obj.id)
                        .collect();

                    for obj_id in controller_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::ControllerCastsSpell,
                            Some(obj_id),
                            None,
                        );
                    }
                }

                // CR 603.2 / CR 102.2: "Whenever an opponent casts a spell."
                // Collect triggers on all permanents whose controller is NOT the caster.
                // In Commander FFA (CR 903.2, CR 102.2), all other players are opponents.
                {
                    let opponent_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller != *player)
                        .map(|obj| obj.id)
                        .collect();

                    let pre_len = triggers.len();
                    for obj_id in opponent_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::OpponentCastsSpell,
                            Some(obj_id),
                            None,
                        );
                    }
                    // Tag opponent-casts triggers with the casting player so
                    // flush_pending_triggers can set Target::Player at index 0.
                    for t in &mut triggers[pre_len..] {
                        t.triggering_player = Some(*player);
                    }
                }
            }

            GameEvent::PermanentTapped { object_id, .. } => {
                // SelfBecomesTapped: fires on the tapped permanent itself.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::SelfBecomesTapped,
                    Some(*object_id),
                    None,
                );
            }

            GameEvent::AttackersDeclared {
                attacking_player,
                attackers,
            } => {
                // SelfAttacks: fires on each creature that is declared as an attacker (CR 508.1m, CR 508.3a).
                // CR 702.86a / CR 508.5: tag each SelfAttacks trigger with the defending player
                // so annihilator (and any future "defending player" attack triggers) can resolve
                // the correct player in multiplayer games (CR 508.5a).
                for (attacker_id, attack_target) in attackers {
                    let pre_len = triggers.len();
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfAttacks,
                        Some(*attacker_id),
                        None,
                    );
                    // Resolve defending player from AttackTarget (CR 508.5).
                    let defending_player = match attack_target {
                        crate::state::combat::AttackTarget::Player(pid) => Some(*pid),
                        crate::state::combat::AttackTarget::Planeswalker(pw_id) => {
                            state.objects.get(pw_id).map(|obj| obj.controller)
                        }
                    };
                    for t in &mut triggers[pre_len..] {
                        t.defending_player_id = defending_player;
                    }

                    // CR 702.116a/b: Tag myriad triggers for special stack handling.
                    // A SelfAttacks trigger is a myriad trigger if its source object has
                    // the Myriad keyword. We check the triggered ability's description
                    // (set by builder.rs) to identify myriad triggers -- they carry
                    // `effect: None` and start with "Myriad". The `is_myriad_trigger`
                    // flag causes flush_pending_triggers to create a MyriadTrigger stack
                    // object (not a plain TriggeredAbility) so resolution.rs can execute
                    // the copy-and-attack logic.
                    for t in &mut triggers[pre_len..] {
                        if let Some(obj) = state.objects.get(&t.source) {
                            if let Some(ta) =
                                obj.characteristics.triggered_abilities.get(t.ability_index)
                            {
                                if ta.effect.is_none() && ta.description.starts_with("Myriad") {
                                    t.is_myriad_trigger = true;
                                }
                            }
                        }
                    }

                    // CR 702.105a: Dethrone -- "Whenever this creature attacks the player
                    // with the most life or tied for most life, put a +1/+1 counter on
                    // this creature."
                    // Only triggers when attacking a Player (not planeswalker/battle).
                    // CR 508.2a: condition checked at declaration time only.
                    if let crate::state::combat::AttackTarget::Player(def_pid) = attack_target {
                        // Find the maximum life total among all active (non-eliminated) players.
                        let defending_life = state
                            .players
                            .get(def_pid)
                            .map(|p| p.life_total)
                            .unwrap_or(i32::MIN);
                        let max_life = state
                            .players
                            .values()
                            .filter(|p| !p.has_lost && !p.has_conceded)
                            .map(|p| p.life_total)
                            .max()
                            .unwrap_or(i32::MIN);

                        if defending_life >= max_life {
                            let pre_len_dethrone = triggers.len();
                            collect_triggers_for_event(
                                state,
                                &mut triggers,
                                TriggerEvent::SelfAttacksPlayerWithMostLife,
                                Some(*attacker_id),
                                None,
                            );
                            // Tag dethrone triggers with defending player for consistency
                            // with other attack triggers (e.g., annihilator).
                            for t in &mut triggers[pre_len_dethrone..] {
                                t.defending_player_id = defending_player;
                            }
                        }
                    }
                }

                // CR 702.83a/b: Exalted — "Whenever a creature you control attacks alone."
                // If exactly one creature is declared as an attacker, fire exalted triggers
                // on ALL permanents controlled by the attacking player (not just the attacker).
                // CR 702.83b: "attacks alone" = exactly one creature declared as attacker.
                if attackers.len() == 1 {
                    let (lone_attacker_id, _) = &attackers[0];
                    let exalted_sources: Vec<ObjectId> = state
                        .objects
                        .values()
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield && obj.controller == *attacking_player
                        })
                        .map(|obj| obj.id)
                        .collect();

                    let pre_len = triggers.len();
                    for obj_id in exalted_sources {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::ControllerCreatureAttacksAlone,
                            Some(obj_id),
                            None,
                        );
                    }
                    // Tag exalted triggers with the lone attacker's ObjectId so
                    // flush_pending_triggers can set Target::Object(attacker_id) at index 0.
                    for t in &mut triggers[pre_len..] {
                        t.exalted_attacker_id = Some(*lone_attacker_id);
                    }
                }
            }

            GameEvent::BlockersDeclared { blockers, .. } => {
                // SelfBlocks: fires on each creature that is blocking (CR 603.5).
                for (blocker_id, _) in blockers {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfBlocks,
                        Some(*blocker_id),
                        None,
                    );
                }
            }

            GameEvent::PermanentTargeted {
                target_id,
                targeting_stack_id,
                targeting_controller,
            } => {
                // CR 702.21a: Ward triggers when this permanent becomes the target
                // of a spell or ability an opponent controls. Only triggers if the
                // targeting player is an opponent (not the permanent's controller).
                if let Some(obj) = state.objects.get(target_id) {
                    if obj.zone == ZoneId::Battlefield && obj.controller != *targeting_controller {
                        let pre_len = triggers.len();
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::SelfBecomesTargetByOpponent,
                            Some(*target_id),
                            None,
                        );
                        // Tag ward triggers with the targeting stack object ID so
                        // flush_pending_triggers can set the correct target on the
                        // ward triggered ability's stack entry (for CounterSpell resolution).
                        for t in &mut triggers[pre_len..] {
                            t.targeting_stack_id = Some(*targeting_stack_id);
                        }
                    }
                }
            }

            GameEvent::CreatureDied {
                new_grave_id,
                controller: death_controller,
                pre_death_counters,
                ..
            } => {
                // CR 603.6c / CR 603.10a / CR 700.4: "When ~ dies" triggers look back in time.
                // The creature is now in the graveyard, but its characteristics (including
                // triggered_abilities) are preserved by move_object_to_zone. Check the graveyard
                // object for SelfDies triggers rather than trying to find the battlefield object
                // (which no longer exists at trigger-check time).
                if let Some(obj) = state.objects.get(new_grave_id) {
                    for (idx, trigger_def) in
                        obj.characteristics.triggered_abilities.iter().enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SelfDies {
                            continue;
                        }

                        // CR 603.4: Check intervening-if clause at trigger time.
                        // Pass pre_death_counters for persist/undying counter checks (CR 702.79a).
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(
                                state,
                                cond,
                                *death_controller,
                                Some(pre_death_counters),
                            ) {
                                continue;
                            }
                        }

                        // CR 702.43a: Detect if this is a Modular trigger. Tag it with
                        // the +1/+1 counter count from last-known information so that
                        // flush_pending_triggers can create a ModularTrigger stack entry.
                        let is_modular = trigger_def.description.contains("Modular (CR 702.43a)");
                        let modular_counter_count = if is_modular {
                            Some(
                                pre_death_counters
                                    .get(&CounterType::PlusOnePlusOne)
                                    .copied()
                                    .unwrap_or(0),
                            )
                        } else {
                            None
                        };

                        triggers.push(PendingTrigger {
                            source: *new_grave_id,
                            ability_index: idx,
                            // CR 603.3a: use the controller captured at death time (before
                            // move_object_to_zone reset it to owner). This correctly handles
                            // stolen creatures — if Player A controls Player B's creature and
                            // it dies, the trigger is controlled by Player A.
                            controller: *death_controller,
                            triggering_event: Some(TriggerEvent::SelfDies),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            is_evoke_sacrifice: false,
                            is_madness_trigger: false,
                            madness_exiled_card: None,
                            madness_cost: None,
                            is_miracle_trigger: false,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            is_unearth_trigger: false,
                            is_exploit_trigger: false,
                            is_modular_trigger: is_modular,
                            modular_counter_count,
                            is_evolve_trigger: false,
                            evolve_entering_creature: None,
                            is_myriad_trigger: false,
                            is_suspend_counter_trigger: false,
                            is_suspend_cast_trigger: false,
                            suspend_card_id: None,
                            is_hideaway_trigger: false,
                            hideaway_count: None,
                            is_partner_with_trigger: false,
                            partner_with_name: None,
                        });
                    }
                }
            }

            GameEvent::AuraFellOff { new_grave_id, .. } => {
                // CR 603.6c / CR 603.10a: "When ~ is put into a graveyard from the battlefield"
                // triggers on Auras fire when the Aura moves to the graveyard via SBA 704.5m.
                // The Aura's characteristics (including triggered_abilities) are preserved in
                // the graveyard object by move_object_to_zone — same look-back pattern as
                // CreatureDied. Controller defaults to owner (as reset by move_object_to_zone).
                if let Some(obj) = state.objects.get(new_grave_id) {
                    let controller = obj.controller;
                    for (idx, trigger_def) in
                        obj.characteristics.triggered_abilities.iter().enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SelfDies {
                            continue;
                        }

                        // CR 603.4: Check intervening-if clause at trigger time.
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(state, cond, controller, None) {
                                continue;
                            }
                        }

                        triggers.push(PendingTrigger {
                            source: *new_grave_id,
                            ability_index: idx,
                            controller,
                            triggering_event: Some(TriggerEvent::SelfDies),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            is_evoke_sacrifice: false,
                            is_madness_trigger: false,
                            madness_exiled_card: None,
                            madness_cost: None,
                            is_miracle_trigger: false,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            is_unearth_trigger: false,
                            is_exploit_trigger: false,
                            is_modular_trigger: false,
                            modular_counter_count: None,
                            is_evolve_trigger: false,
                            evolve_entering_creature: None,
                            is_myriad_trigger: false,
                            is_suspend_counter_trigger: false,
                            is_suspend_cast_trigger: false,
                            suspend_card_id: None,
                            is_hideaway_trigger: false,
                            hideaway_count: None,
                            is_partner_with_trigger: false,
                            partner_with_name: None,
                        });
                    }
                }
            }

            GameEvent::Surveilled { player, .. } => {
                // CR 701.25d: "Whenever you surveil" triggers on all permanents
                // controlled by the surveilling player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == *player)
                    .map(|obj| obj.id)
                    .collect();

                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerSurveils,
                        Some(obj_id),
                        None,
                    );
                }
            }

            GameEvent::Investigated { player, .. } => {
                // CR 701.16a: "Whenever you investigate" triggers on all permanents
                // controlled by the investigating player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == *player)
                    .map(|obj| obj.id)
                    .collect();

                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerInvestigates,
                        Some(obj_id),
                        None,
                    );
                }
            }

            GameEvent::Connived { object_id, .. } => {
                // CR 701.50b: "Whenever [this creature] connives" triggers fire even if
                // the creature left the battlefield before the Connived event is processed.
                // Scryfall ruling (Psychic Pickpocket, 2022-04-29): "If ... that creature
                // has left the battlefield, the creature still connives. Abilities that
                // trigger 'when [that creature] connives' will trigger."
                //
                // `collect_triggers_for_event` enforces a zone == Battlefield check at
                // line 1518 and would skip off-battlefield objects. To comply with CR
                // 701.50b, we bypass the helper and generate the trigger inline,
                // accepting the object in ANY zone.
                if let Some(obj) = state.objects.get(object_id) {
                    for (idx, trigger_def) in
                        obj.characteristics.triggered_abilities.iter().enumerate()
                    {
                        if trigger_def.trigger_on != TriggerEvent::SourceConnives {
                            continue;
                        }
                        // CR 603.4: intervening-if check at trigger time.
                        if let Some(ref cond) = trigger_def.intervening_if {
                            if !check_intervening_if(state, cond, obj.controller, None) {
                                continue;
                            }
                        }
                        triggers.push(PendingTrigger {
                            source: *object_id,
                            ability_index: idx,
                            controller: obj.controller,
                            triggering_event: Some(TriggerEvent::SourceConnives),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            is_evoke_sacrifice: false,
                            is_madness_trigger: false,
                            madness_exiled_card: None,
                            madness_cost: None,
                            is_miracle_trigger: false,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            is_unearth_trigger: false,
                            is_exploit_trigger: false,
                            is_modular_trigger: false,
                            modular_counter_count: None,
                            is_evolve_trigger: false,
                            evolve_entering_creature: None,
                            is_myriad_trigger: false,
                            is_suspend_counter_trigger: false,
                            is_suspend_cast_trigger: false,
                            suspend_card_id: None,
                            is_hideaway_trigger: false,
                            hideaway_count: None,
                            is_partner_with_trigger: false,
                            partner_with_name: None,
                        });
                    }
                }
            }

            GameEvent::CombatDamageDealt { assignments } => {
                // CR 510.3a / CR 603.2: "Whenever ~ deals combat damage to a player"
                // triggers fire for each creature that dealt > 0 combat damage to a player.
                // CR 603.2g: damage with amount == 0 (fully prevented) does not trigger.
                // CR 603.10: NOT a look-back trigger — creature must be on battlefield;
                // collect_triggers_for_event checks obj.zone == Battlefield internally.
                for assignment in assignments {
                    if assignment.amount == 0 {
                        continue; // CR 603.2g: damage was fully prevented
                    }
                    if matches!(assignment.target, CombatDamageTarget::Player(_)) {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::SelfDealsCombatDamageToPlayer,
                            Some(assignment.source),
                            None,
                        );
                    }
                }
            }

            GameEvent::Proliferated { controller, .. } => {
                // CR 701.34: "Whenever you proliferate" triggers on all permanents
                // controlled by the proliferating player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == *controller)
                    .map(|obj| obj.id)
                    .collect();

                for obj_id in controller_sources {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::ControllerProliferates,
                        Some(obj_id),
                        None,
                    );
                }
            }

            _ => {}
        }
    }

    triggers
}

/// Collect triggered abilities of type `event_type` from battlefield permanents.
///
/// If `only_object` is `Some(id)`, only checks that specific object.
/// If `only_object` is `None`, checks all permanents on the battlefield.
///
/// `entering_object` is the object that entered the battlefield to cause this event,
/// if applicable (used by `TriggerDoublerFilter::ArtifactOrCreatureETB` to verify
/// the entering object's card types — CR 603.2d).
fn collect_triggers_for_event(
    state: &GameState,
    triggers: &mut Vec<PendingTrigger>,
    event_type: TriggerEvent,
    only_object: Option<ObjectId>,
    entering_object: Option<ObjectId>,
) {
    let object_ids: Vec<ObjectId> = if let Some(id) = only_object {
        vec![id]
    } else {
        state
            .objects
            .values()
            .filter(|obj| obj.zone == ZoneId::Battlefield)
            .map(|obj| obj.id)
            .collect()
    };

    for obj_id in object_ids {
        let Some(obj) = state.objects.get(&obj_id) else {
            continue;
        };
        if obj.zone != ZoneId::Battlefield {
            continue;
        }

        for (idx, trigger_def) in obj.characteristics.triggered_abilities.iter().enumerate() {
            if trigger_def.trigger_on != event_type {
                continue;
            }

            // CR 603.4: Check intervening-if at trigger time.
            // If the condition is false, the ability does not trigger.
            if let Some(ref cond) = trigger_def.intervening_if {
                if !check_intervening_if(state, cond, obj.controller, None) {
                    continue;
                }
            }

            triggers.push(PendingTrigger {
                source: obj_id,
                ability_index: idx,
                controller: obj.controller,
                triggering_event: Some(event_type.clone()),
                entering_object_id: entering_object,
                targeting_stack_id: None,
                triggering_player: None,
                exalted_attacker_id: None,
                defending_player_id: None,
                is_evoke_sacrifice: false,
                is_madness_trigger: false,
                madness_exiled_card: None,
                madness_cost: None,
                is_miracle_trigger: false,
                miracle_revealed_card: None,
                miracle_cost: None,
                is_unearth_trigger: false,
                is_exploit_trigger: false,
                is_modular_trigger: false,
                modular_counter_count: None,
                is_evolve_trigger: false,
                evolve_entering_creature: None,
                is_myriad_trigger: false,
                is_suspend_counter_trigger: false,
                is_suspend_cast_trigger: false,
                suspend_card_id: None,
                is_hideaway_trigger: false,
                hideaway_count: None,
                is_partner_with_trigger: false,
                partner_with_name: None,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Trigger flushing
// ---------------------------------------------------------------------------

/// Place all pending triggered abilities onto the stack in APNAP order (CR 603.3).
///
/// Called immediately before a player would receive priority. If no pending
/// triggers exist, this is a no-op.
///
/// CR 603.3: "Each time a player would receive priority, the game checks for any
/// triggered abilities that have triggered since the last time a player received
/// priority. If any have triggered, those abilities are put on the stack."
///
/// APNAP ordering (CR 101.4): Active player's triggers go on the stack first
/// (ending up at the bottom), then each non-active player in turn order. The last
/// player's triggers are on top and resolve first.
///
/// Returns events for each ability placed on the stack. Does NOT emit
/// `PriorityGiven` — the caller is responsible for granting priority after.
pub fn flush_pending_triggers(state: &mut GameState) -> Vec<GameEvent> {
    if state.pending_triggers.is_empty() {
        return Vec::new();
    }

    // CR 603.2d: Remove stale TriggerDoubler entries whose source left the battlefield.
    // This prevents accumulation of dead entries from permanents that left the battlefield.
    state.trigger_doublers.retain(|d| {
        state
            .objects
            .get(&d.source)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false)
    });

    // Drain all pending triggers.
    let pending: Vec<PendingTrigger> = state.pending_triggers.iter().cloned().collect();
    state.pending_triggers = im::Vector::new();

    // Build APNAP order starting from the active player.
    let apnap = apnap_order(state);

    // Stable-sort by controller position in APNAP order.
    let mut sorted = pending;
    sorted.sort_by_key(|t| {
        apnap
            .iter()
            .position(|&p| p == t.controller)
            .unwrap_or(usize::MAX)
    });

    let mut events = Vec::new();

    for trigger in sorted {
        // CR 603.2d: Check for Panharmonicon-style trigger doublers.
        // Compute how many times this trigger fires (1 base + additional from doublers).
        let additional_count = compute_trigger_doubling(state, &trigger);

        // CR 702.21a: For Ward triggers, the targeting stack object ID is carried
        // through PendingTrigger.targeting_stack_id. Set it as the triggered
        // ability's target so CounterSpell resolution can find the right stack entry.
        // CR 603.2 / CR 102.2: For OpponentCastsSpell triggers, the casting player
        // is set as Target::Player at index 0 so DeclaredTarget { index: 0 } resolves
        // to the specific opponent who cast the spell (e.g. Rhystic Study resolution).
        let trigger_targets: Vec<SpellTarget> = if let Some(tsid) = trigger.targeting_stack_id {
            vec![SpellTarget {
                target: Target::Object(tsid),
                zone_at_cast: None,
            }]
        } else if let Some(pid) = trigger.triggering_player {
            vec![SpellTarget {
                target: Target::Player(pid),
                zone_at_cast: None,
            }]
        } else if let Some(dp) = trigger.defending_player_id {
            // CR 702.86a / CR 508.5: Annihilator triggers carry the defending player ID.
            // Set as Target::Player at index 0 so PlayerTarget::DeclaredTarget { index: 0 }
            // resolves to the correct defending player for the SacrificePermanents effect.
            vec![SpellTarget {
                target: Target::Player(dp),
                zone_at_cast: None,
            }]
        } else if let Some(attacker_id) = trigger.exalted_attacker_id {
            // CR 702.83a: Exalted triggers carry the lone attacker's ObjectId.
            // Set it as Target::Object at index 0 so CEFilter::DeclaredTarget { index: 0 }
            // resolves to the attacking creature (not the exalted source permanent).
            vec![SpellTarget {
                target: Target::Object(attacker_id),
                zone_at_cast: None,
            }]
        } else {
            vec![]
        };

        // Push the triggered ability onto the stack (1 + additional_count) times.
        for _ in 0..=(additional_count) {
            let stack_id = state.next_object_id();
            // CR 702.74a: Evoke sacrifice triggers use EvokeSacrificeTrigger kind
            // instead of TriggeredAbility to distinguish them at resolution time.
            // CR 702.35a: Madness triggers use MadnessTrigger kind to carry
            // the exiled card ObjectId and madness cost for resolution.
            let kind = if trigger.is_evoke_sacrifice {
                StackObjectKind::EvokeSacrificeTrigger {
                    source_object: trigger.source,
                }
            } else if trigger.is_madness_trigger {
                StackObjectKind::MadnessTrigger {
                    source_object: trigger.source,
                    exiled_card: trigger.madness_exiled_card.unwrap_or(trigger.source),
                    madness_cost: trigger.madness_cost.clone().unwrap_or_default(),
                    owner: trigger.controller,
                }
            } else if trigger.is_miracle_trigger {
                // CR 702.94a: Miracle trigger carries the revealed card and cost.
                StackObjectKind::MiracleTrigger {
                    source_object: trigger.source,
                    revealed_card: trigger.miracle_revealed_card.unwrap_or(trigger.source),
                    miracle_cost: trigger.miracle_cost.clone().unwrap_or_default(),
                    owner: trigger.controller,
                }
            } else if trigger.is_unearth_trigger {
                // CR 702.84a: Unearth delayed exile trigger -- "Exile [this permanent]
                // at the beginning of the next end step."
                StackObjectKind::UnearthTrigger {
                    source_object: trigger.source,
                }
            } else if trigger.is_exploit_trigger {
                // CR 702.110a: Exploit ETB trigger -- "When this creature enters,
                // you may sacrifice a creature."
                StackObjectKind::ExploitTrigger {
                    source_object: trigger.source,
                }
            } else if trigger.is_modular_trigger {
                // CR 702.43a: Modular dies trigger -- "you may put a +1/+1 counter
                // on target artifact creature for each +1/+1 counter on this permanent."
                // Deterministic target selection: first artifact creature on the
                // battlefield by ObjectId ascending (OrdMap is sorted by key).
                // CR 603.3d: If no legal artifact creature target exists, the trigger
                // is not placed on the stack. Use `continue` to skip this trigger.
                let target_id = state
                    .objects
                    .iter()
                    .find(|(_, obj)| {
                        obj.zone == ZoneId::Battlefield
                            && obj.characteristics.card_types.contains(&CardType::Artifact)
                            && obj.characteristics.card_types.contains(&CardType::Creature)
                    })
                    .map(|(id, _)| *id);

                let Some(tid) = target_id else {
                    // No legal artifact creature target -- skip this trigger (CR 603.3d).
                    continue;
                };

                // Override trigger_targets with the selected artifact creature target.
                // (trigger_targets computed above does not apply to modular triggers.)
                let modular_targets = vec![SpellTarget {
                    target: Target::Object(tid),
                    zone_at_cast: Some(ZoneId::Battlefield),
                }];

                let counter_count = trigger.modular_counter_count.unwrap_or(0);
                let stack_id = state.next_object_id();
                let stack_obj = StackObject {
                    id: stack_id,
                    controller: trigger.controller,
                    kind: StackObjectKind::ModularTrigger {
                        source_object: trigger.source,
                        counter_count,
                    },
                    targets: modular_targets,
                    cant_be_countered: false,
                    is_copy: false,
                    cast_with_flashback: false,
                    kicker_times_paid: 0,
                    was_evoked: false,
                    was_bestowed: false,
                    cast_with_madness: false,
                    cast_with_miracle: false,
                    was_escaped: false,
                    cast_with_foretell: false,
                    was_buyback_paid: false,
                    was_suspended: false,
                };
                state.stack_objects.push_back(stack_obj);

                events.push(GameEvent::AbilityTriggered {
                    controller: trigger.controller,
                    source_object_id: trigger.source,
                    stack_object_id: stack_id,
                });

                // For trigger doubling: already handled via additional_count loop below,
                // but modular uses an early-exit path above. We run additional_count
                // copies too. However, for simplicity and correctness, break out of the
                // per-duplication loop by skipping the rest. The doubler case is handled
                // after the if-else chain below -- but since we already pushed the stack
                // object and emitted the event, we must NOT fall through to the bottom
                // of the loop. Use a labeled continue to advance to the next trigger.
                // NOTE: trigger doubling (Panharmonicon) is not applicable to non-ETB
                // triggers, so additional_count will always be 0 here.
                continue;
            } else if trigger.is_evolve_trigger {
                // CR 702.100a: Evolve ETB trigger — "Whenever a creature you control
                // enters, if that creature's P > this creature's P and/or that creature's
                // T > this creature's T, put a +1/+1 counter on this creature."
                // The resolution handler re-checks the intervening-if (CR 603.4).
                StackObjectKind::EvolveTrigger {
                    source_object: trigger.source,
                    entering_creature: trigger.evolve_entering_creature.unwrap_or(trigger.source),
                }
            } else if trigger.is_myriad_trigger {
                // CR 702.116a: Myriad SelfAttacks trigger -- "Whenever this creature
                // attacks, for each opponent other than defending player, create a token
                // copy tapped and attacking that player."
                // The `defending_player_id` was tagged by the AttackersDeclared handler
                // in check_triggers. Fallback to active player if somehow None.
                let defending = trigger
                    .defending_player_id
                    .unwrap_or(state.turn.active_player);
                StackObjectKind::MyriadTrigger {
                    source_object: trigger.source,
                    defending_player: defending,
                }
            } else if trigger.is_suspend_counter_trigger {
                // CR 702.62a: Suspend upkeep counter-removal trigger.
                // "At the beginning of your upkeep, if this card is suspended, remove a
                // time counter from it." This trigger goes on the stack and can be
                // responded to (e.g., Stifle can counter it, preventing counter removal).
                StackObjectKind::SuspendCounterTrigger {
                    source_object: trigger.source,
                    suspended_card: trigger.suspend_card_id.unwrap_or(trigger.source),
                }
            } else if trigger.is_suspend_cast_trigger {
                // CR 702.62a: Suspend cast trigger (last time counter removed).
                // "When the last time counter is removed from this card, if it's exiled,
                // you may play it without paying its mana cost if able."
                StackObjectKind::SuspendCastTrigger {
                    source_object: trigger.source,
                    suspended_card: trigger.suspend_card_id.unwrap_or(trigger.source),
                    owner: trigger.controller,
                }
            } else if trigger.is_hideaway_trigger {
                // CR 702.75a: Hideaway ETB trigger — "When this permanent enters,
                // look at the top N cards of your library. Exile one of them face
                // down and put the rest on the bottom of your library in a random order."
                StackObjectKind::HideawayTrigger {
                    source_object: trigger.source,
                    hideaway_count: trigger.hideaway_count.unwrap_or(4),
                }
            } else if trigger.is_partner_with_trigger {
                // CR 702.124j: Partner With ETB trigger — "When this permanent enters,
                // target player may search their library for a card named [name], reveal
                // it, put it into their hand, then shuffle."
                // Target player: deterministic fallback = the trigger controller (owner).
                StackObjectKind::PartnerWithTrigger {
                    source_object: trigger.source,
                    partner_name: trigger.partner_with_name.clone().unwrap_or_default(),
                    target_player: trigger.controller,
                }
            } else {
                StackObjectKind::TriggeredAbility {
                    source_object: trigger.source,
                    ability_index: trigger.ability_index,
                }
            };
            let stack_obj = StackObject {
                id: stack_id,
                controller: trigger.controller,
                kind,
                targets: trigger_targets.clone(),
                cant_be_countered: false,
                is_copy: false,
                cast_with_flashback: false,
                kicker_times_paid: 0,
                was_evoked: false,
                was_bestowed: false,
                cast_with_madness: false,
                cast_with_miracle: false,
                was_escaped: false,
                cast_with_foretell: false,
                was_buyback_paid: false,
                was_suspended: false,
            };
            state.stack_objects.push_back(stack_obj);

            events.push(GameEvent::AbilityTriggered {
                controller: trigger.controller,
                source_object_id: trigger.source,
                stack_object_id: stack_id,
            });
        }
    }

    if !events.is_empty() {
        // Triggers going on the stack is a game action — reset priority pass count.
        state.turn.players_passed = OrdSet::new();
    }

    events
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns player IDs in APNAP order starting from the active player.
///
/// CR 101.4 (APNAP): Active Player, Non-Active Players in turn order.
pub fn apnap_order(state: &GameState) -> Vec<PlayerId> {
    let active = state.turn.active_player;
    let order = &state.turn.turn_order;
    let n = order.len();
    let start = order.iter().position(|&p| p == active).unwrap_or(0);
    (0..n).map(|i| order[(start + i) % n]).collect()
}

/// CR 603.2d: Compute how many additional times a trigger should fire due to
/// Panharmonicon-style trigger-doubling effects.
///
/// Returns the number of ADDITIONAL triggers beyond the base 1. So a return
/// value of 0 means fire exactly once; 1 means fire twice; etc.
///
/// Each active `TriggerDoubler` whose filter matches the trigger contributes
/// `additional_triggers` extra instances. With two Panharmonicons, an ETB
/// trigger that would fire once instead fires three times (2 extra each).
///
/// Panharmonicon-style rulings (2024): the ability "triggers an additional time"
/// — each Panharmonicon adds another copy; they stack independently.
fn compute_trigger_doubling(state: &GameState, trigger: &PendingTrigger) -> u32 {
    let mut additional = 0u32;

    for doubler in state.trigger_doublers.iter() {
        if doubler_applies_to_trigger(state, doubler, trigger) {
            additional += doubler.additional_triggers;
        }
    }

    additional
}

/// CR 603.2d: Determine whether a specific `TriggerDoubler` applies to the given trigger.
///
/// For `ArtifactOrCreatureETB`: the trigger must be from a permanent entering the
/// battlefield, AND the trigger's source (the permanent with the ability) must be
/// controlled by the doubler's controller, AND the triggering event must be
/// `AnyPermanentEntersBattlefield` caused by an artifact or creature entering.
///
/// TODO: SelfEntersBattlefield triggers (PartnerWith, Hideaway, Exploit) are not doubled
/// by Panharmonicon — fix holistically when addressing trigger doubling for all self-ETB
/// triggers. These keyword ETB triggers use TriggerEvent::SelfEntersBattlefield, but this
/// function only matches TriggerEvent::AnyPermanentEntersBattlefield.
fn doubler_applies_to_trigger(
    state: &GameState,
    doubler: &TriggerDoubler,
    trigger: &PendingTrigger,
) -> bool {
    // Doubler source must still be on the battlefield.
    let source_active = state
        .objects
        .get(&doubler.source)
        .map(|o| o.zone == ZoneId::Battlefield)
        .unwrap_or(false);
    if !source_active {
        return false;
    }

    // The trigger must be controlled by the same player as the doubler.
    if trigger.controller != doubler.controller {
        return false;
    }

    match &doubler.filter {
        TriggerDoublerFilter::ArtifactOrCreatureETB => {
            // The triggering event must be AnyPermanentEntersBattlefield (CR 603.2d).
            let is_etb = matches!(
                trigger.triggering_event,
                Some(TriggerEvent::AnyPermanentEntersBattlefield)
            );
            if !is_etb {
                return false;
            }

            // The entering object must be an artifact or creature (CR 603.2d).
            // Use entering_object_id (set by check_triggers from PermanentEnteredBattlefield event).
            // If entering_object_id is absent, we cannot confirm the type — conservatively skip.
            let entering_id = match trigger.entering_object_id {
                Some(id) => id,
                None => return false,
            };

            // Use calculate_characteristics for type checks under continuous effects,
            // falling back to raw characteristics if the object is no longer in the
            // objects map (e.g., it moved zones since entering).
            let entering_chars =
                crate::rules::layers::calculate_characteristics(state, entering_id).or_else(|| {
                    state
                        .objects
                        .get(&entering_id)
                        .map(|o| o.characteristics.clone())
                });

            entering_chars
                .map(|chars| {
                    use crate::state::types::CardType;
                    chars.card_types.contains(&CardType::Artifact)
                        || chars.card_types.contains(&CardType::Creature)
                })
                .unwrap_or(false)
        }
    }
}

// ---------------------------------------------------------------------------
// Crew handler (CR 702.122)
// ---------------------------------------------------------------------------

/// Handle a CrewVehicle command: validate, tap crew creatures, push crew ability onto the stack.
///
/// CR 702.122a: "Tap any number of other untapped creatures you control with total power N
/// or greater: This permanent becomes an artifact creature until end of turn."
///
/// When the crew ability resolves, an `AddCardTypes({Creature})` continuous effect is
/// registered in Layer 4 (TypeChange) with `UntilEndOfTurn` duration.
///
/// Notable rulings:
/// - Summoning sickness does NOT prevent crewing (ruling): tapping for crew cost is not
///   a {T} activated ability — summoning sickness only prevents those.
/// - Crewing an already-crewed Vehicle is legal but has no effect (ruling).
/// - Becoming a creature via crew does NOT trigger ETB effects (ruling).
pub fn handle_crew_vehicle(
    state: &mut GameState,
    player: PlayerId,
    vehicle: ObjectId,
    crew_creatures: Vec<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use std::collections::HashSet;

    use crate::cards::card_definition::ContinuousEffectDef;
    use crate::rules::layers::calculate_characteristics;
    use crate::state::continuous_effect::{
        EffectDuration, EffectFilter, EffectLayer, LayerModification,
    };
    use crate::state::types::CardType;

    // CR 602.2: Crewing requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // CR 702.61a: If a spell with split second is on the stack, no non-mana
    // abilities can be activated.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; crew ability cannot be activated (CR 702.61a)".into(),
        ));
    }

    // Validate the Vehicle: must be on the battlefield, controlled by the player,
    // and must have KeywordAbility::Crew(n). Use calculate_characteristics for
    // layer correctness (e.g., Humility may have removed the keyword).
    let crew_cost_n: u32 = {
        let obj = state.object(vehicle)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(vehicle));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: vehicle,
            });
        }
        // Use layer-computed characteristics to account for continuous effects.
        let chars = calculate_characteristics(state, vehicle).or_else(|| {
            state
                .objects
                .get(&vehicle)
                .map(|o| o.characteristics.clone())
        });
        let crew_n = chars.as_ref().and_then(|c| {
            c.keywords.iter().find_map(|kw| {
                if let KeywordAbility::Crew(n) = kw {
                    Some(*n)
                } else {
                    None
                }
            })
        });
        crew_n.ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "object {:?} does not have the Crew keyword (CR 702.122a)",
                vehicle
            ))
        })?
    };

    // Validate crew_creatures is non-empty (you must tap at least one creature).
    if crew_creatures.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "must provide at least one creature to crew the vehicle (CR 702.122a)".into(),
        ));
    }

    // CR 702.122a: Validate uniqueness — no duplicates in crew_creatures.
    let mut seen: HashSet<ObjectId> = HashSet::new();
    for &id in &crew_creatures {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate creature {:?} in crew_creatures (CR 702.122a)",
                id
            )));
        }
    }

    // CR 702.122a: Validate each crew creature — must be an untapped creature
    // you control on the battlefield, and must not be the vehicle itself.
    // Also sum total power for the crew cost threshold check.
    // Note: summoning sickness does NOT prevent crewing (ruling under CR 702.122a);
    // tapping for crew cost is not a {T} activated ability.
    let mut total_power: i32 = 0;
    for &id in &crew_creatures {
        // CR 702.122a: "other" — vehicle cannot crew itself.
        if id == vehicle {
            return Err(GameStateError::InvalidCommand(
                "a vehicle cannot be used to crew itself (CR 702.122a: 'other untapped creatures')"
                    .into(),
            ));
        }

        let obj = state
            .objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))?;

        // Must be on the battlefield.
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(id));
        }

        // Must be controlled by the player.
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: id,
            });
        }

        // Must be untapped (CR 702.122a: "untapped creatures").
        if obj.status.tapped {
            return Err(GameStateError::InvalidCommand(format!(
                "creature {:?} is already tapped and cannot be used to crew (CR 702.122a)",
                id
            )));
        }

        // Must be a creature (use layer-computed characteristics).
        let chars = calculate_characteristics(state, id)
            .or_else(|| state.objects.get(&id).map(|o| o.characteristics.clone()));
        let is_creature = chars
            .as_ref()
            .map(|c| c.card_types.contains(&CardType::Creature))
            .unwrap_or(false);
        if !is_creature {
            return Err(GameStateError::InvalidCommand(format!(
                "object {:?} is not a creature and cannot be used to crew (CR 702.122a)",
                id
            )));
        }

        // Accumulate power for the total power check.
        let power = chars.and_then(|c| c.power).unwrap_or(0);
        total_power = total_power.saturating_add(power);
    }

    // CR 702.122a: Total power of tapped creatures must be >= N.
    if total_power < crew_cost_n as i32 {
        return Err(GameStateError::InvalidCommand(format!(
            "total power of crew creatures ({}) is less than Crew {} cost (CR 702.122a)",
            total_power, crew_cost_n
        )));
    }

    // Pay the cost: tap all crew creatures (CR 602.2b analog for crew cost).
    let mut events = Vec::new();
    for &id in &crew_creatures {
        if let Some(obj) = state.objects.get_mut(&id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: id,
        });
    }

    // Push the crew ability onto the stack as an activated ability.
    // The embedded effect is `ApplyContinuousEffect` that adds `Creature` type
    // in Layer 4 with `UntilEndOfTurn` duration, targeting the vehicle (source).
    let stack_id = state.next_object_id();

    // Build the embedded effect: AddCardTypes({Creature}) in Layer 4, on the source.
    let effect_def = ContinuousEffectDef {
        layer: EffectLayer::TypeChange,
        modification: LayerModification::AddCardTypes(im::OrdSet::from(vec![CardType::Creature])),
        filter: EffectFilter::Source, // resolved to SingleObject(vehicle) at execution
        duration: EffectDuration::UntilEndOfTurn,
    };
    let embedded_effect = crate::cards::card_definition::Effect::ApplyContinuousEffect {
        effect_def: Box::new(effect_def),
    };

    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::ActivatedAbility {
            source_object: vehicle,
            ability_index: 0, // synthetic — crew ability has no index in activated_abilities
            embedded_effect: Some(Box::new(embedded_effect)),
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
    };
    state.stack_objects.push_back(stack_obj);

    // CR 602.2e / CR 116.3b: After activating, the active player receives priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: vehicle,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}

/// Evaluate an intervening-if condition against the current game state (CR 603.4).
///
/// `pre_death_counters` — counters captured from the creature just before it left
/// the battlefield. Required for `SourceHadNoCounterOfType` checks (persist/undying).
/// Pass `None` for all non-death trigger contexts.
pub fn check_intervening_if(
    state: &GameState,
    cond: &InterveningIf,
    controller: PlayerId,
    pre_death_counters: Option<&im::OrdMap<crate::state::types::CounterType, u32>>,
) -> bool {
    match cond {
        InterveningIf::ControllerLifeAtLeast(n) => state
            .players
            .get(&controller)
            .map(|p| p.life_total >= *n as i32)
            .unwrap_or(false),
        // CR 702.79a / CR 702.93a: "if it had no [counter type] counters on it"
        // Checked against last-known-information (pre-death counters) at trigger time.
        // At resolution time, caller passes None; the condition is treated as true
        // (the MoveZone effect will silently no-op if the source left the graveyard).
        InterveningIf::SourceHadNoCounterOfType(ct) => pre_death_counters
            .map(|counters| !counters.contains_key(ct))
            .unwrap_or(true),
    }
}
