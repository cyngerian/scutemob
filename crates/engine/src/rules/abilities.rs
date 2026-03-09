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
use crate::state::types::AltCostKind;
use crate::state::stack::{StackObject, StackObjectKind, TriggerData};
use crate::state::stubs::{
    PendingTrigger, PendingTriggerKind, TriggerDoubler, TriggerDoublerFilter,
};
use crate::state::targeting::{SpellTarget, Target};
use crate::state::types::{CardType, ChampionFilter, CounterType, KeywordAbility};
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
    discard_card: Option<ObjectId>,
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

    // CR 702.67a / CR 601.2c: Fortify abilities can only target "a land you control."
    // Validate target type and controller BEFORE spending any costs, so that mana is
    // not wasted when the activation is illegal.
    if matches!(
        &embedded_effect,
        Some(crate::cards::card_definition::Effect::AttachFortification { .. })
    ) {
        // CR 301.6: A Fortification that's also a creature can't fortify a land.
        // Check source (the Fortification itself) using layer-resolved characteristics.
        let source_is_creature = {
            let layer_chars = crate::rules::layers::calculate_characteristics(state, source)
                .or_else(|| {
                    state
                        .objects
                        .get(&source)
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
        if source_is_creature {
            return Err(GameStateError::InvalidTarget(
                "a Fortification that's also a creature can't fortify a land (CR 301.6)".into(),
            ));
        }

        if let Some(Target::Object(target_id)) = targets.first() {
            let target_id = *target_id;
            // Check: target must be a land on the battlefield controlled by the
            // activating player. Use layer-computed characteristics for correctness
            // under continuous effects (e.g. non-land permanents that became lands).
            let on_battlefield_and_controlled = state
                .objects
                .get(&target_id)
                .map(|obj| {
                    obj.zone == crate::state::zone::ZoneId::Battlefield && obj.controller == player
                })
                .unwrap_or(false);
            let is_land = {
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
                            .contains(&crate::state::types::CardType::Land)
                    })
                    .unwrap_or(false)
            };
            if !on_battlefield_and_controlled {
                return Err(GameStateError::InvalidTarget(
                    "fortify target must be a land you control on the battlefield".into(),
                ));
            }
            if !is_land {
                return Err(GameStateError::InvalidTarget(
                    "fortify target must be a land".into(),
                ));
            }
        }
    }

    // CR 702.151a: Reconfigure unattach ability -- "Activate only if this permanent is
    // attached to a creature." Validate BEFORE spending any costs.
    if matches!(
        &embedded_effect,
        Some(crate::cards::card_definition::Effect::DetachEquipment { .. })
    ) {
        let is_attached = state
            .objects
            .get(&source)
            .and_then(|obj| obj.attached_to)
            .is_some();
        if !is_attached {
            return Err(GameStateError::InvalidCommand(
                "reconfigure unattach: permanent must be attached to a creature".into(),
            ));
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

    // CR 602.2 / CR 111.10g: Pay discard-a-card cost (e.g., Blood token activation).
    // The discard is a cost, not an effect — it happens at activation time, before the
    // ability goes on the stack. The caller must supply discard_card: Some(ObjectId)
    // if the ability cost requires a discard.
    if ability_cost.discard_card {
        let card_to_discard = discard_card.ok_or_else(|| {
            GameStateError::InvalidCommand(
                "ability requires discarding a card as cost: discard_card must be Some (CR 602.2)"
                    .into(),
            )
        })?;
        // Validate the card is in the player's hand.
        {
            let card_obj = state.object(card_to_discard)?;
            if card_obj.zone != ZoneId::Hand(player) {
                return Err(GameStateError::InvalidCommand(
                    "discard cost: card must be in your hand (CR 602.2)".into(),
                ));
            }
        }
        // Move card from hand to graveyard.
        let (new_grave_id, _) =
            state.move_object_to_zone(card_to_discard, ZoneId::Graveyard(player))?;
        events.push(GameEvent::CardDiscarded {
            player,
            object_id: card_to_discard,
            new_id: new_grave_id,
        });
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

    // CR 701.61a: Pay forage cost — "Exile three cards from your graveyard or sacrifice a Food."
    // Deterministic fallback (M9.5): prefer Food sacrifice when both options are available.
    if ability_cost.forage {
        // Collect Food artifacts controlled by this player on the battlefield (phased in).
        let food_subtype = crate::state::types::SubType("Food".to_string());
        let mut food_ids: Vec<ObjectId> = state
            .objects
            .iter()
            .filter_map(|(&id, obj)| {
                if obj.zone == ZoneId::Battlefield && obj.controller == player && obj.is_phased_in()
                {
                    // Use layer-resolved characteristics to respect continuous effects.
                    let chars = crate::rules::layers::calculate_characteristics(state, id)
                        .unwrap_or_else(|| obj.characteristics.clone());
                    if chars.subtypes.contains(&food_subtype) {
                        Some(id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        food_ids.sort(); // deterministic: smallest ObjectId first

        // Collect graveyard cards for the exile-3 option.
        let mut grave_ids: Vec<ObjectId> = state
            .objects
            .iter()
            .filter_map(|(&id, obj)| {
                if obj.zone == ZoneId::Graveyard(player) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();
        grave_ids.sort(); // deterministic: smallest ObjectId first

        let has_food = !food_ids.is_empty();
        let has_three_grave = grave_ids.len() >= 3;

        if !has_food && !has_three_grave {
            return Err(GameStateError::InvalidCommand(
                "cannot forage: need a Food you control or 3+ cards in your graveyard (CR 701.61a)"
                    .into(),
            ));
        }

        if has_food {
            // Sacrifice a Food (deterministic: lowest ObjectId).
            let food_id = food_ids[0];
            let owner = state.object(food_id)?.owner;
            let (new_grave_id, _) = state.move_object_to_zone(food_id, ZoneId::Graveyard(owner))?;
            events.push(GameEvent::PermanentDestroyed {
                object_id: food_id,
                new_grave_id,
            });
        } else {
            // Exile 3 cards from graveyard (deterministic: lowest ObjectId order).
            let to_exile: Vec<ObjectId> = grave_ids.into_iter().take(3).collect();
            for id in to_exile {
                let (new_exile_id, _) = state.move_object_to_zone(id, ZoneId::Exile)?;
                events.push(GameEvent::ObjectExiled {
                    player,
                    object_id: id,
                    new_exile_id,
                });
            }
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: triggered abilities are not cleave casts.
        was_cleaved: false,
        // CR 702.42a: triggered/copy abilities are not entwine casts.
        // CR 702.120a: triggered abilities have no escalate modes paid.
        // CR 702.47a: triggered abilities have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: triggered abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: triggered/activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
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
            kind: PendingTriggerKind::Madness,
            triggering_event: None,
            entering_object_id: None,
            targeting_stack_id: None,
            triggering_player: None,
            exalted_attacker_id: None,
            defending_player_id: None,
            madness_exiled_card: Some(new_grave_id),
            madness_cost,
            miracle_revealed_card: None,
            miracle_cost: None,
            modular_counter_count: None,
            evolve_entering_creature: None,
            suspend_card_id: None,
            hideaway_count: None,
            partner_with_name: None,
            ingest_target_player: None,
            flanking_blocker_id: None,
            rampage_n: None,
            provoke_target_creature: None,
            renown_n: None,
            poisonous_n: None,
            poisonous_target_player: None,
            enlist_enlisted_creature: None,
            encore_activator: None,
            echo_cost: None,
            cumulative_upkeep_cost: None,
            recover_cost: None,
            recover_card: None,
            graft_entering_creature: None,
            backup_abilities: None,
            backup_n: None,
            champion_filter: None,
            champion_exiled_card: None,
            soulbond_pair_target: None,
            squad_count: None,
            gift_opponent: None,
            cipher_encoded_card_id: None,
            cipher_encoded_object_id: None,
            haunt_source_object_id: None,
            haunt_source_card_id: None,
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: triggered abilities are not cleave casts.
        was_cleaved: false,
        // CR 702.42a: triggered/copy abilities are not entwine casts.
        // CR 702.120a: triggered abilities have no escalate modes paid.
        // CR 702.47a: triggered abilities have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: triggered abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: triggered/activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
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

/// CR 702.59a: Look up the recover cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Recover { cost }`, or `None`
/// if the card has no definition or no recover ability defined.
fn find_recover_cost(
    card_id: &Option<crate::state::player::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Recover { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Forecast (CR 702.57)
// ---------------------------------------------------------------------------

/// Handle an ActivateForecast command: validate timing/zone/once-per-turn,
/// pay mana cost, push forecast ability onto stack.
///
/// CR 702.57a: Forecast is an activated ability from hand.
/// CR 702.57b: May only be activated during the upkeep step of the card's owner,
/// and only once each turn. The card is revealed but stays in hand.
pub fn handle_activate_forecast(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::turn::Step;

    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // 2. Split second check (CR 702.61a): Forecast is an activated ability, not a mana
    //    ability. It cannot be activated while a spell with split second is on the stack.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; forecast cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 3. Upkeep check (CR 702.57b): only during the upkeep step.
    if state.turn.step != Step::Upkeep {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateForecast: forecast may only be activated during the upkeep step (CR 702.57b); \
             current step is {:?}",
            state.turn.step
        )));
    }

    // 4. Owner's upkeep check (CR 702.57b): the card's owner must be the active player.
    //    In multiplayer, only during the turn of the card's owner.
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateForecast: forecast may only be activated during the owner's upkeep (CR 702.57b); \
             active player is {:?}, activating player is {:?}",
            state.turn.active_player, player
        )));
    }

    // 5. Zone check (CR 702.57a): card must be in Hand(player).
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateForecast: card {:?} is not in Hand({:?}); \
                 forecast can only be activated from hand (CR 702.57a)",
                card, player
            )));
        }
    }

    // 6. Keyword check (CR 702.57a): card must have KeywordAbility::Forecast.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Forecast)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateForecast: card {:?} does not have the Forecast keyword (CR 702.57a)",
                card
            )));
        }
    }

    // 7. Once-per-turn check (CR 702.57b): card must not have already used forecast this turn.
    let card_id_opt = state.object(card)?.card_id.clone();
    if let Some(ref cid) = card_id_opt {
        if state.forecast_used_this_turn.contains(cid) {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateForecast: card {:?} has already activated its forecast this turn (CR 702.57b)",
                card
            )));
        }
    }

    // 8. Look up cost and effect from AbilityDefinition::Forecast in card registry.
    let registry = state.card_registry.clone();
    let (forecast_cost, forecast_effect) = card_id_opt
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Forecast { cost, effect } = a {
                    Some((cost.clone(), effect.clone()))
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "ActivateForecast: card {:?} has no AbilityDefinition::Forecast entry",
                card
            ))
        })?;

    // 9. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if forecast_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &forecast_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &forecast_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: forecast_cost,
        });
    }

    // 10. Mark forecast as used for this turn (CR 702.57b — once per card per turn).
    if let Some(cid) = card_id_opt {
        state.forecast_used_this_turn = state.forecast_used_this_turn.update(cid);
    }

    // 11. Push forecast ability onto stack.
    // The card stays in hand — no zone move.
    // Convert Vec<Target> → Vec<SpellTarget> capturing zone at activation time (CR 601.2c).
    let spell_targets: Vec<SpellTarget> = targets
        .into_iter()
        .map(|t| {
            let zone_at_cast = match &t {
                Target::Object(id) => state.objects.get(id).map(|obj| obj.zone),
                Target::Player(_) => None,
            };
            SpellTarget {
                target: t,
                zone_at_cast,
            }
        })
        .collect();
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::ForecastAbility {
            source_object: card,
            embedded_effect: Box::new(forecast_effect),
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: forecast abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: forecast abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: forecast abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(stack_obj);

    // 12. Reset priority (CR 602.2e): active player gets priority.
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

// ---------------------------------------------------------------------------
// Bloodrush (CR 207.2c — ability word; underlying mechanics: CR 602)
// ---------------------------------------------------------------------------

/// Handle an ActivateBloodrush command: validate zone/target/mana, discard self
/// as cost, and push BloodrushAbility onto the stack.
///
/// CR 207.2c: Bloodrush is an ability word. The underlying ability is an activated
/// ability (CR 602) of the form:
/// "{cost}, Discard this card: Target attacking creature gets +N/+N
/// [and gains {keyword}] until end of turn."
///
/// Key rules:
/// - CR 602.2a: The card is in a hidden zone (hand); it is revealed during activation.
/// - CR 602.2b: The discard is the additional cost; paid before ability goes on stack.
/// - CR 115: "Target attacking creature" — target must be in `state.combat.attackers`.
/// - CR 702.61a: Cannot activate while split second is on the stack.
pub fn handle_activate_bloodrush(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    target: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // 2. Split second check (CR 702.61a): Bloodrush is an activated ability, not a mana
    //    ability. It cannot be activated while a spell with split second is on the stack.
    if crate::rules::casting::has_split_second_on_stack(state) {
        return Err(GameStateError::InvalidCommand(
            "a spell with split second is on the stack; bloodrush cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 3. Zone check (CR 602.2a): card must be in Hand(player).
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Hand(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: card {:?} is not in Hand({:?}); \
                 bloodrush can only be activated from hand (CR 602.2a)",
                card, player
            )));
        }
    }

    // 4. AbilityDefinition check: card must have AbilityDefinition::Bloodrush.
    //    We look up from the card registry, not the characteristics keywords,
    //    because bloodrush is an ability word (not a KeywordAbility variant).
    let card_id_opt = state.object(card)?.card_id.clone();
    let registry = state.card_registry.clone();
    let bloodrush_def = card_id_opt
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Bloodrush {
                    cost,
                    power_boost,
                    toughness_boost,
                    grants_keyword,
                } = a
                {
                    Some((
                        cost.clone(),
                        *power_boost,
                        *toughness_boost,
                        grants_keyword.clone(),
                    ))
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: card {:?} has no AbilityDefinition::Bloodrush entry",
                card
            ))
        })?;
    let (bloodrush_cost, power_boost, toughness_boost, grants_keyword) = bloodrush_def;

    // 5. Target validation (CR 115): target must be on the battlefield as a creature
    //    AND currently registered as an attacker in CombatState.
    {
        let target_obj = state.objects.get(&target).ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: target {:?} does not exist",
                target
            ))
        })?;
        if !matches!(target_obj.zone, ZoneId::Battlefield) {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: target {:?} is not on the battlefield (CR 115)",
                target
            )));
        }
        if !target_obj
            .characteristics
            .card_types
            .contains(&crate::state::types::CardType::Creature)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateBloodrush: target {:?} is not a creature (CR 115)",
                target
            )));
        }
    }
    let is_attacking = state
        .combat
        .as_ref()
        .map(|c| c.attackers.contains_key(&target))
        .unwrap_or(false);
    if !is_attacking {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateBloodrush: target {:?} is not an attacking creature (CR 115). \
             Bloodrush requires 'target attacking creature'.",
            target
        )));
    }

    // 6. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if bloodrush_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &bloodrush_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &bloodrush_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: bloodrush_cost,
        });
    }

    // 7. Discard self as cost (CR 602.2b): the card goes to graveyard before
    //    the ability goes on the stack. Check for Madness first (CR 702.35a).
    let owner = state.object(card)?.owner;
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

    // Emit CardDiscarded (CR 701.8).
    events.push(GameEvent::CardDiscarded {
        player,
        object_id: card,
        new_id: new_grave_id,
    });

    // Handle Madness if present (CR 702.35a): queue Madness trigger.
    if has_madness {
        let madness_cost = card_id_opt.as_ref().and_then(|cid| {
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
        state
            .pending_triggers
            .push_back(crate::state::stubs::PendingTrigger {
                source: new_grave_id,
                ability_index: 0,
                controller: player,
                kind: crate::state::stubs::PendingTriggerKind::Madness,
                triggering_event: None,
                entering_object_id: None,
                targeting_stack_id: None,
                triggering_player: None,
                exalted_attacker_id: None,
                defending_player_id: None,
                madness_exiled_card: Some(new_grave_id),
                madness_cost,
                miracle_revealed_card: None,
                miracle_cost: None,
                modular_counter_count: None,
                evolve_entering_creature: None,
                suspend_card_id: None,
                hideaway_count: None,
                partner_with_name: None,
                ingest_target_player: None,
                flanking_blocker_id: None,
                rampage_n: None,
                provoke_target_creature: None,
                renown_n: None,
                poisonous_n: None,
                poisonous_target_player: None,
                enlist_enlisted_creature: None,
                encore_activator: None,
                echo_cost: None,
                cumulative_upkeep_cost: None,
                recover_cost: None,
                recover_card: None,
                graft_entering_creature: None,
                backup_abilities: None,
                backup_n: None,
                champion_filter: None,
                champion_exiled_card: None,
                soulbond_pair_target: None,
                squad_count: None,
                gift_opponent: None,
                cipher_encoded_card_id: None,
                cipher_encoded_object_id: None,
                haunt_source_object_id: None,
                haunt_source_card_id: None,
            });
    }

    // 8. Push BloodrushAbility onto stack (CR 602.2c).
    //    The source card is now in the graveyard; source_object records the
    //    pre-discard ObjectId for attribution only.
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::BloodrushAbility {
            source_object: card,
            target_creature: target,
            power_boost,
            toughness_boost,
            grants_keyword,
        },
        targets: vec![SpellTarget {
            target: Target::Object(target),
            zone_at_cast: state.objects.get(&target).map(|o| o.zone),
        }],
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        // CR 701.59c: activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
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

    // CR 702.21a: Emit PermanentTargeted so Ward triggers fire when the target
    // creature has Ward. Mirrors the pattern in handle_activate_ability (lines
    // 464-470) which emits this event for every battlefield permanent targeted
    // by an activated ability.
    events.push(GameEvent::PermanentTargeted {
        target_id: target,
        targeting_stack_id: stack_id,
        targeting_controller: player,
    });

    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: triggered abilities are not cleave casts.
        was_cleaved: false,
        // CR 702.42a: triggered/copy abilities are not entwine casts.
        // CR 702.120a: triggered abilities have no escalate modes paid.
        // CR 702.47a: triggered abilities have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: triggered abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: triggered/activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
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
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Unearth, .. }`,
/// or `None` if the card has no definition or no unearth ability defined.
fn get_unearth_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility { kind: AltCostKind::Unearth, cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Ninjutsu (CR 702.49)
// ---------------------------------------------------------------------------

/// Handle an ActivateNinjutsu command: validate, pay cost, return attacker to
/// hand as a cost, then push ninjutsu ability onto the stack.
///
/// CR 702.49a: Ninjutsu is an activated ability from hand.
/// CR 702.49c: May only be activated when an unblocked attacker exists.
/// CR 702.49d: Commander ninjutsu also functions from the command zone.
pub fn handle_ninjutsu(
    state: &mut GameState,
    player: PlayerId,
    ninja_card: ObjectId,
    attacker_to_return: ObjectId,
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
            "a spell with split second is on the stack; ninjutsu cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 3. Combat phase + step check (CR 702.49c): must be in combat phase,
    //    at DeclareBlockers or later (not DeclareAttackers or BeginningOfCombat --
    //    before blockers are declared, creatures are neither blocked nor unblocked).
    {
        use crate::state::turn::Step;
        let step = state.turn.step;
        let valid_step = matches!(
            step,
            Step::DeclareBlockers
                | Step::FirstStrikeDamage
                | Step::CombatDamage
                | Step::EndOfCombat
        );
        if !valid_step {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateNinjutsu: ninjutsu can only be activated during DeclareBlockers, \
                 FirstStrikeDamage, CombatDamage, or EndOfCombat steps (CR 702.49c); \
                 current step is {:?}",
                step
            )));
        }
    }

    // 4. Combat state must exist (safety check).
    if state.combat.is_none() {
        return Err(GameStateError::InvalidCommand(
            "ActivateNinjutsu: no active combat state (CR 702.49c)".into(),
        ));
    }

    // 5. Zone check (CR 702.49a/d): ninja card must be in player's hand, OR,
    //    if it has CommanderNinjutsu, in the command zone ZoneId::Command(player).
    //    CRITICAL: ZoneId::Command(player), NOT ZoneId::CommandZone.
    let ninja_zone = {
        let obj = state.object(ninja_card)?;
        obj.zone
    };
    let has_commander_ninjutsu = state
        .object(ninja_card)?
        .characteristics
        .keywords
        .contains(&KeywordAbility::CommanderNinjutsu);
    let in_hand = ninja_zone == ZoneId::Hand(player);
    let in_command_zone = has_commander_ninjutsu && ninja_zone == ZoneId::Command(player);
    if !in_hand && !in_command_zone {
        return Err(GameStateError::InvalidCommand(format!(
            "ActivateNinjutsu: ninja card {:?} is not in hand or command zone (CR 702.49a/d)",
            ninja_card
        )));
    }
    let from_command_zone = in_command_zone;

    // 6. Keyword check (CR 702.49a/d): card must have Ninjutsu or CommanderNinjutsu.
    {
        let obj = state.object(ninja_card)?;
        let has_ninjutsu = obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Ninjutsu);
        if !has_ninjutsu && !has_commander_ninjutsu {
            return Err(GameStateError::InvalidCommand(format!(
                "ActivateNinjutsu: card {:?} does not have Ninjutsu or CommanderNinjutsu keyword \
                 (CR 702.49a)",
                ninja_card
            )));
        }
    }

    // 7. Attacker validation (CR 702.49c): attacker must be on battlefield,
    //    controlled by player, in combat.attackers, and unblocked.
    {
        let obj = state.object(attacker_to_return)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::InvalidCommand(
                "ActivateNinjutsu: attacker is not on the battlefield".into(),
            ));
        }
        if obj.controller != player {
            return Err(GameStateError::InvalidCommand(
                "ActivateNinjutsu: attacker is not controlled by the activating player".into(),
            ));
        }
    }
    let combat = state.combat.as_ref().ok_or_else(|| {
        GameStateError::InvalidCommand("ActivateNinjutsu: no active combat state".into())
    })?;
    if !combat.attackers.contains_key(&attacker_to_return) {
        return Err(GameStateError::InvalidCommand(
            "ActivateNinjutsu: attacker is not an attacking creature (CR 702.49c)".into(),
        ));
    }
    if combat.is_blocked(attacker_to_return) {
        return Err(GameStateError::InvalidCommand(
            "ActivateNinjutsu: attacker is blocked; ninjutsu requires an unblocked attacker \
             (CR 702.49c)"
                .into(),
        ));
    }

    // 8. Capture attack target BEFORE returning the attacker (CR 702.49c):
    //    the ninja inherits the attack target of the returned creature.
    let attack_target = state
        .combat
        .as_ref()
        .and_then(|c| c.attackers.get(&attacker_to_return).cloned())
        .ok_or_else(|| {
            GameStateError::InvalidCommand(
                "ActivateNinjutsu: could not retrieve attack target from combat state".into(),
            )
        })?;

    // 9. Cost lookup: find AbilityDefinition::Ninjutsu or ::CommanderNinjutsu.
    let card_id_opt = state.object(ninja_card)?.card_id.clone();
    let ninjutsu_cost_opt = get_ninjutsu_cost(&card_id_opt, &state.card_registry.clone());
    let ninjutsu_cost = match ninjutsu_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "ActivateNinjutsu: no ninjutsu cost found in card definition (CR 702.49a)".into(),
            ));
        }
    };

    // 10. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if ninjutsu_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &ninjutsu_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &ninjutsu_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: ninjutsu_cost.clone(),
        });
    }

    // 11. Return attacker to its OWNER's hand (cost, CR 702.49a).
    //     "Return an unblocked attacking creature you control to its owner's hand."
    //     NOT the controller's hand -- in multiplayer theft, the attacker goes
    //     to the original owner's hand.
    let attacker_owner = state.object(attacker_to_return)?.owner;
    let (new_hand_id, _old) =
        state.move_object_to_zone(attacker_to_return, ZoneId::Hand(attacker_owner))?;
    // Remove attacker from combat.attackers: move_object_to_zone doesn't touch
    // CombatState, so the old ObjectId is now stale (CR 400.7) and must be removed.
    if let Some(combat) = state.combat.as_mut() {
        combat.attackers.remove(&attacker_to_return);
    }
    events.push(GameEvent::ObjectReturnedToHand {
        player: attacker_owner,
        object_id: attacker_to_return,
        new_hand_id,
    });

    // 12. Push ninjutsu ability onto stack as NinjutsuAbility.
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::NinjutsuAbility {
            source_object: ninja_card,
            ninja_card,
            attack_target: attack_target.clone(),
            from_command_zone,
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: triggered abilities are not cleave casts.
        was_cleaved: false,
        // CR 702.42a: triggered/copy abilities are not entwine casts.
        // CR 702.120a: triggered abilities have no escalate modes paid.
        // CR 702.47a: triggered abilities have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: triggered abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: triggered/activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(stack_obj);

    // 13. Reset priority (CR 602.2e): active player gets priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: ninja_card,
        stack_object_id: stack_id,
    });
    events.push(GameEvent::PriorityGiven { player: active });

    Ok(events)
}

/// CR 702.49a: Look up the ninjutsu cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Ninjutsu { cost }` or
/// `AbilityDefinition::CommanderNinjutsu { cost }`, or `None` if the card has
/// no definition or no ninjutsu ability defined.
fn get_ninjutsu_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| match a {
                AbilityDefinition::Ninjutsu { cost }
                | AbilityDefinition::CommanderNinjutsu { cost } => Some(cost.clone()),
                _ => None,
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Embalm (CR 702.128)
// ---------------------------------------------------------------------------

/// Handle an EmbalmCard command: validate, pay cost, exile card, push embalm ability
/// onto the stack.
///
/// CR 702.128a: Embalm is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: Create a token that's a copy of
/// this card, except it's white, it has no mana cost, and it's a Zombie in addition
/// to its other types. Activate only as a sorcery."
///
/// KEY DIFFERENCE FROM UNEARTH: the card is exiled as part of the activation cost
/// (before the ability goes on the stack), not when the ability resolves.
pub fn handle_embalm_card(
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
            "a spell with split second is on the stack; embalm cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 3. Zone check (CR 702.128a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "EmbalmCard: card {:?} is not in Graveyard({:?}); embalm can only be activated from your graveyard (CR 702.128a)",
                card, player
            )));
        }
    }

    // 4. Keyword check (CR 702.128a): card must have KeywordAbility::Embalm.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Embalm)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "EmbalmCard: card {:?} does not have the Embalm keyword (CR 702.128a)",
                card
            )));
        }
    }

    // 5. Sorcery speed check (CR 702.128a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "EmbalmCard: embalm can only be activated during your own turn (CR 702.128a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "EmbalmCard: embalm can only be activated during a main phase (CR 702.128a)".into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "EmbalmCard: embalm can only be activated with an empty stack (CR 702.128a)".into(),
            ));
        }
    }

    // 6. Look up embalm cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let embalm_cost_opt = get_embalm_cost(&card_id_opt, &state.card_registry.clone());
    let embalm_cost = match embalm_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "EmbalmCard: no embalm cost found in card definition (CR 702.128a)".into(),
            ));
        }
    };

    // 7. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if embalm_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &embalm_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &embalm_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: embalm_cost.clone(),
        });
    }

    // 8. Capture the card_id BEFORE exiling (object identity is reset on zone change,
    //    CR 400.7 -- but card_id is the registry key and survives the move).
    //    We need it for EmbalmAbility so resolution can find the CardDefinition.
    let source_card_id = state.object(card)?.card_id.clone();

    // 9. Exile the card from graveyard as cost payment (CR 702.128a: "[Cost], Exile
    //    this card from your graveyard"). CRITICAL DIFFERENCE FROM UNEARTH:
    //    the card is exiled immediately as part of cost payment, not at resolution.
    //    Ruling 2017-07-14: "Once you've activated an embalm ability, the card is
    //    immediately exiled. Opponents can't try to stop the ability by exiling the
    //    card with an effect."
    let (exile_id, _old) = state.move_object_to_zone(card, ZoneId::Exile)?;
    events.push(GameEvent::ObjectExiled {
        player,
        object_id: card,
        new_exile_id: exile_id,
    });

    // 10. Push the embalm ability onto the stack as EmbalmAbility.
    //     We store source_card_id (the registry key) instead of the ObjectId
    //     because the card's ObjectId is now dead (zone change, CR 400.7).
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::EmbalmAbility { source_card_id },
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: triggered abilities are not cleave casts.
        was_cleaved: false,
        // CR 702.42a: triggered/copy abilities are not entwine casts.
        // CR 702.120a: triggered abilities have no escalate modes paid.
        // CR 702.47a: triggered abilities have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: triggered abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: triggered/activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(stack_obj);

    // 11. Reset priority (CR 602.2e): active player gets priority.
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

/// CR 702.128a: Look up the embalm cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Embalm, .. }`,
/// or `None` if the card has no definition or no embalm ability defined.
fn get_embalm_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility { kind: AltCostKind::Embalm, cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Eternalize (CR 702.129)
// ---------------------------------------------------------------------------

/// Handle an EternalizeCard command: validate, pay cost, exile card, push eternalize ability
/// onto the stack.
///
/// CR 702.129a: Eternalize is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: Create a token that's a copy of
/// this card, except it's black, it's 4/4, it has no mana cost, and it's a Zombie
/// in addition to its other types. Activate only as a sorcery."
///
/// KEY DIFFERENCE FROM UNEARTH: the card is exiled as part of the activation cost
/// (before the ability goes on the stack), not when the ability resolves.
pub fn handle_eternalize_card(
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
            "a spell with split second is on the stack; eternalize cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 3. Zone check (CR 702.129a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "EternalizeCard: card {:?} is not in Graveyard({:?}); eternalize can only be activated from your graveyard (CR 702.129a)",
                card, player
            )));
        }
    }

    // 4. Keyword check (CR 702.129a): card must have KeywordAbility::Eternalize.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Eternalize)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "EternalizeCard: card {:?} does not have the Eternalize keyword (CR 702.129a)",
                card
            )));
        }
    }

    // 5. Sorcery speed check (CR 702.129a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "EternalizeCard: eternalize can only be activated during your own turn (CR 702.129a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "EternalizeCard: eternalize can only be activated during a main phase (CR 702.129a)"
                    .into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "EternalizeCard: eternalize can only be activated with an empty stack (CR 702.129a)"
                    .into(),
            ));
        }
    }

    // 6. Look up eternalize cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let eternalize_cost_opt = get_eternalize_cost(&card_id_opt, &state.card_registry.clone());
    let eternalize_cost = match eternalize_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "EternalizeCard: no eternalize cost found in card definition (CR 702.129a)".into(),
            ));
        }
    };

    // 7. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if eternalize_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &eternalize_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &eternalize_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: eternalize_cost.clone(),
        });
    }

    // 8. Capture the card_id and name BEFORE exiling (object identity is reset on zone
    //    change, CR 400.7 -- but card_id is the registry key and survives the move).
    //    We need both for EternalizeAbility so resolution can find the CardDefinition
    //    and the TUI can display the card name.
    let source_card_id = state.object(card)?.card_id.clone();
    let source_name = state.object(card)?.characteristics.name.clone();

    // 9. Exile the card from graveyard as cost payment (CR 702.129a: "[Cost], Exile
    //    this card from your graveyard"). CRITICAL DIFFERENCE FROM UNEARTH:
    //    the card is exiled immediately as part of cost payment, not at resolution.
    //    Ruling 2017-07-14: "Once you've activated an eternalize ability, the card is
    //    immediately exiled. Opponents can't try to stop the ability by exiling the
    //    card with an effect."
    let (exile_id, _old) = state.move_object_to_zone(card, ZoneId::Exile)?;
    events.push(GameEvent::ObjectExiled {
        player,
        object_id: card,
        new_exile_id: exile_id,
    });

    // 10. Push the eternalize ability onto the stack as EternalizeAbility.
    //     We store source_card_id (the registry key) instead of the ObjectId
    //     because the card's ObjectId is now dead (zone change, CR 400.7).
    //     We also store source_name for TUI display purposes.
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::EternalizeAbility {
            source_card_id,
            source_name,
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: triggered abilities are not cleave casts.
        was_cleaved: false,
        // CR 702.42a: triggered/copy abilities are not entwine casts.
        // CR 702.120a: triggered abilities have no escalate modes paid.
        // CR 702.47a: triggered abilities have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: triggered abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: triggered/activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(stack_obj);

    // 11. Reset priority (CR 602.2e): active player gets priority.
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

/// CR 702.129a: Look up the eternalize cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Eternalize, .. }`,
/// or `None` if the card has no definition or no eternalize ability defined.
fn get_eternalize_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility { kind: AltCostKind::Eternalize, cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Encore (CR 702.141)
// ---------------------------------------------------------------------------

/// Handle an EncoreCard command: validate, pay cost, exile card, push encore ability
/// onto the stack.
///
/// CR 702.141a: Encore is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: For each opponent, create a token
/// that's a copy of this card that attacks that opponent this turn if able. The tokens
/// gain haste. Sacrifice them at the beginning of the next end step. Activate only
/// as a sorcery."
///
/// KEY DIFFERENCE FROM UNEARTH: the card is exiled as part of the activation cost
/// (before the ability goes on the stack), not when the ability resolves.
/// KEY DIFFERENCE FROM EMBALM/ETERNALIZE: tokens copy original characteristics without
/// modification (no color change, no P/T change, no type addition).
pub fn handle_encore_card(
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
            "a spell with split second is on the stack; encore cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 3. Zone check (CR 702.141a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "EncoreCard: card {:?} is not in Graveyard({:?}); encore can only be activated from your graveyard (CR 702.141a)",
                card, player
            )));
        }
    }

    // 4. Keyword check (CR 702.141a): card must have KeywordAbility::Encore.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Encore)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "EncoreCard: card {:?} does not have the Encore keyword (CR 702.141a)",
                card
            )));
        }
    }

    // 5. Sorcery speed check (CR 702.141a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "EncoreCard: encore can only be activated during your own turn (CR 702.141a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "EncoreCard: encore can only be activated during a main phase (CR 702.141a)".into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "EncoreCard: encore can only be activated with an empty stack (CR 702.141a)".into(),
            ));
        }
    }

    // 6. Look up encore cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let encore_cost_opt = get_encore_cost(&card_id_opt, &state.card_registry.clone());
    let encore_cost = match encore_cost_opt {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "EncoreCard: no encore cost found in card definition (CR 702.141a)".into(),
            ));
        }
    };

    // 7. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if encore_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &encore_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &encore_cost);
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: encore_cost.clone(),
        });
    }

    // 8. Capture the card_id BEFORE exiling (object identity is reset on zone change,
    //    CR 400.7 -- but card_id is the registry key and survives the move).
    //    We need it for EncoreAbility so resolution can find the CardDefinition.
    let source_card_id = state.object(card)?.card_id.clone();

    // 9. Exile the card from graveyard as cost payment (CR 702.141a: "[Cost], Exile
    //    this card from your graveyard"). CRITICAL DIFFERENCE FROM UNEARTH:
    //    the card is exiled immediately as part of cost payment, not at resolution.
    //    Ruling: "Once you've activated an encore ability, the card is
    //    immediately exiled. Opponents can't try to stop the ability by exiling the
    //    card with an effect."
    let (exile_id, _old) = state.move_object_to_zone(card, ZoneId::Exile)?;
    events.push(GameEvent::ObjectExiled {
        player,
        object_id: card,
        new_exile_id: exile_id,
    });

    // 10. Push the encore ability onto the stack as EncoreAbility.
    //     We store source_card_id (the registry key) instead of the ObjectId
    //     because the card's ObjectId is now dead (zone change, CR 400.7).
    //     We also store the activator to determine token targets at resolution.
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::EncoreAbility {
            source_card_id,
            activator: player,
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: triggered abilities are not cleave casts.
        was_cleaved: false,
        // CR 702.42a: triggered/copy abilities are not entwine casts.
        // CR 702.120a: triggered abilities have no escalate modes paid.
        // CR 702.47a: triggered abilities have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: triggered abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: triggered/activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(stack_obj);

    // 11. Reset priority (CR 602.2e): active player gets priority.
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

/// CR 702.141a: Look up the encore cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::AltCastAbility { kind: AltCostKind::Encore, .. }`,
/// or `None` if the card has no definition or no encore ability defined.
fn get_encore_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::AltCastAbility { kind: AltCostKind::Encore, cost, .. } = a {
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
                    if obj.cast_alt_cost == Some(crate::state::types::AltCostKind::Evoke) {
                        let evoke_trigger = PendingTrigger {
                            source: *object_id,
                            ability_index: 0, // unused for evoke sacrifice
                            controller: obj.controller,
                            kind: PendingTriggerKind::Evoke,
                            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                            entering_object_id: Some(*object_id),
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
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
                                kind: PendingTriggerKind::Exploit,
                                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                                entering_object_id: Some(*object_id),
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                madness_exiled_card: None,
                                madness_cost: None,
                                miracle_revealed_card: None,
                                miracle_cost: None,
                                modular_counter_count: None,
                                evolve_entering_creature: None,
                                suspend_card_id: None,
                                hideaway_count: None,
                                partner_with_name: None,
                                ingest_target_player: None,
                                flanking_blocker_id: None,
                                rampage_n: None,
                                provoke_target_creature: None,
                                renown_n: None,
                                poisonous_n: None,
                                poisonous_target_player: None,
                                enlist_enlisted_creature: None,
                                encore_activator: None,
                                echo_cost: None,
                                cumulative_upkeep_cost: None,
                                recover_cost: None,
                                recover_card: None,
                                graft_entering_creature: None,
                                backup_abilities: None,
                                backup_n: None,
                                champion_filter: None,
                                champion_exiled_card: None,
                                soulbond_pair_target: None,
                                squad_count: None,
                                gift_opponent: None,
                                cipher_encoded_card_id: None,
                                cipher_encoded_object_id: None,
                                haunt_source_object_id: None,
                                haunt_source_card_id: None,
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
                            kind: PendingTriggerKind::Hideaway,
                            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                            entering_object_id: Some(*object_id),
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: Some(n),
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
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
                                kind: PendingTriggerKind::PartnerWith,
                                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                                entering_object_id: Some(*object_id),
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                madness_exiled_card: None,
                                madness_cost: None,
                                miracle_revealed_card: None,
                                miracle_cost: None,
                                modular_counter_count: None,
                                evolve_entering_creature: None,
                                suspend_card_id: None,
                                hideaway_count: None,
                                partner_with_name: Some(name),
                                ingest_target_player: None,
                                flanking_blocker_id: None,
                                rampage_n: None,
                                provoke_target_creature: None,
                                renown_n: None,
                                poisonous_n: None,
                                poisonous_target_player: None,
                                enlist_enlisted_creature: None,
                                encore_activator: None,
                                echo_cost: None,
                                cumulative_upkeep_cost: None,
                                recover_cost: None,
                                recover_card: None,
                                graft_entering_creature: None,
                                backup_abilities: None,
                                backup_n: None,
                                champion_filter: None,
                                champion_exiled_card: None,
                                soulbond_pair_target: None,
                                squad_count: None,
                                gift_opponent: None,
                                cipher_encoded_card_id: None,
                                cipher_encoded_object_id: None,
                                haunt_source_object_id: None,
                                haunt_source_card_id: None,
                            });
                        }
                    }
                }

                // CR 702.165a: Backup -- "When this creature enters, put N +1/+1 counters
                // on target creature. If that's another creature, it also gains the non-backup
                // abilities of this creature printed below this one until end of turn."
                //
                // CR 702.165d: Abilities are determined at trigger time (snapshot when trigger
                // fires, not at resolution). Stored in backup_abilities on PendingTrigger.
                // CR 702.165c: Only printed abilities (from card definition), not gained ones.
                // CR 702.165a: Only abilities printed BELOW the Backup entry in the definition.
                {
                    if let Some(obj) = state.objects.get(object_id) {
                        let controller = obj.controller;
                        let card_id = obj.card_id.clone();
                        if let Some(cid) = card_id {
                            if let Some(def) = state.card_registry.get(cid) {
                                // Find all Backup(N) instances and their positions.
                                for (idx, ability) in def.abilities.iter().enumerate() {
                                    if let crate::cards::card_definition::AbilityDefinition::Keyword(
                                        KeywordAbility::Backup(n),
                                    ) = ability
                                    {
                                        // CR 702.165d: Snapshot abilities below this Backup entry.
                                        // CR 702.165a: "non-backup abilities printed below this one"
                                        // CR 702.165c: Only printed abilities.
                                        let abilities_below: Vec<KeywordAbility> = def.abilities
                                            [idx + 1..]
                                            .iter()
                                            .filter_map(|a| match a {
                                                crate::cards::card_definition::AbilityDefinition::Keyword(kw)
                                                    if !matches!(kw, KeywordAbility::Backup(_)) =>
                                                {
                                                    Some(kw.clone())
                                                }
                                                _ => None,
                                            })
                                            .collect();

                                        triggers.push(PendingTrigger {
                                            source: *object_id,
                                            ability_index: idx,
                                            controller,
                                            kind: PendingTriggerKind::Backup,
                                            triggering_event: Some(
                                                TriggerEvent::SelfEntersBattlefield,
                                            ),
                                            entering_object_id: Some(*object_id),
                                            targeting_stack_id: None,
                                            triggering_player: None,
                                            exalted_attacker_id: None,
                                            defending_player_id: None,
                                            madness_exiled_card: None,
                                            madness_cost: None,
                                            miracle_revealed_card: None,
                                            miracle_cost: None,
                                            modular_counter_count: None,
                                            evolve_entering_creature: None,
                                            suspend_card_id: None,
                                            hideaway_count: None,
                                            partner_with_name: None,
                                            ingest_target_player: None,
                                            flanking_blocker_id: None,
                                            rampage_n: None,
                                            provoke_target_creature: None,
                                            renown_n: None,
                                            poisonous_n: None,
                                            poisonous_target_player: None,
                                            enlist_enlisted_creature: None,
                                            encore_activator: None,
                                            echo_cost: None,
                                            cumulative_upkeep_cost: None,
                                            recover_cost: None,
                                            recover_card: None,
                                            graft_entering_creature: None,
                                            backup_abilities: Some(abilities_below),
                                            backup_n: Some(*n),
                                            champion_filter: None,
                                            champion_exiled_card: None,
                                            soulbond_pair_target: None,
                                            squad_count: None,
                gift_opponent: None,
                cipher_encoded_card_id: None,
                cipher_encoded_object_id: None,
                haunt_source_object_id: None,
                haunt_source_card_id: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                // CR 702.72a: Champion ETB trigger.
                // "When this permanent enters, sacrifice it unless you exile
                // another [object] you control."
                {
                    if let Some(obj) = state.objects.get(object_id) {
                        if obj
                            .characteristics
                            .keywords
                            .contains(&KeywordAbility::Champion)
                        {
                            let controller = obj.controller;
                            // Look up champion filter from card registry.
                            let filter = obj
                                .card_id
                                .as_ref()
                                .and_then(|cid| state.card_registry.get(cid.clone()))
                                .and_then(|def| {
                                    def.abilities.iter().find_map(|a| {
                                        if let crate::cards::card_definition::AbilityDefinition::Champion {
                                            filter,
                                        } = a
                                        {
                                            Some(filter.clone())
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .unwrap_or(ChampionFilter::AnyCreature);

                            triggers.push(PendingTrigger {
                                source: *object_id,
                                ability_index: 0,
                                controller,
                                kind: PendingTriggerKind::ChampionETB,
                                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                                entering_object_id: Some(*object_id),
                                targeting_stack_id: None,
                                triggering_player: None,
                                exalted_attacker_id: None,
                                defending_player_id: None,
                                madness_exiled_card: None,
                                madness_cost: None,
                                miracle_revealed_card: None,
                                miracle_cost: None,
                                modular_counter_count: None,
                                evolve_entering_creature: None,
                                suspend_card_id: None,
                                hideaway_count: None,
                                partner_with_name: None,
                                ingest_target_player: None,
                                flanking_blocker_id: None,
                                rampage_n: None,
                                provoke_target_creature: None,
                                renown_n: None,
                                poisonous_n: None,
                                poisonous_target_player: None,
                                enlist_enlisted_creature: None,
                                encore_activator: None,
                                echo_cost: None,
                                cumulative_upkeep_cost: None,
                                recover_cost: None,
                                recover_card: None,
                                graft_entering_creature: None,
                                backup_abilities: None,
                                backup_n: None,
                                champion_filter: Some(filter),
                                champion_exiled_card: None,
                                soulbond_pair_target: None,
                                squad_count: None,
                                gift_opponent: None,
                                cipher_encoded_card_id: None,
                                cipher_encoded_object_id: None,
                                haunt_source_object_id: None,
                                haunt_source_card_id: None,
                            });
                        }
                    }
                }

                // CR 702.95a: Soulbond — two ETB triggered abilities:
                //   Trigger 1 (SelfETB): When a creature with soulbond enters, if its
                //   controller controls another unpaired creature, pair them.
                //   Trigger 2 (OtherETB): When any creature enters, for each unpaired
                //   soulbond creature controlled by the same player, pair them.
                //
                // CR 603.4: Intervening-if — "you control another unpaired creature" is
                // checked at trigger time AND at resolution.
                {
                    let entering_controller = state.objects.get(object_id).map(|o| o.controller);
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
                        if let Some(controller) = entering_controller {
                            // Trigger 1 (SoulbondSelfETB): entering creature itself has Soulbond.
                            let entering_has_soulbond = {
                                let base = state
                                    .objects
                                    .get(object_id)
                                    .map(|o| {
                                        o.characteristics
                                            .keywords
                                            .contains(&KeywordAbility::Soulbond)
                                    })
                                    .unwrap_or(false);
                                let layer = crate::rules::layers::calculate_characteristics(
                                    state, *object_id,
                                )
                                .map(|c| c.keywords.contains(&KeywordAbility::Soulbond))
                                .unwrap_or(false);
                                base || layer
                            };

                            if entering_has_soulbond {
                                // Intervening-if: controller has another unpaired creature.
                                let pair_target: Option<ObjectId> = state
                                    .objects
                                    .values()
                                    .find(|obj| {
                                        obj.zone == ZoneId::Battlefield
                                            && obj.is_phased_in()
                                            && obj.controller == controller
                                            && obj.id != *object_id
                                            && obj.paired_with.is_none()
                                            && obj
                                                .characteristics
                                                .card_types
                                                .contains(&CardType::Creature)
                                    })
                                    .map(|obj| obj.id);

                                if let Some(partner_id) = pair_target {
                                    triggers.push(PendingTrigger {
                                        source: *object_id,
                                        ability_index: 0,
                                        controller,
                                        kind: PendingTriggerKind::SoulbondSelfETB,
                                        triggering_event: Some(
                                            TriggerEvent::AnyPermanentEntersBattlefield,
                                        ),
                                        entering_object_id: Some(*object_id),
                                        targeting_stack_id: None,
                                        triggering_player: None,
                                        exalted_attacker_id: None,
                                        defending_player_id: None,
                                        madness_exiled_card: None,
                                        madness_cost: None,
                                        miracle_revealed_card: None,
                                        miracle_cost: None,
                                        modular_counter_count: None,
                                        evolve_entering_creature: None,
                                        suspend_card_id: None,
                                        hideaway_count: None,
                                        partner_with_name: None,
                                        ingest_target_player: None,
                                        flanking_blocker_id: None,
                                        rampage_n: None,
                                        provoke_target_creature: None,
                                        renown_n: None,
                                        poisonous_n: None,
                                        poisonous_target_player: None,
                                        enlist_enlisted_creature: None,
                                        encore_activator: None,
                                        echo_cost: None,
                                        cumulative_upkeep_cost: None,
                                        recover_cost: None,
                                        recover_card: None,
                                        graft_entering_creature: None,
                                        backup_abilities: None,
                                        backup_n: None,
                                        champion_filter: None,
                                        champion_exiled_card: None,
                                        soulbond_pair_target: Some(partner_id),
                                        squad_count: None,
                                        gift_opponent: None,
                                        cipher_encoded_card_id: None,
                                        cipher_encoded_object_id: None,
                                        haunt_source_object_id: None,
                                        haunt_source_card_id: None,
                                    });
                                }
                            }

                            // Trigger 2 (SoulbondOtherETB): other unpaired soulbond creatures
                            // controlled by same player pair with the entering creature.
                            // The entering creature must also be unpaired (checked at trigger time).
                            let entering_is_unpaired = state
                                .objects
                                .get(object_id)
                                .map(|o| o.paired_with.is_none())
                                .unwrap_or(false);

                            if entering_is_unpaired {
                                let soulbond_sources: Vec<(ObjectId, PlayerId)> =
                                    state
                                        .objects
                                        .values()
                                        .filter(|obj| {
                                            obj.zone == ZoneId::Battlefield
                                            && obj.is_phased_in()
                                            && obj.controller == controller
                                            && obj.id != *object_id
                                            && obj.paired_with.is_none()
                                            && obj.characteristics.card_types
                                                .contains(&CardType::Creature)
                                            && (obj.characteristics.keywords
                                                .contains(&KeywordAbility::Soulbond)
                                                || crate::rules::layers::calculate_characteristics(
                                                    state, obj.id,
                                                )
                                                .map(|c| {
                                                    c.keywords.contains(&KeywordAbility::Soulbond)
                                                })
                                                .unwrap_or(false))
                                        })
                                        .map(|obj| (obj.id, obj.controller))
                                        .collect();

                                for (sb_id, sb_controller) in soulbond_sources {
                                    // Skip if sb_id has Soulbond and already fired SelfETB for this
                                    // same pair (sb_id == object_id handled by filter above).
                                    // This arm fires for OTHER soulbond creatures pairing INTO
                                    // the entering creature — only skip if entering creature itself
                                    // has soulbond (handled by Trigger 1 above).
                                    if entering_has_soulbond && sb_id == *object_id {
                                        continue;
                                    }
                                    triggers.push(PendingTrigger {
                                        source: sb_id,
                                        ability_index: 0,
                                        controller: sb_controller,
                                        kind: PendingTriggerKind::SoulbondOtherETB,
                                        triggering_event: Some(
                                            TriggerEvent::AnyPermanentEntersBattlefield,
                                        ),
                                        entering_object_id: Some(*object_id),
                                        targeting_stack_id: None,
                                        triggering_player: None,
                                        exalted_attacker_id: None,
                                        defending_player_id: None,
                                        madness_exiled_card: None,
                                        madness_cost: None,
                                        miracle_revealed_card: None,
                                        miracle_cost: None,
                                        modular_counter_count: None,
                                        evolve_entering_creature: None,
                                        suspend_card_id: None,
                                        hideaway_count: None,
                                        partner_with_name: None,
                                        ingest_target_player: None,
                                        flanking_blocker_id: None,
                                        rampage_n: None,
                                        provoke_target_creature: None,
                                        renown_n: None,
                                        poisonous_n: None,
                                        poisonous_target_player: None,
                                        enlist_enlisted_creature: None,
                                        encore_activator: None,
                                        echo_cost: None,
                                        cumulative_upkeep_cost: None,
                                        recover_cost: None,
                                        recover_card: None,
                                        graft_entering_creature: None,
                                        backup_abilities: None,
                                        backup_n: None,
                                        champion_filter: None,
                                        champion_exiled_card: None,
                                        soulbond_pair_target: Some(*object_id),
                                        squad_count: None,
                                        gift_opponent: None,
                                        cipher_encoded_card_id: None,
                                        cipher_encoded_object_id: None,
                                        haunt_source_object_id: None,
                                        haunt_source_card_id: None,
                                    });
                                }
                            }
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
                                        && obj.is_phased_in()
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
                                            kind: PendingTriggerKind::Evolve,
                                            triggering_event: Some(
                                                TriggerEvent::AnyPermanentEntersBattlefield,
                                            ),
                                            entering_object_id: Some(*object_id),
                                            targeting_stack_id: None,
                                            triggering_player: None,
                                            exalted_attacker_id: None,
                                            defending_player_id: None,
                                            madness_exiled_card: None,
                                            madness_cost: None,
                                            miracle_revealed_card: None,
                                            miracle_cost: None,
                                            modular_counter_count: None,
                                            evolve_entering_creature: Some(*object_id),
                                            suspend_card_id: None,
                                            hideaway_count: None,
                                            partner_with_name: None,
                                            ingest_target_player: None,
                                            flanking_blocker_id: None,
                                            rampage_n: None,
                                            provoke_target_creature: None,
                                            renown_n: None,
                                            poisonous_n: None,
                                            poisonous_target_player: None,
                                            enlist_enlisted_creature: None,
                                            encore_activator: None,
                                            echo_cost: None,
                                            cumulative_upkeep_cost: None,
                                            recover_cost: None,
                                            recover_card: None,
                                            graft_entering_creature: None,
                                            backup_abilities: None,
                                            backup_n: None,
                                            champion_filter: None,
                                            champion_exiled_card: None,
                                            soulbond_pair_target: None,
                                            squad_count: None,
                                            gift_opponent: None,
                                            cipher_encoded_card_id: None,
                                            cipher_encoded_object_id: None,
                                            haunt_source_object_id: None,
                                            haunt_source_card_id: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                // CR 702.58a: Graft -- "Whenever another creature enters, if this
                // permanent has a +1/+1 counter on it, you may move a +1/+1 counter
                // from this permanent onto that creature."
                //
                // CR 702.58b: Multiple instances each trigger separately.
                // Differences from Evolve:
                // - Fires for ANY player's creature entering (not just controller's)
                // - Has intervening-if: source must have a +1/+1 counter
                // - "Another creature" -- source entering does NOT trigger itself
                {
                    // Only creatures entering trigger Graft (CR 702.58a).
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
                        // Collect all battlefield permanents with Graft that:
                        // 1. Are not the entering creature itself ("another creature")
                        // 2. Have at least one +1/+1 counter (intervening-if check at trigger time, CR 603.4)
                        let graft_sources: Vec<(ObjectId, PlayerId, usize)> = state
                            .objects
                            .iter()
                            .filter(|(id, obj)| {
                                obj.zone == ZoneId::Battlefield
                                    && **id != *object_id
                                    && obj.is_phased_in()
                                    && obj
                                        .counters
                                        .get(&CounterType::PlusOnePlusOne)
                                        .copied()
                                        .unwrap_or(0)
                                        > 0
                            })
                            .filter_map(|(id, obj)| {
                                let chars =
                                    crate::rules::layers::calculate_characteristics(state, *id)
                                        .unwrap_or_else(|| obj.characteristics.clone());
                                let graft_count = chars
                                    .keywords
                                    .iter()
                                    .filter(|kw| matches!(kw, KeywordAbility::Graft(_)))
                                    .count();
                                if graft_count > 0 {
                                    Some((*id, obj.controller, graft_count))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        for (graft_id, graft_controller, graft_count) in graft_sources {
                            for _ in 0..graft_count {
                                triggers.push(PendingTrigger {
                                    source: graft_id,
                                    ability_index: 0, // unused for graft triggers
                                    controller: graft_controller,
                                    kind: PendingTriggerKind::Graft,
                                    triggering_event: Some(
                                        TriggerEvent::AnyPermanentEntersBattlefield,
                                    ),
                                    entering_object_id: Some(*object_id),
                                    targeting_stack_id: None,
                                    triggering_player: None,
                                    exalted_attacker_id: None,
                                    defending_player_id: None,
                                    madness_exiled_card: None,
                                    madness_cost: None,
                                    miracle_revealed_card: None,
                                    miracle_cost: None,
                                    modular_counter_count: None,
                                    evolve_entering_creature: None,
                                    suspend_card_id: None,
                                    hideaway_count: None,
                                    partner_with_name: None,
                                    ingest_target_player: None,
                                    flanking_blocker_id: None,
                                    rampage_n: None,
                                    provoke_target_creature: None,
                                    renown_n: None,
                                    poisonous_n: None,
                                    poisonous_target_player: None,
                                    enlist_enlisted_creature: None,
                                    encore_activator: None,
                                    echo_cost: None,
                                    cumulative_upkeep_cost: None,
                                    recover_cost: None,
                                    recover_card: None,
                                    graft_entering_creature: Some(*object_id),
                                    backup_abilities: None,
                                    backup_n: None,
                                    champion_filter: None,
                                    champion_exiled_card: None,
                                    soulbond_pair_target: None,
                                    squad_count: None,
                                    gift_opponent: None,
                                    cipher_encoded_card_id: None,
                                    cipher_encoded_object_id: None,
                                    haunt_source_object_id: None,
                                    haunt_source_card_id: None,
                                });
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
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == *player
                        })
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
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == *player
                        })
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
                        .filter(|obj| {
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller != *player
                        })
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
                    // `effect: None` and start with "Myriad". The `kind` field is set to
                    // `PendingTriggerKind::Myriad` so flush_pending_triggers creates a
                    // MyriadTrigger stack object (not a plain TriggeredAbility).
                    for t in &mut triggers[pre_len..] {
                        if let Some(obj) = state.objects.get(&t.source) {
                            if let Some(ta) =
                                obj.characteristics.triggered_abilities.get(t.ability_index)
                            {
                                if ta.effect.is_none() && ta.description.starts_with("Myriad") {
                                    t.kind = PendingTriggerKind::Myriad;
                                }
                            }
                        }
                    }

                    // CR 702.39a/b: Tag provoke triggers for special stack handling.
                    // A SelfAttacks trigger is a provoke trigger if the triggered ability
                    // description starts with "Provoke" (set by builder.rs). At collection
                    // time, select a target creature the defending player controls
                    // (deterministic: first by ObjectId order in OrdMap).
                    // CR 603.3d: If no valid target exists, provoke_target_creature is None
                    // and the trigger will not be placed on the stack in flush_pending_triggers.
                    // CR 702.39b: When a creature has multiple Provoke instances, each trigger
                    // independently selects a target. Track already-assigned targets so that
                    // successive triggers from the same attacker pick different creatures.
                    let mut provoke_targets_used: Vec<ObjectId> = Vec::new();
                    for t in &mut triggers[pre_len..] {
                        if let Some(obj) = state.objects.get(&t.source) {
                            if let Some(ta) =
                                obj.characteristics.triggered_abilities.get(t.ability_index)
                            {
                                if ta.description.starts_with("Provoke") {
                                    t.kind = PendingTriggerKind::Provoke;

                                    // Select target: first creature controlled by defending player
                                    // that has not already been claimed by a prior provoke trigger
                                    // from this attacker this combat.
                                    if let Some(dp) = defending_player {
                                        let target = state
                                            .objects
                                            .values()
                                            .filter(|o| {
                                                o.zone == ZoneId::Battlefield
                                                    && o.controller == dp
                                                    && !provoke_targets_used.contains(&o.id)
                                                    && crate::rules::layers::calculate_characteristics(
                                                        state, o.id,
                                                    )
                                                    .map(|c| {
                                                        c.card_types.contains(&CardType::Creature)
                                                    })
                                                    .unwrap_or(false)
                                            })
                                            .map(|o| o.id)
                                            .next(); // OrdMap iteration is by ObjectId order
                                        t.provoke_target_creature = target;
                                        if let Some(tid) = target {
                                            provoke_targets_used.push(tid);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // CR 702.121a/b: Tag melee triggers for special stack handling.
                    // A SelfAttacks trigger is a melee trigger if its triggered ability
                    // description starts with "Melee" (set by builder.rs). Unlike
                    // Rampage which needs an N value, Melee always gives +1/+1 per
                    // opponent attacked -- no parameter to carry.
                    for t in &mut triggers[pre_len..] {
                        if let Some(obj) = state.objects.get(&t.source) {
                            if let Some(ta) =
                                obj.characteristics.triggered_abilities.get(t.ability_index)
                            {
                                if ta.effect.is_none() && ta.description.starts_with("Melee") {
                                    t.kind = PendingTriggerKind::Melee;
                                }
                            }
                        }
                    }

                    // CR 702.154a: Enlist trigger post-processing.
                    // Each enlist pairing from combat.enlist_pairings for this attacker
                    // should match one "Enlist"-prefixed placeholder TriggeredAbilityDef.
                    // - If a pairing exists, tag the trigger with is_enlist_trigger=true
                    //   and the enlisted creature's ObjectId.
                    // - If no pairing exists for a given Enlist placeholder trigger,
                    //   REMOVE it (the player chose not to use that Enlist instance).
                    {
                        let enlist_pairings_for_attacker: Vec<ObjectId> = state
                            .combat
                            .as_ref()
                            .map(|c| {
                                c.enlist_pairings
                                    .iter()
                                    .filter(|(aid, _)| aid == attacker_id)
                                    .map(|(_, eid)| *eid)
                                    .collect()
                            })
                            .unwrap_or_default();

                        // Collect indices of Enlist placeholder triggers from this batch.
                        let mut enlist_trigger_indices: Vec<usize> = Vec::new();
                        for (i, t) in triggers[pre_len..].iter().enumerate() {
                            if let Some(obj) = state.objects.get(&t.source) {
                                if let Some(ta) =
                                    obj.characteristics.triggered_abilities.get(t.ability_index)
                                {
                                    if ta.description.starts_with("Enlist") {
                                        enlist_trigger_indices.push(pre_len + i);
                                    }
                                }
                            }
                        }

                        // Match pairings to placeholder triggers.
                        // Tag matched triggers; mark unmatched for removal.
                        let mut indices_to_remove: Vec<usize> = Vec::new();
                        let mut pairing_iter = enlist_pairings_for_attacker.iter();
                        for &idx in &enlist_trigger_indices {
                            if let Some(&enlisted_id) = pairing_iter.next() {
                                triggers[idx].kind = PendingTriggerKind::Enlist;
                                triggers[idx].enlist_enlisted_creature = Some(enlisted_id);
                            } else {
                                // No pairing for this Enlist instance -- mark for removal.
                                indices_to_remove.push(idx);
                            }
                        }

                        // Remove unmatched Enlist placeholder triggers (reverse order to
                        // preserve indices).
                        for &idx in indices_to_remove.iter().rev() {
                            triggers.remove(idx);
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

                    // CR 702.149a: Training -- "Whenever this creature and at least one
                    // other creature with power greater than this creature's power attack,
                    // put a +1/+1 counter on this creature."
                    // The condition is: among ALL attackers declared in this batch, at
                    // least one other creature has strictly greater power than this creature.
                    // CR 508.2a: condition checked at declaration time only.
                    // Ruling 2021-11-19: "triggers only when both that creature and a
                    // creature with greater power are declared as attackers."
                    {
                        // Get the power of the current attacker (layer-aware).
                        let attacker_power =
                            crate::rules::layers::calculate_characteristics(state, *attacker_id)
                                .and_then(|c| c.power)
                                .unwrap_or(0);

                        // Check if any OTHER attacker in this batch has strictly greater power.
                        let has_greater_power_ally = attackers.iter().any(|(other_id, _)| {
                            *other_id != *attacker_id && {
                                let other_power = crate::rules::layers::calculate_characteristics(
                                    state, *other_id,
                                )
                                .and_then(|c| c.power)
                                .unwrap_or(0);
                                other_power > attacker_power
                            }
                        });

                        if has_greater_power_ally {
                            let pre_len_training = triggers.len();
                            collect_triggers_for_event(
                                state,
                                &mut triggers,
                                TriggerEvent::SelfAttacksWithGreaterPowerAlly,
                                Some(*attacker_id),
                                None,
                            );
                            // Tag training triggers with defending player for consistency
                            // with other attack triggers.
                            for t in &mut triggers[pre_len_training..] {
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
                            obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj.controller == *attacking_player
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

            GameEvent::BlockersDeclared {
                blockers,
                defending_player,
            } => {
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

                // CR 702.25a: Flanking -- "Whenever this creature becomes blocked by
                // a creature without flanking, the blocking creature gets -1/-1 until
                // end of turn."
                // CR 702.25b: Multiple instances trigger separately.
                // CR 509.3f: The "without flanking" check is at declaration time.
                for (blocker_id, attacker_id) in blockers {
                    let attacker_obj = match state.objects.get(attacker_id) {
                        Some(obj) if obj.zone == ZoneId::Battlefield && obj.is_phased_in() => {
                            obj.clone()
                        }
                        _ => continue,
                    };
                    if !attacker_obj
                        .characteristics
                        .keywords
                        .contains(&KeywordAbility::Flanking)
                    {
                        continue;
                    }

                    // Check that the blocker does NOT have flanking (CR 702.25a).
                    let blocker_has_flanking = state
                        .objects
                        .get(blocker_id)
                        .map(|b| {
                            b.characteristics
                                .keywords
                                .contains(&KeywordAbility::Flanking)
                        })
                        .unwrap_or(false);
                    if blocker_has_flanking {
                        continue;
                    }

                    // Count flanking instances from card definition (CR 702.25b).
                    let flanking_count = attacker_obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter(|a| {
                                    matches!(
                                        a,
                                        AbilityDefinition::Keyword(KeywordAbility::Flanking)
                                    )
                                })
                                .count()
                        })
                        .unwrap_or(1)
                        .max(1);

                    let controller = attacker_obj.controller;
                    let source_id = attacker_obj.id;
                    for _ in 0..flanking_count {
                        triggers.push(PendingTrigger {
                            source: source_id,
                            ability_index: 0, // unused for flanking triggers
                            controller,
                            kind: PendingTriggerKind::Flanking,
                            triggering_event: Some(TriggerEvent::SelfBlocks),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: Some(*blocker_id),
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
                        });
                    }
                }

                // CR 509.1h / CR 702.45a / CR 702.23a: SelfBecomesBlocked -- fires
                // on each ATTACKER that has at least one blocker declared against it.
                // Collect unique attacker IDs to ensure each triggers only once
                // (CR 509.3c: "generally triggers only once each combat").
                let mut blocked_attackers: Vec<ObjectId> = blockers
                    .iter()
                    .map(|(_, attacker_id)| *attacker_id)
                    .collect();
                blocked_attackers.sort();
                blocked_attackers.dedup();

                for attacker_id in blocked_attackers {
                    let pre_len = triggers.len();
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfBecomesBlocked,
                        Some(attacker_id),
                        None,
                    );

                    // CR 702.23a: Tag Rampage triggers with kind=Rampage and rampage_n.
                    // Each Rampage(n) keyword on the attacker generates a TriggeredAbilityDef
                    // with description starting "Rampage N (CR 702.23a):". We detect these
                    // and set the custom StackObjectKind by tagging the PendingTrigger.
                    if let Some(obj) = state.objects.get(&attacker_id) {
                        for t in &mut triggers[pre_len..] {
                            if let Some(ability_def) =
                                obj.characteristics.triggered_abilities.get(t.ability_index)
                            {
                                if ability_def.description.starts_with("Rampage") {
                                    // Find the matching Rampage(n) keyword for this trigger.
                                    // Each Rampage instance generates its own TriggeredAbilityDef
                                    // with a unique description containing "Rampage {n}".
                                    for kw in &obj.characteristics.keywords {
                                        if let KeywordAbility::Rampage(n) = kw {
                                            if ability_def
                                                .description
                                                .contains(&format!("Rampage {n}"))
                                            {
                                                t.kind = PendingTriggerKind::Rampage;
                                                t.rampage_n = Some(*n);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // CR 509.3c / CR 702.130a: Tag all SelfBecomesBlocked triggers with
                    // the defending player so flush_pending_triggers sets Target::Player at
                    // index 0. This enables PlayerTarget::DeclaredTarget { index: 0 } in
                    // Afflict's LoseLife effect to resolve to the correct defending player
                    // in multiplayer games (CR 508.5). Bushido and Rampage target the
                    // source object rather than a player, so tagging defending_player_id
                    // has no effect on them (flush_pending_triggers only uses it for the
                    // LoseLife path via DeclaredTarget; Bushido/Rampage use Source/custom).
                    for t in &mut triggers[pre_len..] {
                        t.defending_player_id = Some(*defending_player);
                    }
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
                    if obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller != *targeting_controller
                    {
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
                object_id: pre_death_object_id,
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
                            kind: if is_modular {
                                PendingTriggerKind::Modular
                            } else {
                                PendingTriggerKind::Normal
                            },
                            triggering_event: Some(TriggerEvent::SelfDies),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
                        });
                    }
                }

                // CR 702.59a: Recover triggers. When a creature enters a player's
                // graveyard from the battlefield, each Recover card in that same
                // player's graveyard triggers independently.
                //
                // The dying creature itself CAN trigger its own Recover (if it has
                // Recover) because it is now in the graveyard when the event is
                // processed (CR 702.59a: "while the card with recover is in a player's
                // graveyard").
                //
                // Identify the owner's graveyard by looking at the new_grave_id object.
                if let Some(dead_obj) = state.objects.get(new_grave_id) {
                    let owner_gy = crate::state::zone::ZoneId::Graveyard(dead_obj.owner);
                    // Collect Recover cards in the owner's graveyard.
                    // Use a snapshot to avoid borrow conflicts during iteration.
                    let recover_cards: Vec<(ObjectId, ManaCost, PlayerId)> = state
                        .objects
                        .iter()
                        .filter_map(|(&obj_id, obj)| {
                            if obj.zone != owner_gy {
                                return None;
                            }
                            // Quick check: does this object have the Recover keyword marker?
                            if !obj
                                .characteristics
                                .keywords
                                .iter()
                                .any(|kw| *kw == KeywordAbility::Recover)
                            {
                                return None;
                            }
                            // Look up the recover cost from the card registry.
                            let cost = find_recover_cost(&obj.card_id, &state.card_registry)?;
                            Some((obj_id, cost, obj.owner))
                        })
                        .collect();

                    for (recover_id, cost, card_owner) in recover_cards {
                        triggers.push(PendingTrigger {
                            source: recover_id,
                            ability_index: 0, // unused for recover triggers
                            controller: card_owner,
                            kind: PendingTriggerKind::Recover,
                            triggering_event: Some(TriggerEvent::SelfDies),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: Some(cost),
                            recover_card: Some(recover_id),
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
                        });
                    }
                }

                // CR 702.72a: Champion LTB trigger. When a Champion permanent leaves the
                // battlefield (here: dies), check if it had a champion_exiled_card and
                // fire the LTB trigger to return that card to the battlefield.
                //
                // CR 603.10a: LTB triggers look back in time -- champion_exiled_card is
                // preserved in move_object_to_zone so we can read it from the graveyard object.
                if let Some(dead_obj) = state.objects.get(new_grave_id) {
                    if let Some(exiled_id) = dead_obj.champion_exiled_card {
                        let champion_controller = *death_controller;
                        triggers.push(PendingTrigger {
                            source: *new_grave_id,
                            ability_index: 0,
                            controller: champion_controller,
                            kind: PendingTriggerKind::ChampionLTB,
                            triggering_event: Some(TriggerEvent::SelfDies),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: Some(exiled_id),
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
                        });
                    }
                }

                // CR 702.55b: When a creature with Haunt dies, exile the dying creature
                // haunting another target creature.
                // Look back in time via new_grave_id to check if the dead creature had Haunt.
                if let Some(dead_obj) = state.objects.get(new_grave_id) {
                    if dead_obj
                        .characteristics
                        .keywords
                        .iter()
                        .any(|kw| *kw == KeywordAbility::Haunt)
                    {
                        let haunt_controller = *death_controller;
                        let haunt_card_id = dead_obj.card_id.clone();
                        triggers.push(PendingTrigger {
                            source: *new_grave_id,
                            ability_index: 0,
                            controller: haunt_controller,
                            kind: PendingTriggerKind::HauntExile,
                            triggering_event: Some(TriggerEvent::SelfDies),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: Some(*new_grave_id),
                            haunt_source_card_id: haunt_card_id,
                        });
                    }
                }

                // CR 702.55c: When the creature a haunt card haunts dies, fire the haunted
                // creature dies trigger for each haunt card in exile that targets this creature.
                // Scan exile for objects whose haunting_target matches the pre-death battlefield ID.
                {
                    let dying_id = *pre_death_object_id;
                    let haunt_exiled: Vec<(
                        ObjectId,
                        Option<crate::state::player::CardId>,
                        PlayerId,
                    )> = state
                        .objects
                        .iter()
                        .filter_map(|(&exiled_obj_id, obj)| {
                            // Must be in the exile zone.
                            if obj.zone != crate::state::zone::ZoneId::Exile {
                                return None;
                            }
                            // Must haunt the dying creature (pre-death battlefield ObjectId).
                            if obj.haunting_target != Some(dying_id) {
                                return None;
                            }
                            Some((exiled_obj_id, obj.card_id.clone(), obj.controller))
                        })
                        .collect();

                    for (haunt_obj_id, haunt_card_id, haunt_controller) in haunt_exiled {
                        triggers.push(PendingTrigger {
                            source: haunt_obj_id,
                            ability_index: 0,
                            controller: haunt_controller,
                            kind: PendingTriggerKind::HauntedCreatureDies,
                            triggering_event: Some(TriggerEvent::HauntedCreatureDies),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: Some(haunt_obj_id),
                            haunt_source_card_id: haunt_card_id,
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
                            kind: PendingTriggerKind::Normal,
                            triggering_event: Some(TriggerEvent::SelfDies),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
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
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *player
                    })
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
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *player
                    })
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

            GameEvent::Amassed { player, .. } => {
                // CR 701.47a: "Whenever you amass" triggers on all permanents
                // controlled by the amassing player. No TriggerEvent::ControllerAmasses
                // exists yet (no card currently uses this trigger condition), so this
                // arm is a no-op placeholder for forward compatibility. When a card
                // with "whenever you amass" is implemented, add a TriggerEvent variant
                // and update collect_triggers_for_event here.
                let _ = player;
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
                            kind: PendingTriggerKind::Normal,
                            triggering_event: Some(TriggerEvent::SourceConnives),
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: None,
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
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

                        // CR 702.115a: Ingest -- "Whenever this creature deals combat
                        // damage to a player, that player exiles the top card of
                        // their library."
                        // CR 702.115b: Multiple instances trigger separately.
                        if let Some(obj) = state.objects.get(&assignment.source) {
                            if obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && obj
                                    .characteristics
                                    .keywords
                                    .contains(&KeywordAbility::Ingest)
                            {
                                // Already guaranteed by the `if matches!(..., Player(_))`
                                // guard above — use `let...else` instead of unreachable!().
                                let CombatDamageTarget::Player(damaged_player) = &assignment.target
                                else {
                                    continue;
                                };
                                let damaged_player = *damaged_player;

                                // Count ingest instances from card definition for
                                // CR 702.115b: multiple instances trigger separately.
                                let ingest_count = obj
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
                                                        KeywordAbility::Ingest
                                                    )
                                                )
                                            })
                                            .count()
                                    })
                                    .unwrap_or(1)
                                    .max(1);

                                let controller = obj.controller;
                                let source_id = obj.id;
                                for _ in 0..ingest_count {
                                    triggers.push(PendingTrigger {
                                        source: source_id,
                                        ability_index: 0, // unused for ingest triggers
                                        controller,
                                        kind: PendingTriggerKind::Ingest,
                                        triggering_event: Some(
                                            TriggerEvent::SelfDealsCombatDamageToPlayer,
                                        ),
                                        entering_object_id: None,
                                        targeting_stack_id: None,
                                        triggering_player: None,
                                        exalted_attacker_id: None,
                                        defending_player_id: None,
                                        madness_exiled_card: None,
                                        madness_cost: None,
                                        miracle_revealed_card: None,
                                        miracle_cost: None,
                                        modular_counter_count: None,
                                        evolve_entering_creature: None,
                                        suspend_card_id: None,
                                        hideaway_count: None,
                                        partner_with_name: None,
                                        ingest_target_player: Some(damaged_player),
                                        flanking_blocker_id: None,
                                        rampage_n: None,
                                        provoke_target_creature: None,
                                        renown_n: None,
                                        poisonous_n: None,
                                        poisonous_target_player: None,
                                        enlist_enlisted_creature: None,
                                        encore_activator: None,
                                        echo_cost: None,
                                        cumulative_upkeep_cost: None,
                                        recover_cost: None,
                                        recover_card: None,
                                        graft_entering_creature: None,
                                        backup_abilities: None,
                                        backup_n: None,
                                        champion_filter: None,
                                        champion_exiled_card: None,
                                        soulbond_pair_target: None,
                                        squad_count: None,
                                        gift_opponent: None,
                                        cipher_encoded_card_id: None,
                                        cipher_encoded_object_id: None,
                                        haunt_source_object_id: None,
                                        haunt_source_card_id: None,
                                    });
                                }
                            }
                        }

                        // CR 702.112a: Renown N -- "When this creature deals combat
                        // damage to a player, if it isn't renowned, put N +1/+1
                        // counters on it and it becomes renowned."
                        // CR 702.112c: Multiple instances trigger separately.
                        // CR 603.4: Intervening-if -- checked here at trigger time
                        // (is_renowned must be false) and again at resolution time.
                        if let Some(obj) = state.objects.get(&assignment.source) {
                            if obj.zone == ZoneId::Battlefield
                                && obj.is_phased_in()
                                && !obj
                                    .designations
                                    .contains(crate::state::game_object::Designations::RENOWNED)
                            // CR 603.4: intervening-if at trigger time
                            {
                                // Collect Renown N values from card definition.
                                // CR 702.112c: Each keyword instance triggers separately.
                                let renown_values: Vec<u32> = obj
                                    .card_id
                                    .as_ref()
                                    .and_then(|cid| state.card_registry.get(cid.clone()))
                                    .map(|def| {
                                        def.abilities
                                            .iter()
                                            .filter_map(|a| match a {
                                                AbilityDefinition::Keyword(
                                                    KeywordAbility::Renown(n),
                                                ) => Some(*n),
                                                _ => None,
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_else(|| {
                                        // Fallback: check keywords on the object itself
                                        obj.characteristics
                                            .keywords
                                            .iter()
                                            .filter_map(|kw| match kw {
                                                KeywordAbility::Renown(n) => Some(*n),
                                                _ => None,
                                            })
                                            .collect()
                                    });

                                let controller = obj.controller;
                                let source_id = obj.id;
                                for n in renown_values {
                                    triggers.push(PendingTrigger {
                                        source: source_id,
                                        ability_index: 0, // unused for renown triggers
                                        controller,
                                        kind: PendingTriggerKind::Renown,
                                        triggering_event: Some(
                                            TriggerEvent::SelfDealsCombatDamageToPlayer,
                                        ),
                                        entering_object_id: None,
                                        targeting_stack_id: None,
                                        triggering_player: None,
                                        exalted_attacker_id: None,
                                        defending_player_id: None,
                                        madness_exiled_card: None,
                                        madness_cost: None,
                                        miracle_revealed_card: None,
                                        miracle_cost: None,
                                        modular_counter_count: None,
                                        evolve_entering_creature: None,
                                        suspend_card_id: None,
                                        hideaway_count: None,
                                        partner_with_name: None,
                                        ingest_target_player: None,
                                        flanking_blocker_id: None,
                                        rampage_n: None,
                                        provoke_target_creature: None,
                                        renown_n: Some(n),
                                        poisonous_n: None,
                                        poisonous_target_player: None,
                                        enlist_enlisted_creature: None,
                                        encore_activator: None,
                                        echo_cost: None,
                                        cumulative_upkeep_cost: None,
                                        recover_cost: None,
                                        recover_card: None,
                                        graft_entering_creature: None,
                                        backup_abilities: None,
                                        backup_n: None,
                                        champion_filter: None,
                                        champion_exiled_card: None,
                                        soulbond_pair_target: None,
                                        squad_count: None,
                                        gift_opponent: None,
                                        cipher_encoded_card_id: None,
                                        cipher_encoded_object_id: None,
                                        haunt_source_object_id: None,
                                        haunt_source_card_id: None,
                                    });
                                }
                            }
                        }

                        // CR 702.70a: Poisonous N -- "Whenever this creature deals combat
                        // damage to a player, that player gets N poison counters."
                        // CR 702.70b: Multiple instances trigger separately.
                        if let Some(obj) = state.objects.get(&assignment.source) {
                            if obj.zone == ZoneId::Battlefield && obj.is_phased_in() {
                                // Already guaranteed by the outer `if matches!(..., Player(_))`
                                // guard -- use `let...else` for safety.
                                let CombatDamageTarget::Player(damaged_player) = &assignment.target
                                else {
                                    continue;
                                };
                                let damaged_player = *damaged_player;

                                // Collect Poisonous N values from card definition.
                                // CR 702.70b: Each keyword instance triggers separately.
                                let poisonous_values: Vec<u32> = obj
                                    .card_id
                                    .as_ref()
                                    .and_then(|cid| state.card_registry.get(cid.clone()))
                                    .map(|def| {
                                        def.abilities
                                            .iter()
                                            .filter_map(|a| match a {
                                                AbilityDefinition::Keyword(
                                                    KeywordAbility::Poisonous(n),
                                                ) => Some(*n),
                                                _ => None,
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_else(|| {
                                        // Fallback: check keywords on the object itself
                                        obj.characteristics
                                            .keywords
                                            .iter()
                                            .filter_map(|kw| match kw {
                                                KeywordAbility::Poisonous(n) => Some(*n),
                                                _ => None,
                                            })
                                            .collect()
                                    });

                                let controller = obj.controller;
                                let source_id = obj.id;
                                for n in poisonous_values {
                                    triggers.push(PendingTrigger {
                                        source: source_id,
                                        ability_index: 0, // unused for poisonous triggers
                                        controller,
                                        kind: PendingTriggerKind::Poisonous,
                                        triggering_event: Some(
                                            TriggerEvent::SelfDealsCombatDamageToPlayer,
                                        ),
                                        entering_object_id: None,
                                        targeting_stack_id: None,
                                        triggering_player: None,
                                        exalted_attacker_id: None,
                                        defending_player_id: None,
                                        madness_exiled_card: None,
                                        madness_cost: None,
                                        miracle_revealed_card: None,
                                        miracle_cost: None,
                                        modular_counter_count: None,
                                        evolve_entering_creature: None,
                                        suspend_card_id: None,
                                        hideaway_count: None,
                                        partner_with_name: None,
                                        ingest_target_player: None,
                                        flanking_blocker_id: None,
                                        rampage_n: None,
                                        provoke_target_creature: None,
                                        renown_n: None,
                                        poisonous_n: Some(n),
                                        poisonous_target_player: Some(damaged_player),
                                        enlist_enlisted_creature: None,
                                        encore_activator: None,
                                        echo_cost: None,
                                        cumulative_upkeep_cost: None,
                                        recover_cost: None,
                                        recover_card: None,
                                        graft_entering_creature: None,
                                        backup_abilities: None,
                                        backup_n: None,
                                        champion_filter: None,
                                        champion_exiled_card: None,
                                        soulbond_pair_target: None,
                                        squad_count: None,
                                        gift_opponent: None,
                                        cipher_encoded_card_id: None,
                                        cipher_encoded_object_id: None,
                                        haunt_source_object_id: None,
                                        haunt_source_card_id: None,
                                    });
                                }
                            }
                        }

                        // CR 702.99b: Cipher -- "Whenever this creature deals combat damage to a
                        // player, you may copy the encoded card and cast the copy without paying
                        // its mana cost."  One trigger per encoded card per damaged player.
                        // CR 702.99c: If the encoded card left exile, the trigger still goes on
                        // the stack but does nothing at resolution (checked in resolution.rs).
                        if assignment.amount > 0 {
                            if let Some(obj) = state.objects.get(&assignment.source) {
                                if obj.zone == ZoneId::Battlefield && obj.is_phased_in() {
                                    let CombatDamageTarget::Player(_damaged_player) =
                                        &assignment.target
                                    else {
                                        // already guarded by outer matches! check
                                        continue;
                                    };
                                    if !obj.encoded_cards.is_empty() {
                                        let controller = obj.controller;
                                        let source_id = obj.id;
                                        let encoded = obj.encoded_cards.clone();
                                        for (exiled_obj_id, card_id) in encoded {
                                            triggers.push(PendingTrigger {
                                                source: source_id,
                                                ability_index: 0,
                                                controller,
                                                kind: PendingTriggerKind::CipherCombatDamage,
                                                triggering_event: Some(
                                                    TriggerEvent::SelfDealsCombatDamageToPlayer,
                                                ),
                                                entering_object_id: None,
                                                targeting_stack_id: None,
                                                triggering_player: None,
                                                exalted_attacker_id: None,
                                                defending_player_id: None,
                                                madness_exiled_card: None,
                                                madness_cost: None,
                                                miracle_revealed_card: None,
                                                miracle_cost: None,
                                                modular_counter_count: None,
                                                evolve_entering_creature: None,
                                                suspend_card_id: None,
                                                hideaway_count: None,
                                                partner_with_name: None,
                                                ingest_target_player: None,
                                                flanking_blocker_id: None,
                                                rampage_n: None,
                                                provoke_target_creature: None,
                                                renown_n: None,
                                                poisonous_n: None,
                                                poisonous_target_player: None,
                                                enlist_enlisted_creature: None,
                                                encore_activator: None,
                                                echo_cost: None,
                                                cumulative_upkeep_cost: None,
                                                recover_cost: None,
                                                recover_card: None,
                                                graft_entering_creature: None,
                                                backup_abilities: None,
                                                backup_n: None,
                                                champion_filter: None,
                                                champion_exiled_card: None,
                                                soulbond_pair_target: None,
                                                squad_count: None,
                                                gift_opponent: None,
                                                cipher_encoded_card_id: Some(card_id),
                                                cipher_encoded_object_id: Some(exiled_obj_id),
                                                haunt_source_object_id: None,
                                                haunt_source_card_id: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // CR 207.2c / CR 120.3: Enrage -- "Whenever this creature is dealt damage."
                // Collect unique creature ObjectIds that received > 0 combat damage in this
                // simultaneous damage step. Per ruling 2018-01-19, multiple simultaneous
                // sources trigger Enrage only once per creature per damage event.
                // CR 603.2g: amount == 0 (fully prevented) does not trigger.
                let mut damaged_creatures: Vec<ObjectId> = Vec::new();
                for assignment in assignments {
                    if assignment.amount == 0 {
                        continue;
                    }
                    if let CombatDamageTarget::Creature(creature_id) = &assignment.target {
                        if !damaged_creatures.contains(creature_id) {
                            damaged_creatures.push(*creature_id);
                        }
                    }
                }
                for creature_id in damaged_creatures {
                    collect_triggers_for_event(
                        state,
                        &mut triggers,
                        TriggerEvent::SelfIsDealtDamage,
                        Some(creature_id),
                        None,
                    );
                }
            }

            GameEvent::Proliferated { controller, .. } => {
                // CR 701.34: "Whenever you proliferate" triggers on all permanents
                // controlled by the proliferating player.
                let controller_sources: Vec<ObjectId> = state
                    .objects
                    .values()
                    .filter(|obj| {
                        obj.zone == ZoneId::Battlefield
                            && obj.is_phased_in()
                            && obj.controller == *controller
                    })
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

            // CR 702.72a: Champion LTB trigger -- when the champion permanent is destroyed
            // (non-creature), check champion_exiled_card on the graveyard object.
            GameEvent::PermanentDestroyed { new_grave_id, .. } => {
                if let Some(dead_obj) = state.objects.get(new_grave_id) {
                    if let Some(exiled_id) = dead_obj.champion_exiled_card {
                        let champion_controller = dead_obj.controller;
                        triggers.push(PendingTrigger {
                            source: *new_grave_id,
                            ability_index: 0,
                            controller: champion_controller,
                            kind: PendingTriggerKind::ChampionLTB,
                            triggering_event: None,
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: Some(exiled_id),
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
                        });
                    }
                }
            }

            // CR 702.72a: Champion LTB trigger -- when the champion permanent is exiled,
            // check champion_exiled_card on the exile-zone object.
            GameEvent::ObjectExiled { new_exile_id, .. } => {
                if let Some(exiled_obj) = state.objects.get(new_exile_id) {
                    // CR 607.2a / CR 603.10a: rely solely on champion_exiled_card (linked-ability
                    // tracking), not on keyword presence. The keyword may have been removed (e.g.
                    // Humility) before the permanent left the battlefield; the LTB trigger still
                    // fires because the championed-card designation is a linked-ability state, not
                    // a keyword-dependent state.
                    if let Some(exiled_card_id) = exiled_obj.champion_exiled_card {
                        let champion_controller = exiled_obj.controller;
                        triggers.push(PendingTrigger {
                            source: *new_exile_id,
                            ability_index: 0,
                            controller: champion_controller,
                            kind: PendingTriggerKind::ChampionLTB,
                            triggering_event: None,
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: Some(exiled_card_id),
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
                        });
                    }
                }
            }

            // CR 702.72a: Champion LTB trigger -- when the champion permanent bounces to hand,
            // check champion_exiled_card on the hand object.
            GameEvent::ObjectReturnedToHand { new_hand_id, .. } => {
                if let Some(hand_obj) = state.objects.get(new_hand_id) {
                    // CR 607.2a / CR 603.10a: rely solely on champion_exiled_card (linked-ability
                    // tracking), not on keyword presence. The keyword may have been removed before
                    // the permanent bounced; the LTB trigger still fires per CR 607.2a.
                    if let Some(exiled_id) = hand_obj.champion_exiled_card {
                        let champion_controller = hand_obj.controller;
                        triggers.push(PendingTrigger {
                            source: *new_hand_id,
                            ability_index: 0,
                            controller: champion_controller,
                            kind: PendingTriggerKind::ChampionLTB,
                            triggering_event: None,
                            entering_object_id: None,
                            targeting_stack_id: None,
                            triggering_player: None,
                            exalted_attacker_id: None,
                            defending_player_id: None,
                            madness_exiled_card: None,
                            madness_cost: None,
                            miracle_revealed_card: None,
                            miracle_cost: None,
                            modular_counter_count: None,
                            evolve_entering_creature: None,
                            suspend_card_id: None,
                            hideaway_count: None,
                            partner_with_name: None,
                            ingest_target_player: None,
                            flanking_blocker_id: None,
                            rampage_n: None,
                            provoke_target_creature: None,
                            renown_n: None,
                            poisonous_n: None,
                            poisonous_target_player: None,
                            enlist_enlisted_creature: None,
                            encore_activator: None,
                            echo_cost: None,
                            cumulative_upkeep_cost: None,
                            recover_cost: None,
                            recover_card: None,
                            graft_entering_creature: None,
                            backup_abilities: None,
                            backup_n: None,
                            champion_filter: None,
                            champion_exiled_card: Some(exiled_id),
                            soulbond_pair_target: None,
                            squad_count: None,
                            gift_opponent: None,
                            cipher_encoded_card_id: None,
                            cipher_encoded_object_id: None,
                            haunt_source_object_id: None,
                            haunt_source_card_id: None,
                        });
                    }
                }
            }

            // CR 207.2c / CR 120.3: Enrage -- "Whenever this creature is dealt damage."
            // Non-combat damage to a creature fires SelfIsDealtDamage on that creature.
            // CR 603.2g: amount == 0 (fully prevented) does not trigger.
            GameEvent::DamageDealt { target, amount, .. } => {
                if *amount > 0 {
                    if let CombatDamageTarget::Creature(creature_id) = target {
                        collect_triggers_for_event(
                            state,
                            &mut triggers,
                            TriggerEvent::SelfIsDealtDamage,
                            Some(*creature_id),
                            None,
                        );
                    }
                }
            }

            // CR 702.140d: "Whenever this creature mutates" — fires on the merged permanent.
            // The merged permanent is the same object (same ObjectId) as the target permanent
            // before merging. After the merge, it has ALL abilities from ALL components
            // (via the layer system). We fire SelfMutates on the merged permanent itself.
            //
            // CR 729.2c: The merged permanent is NOT new — it did not enter the battlefield.
            // No ETB triggers fire. Only SelfMutates triggers fire.
            GameEvent::CreatureMutated { object_id, .. } => {
                // collect_triggers_for_event checks zone == Battlefield, which is correct:
                // the merged permanent must still be on the battlefield to fire this trigger.
                collect_triggers_for_event(
                    state,
                    &mut triggers,
                    TriggerEvent::SelfMutates,
                    Some(*object_id),
                    None,
                );
            }

            // CR 708.8 / CR 702.37e: "When this permanent is turned face up" triggers.
            // Fire the TurnFaceUp pending trigger for any WhenTurnedFaceUp ability in the
            // permanent's CardDefinition. The permanent is now face-up; look up its card_id
            // to find the definition. ETB abilities do NOT fire (CR 708.8).
            GameEvent::PermanentTurnedFaceUp {
                player: _,
                permanent,
            } => {
                use crate::cards::card_definition::{AbilityDefinition, TriggerCondition};
                // The permanent is now face-up — its card_id is accessible.
                let card_id = state.objects.get(permanent).and_then(|o| o.card_id.clone());
                let controller_opt = state.objects.get(permanent).map(|o| o.controller);
                if let (Some(cid), Some(ctrl)) = (card_id, controller_opt) {
                    let def_opt = state.card_registry.get(cid);
                    if let Some(def) = def_opt {
                        for (idx, ability) in def.abilities.iter().enumerate() {
                            if let AbilityDefinition::Triggered {
                                trigger_condition: TriggerCondition::WhenTurnedFaceUp,
                                ..
                            } = ability
                            {
                                triggers.push(PendingTrigger {
                                    source: *permanent,
                                    ability_index: idx,
                                    controller: ctrl,
                                    kind: crate::state::stubs::PendingTriggerKind::TurnFaceUp,
                                    triggering_event: None,
                                    entering_object_id: None,
                                    targeting_stack_id: None,
                                    triggering_player: None,
                                    exalted_attacker_id: None,
                                    defending_player_id: None,
                                    madness_exiled_card: None,
                                    madness_cost: None,
                                    miracle_revealed_card: None,
                                    miracle_cost: None,
                                    modular_counter_count: None,
                                    evolve_entering_creature: None,
                                    suspend_card_id: None,
                                    hideaway_count: None,
                                    partner_with_name: None,
                                    ingest_target_player: None,
                                    flanking_blocker_id: None,
                                    rampage_n: None,
                                    provoke_target_creature: None,
                                    renown_n: None,
                                    poisonous_n: None,
                                    poisonous_target_player: None,
                                    enlist_enlisted_creature: None,
                                    encore_activator: None,
                                    echo_cost: None,
                                    cumulative_upkeep_cost: None,
                                    recover_cost: None,
                                    recover_card: None,
                                    graft_entering_creature: None,
                                    backup_abilities: None,
                                    backup_n: None,
                                    champion_filter: None,
                                    champion_exiled_card: None,
                                    soulbond_pair_target: None,
                                    squad_count: None,
                                    gift_opponent: None,
                                    cipher_encoded_card_id: None,
                                    cipher_encoded_object_id: None,
                                    haunt_source_object_id: None,
                                    haunt_source_card_id: None,
                                });
                            }
                        }
                    }
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
            .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
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

        // CR 708.3: Face-down permanents have no triggered abilities.
        // A permanent entering the battlefield face-down (via Manifest, Cloak, or Morph cast)
        // must not fire its ETB triggered abilities. The morph cast path suppresses this at
        // resolution; here we suppress it for any face-down permanent receiving
        // SelfEntersBattlefield — covering Manifest and Cloak effect paths.
        if obj.status.face_down
            && obj.face_down_as.is_some()
            && event_type == TriggerEvent::SelfEntersBattlefield
        {
            continue;
        }

        for (idx, trigger_def) in obj.characteristics.triggered_abilities.iter().enumerate() {
            if trigger_def.trigger_on != event_type {
                continue;
            }

            // CR 603.2 / CR 207.2c: Apply ETB filter for Alliance and similar
            // "whenever [another] [creature] [you control] enters" triggers.
            // All filter conditions must pass (AND logic).
            if let Some(ref etb_filter) = trigger_def.etb_filter {
                if let Some(entering_id) = entering_object {
                    // exclude_self: "another" qualifier -- skip if the entering
                    // permanent IS the trigger source.
                    if etb_filter.exclude_self && obj_id == entering_id {
                        continue;
                    }
                    if let Some(entering_obj) = state.objects.get(&entering_id) {
                        // creature_only: entering permanent must be a creature.
                        if etb_filter.creature_only
                            && !entering_obj
                                .characteristics
                                .card_types
                                .contains(&CardType::Creature)
                        {
                            continue;
                        }
                        // controller_you: entering permanent must share controller
                        // with the trigger source's controller.
                        if etb_filter.controller_you && entering_obj.controller != obj.controller {
                            continue;
                        }
                    } else {
                        // Entering object not found -- skip conservatively.
                        continue;
                    }
                }
                // If no entering_object provided but filter is set, skip --
                // ETB filters require knowing the entering object.
                else {
                    continue;
                }
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
                kind: PendingTriggerKind::Normal,
                triggering_event: Some(event_type.clone()),
                entering_object_id: entering_object,
                targeting_stack_id: None,
                triggering_player: None,
                exalted_attacker_id: None,
                defending_player_id: None,
                madness_exiled_card: None,
                madness_cost: None,
                miracle_revealed_card: None,
                miracle_cost: None,
                modular_counter_count: None,
                evolve_entering_creature: None,
                suspend_card_id: None,
                hideaway_count: None,
                partner_with_name: None,
                ingest_target_player: None,
                flanking_blocker_id: None,
                rampage_n: None,
                provoke_target_creature: None,
                renown_n: None,
                poisonous_n: None,
                poisonous_target_player: None,
                enlist_enlisted_creature: None,
                encore_activator: None,
                echo_cost: None,
                cumulative_upkeep_cost: None,
                recover_cost: None,
                recover_card: None,
                graft_entering_creature: None,
                backup_abilities: None,
                backup_n: None,
                champion_filter: None,
                champion_exiled_card: None,
                soulbond_pair_target: None,
                squad_count: None,
                gift_opponent: None,
                cipher_encoded_card_id: None,
                cipher_encoded_object_id: None,
                haunt_source_object_id: None,
                haunt_source_card_id: None,
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
        } else if trigger.kind == PendingTriggerKind::Provoke {
            // CR 702.39a: Provoke triggers target the provoked creature.
            // Set it as Target::Object so target legality can be checked at resolution.
            if let Some(provoked) = trigger.provoke_target_creature {
                vec![SpellTarget {
                    target: Target::Object(provoked),
                    zone_at_cast: Some(ZoneId::Battlefield),
                }]
            } else {
                vec![]
            }
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
            let kind = match trigger.kind {
                PendingTriggerKind::Evoke => StackObjectKind::KeywordTrigger {
                    source_object: trigger.source,
                    keyword: KeywordAbility::Evoke,
                    data: TriggerData::DelayedZoneChange,
                },
                PendingTriggerKind::Madness => StackObjectKind::MadnessTrigger {
                    source_object: trigger.source,
                    exiled_card: trigger.madness_exiled_card.unwrap_or(trigger.source),
                    madness_cost: trigger.madness_cost.clone().unwrap_or_default(),
                    owner: trigger.controller,
                },
                PendingTriggerKind::Miracle => {
                    // CR 702.94a: Miracle trigger carries the revealed card and cost.
                    StackObjectKind::MiracleTrigger {
                        source_object: trigger.source,
                        revealed_card: trigger.miracle_revealed_card.unwrap_or(trigger.source),
                        miracle_cost: trigger.miracle_cost.clone().unwrap_or_default(),
                        owner: trigger.controller,
                    }
                }
                PendingTriggerKind::Unearth => {
                    // CR 702.84a: Unearth delayed exile trigger -- "Exile [this permanent]
                    // at the beginning of the next end step."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Unearth,
                        data: TriggerData::DelayedZoneChange,
                    }
                }
                PendingTriggerKind::Exploit => {
                    // CR 702.110a: Exploit ETB trigger -- "When this creature enters,
                    // you may sacrifice a creature."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Exploit,
                        data: TriggerData::Simple,
                    }
                }
                PendingTriggerKind::Modular => {
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
                                && obj.is_phased_in()
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
                        kind: StackObjectKind::KeywordTrigger {
                            source_object: trigger.source,
                            keyword: KeywordAbility::Modular(counter_count),
                            data: TriggerData::DeathModular { counter_count },
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
                        was_overloaded: false,
                        cast_with_jump_start: false,
                        cast_with_aftermath: false,
                        was_dashed: false,
                        was_blitzed: false,
                        was_plotted: false,
                        was_prototyped: false,
                        was_impended: false,
                        was_bargained: false,
                        was_surged: false,
                        was_casualty_paid: false,
                        // CR 702.148a: storm copies are not cleave casts.
                        was_cleaved: false,
                        // CR 702.42a: storm copies are not entwine casts.
                        // CR 702.120a: storm copies have no escalate modes paid.
                        // CR 702.47a: storm copies have no spliced effects.
                        spliced_effects: vec![],
                        spliced_card_ids: vec![],
                        // CR 700.2g: storm copies inherit modes_chosen from the original.
                        // (Storm copies are handled via copy_spell_on_stack in copy.rs
                        //  which propagates modes_chosen; this site is a fallback stub.)
                        modes_chosen: vec![],
                        // CR 702.102a: storm copies are never fused spells.
                        x_value: 0,
                        // CR 701.59c: storm copies are never collect evidence casts.
                        evidence_collected: false,
                        // CR 702.174a: triggered ability stack objects are never gift casts.
                        is_cast_transformed: false,
                        additional_costs: vec![],
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
                }
                PendingTriggerKind::Evolve => {
                    // CR 702.100a: Evolve ETB trigger — "Whenever a creature you control
                    // enters, if that creature's P > this creature's P and/or that creature's
                    // T > this creature's T, put a +1/+1 counter on this creature."
                    // The resolution handler re-checks the intervening-if (CR 603.4).
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Evolve,
                        data: TriggerData::EvolveTrigger {
                            entering_creature: trigger
                                .evolve_entering_creature
                                .unwrap_or(trigger.source),
                        },
                    }
                }
                PendingTriggerKind::Myriad => {
                    // CR 702.116a: Myriad SelfAttacks trigger -- "Whenever this creature
                    // attacks, for each opponent other than defending player, create a token
                    // copy tapped and attacking that player."
                    // The `defending_player_id` was tagged by the AttackersDeclared handler
                    // in check_triggers. Fallback to active player if somehow None.
                    let defending = trigger
                        .defending_player_id
                        .unwrap_or(state.turn.active_player);
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Myriad,
                        data: TriggerData::MyriadAttack {
                            defending_player: defending,
                        },
                    }
                }
                PendingTriggerKind::SuspendCounter => {
                    // CR 702.62a: Suspend upkeep counter-removal trigger.
                    // "At the beginning of your upkeep, if this card is suspended, remove a
                    // time counter from it." This trigger goes on the stack and can be
                    // responded to (e.g., Stifle can counter it, preventing counter removal).
                    StackObjectKind::SuspendCounterTrigger {
                        source_object: trigger.source,
                        suspended_card: trigger.suspend_card_id.unwrap_or(trigger.source),
                    }
                }
                PendingTriggerKind::SuspendCast => {
                    // CR 702.62a: Suspend cast trigger (last time counter removed).
                    // "When the last time counter is removed from this card, if it's exiled,
                    // you may play it without paying its mana cost if able."
                    StackObjectKind::SuspendCastTrigger {
                        source_object: trigger.source,
                        suspended_card: trigger.suspend_card_id.unwrap_or(trigger.source),
                        owner: trigger.controller,
                    }
                }
                PendingTriggerKind::Hideaway => {
                    // CR 702.75a: Hideaway ETB trigger — "When this permanent enters,
                    // look at the top N cards of your library. Exile one of them face
                    // down and put the rest on the bottom of your library in a random order."
                    let hide_count = trigger.hideaway_count.unwrap_or(4);
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Hideaway(hide_count),
                        data: TriggerData::ETBHideaway {
                            count: hide_count,
                        },
                    }
                }
                PendingTriggerKind::PartnerWith => {
                    // CR 702.124j: Partner With ETB trigger — "When this permanent enters,
                    // target player may search their library for a card named [name], reveal
                    // it, put it into their hand, then shuffle."
                    // Target player: deterministic fallback = the trigger controller (owner).
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::PartnerWith(trigger.partner_with_name.clone().unwrap_or_default()),
                        data: TriggerData::ETBPartnerWith {
                            partner_name: trigger.partner_with_name.clone().unwrap_or_default(),
                            target_player: trigger.controller,
                        },
                    }
                }
                PendingTriggerKind::Ingest => {
                    // CR 702.115a: Ingest combat damage trigger — "Whenever this creature
                    // deals combat damage to a player, that player exiles the top card of
                    // their library."
                    // `ingest_target_player` carries the damaged player's ID.
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Ingest,
                        data: TriggerData::IngestExile {
                            target_player: trigger.ingest_target_player.unwrap_or(trigger.controller),
                        },
                    }
                }
                PendingTriggerKind::Flanking => {
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Flanking,
                        data: TriggerData::CombatFlanking {
                            blocker: trigger.flanking_blocker_id.unwrap_or(trigger.source),
                        },
                    }
                }
                PendingTriggerKind::Rampage => {
                    let n = trigger.rampage_n.unwrap_or(1);
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Rampage(n),
                        data: TriggerData::CombatRampage { n },
                    }
                }
                PendingTriggerKind::Provoke => {
                    if let Some(provoked) = trigger.provoke_target_creature {
                        StackObjectKind::KeywordTrigger {
                            source_object: trigger.source,
                            keyword: KeywordAbility::Provoke,
                            data: TriggerData::CombatProvoke { target: provoked },
                        }
                    } else {
                        continue;
                    }
                }
                PendingTriggerKind::Renown => {
                    let n = trigger.renown_n.unwrap_or(1);
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Renown(n),
                        data: TriggerData::RenownDamage { n },
                    }
                }
                PendingTriggerKind::Melee => {
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Melee,
                        data: TriggerData::Simple,
                    }
                }
                PendingTriggerKind::Poisonous => {
                    let n = trigger.poisonous_n.unwrap_or(1);
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Poisonous(n),
                        data: TriggerData::CombatPoisonous {
                            target_player: trigger
                                .poisonous_target_player
                                .unwrap_or(trigger.controller),
                            n,
                        },
                    }
                }
                PendingTriggerKind::Enlist => {
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Enlist,
                        data: TriggerData::CombatEnlist {
                            enlisted: trigger
                                .enlist_enlisted_creature
                                .unwrap_or(trigger.source),
                        },
                    }
                }
                PendingTriggerKind::EncoreSacrifice => {
                    // CR 702.141a: Encore delayed sacrifice trigger -- "Sacrifice them
                    // at the beginning of the next end step."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Encore,
                        data: TriggerData::EncoreSacrifice {
                            activator: trigger.encore_activator.unwrap_or(trigger.controller),
                        },
                    }
                }
                PendingTriggerKind::DashReturn => {
                    // CR 702.109a: Dash delayed return trigger -- "return the permanent to
                    // its owner's hand at the beginning of the next end step."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Dash,
                        data: TriggerData::DelayedZoneChange,
                    }
                }
                PendingTriggerKind::BlitzSacrifice => {
                    // CR 702.152a: Blitz delayed sacrifice trigger -- "sacrifice the
                    // permanent at the beginning of the next end step."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Blitz,
                        data: TriggerData::DelayedZoneChange,
                    }
                }
                // ImpendingCounter: migrated to KeywordTrigger
                // VanishingCounter and VanishingSacrifice: migrated to KeywordTrigger
                // FadingUpkeep: migrated to KeywordTrigger
                // EchoUpkeep: migrated to KeywordTrigger
                // CumulativeUpkeep: migrated to KeywordTrigger
                PendingTriggerKind::Recover => {
                    // CR 702.59a: Recover trigger.
                    // "When a creature is put into your graveyard from the battlefield,
                    // you may pay [cost]. If you do, return this card from your graveyard
                    // to your hand. Otherwise, exile this card."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Recover,
                        data: TriggerData::DeathRecover {
                            recover_card: trigger.recover_card.unwrap_or(trigger.source),
                            recover_cost: trigger.recover_cost.clone().unwrap_or_default(),
                        },
                    }
                }
                PendingTriggerKind::Graft => {
                    // CR 702.58a: Graft trigger.
                    // "Whenever another creature enters, if this permanent has a +1/+1
                    // counter on it, you may move a +1/+1 counter from this permanent
                    // onto that creature."
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Graft(0),
                        data: TriggerData::ETBGraft {
                            entering_creature: trigger
                                .graft_entering_creature
                                .unwrap_or(trigger.source),
                        },
                    }
                }
                PendingTriggerKind::Backup => {
                    // CR 702.165a: Backup ETB trigger.
                    // Default target = self (gets counters but no abilities per CR 702.165a).
                    // In real play the controller chooses; deterministic default = source.
                    let target = trigger.source;
                    let n = trigger.backup_n.unwrap_or(1);
                    let abilities = trigger.backup_abilities.clone().unwrap_or_default();
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Backup(n),
                        data: TriggerData::ETBBackup {
                            target,
                            count: n,
                            // Self-targeting: no abilities granted (CR 702.165a "if that's another creature").
                            abilities: if target == trigger.source {
                                vec![]
                            } else {
                                abilities
                            },
                        },
                    }
                }
                PendingTriggerKind::ChampionETB => {
                    // CR 702.72a: Champion ETB trigger.
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Champion,
                        data: TriggerData::ETBChampion {
                            filter: trigger
                                .champion_filter
                                .clone()
                                .unwrap_or(ChampionFilter::AnyCreature),
                        },
                    }
                }
                PendingTriggerKind::ChampionLTB => {
                    // CR 702.72a: Champion LTB trigger.
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Champion,
                        data: TriggerData::LTBChampion {
                            exiled_card: trigger.champion_exiled_card.unwrap_or(trigger.source),
                        },
                    }
                }
                PendingTriggerKind::SoulbondSelfETB | PendingTriggerKind::SoulbondOtherETB => {
                    // CR 702.95a: Soulbond ETB triggers (self-ETB and other-ETB).
                    // source = soulbond creature; pair_target = the creature to pair with.
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Soulbond,
                        data: TriggerData::ETBSoulbond {
                            pair_target: trigger.soulbond_pair_target.unwrap_or(trigger.source),
                        },
                    }
                }
                PendingTriggerKind::RavenousDraw => {
                    // CR 702.156a: Ravenous draw trigger. Read x_value from the GameObject
                    // (stored at ETB time per CR 107.3m). Intervening-if re-check happens
                    // at resolution.
                    let x_value = state
                        .objects
                        .get(&trigger.source)
                        .map(|o| o.x_value)
                        .unwrap_or(0);
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Ravenous,
                        data: TriggerData::ETBRavenousDraw {
                            permanent: trigger.source,
                            x_value,
                        },
                    }
                }
                PendingTriggerKind::SquadETB => {
                    // CR 702.157a: Squad ETB trigger. Read squad_count from the trigger
                    // (stored at trigger-queue time from the permanent's squad_count field).
                    // At resolution, creates squad_count token copies of the source creature.
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Squad,
                        data: TriggerData::ETBSquad {
                            count: trigger.squad_count.unwrap_or(0),
                        },
                    }
                }
                PendingTriggerKind::OffspringETB => {
                    // CR 702.175a: Offspring ETB trigger. The source_object is the creature
                    // that entered with offspring cost paid. At resolution, creates 1 token
                    // copy except it's 1/1. Uses LKI if source has left the battlefield.
                    // Capture source_card_id now (while source is on battlefield) for LKI
                    // fallback at resolution time (ruling 2024-07-26).
                    let source_card_id = state
                        .objects
                        .get(&trigger.source)
                        .and_then(|o| o.card_id.clone());
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Offspring,
                        data: TriggerData::ETBOffspring {
                            source_card_id,
                        },
                    }
                }
                PendingTriggerKind::GiftETB => {
                    // CR 702.174b: Gift ETB trigger. The source_object is the permanent
                    // that entered with gift cost paid. At resolution, gives the chosen
                    // opponent a gift defined by AbilityDefinition::Gift { gift_type }.
                    // Capture source_card_id for LKI fallback.
                    let source_card_id = state
                        .objects
                        .get(&trigger.source)
                        .and_then(|o| o.card_id.clone());
                    let gift_opponent = match trigger.gift_opponent {
                        Some(p) => p,
                        None => {
                            // No gift opponent — skip this trigger (should not happen).
                            continue;
                        }
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Gift,
                        data: TriggerData::ETBGift {
                            source_card_id,
                            gift_opponent,
                        },
                    }
                }
                PendingTriggerKind::CipherCombatDamage => {
                    // CR 702.99a: Cipher combat damage trigger -- "Whenever [encoded creature]
                    // deals combat damage to a player, you may copy the encoded card and you
                    // may cast the copy without paying its mana cost."
                    //
                    // The trigger carries the encoded card info (captured at trigger time).
                    // At resolution, verify the encoded card still exists in exile (CR 702.99c).
                    let encoded_card_id = match trigger.cipher_encoded_card_id.clone() {
                        Some(id) => id,
                        None => continue, // Missing card id — skip (should not happen).
                    };
                    let encoded_object_id = match trigger.cipher_encoded_object_id {
                        Some(id) => id,
                        None => continue, // Missing object id — skip (should not happen).
                    };
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Cipher,
                        data: TriggerData::CipherDamage {
                            source_creature: trigger.source,
                            encoded_card_id,
                            encoded_object_id,
                        },
                    }
                }
                PendingTriggerKind::HauntExile => {
                    // CR 702.55a: Haunt exile trigger -- "When this creature dies / this spell
                    // is put into a graveyard during its resolution, exile it haunting target
                    // creature." The haunt_source_object_id is the graveyard ObjectId.
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Haunt,
                        data: TriggerData::DeathHauntExile {
                            haunt_card: trigger.haunt_source_object_id.unwrap_or(trigger.source),
                            haunt_card_id: trigger.haunt_source_card_id.clone(),
                        },
                    }
                }
                PendingTriggerKind::HauntedCreatureDies => {
                    // CR 702.55c: Haunted creature dies trigger -- fires the haunt card's
                    // effect from exile when the creature it haunts dies.
                    // The haunt_source_object_id is the exiled haunt card's ObjectId.
                    StackObjectKind::KeywordTrigger {
                        source_object: trigger.source,
                        keyword: KeywordAbility::Haunt,
                        data: TriggerData::DeathHauntedCreatureDies {
                            haunt_source: trigger.haunt_source_object_id.unwrap_or(trigger.source),
                            haunt_card_id: trigger.haunt_source_card_id.clone(),
                        },
                    }
                }
                // CR 708.8 / CR 702.37e: "When this permanent is turned face up" trigger.
                // The source is the permanent itself; card_id is looked up from the object.
                PendingTriggerKind::TurnFaceUp => {
                    let source_card_id = state
                        .objects
                        .get(&trigger.source)
                        .and_then(|o| o.card_id.clone());
                    StackObjectKind::TurnFaceUpTrigger {
                        permanent: trigger.source,
                        source_card_id,
                        ability_index: trigger.ability_index,
                    }
                }
                PendingTriggerKind::Normal => StackObjectKind::TriggeredAbility {
                    source_object: trigger.source,
                    ability_index: trigger.ability_index,
                },
                PendingTriggerKind::KeywordTrigger {
                    ref keyword,
                    ref data,
                } => StackObjectKind::KeywordTrigger {
                    source_object: trigger.source,
                    keyword: keyword.clone(),
                    data: data.clone(),
                },
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
                was_overloaded: false,
                cast_with_jump_start: false,
                cast_with_aftermath: false,
                was_dashed: false,
                was_blitzed: false,
                was_plotted: false,
                was_prototyped: false,
                was_impended: false,
                was_bargained: false,
                was_surged: false,
                was_casualty_paid: false,
                // CR 702.148a: myriad copies are not cleave casts.
                was_cleaved: false,
                // CR 702.42a: myriad copies are not entwine casts.
                // CR 702.120a: myriad copies have no escalate modes paid.
                // CR 702.47a: myriad copies have no spliced effects.
                spliced_effects: vec![],
                spliced_card_ids: vec![],
                // CR 700.2a: myriad attack copies are not modal spells; no modes chosen.
                modes_chosen: vec![],
                // CR 702.102a: myriad attack copies are never fused spells.
                x_value: 0,
                // CR 701.59c: myriad attack copies are never collect evidence casts.
                evidence_collected: false,
                // CR 702.157a: triggered ability stack objects have no squad cost payments.
                // CR 702.174a: triggered ability stack objects are never gift casts.
                is_cast_transformed: false,
                additional_costs: vec![],
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
    // MR-M3-11: active player must always be in turn_order; assert in debug builds.
    let start_pos = order.iter().position(|&p| p == active);
    debug_assert!(
        start_pos.is_some(),
        "apnap_order: active player {:?} not found in turn_order {:?}",
        active,
        order
    );
    let start = start_pos.unwrap_or(0);
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: triggered abilities are not cleave casts.
        was_cleaved: false,
        // CR 702.42a: triggered/copy abilities are not entwine casts.
        // CR 702.120a: triggered abilities have no escalate modes paid.
        // CR 702.47a: triggered abilities have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: triggered abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: triggered abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: triggered/activated abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
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

/// CR 702.171a: Handle the `SaddleMount` command.
///
/// Validates that:
/// - The player holds priority (CR 602.2).
/// - No split-second spell is on the stack (CR 702.61a).
/// - Sorcery-speed restriction (CR 702.171a): active player's turn, main phase, empty stack.
/// - The Mount is on the battlefield and controlled by the player.
/// - The Mount has `KeywordAbility::Saddle(n)` in layer-resolved characteristics.
/// - Each saddling creature is an untapped creature controlled by the player (not the Mount).
/// - Total power of saddling creatures >= N.
/// - No duplicate creature IDs.
///
/// On success: taps all saddling creatures, pushes `StackObjectKind::SaddleAbility` onto
/// the stack, and grants priority to the active player.
///
/// Key differences from `handle_crew_vehicle`:
/// - Sorcery-speed only (CR 702.171a): active player, main phase, empty stack.
/// - No layer-4 type change: Mount is already a creature. Sets `is_saddled` flag instead.
/// - Ruling 2024-04-12: activating saddle on an already-saddled Mount is legal.
pub fn handle_saddle_mount(
    state: &mut GameState,
    player: PlayerId,
    mount: ObjectId,
    saddle_creatures: Vec<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use std::collections::HashSet;

    use crate::rules::layers::calculate_characteristics;
    use crate::state::types::CardType;

    // CR 602.2: Saddling requires priority.
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
            "a spell with split second is on the stack; saddle ability cannot be activated (CR 702.61a)".into(),
        ));
    }

    // CR 702.171a: "Activate only as a sorcery." Enforce sorcery-speed:
    // - Must be the active player's turn.
    // - Must be a main phase (PreCombatMain or PostCombatMain).
    // - Stack must be empty.
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(
            "saddle ability can only be activated during your own turn (CR 702.171a: 'activate only as a sorcery')".into(),
        ));
    }
    let in_main_phase = matches!(
        state.turn.step,
        crate::state::turn::Step::PreCombatMain | crate::state::turn::Step::PostCombatMain
    );
    if !in_main_phase {
        return Err(GameStateError::InvalidCommand(
            "saddle ability can only be activated during a main phase (CR 702.171a: 'activate only as a sorcery')".into(),
        ));
    }
    if !state.stack_objects.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "saddle ability can only be activated when the stack is empty (CR 702.171a: 'activate only as a sorcery')".into(),
        ));
    }

    // Validate the Mount: must be on the battlefield, controlled by the player,
    // and must have KeywordAbility::Saddle(n). Use calculate_characteristics for
    // layer correctness (e.g., Humility may have removed the keyword).
    let saddle_cost_n: u32 = {
        let obj = state.object(mount)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(mount));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: mount,
            });
        }
        // Use layer-computed characteristics to account for continuous effects.
        let chars = calculate_characteristics(state, mount)
            .or_else(|| state.objects.get(&mount).map(|o| o.characteristics.clone()));
        let saddle_n = chars.as_ref().and_then(|c| {
            c.keywords.iter().find_map(|kw| {
                if let KeywordAbility::Saddle(n) = kw {
                    Some(*n)
                } else {
                    None
                }
            })
        });
        saddle_n.ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "object {:?} does not have the Saddle keyword (CR 702.171a)",
                mount
            ))
        })?
    };

    // Validate saddle_creatures is non-empty (you must tap at least one creature).
    if saddle_creatures.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "must provide at least one creature to saddle the mount (CR 702.171a)".into(),
        ));
    }

    // CR 702.171a: Validate uniqueness — no duplicates in saddle_creatures.
    let mut seen: HashSet<ObjectId> = HashSet::new();
    for &id in &saddle_creatures {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate creature {:?} in saddle_creatures (CR 702.171a)",
                id
            )));
        }
    }

    // CR 702.171a: Validate each saddling creature — must be an untapped creature
    // you control on the battlefield, and must not be the mount itself.
    // Also sum total power for the saddle cost threshold check.
    // Note: summoning sickness does NOT prevent saddling (same ruling as Crew);
    // tapping for saddle cost is not a {T} activated ability.
    let mut total_power: i32 = 0;
    for &id in &saddle_creatures {
        // CR 702.171a: "other" — mount cannot saddle itself.
        if id == mount {
            return Err(GameStateError::InvalidCommand(
                "a mount cannot be used to saddle itself (CR 702.171a: 'other untapped creatures')"
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

        // Must be untapped (CR 702.171a: "untapped creatures").
        if obj.status.tapped {
            return Err(GameStateError::InvalidCommand(format!(
                "creature {:?} is already tapped and cannot be used to saddle (CR 702.171a)",
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
                "object {:?} is not a creature and cannot be used to saddle (CR 702.171a)",
                id
            )));
        }

        // Accumulate power for the total power check.
        let power = chars.and_then(|c| c.power).unwrap_or(0);
        total_power = total_power.saturating_add(power);
    }

    // CR 702.171a: Total power of tapped creatures must be >= N.
    if total_power < saddle_cost_n as i32 {
        return Err(GameStateError::InvalidCommand(format!(
            "total power of saddle creatures ({}) is less than Saddle {} cost (CR 702.171a)",
            total_power, saddle_cost_n
        )));
    }

    // Pay the cost: tap all saddling creatures (CR 602.2b analog for saddle cost).
    let mut events = Vec::new();
    for &id in &saddle_creatures {
        if let Some(obj) = state.objects.get_mut(&id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: id,
        });
    }

    // Push the saddle ability onto the stack.
    // When resolved, `SaddleAbility` sets `is_saddled = true` on the Mount (resolution.rs).
    let stack_id = state.next_object_id();

    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::SaddleAbility {
            source_object: mount,
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(stack_obj);

    // CR 602.2e / CR 116.3b: After activating, the active player receives priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    events.push(GameEvent::AbilityActivated {
        player,
        source_object_id: mount,
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

// ---------------------------------------------------------------------------
// Scavenge (CR 702.97)
// ---------------------------------------------------------------------------

/// Handle a ScavengeCard command: validate, pay cost, snapshot power, exile card,
/// push scavenge ability onto the stack targeting the specified creature.
///
/// CR 702.97a: Scavenge is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: Put a number of +1/+1 counters
/// equal to the power of the card you exiled on target creature. Activate only
/// as a sorcery."
///
/// KEY RULE: Power is snapshotted BEFORE exile (Varolz ruling 2013-04-15 -- "the
/// number of counters that a card's scavenge ability puts on a creature is based on
/// the card's power as it last existed in the graveyard").
pub fn handle_scavenge_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    target_creature: ObjectId,
) -> Result<Vec<crate::rules::events::GameEvent>, GameStateError> {
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
            "a spell with split second is on the stack; scavenge cannot be activated (CR 702.61a)"
                .into(),
        ));
    }

    // 3. Zone check (CR 702.97a): card must be in player's own graveyard.
    {
        let obj = state.object(card)?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "ScavengeCard: card {:?} is not in Graveyard({:?}); scavenge can only be activated from your graveyard (CR 702.97a)",
                card, player
            )));
        }
    }

    // 4. Keyword check (CR 702.97a): card must have KeywordAbility::Scavenge.
    {
        let obj = state.object(card)?;
        if !obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Scavenge)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "ScavengeCard: card {:?} does not have the Scavenge keyword (CR 702.97a)",
                card
            )));
        }
    }

    // 5. Sorcery speed check (CR 702.97a: "activate only as a sorcery").
    //    Active player only, main phase only (PreCombatMain or PostCombatMain), empty stack.
    {
        use crate::state::turn::Step;
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: scavenge can only be activated during your own turn (CR 702.97a)"
                    .into(),
            ));
        }
        let step = state.turn.step;
        if step != Step::PreCombatMain && step != Step::PostCombatMain {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: scavenge can only be activated during a main phase (CR 702.97a)"
                    .into(),
            ));
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: scavenge can only be activated with an empty stack (CR 702.97a)"
                    .into(),
            ));
        }
    }

    // 6. Target validation: target_creature must be a creature on the battlefield.
    {
        let target_on_battlefield = state
            .objects
            .get(&target_creature)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        if !target_on_battlefield {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: target_creature is not on the battlefield (CR 702.97a)".into(),
            ));
        }
        let target_is_creature =
            crate::rules::layers::calculate_characteristics(state, target_creature)
                .map(|c| c.card_types.contains(&CardType::Creature))
                .unwrap_or(false);
        if !target_is_creature {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: target_creature is not a creature (CR 702.97a)".into(),
            ));
        }
    }

    // 7. Look up scavenge cost from CardRegistry.
    let card_id_opt = state.object(card)?.card_id.clone();
    let scavenge_cost = match get_scavenge_cost(&card_id_opt, &state.card_registry.clone()) {
        Some(cost) => cost,
        None => {
            return Err(GameStateError::InvalidCommand(
                "ScavengeCard: no scavenge cost found in card definition (CR 702.97a)".into(),
            ));
        }
    };

    // 8. Pay mana cost (CR 602.2b).
    let mut events = Vec::new();
    if scavenge_cost.mana_value() > 0 {
        let player_state = state.player_mut(player)?;
        if !casting::can_pay_cost(&player_state.mana_pool, &scavenge_cost) {
            return Err(GameStateError::InsufficientMana);
        }
        casting::pay_cost(&mut player_state.mana_pool, &scavenge_cost);
        events.push(crate::rules::events::GameEvent::ManaCostPaid {
            player,
            cost: scavenge_cost.clone(),
        });
    }

    // 9. Snapshot power BEFORE exile (Varolz ruling 2013-04-15: "the number of counters
    //    is based on the card's power as it last existed in the graveyard").
    //    Use layer-resolved characteristics to capture any in-graveyard modifiers.
    let power_snapshot: u32 = crate::rules::layers::calculate_characteristics(state, card)
        .and_then(|c| c.power)
        .map(|p| p.max(0) as u32)
        .unwrap_or(0);

    // Capture source_card_id BEFORE exiling (registry key survives zone change, CR 400.7).
    let source_card_id = state.object(card)?.card_id.clone();

    // 10. Exile the card from graveyard as cost payment (CR 702.97a: "[Cost], Exile this
    //     card from your graveyard"). The card is exiled immediately at activation time.
    //     Ruling 2013-04-15: "Once the ability is activated and the cost is paid, it's too
    //     late to stop the ability by trying to remove the card from the graveyard."
    let (exile_id, _old) = state.move_object_to_zone(card, ZoneId::Exile)?;
    events.push(crate::rules::events::GameEvent::ObjectExiled {
        player,
        object_id: card,
        new_exile_id: exile_id,
    });

    // 11. Push the ScavengeAbility onto the stack with the target creature.
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::ScavengeAbility {
            source_card_id,
            power_snapshot,
        },
        targets: vec![SpellTarget {
            target: Target::Object(target_creature),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
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
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        // CR 700.2a: scavenge abilities are not modal spells; no modes chosen.
        modes_chosen: vec![],
        // CR 702.102a: scavenge abilities are never fused spells.
        x_value: 0,
        // CR 701.59c: scavenge abilities are never collect evidence casts.
        evidence_collected: false,
        // CR 702.174a: triggered ability stack objects are never gift casts.
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(stack_obj);

    // 12. Reset priority (CR 602.2e): active player gets priority.
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);

    events.push(crate::rules::events::GameEvent::AbilityActivated {
        player,
        source_object_id: card,
        stack_object_id: stack_id,
    });
    events.push(crate::rules::events::GameEvent::PriorityGiven { player: active });

    Ok(events)
}

/// CR 702.97a: Look up the scavenge cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Scavenge { cost }`, or `None`
/// if the card has no definition or no scavenge ability defined.
fn get_scavenge_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| match a {
                AbilityDefinition::Scavenge { cost } => Some(cost.clone()),
                _ => None,
            })
        })
    })
}
