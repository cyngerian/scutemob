//! Casting spells (CR 601).
//!
//! A spell is cast by moving a card from the caster's hand to the Stack zone
//! and placing a `StackObject` onto `GameState::stack_objects`. After casting,
//! the active player receives priority (CR 601.2i).
//!
//! Casting speed (CR 601.3):
//! - **Instant speed**: Instants, and any spell with Flash (CR 702.36), may be
//!   cast whenever the player has priority.
//! - **Sorcery speed**: All other spells may only be cast during the active
//!   player's precombat or postcombat main phase while the stack is empty
//!   (CR 307.1).
//!
//! **Targets (CR 601.2c)**: Targets are announced at cast time and validated
//! for existence. The zone of each object target is recorded for the fizzle
//! rule checked at resolution (CR 608.2b).
//!
//! **Mana cost (CR 601.2f-h)**: If the spell has a mana cost, the caster's
//! mana pool must cover it. The cost is deducted from the pool when the spell
//! is cast. Spells with no mana cost (e.g., `mana_cost: None`) are cast for free.

use im::OrdSet;

use crate::cards::card_definition::{AbilityDefinition, TargetController, TargetRequirement};
use crate::rules::commander::apply_commander_tax;
use crate::rules::layers::calculate_characteristics;
use crate::state::error::GameStateError;
use crate::state::game_object::{Characteristics, ManaCost, ObjectId};
use crate::state::player::PlayerId;
use crate::state::stack::{StackObject, StackObjectKind};
use crate::state::targeting::{SpellTarget, Target};
use crate::state::turn::Step;
use crate::state::types::{CardType, KeywordAbility, SubType};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::GameEvent;

/// Handle a CastSpell command: move a card from hand to the stack.
///
/// Validates the casting window, validates targets, pays the mana cost, moves
/// the card to `ZoneId::Stack`, creates a `StackObject`, resets priority to
/// the active player (CR 601.2i), and returns the events produced.
pub fn handle_cast_spell(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // CR 601.2: Casting a spell requires priority.
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }

    // Fetch the card and validate it is in the player's hand, command zone, or graveyard
    // (with flashback). CR 903.8: A player may cast their commander from the command zone.
    // CR 702.34a: A card with flashback may be cast from its owner's graveyard.
    let (casting_from_command_zone, casting_with_flashback, card_id, base_mana_cost) = {
        let card_obj = state.object(card)?;
        let casting_from_command_zone = card_obj.zone == ZoneId::Command(player);
        let casting_from_graveyard = card_obj.zone == ZoneId::Graveyard(player);

        // CR 702.34a: Flashback — allowed if card has the flashback keyword and is in graveyard.
        let casting_with_flashback = casting_from_graveyard
            && card_obj
                .characteristics
                .keywords
                .contains(&KeywordAbility::Flashback);

        if card_obj.zone != ZoneId::Hand(player)
            && !casting_from_command_zone
            && !casting_with_flashback
        {
            return Err(GameStateError::InvalidCommand(
                "card is not in your hand".into(),
            ));
        }
        (
            casting_from_command_zone,
            casting_with_flashback,
            card_obj.card_id.clone(),
            card_obj.characteristics.mana_cost.clone(),
        )
    };

    // CR 903.8: Only a player's own commander may be cast from the command zone.
    if casting_from_command_zone {
        let player_state = state.player(player)?;
        let is_commander = card_id
            .as_ref()
            .map(|cid| player_state.commander_ids.contains(cid))
            .unwrap_or(false);
        if !is_commander {
            return Err(GameStateError::InvalidCommand(
                "only your own commander may be cast from the command zone".into(),
            ));
        }
    }

    // Use calculate_characteristics for type/keyword checks to respect continuous effects
    // (CR 613). Falls back to raw characteristics if the object is not found (command zone
    // objects may not participate in layer calculations).
    let chars = calculate_characteristics(state, card).unwrap_or_else(|| {
        state
            .object(card)
            .map(|o| o.characteristics.clone())
            .unwrap_or_default()
    });

    // Lands are not cast — they are played as a special action (CR 305.1).
    if chars.card_types.contains(&CardType::Land) {
        return Err(GameStateError::InvalidCommand(
            "lands are played with PlayLand, not cast".into(),
        ));
    }

    // Determine casting speed (CR 601.3).
    let is_instant_speed = chars.card_types.contains(&CardType::Instant)
        || chars.keywords.contains(&KeywordAbility::Flash);

    // CR 702.34a: Flashback — type validation: only instants and sorceries can use flashback.
    // CR 702.34a: "You may cast this card from your graveyard if the resulting spell is an
    // instant or sorcery spell."
    if casting_with_flashback {
        let is_instant_or_sorcery = chars.card_types.contains(&CardType::Instant)
            || chars.card_types.contains(&CardType::Sorcery);
        if !is_instant_or_sorcery {
            return Err(GameStateError::InvalidCommand(
                "flashback can only be used on instants and sorceries".into(),
            ));
        }
    }

    // CR 702.34a / CR 903.8: Determine the cost to pay.
    // - Flashback: pay the flashback cost (alternative cost, CR 118.9) instead of mana cost.
    // - Command zone: pay mana cost + commander tax (additional cost, CR 903.8).
    // - Otherwise: pay the printed mana cost.
    // CR 118.9d: additional costs apply on top of the alternative cost — commander tax
    // applies on top of flashback if applicable (rare, but handled correctly here).
    let mana_cost: Option<ManaCost> = if casting_with_flashback {
        // CR 702.34a: Pay flashback cost instead of mana cost.
        // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
        get_flashback_cost(&card_id, &state.card_registry)
    } else if casting_from_command_zone {
        // CR 903.8: Apply commander tax (additional cost on top of mana cost).
        let tax = {
            let player_state = state.player(player)?;
            card_id
                .as_ref()
                .and_then(|cid| player_state.commander_tax.get(cid).copied())
                .unwrap_or(0)
        };
        base_mana_cost.map(|cost| apply_commander_tax(&cost, tax))
    } else {
        base_mana_cost
    };

    // Validate casting window.
    if !is_instant_speed {
        // Sorcery speed: active player only, main phase, empty stack (CR 307.1).
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "sorcery-speed spells can only be cast during your own turn".into(),
            ));
        }
        if !matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain) {
            return Err(GameStateError::NotMainPhase);
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::StackNotEmpty);
        }
    }

    // Look up target requirements and cant_be_countered from the card definition (CR 601.2c).
    let (requirements, cant_be_countered): (Vec<TargetRequirement>, bool) = {
        let registry = state.card_registry.clone();
        card_id
            .clone()
            .and_then(|cid| registry.get(cid))
            .and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Spell {
                        targets,
                        cant_be_countered,
                        ..
                    } = a
                    {
                        Some((targets.clone(), *cant_be_countered))
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default()
    };

    // CR 601.2c: Validate and record targets at cast time.
    // Pass source characteristics for protection-from checks (CR 702.16b).
    let spell_targets = validate_targets(state, &targets, &requirements, player, Some(&chars))?;

    // CR 702.5a / 303.4a: Aura spells require exactly one target matching the Enchant restriction.
    // The Enchant keyword defines the target restriction — it is derived from the card's
    // keywords rather than from an explicit TargetRequirement (which applies to instants/sorceries).
    if chars.subtypes.contains(&SubType("Aura".to_string()))
        && chars.card_types.contains(&CardType::Enchantment)
    {
        if let Some(enchant_target) = super::sba::get_enchant_target(&chars.keywords) {
            // CR 303.4a: Aura spell must target exactly one legal object.
            if spell_targets.is_empty() {
                return Err(GameStateError::InvalidCommand(
                    "Aura spells require exactly one target (CR 303.4a)".into(),
                ));
            }
            for st in &spell_targets {
                if let Target::Object(target_id) = st.target {
                    // CR 303.4a / 115.4: The target must be on the battlefield.
                    // Auras can only enchant permanents (or players for Enchant Player).
                    // A creature in the graveyard or hand is not a legal Aura target.
                    let is_on_battlefield = state
                        .objects
                        .get(&target_id)
                        .map(|o| o.zone == ZoneId::Battlefield)
                        .unwrap_or(false);
                    if !is_on_battlefield {
                        return Err(GameStateError::InvalidTarget(
                            "Aura target must be on the battlefield (CR 303.4a)".into(),
                        ));
                    }
                    let target_chars = calculate_characteristics(state, target_id).or_else(|| {
                        state
                            .objects
                            .get(&target_id)
                            .map(|o| o.characteristics.clone())
                    });
                    if let Some(tc) = target_chars {
                        if !super::sba::matches_enchant_target(&enchant_target, &tc) {
                            return Err(GameStateError::InvalidTarget(format!(
                                "target does not match Enchant restriction ({enchant_target:?})"
                            )));
                        }
                    }
                }
            }
        }
    }

    // CR 601.2f-h: Pay the mana cost if the card has one.
    let mut events = Vec::new();

    if let Some(ref cost) = mana_cost {
        if cost.mana_value() > 0 {
            // Check the player has enough mana.
            let player_state = state.player_mut(player)?;
            if !can_pay_cost(&player_state.mana_pool, cost) {
                return Err(GameStateError::InsufficientMana);
            }
            pay_cost(&mut player_state.mana_pool, cost);
        }
        // CR 601.2f: ManaCostPaid is emitted for all costs, including {0}.
        events.push(GameEvent::ManaCostPaid {
            player,
            cost: cost.clone(),
        });
    }

    // CR 601.2c: Move the card to the Stack zone (CR 400.7: new ObjectId).
    let (new_card_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Stack)?;

    // CR 601.2: Create the StackObject and push it (LIFO — last in, first out).
    let stack_entry_id = state.next_object_id();

    // CR 702.21a: Collect battlefield object targets before moving spell_targets into
    // the stack object. These are used to emit PermanentTargeted events for Ward.
    let battlefield_targets: Vec<ObjectId> = spell_targets
        .iter()
        .filter_map(|st| {
            if let Target::Object(id) = st.target {
                // Only objects that were on the battlefield at cast time trigger ward.
                if matches!(st.zone_at_cast, Some(ZoneId::Battlefield)) {
                    return Some(id);
                }
            }
            None
        })
        .collect();

    let stack_obj = StackObject {
        id: stack_entry_id,
        controller: player,
        kind: StackObjectKind::Spell {
            source_object: new_card_id,
        },
        targets: spell_targets,
        cant_be_countered,
        is_copy: false,
        cast_with_flashback: casting_with_flashback,
    };
    state.stack_objects.push_back(stack_obj);

    // CR 903.8: Increment commander tax counter if cast from command zone.
    let commander_tax_paid = if casting_from_command_zone {
        if let Some(ref cid) = card_id {
            let player_state = state.player_mut(player)?;
            let count = player_state.commander_tax.entry(cid.clone()).or_insert(0);
            let tax = *count;
            *count += 1;
            tax
        } else {
            0
        }
    } else {
        0
    };

    // CR 601.2i: "Then the active player receives priority."
    // Reset the priority round — a game action occurred.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(state.turn.active_player);

    events.push(GameEvent::SpellCast {
        player,
        stack_object_id: stack_entry_id,
        source_object_id: new_card_id,
    });

    // CR 702.21a: Emit PermanentTargeted for each battlefield permanent that this
    // spell targets. These events drive Ward trigger checks in check_triggers.
    // `targeting_stack_id` is the stack entry's own ObjectId so the ward CounterSpell
    // effect can locate it via direct stack ID match (so.id == id).
    for target_id in battlefield_targets {
        events.push(GameEvent::PermanentTargeted {
            target_id,
            targeting_stack_id: stack_entry_id,
            targeting_controller: player,
        });
    }

    // CR 702.40a: Track spells cast this turn for storm count.
    // Increment after the spell enters the stack (it is now a spell cast this turn).
    // NOTE: The increment happens here, before the storm trigger is queued, so that
    // `storm_count()` (which uses `spells_cast_this_turn - 1`) yields the correct
    // count of OTHER spells cast before this one. If this ordering changes, storm
    // count would be wrong.
    if let Some(ps) = state.players.get_mut(&player) {
        ps.spells_cast_this_turn += 1;
    }

    // CR 702.40a: Storm — "When you cast this spell, copy it for each other spell
    // cast before it this turn." Storm is a triggered ability (CR 702.40a). It goes
    // on the stack above the original spell and resolves through normal priority.
    // The storm count is captured now (at trigger time) because spells_cast_this_turn
    // could change before the trigger resolves (e.g., cascade casting another spell).
    if chars.keywords.contains(&KeywordAbility::Storm) {
        let count = crate::rules::copy::storm_count(state, player);
        let trigger_id = state.next_object_id();
        let trigger_obj = StackObject {
            id: trigger_id,
            controller: player,
            kind: StackObjectKind::StormTrigger {
                source_object: new_card_id,
                original_stack_id: stack_entry_id,
                storm_count: count,
            },
            targets: vec![],
            cant_be_countered: false,
            is_copy: false,
            cast_with_flashback: false,
        };
        state.stack_objects.push_back(trigger_obj);
        events.push(GameEvent::AbilityTriggered {
            controller: player,
            source_object_id: new_card_id,
            stack_object_id: trigger_id,
        });
    }

    // CR 702.85a: Cascade — "When you cast this spell, exile cards from the top of
    // your library until you exile a nonland card whose mana value is less than this
    // spell's mana value. You may cast that card without paying its mana cost."
    // Cascade is a triggered ability (CR 702.85a). It goes on the stack above the
    // original spell and resolves through normal priority. The mana value is captured
    // now (at trigger time) per CR 702.85a.
    if chars.keywords.contains(&KeywordAbility::Cascade) {
        let spell_mv = chars
            .mana_cost
            .as_ref()
            .map(|mc| mc.mana_value())
            .unwrap_or(0);
        let trigger_id = state.next_object_id();
        let trigger_obj = StackObject {
            id: trigger_id,
            controller: player,
            kind: StackObjectKind::CascadeTrigger {
                source_object: new_card_id,
                spell_mana_value: spell_mv,
            },
            targets: vec![],
            cant_be_countered: false,
            is_copy: false,
            cast_with_flashback: false,
        };
        state.stack_objects.push_back(trigger_obj);
        events.push(GameEvent::AbilityTriggered {
            controller: player,
            source_object_id: new_card_id,
            stack_object_id: trigger_id,
        });
    }

    // CR 903.8: Emit commander-specific event when casting from command zone.
    if casting_from_command_zone {
        if let Some(cid) = card_id {
            events.push(GameEvent::CommanderCastFromCommandZone {
                player,
                card_id: cid,
                tax_paid: commander_tax_paid,
            });
        }
    }

    events.push(GameEvent::PriorityGiven {
        player: state.turn.active_player,
    });

    Ok(events)
}

/// CR 702.34a / CR 118.9: Look up the flashback cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Flashback { cost }`, or `None`
/// if the card has no definition or no flashback ability defined. When `None` is returned,
/// no mana payment is required (free flashback — rare, but correct per CR 118.9).
fn get_flashback_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Flashback { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}

/// CR 601.2c: Validate targets at cast time and snapshot their current zones.
///
/// For each target:
/// - Player: must be an active (non-eliminated) player matching the requirement
/// - Object: must exist, pass hexproof/shroud/protection checks, and satisfy the TargetRequirement
///
/// `requirements` is indexed in parallel with `targets` (requirements[i] applies to
/// targets[i]). If there are fewer requirements than targets, extra targets are
/// existence-only validated (no type restriction). This handles cards without
/// definitions registered at cast time.
///
/// `source_chars` is the characteristics of the spell being cast, used for protection-from
/// checks (CR 702.16b). Pass `None` when unavailable (protection check is skipped).
fn validate_targets(
    state: &GameState,
    targets: &[Target],
    requirements: &[TargetRequirement],
    caster: PlayerId,
    source_chars: Option<&Characteristics>,
) -> Result<Vec<SpellTarget>, GameStateError> {
    let mut spell_targets = Vec::with_capacity(targets.len());

    for (i, target) in targets.iter().enumerate() {
        let req = requirements.get(i);

        let spell_target = match target {
            Target::Player(id) => {
                let player = state
                    .players
                    .get(id)
                    .ok_or(GameStateError::PlayerNotFound(*id))?;
                if player.has_lost || player.has_conceded {
                    return Err(GameStateError::InvalidTarget(format!(
                        "player {:?} is not an active player",
                        id
                    )));
                }
                // CR 601.2c: Validate the target satisfies the declared requirement.
                if let Some(req) = req {
                    validate_player_satisfies_requirement(*id, req)?;
                }
                SpellTarget {
                    target: Target::Player(*id),
                    zone_at_cast: None, // Players are not in a zone
                }
            }
            Target::Object(id) => {
                // Object targets are always looked up in state.objects.
                // Spells on the stack are also in state.objects (zone == ZoneId::Stack);
                // StackObject entries in state.stack_objects have separate IDs used
                // internally by the engine, not as targets.
                let obj = state
                    .objects
                    .get(id)
                    .ok_or(GameStateError::ObjectNotFound(*id))?;

                // CR 702.11a / CR 702.18a / CR 702.16b: Hexproof, shroud, and protection.
                super::validate_target_protection(
                    &obj.characteristics.keywords,
                    obj.controller,
                    caster,
                    source_chars,
                )?;

                // CR 601.2c: Validate the target satisfies the declared requirement.
                if let Some(req) = req {
                    validate_object_satisfies_requirement(state, *id, req, caster)?;
                }

                SpellTarget {
                    target: Target::Object(*id),
                    zone_at_cast: Some(obj.zone),
                }
            }
        };
        spell_targets.push(spell_target);
    }

    Ok(spell_targets)
}

/// CR 601.2c: Check that a player target satisfies a requirement.
///
/// Player targets are valid for `TargetPlayer`, `TargetCreatureOrPlayer`,
/// `TargetAny`, and `TargetPlayerOrPlaneswalker`. All other requirements
/// expect an object, so a player target is illegal.
fn validate_player_satisfies_requirement(
    id: PlayerId,
    req: &TargetRequirement,
) -> Result<(), GameStateError> {
    match req {
        TargetRequirement::TargetPlayer
        | TargetRequirement::TargetCreatureOrPlayer
        | TargetRequirement::TargetAny
        | TargetRequirement::TargetPlayerOrPlaneswalker => Ok(()),
        _ => Err(GameStateError::InvalidTarget(format!(
            "player {:?} does not satisfy requirement {:?} (expected an object)",
            id, req
        ))),
    }
}

/// CR 601.2c: Check that an object target satisfies a `TargetRequirement`.
///
/// Uses `calculate_characteristics` for type/keyword checks to respect
/// continuous effects (e.g., type-changing effects from the layer system).
fn validate_object_satisfies_requirement(
    state: &GameState,
    id: ObjectId,
    req: &TargetRequirement,
    caster: PlayerId,
) -> Result<(), GameStateError> {
    // All requirements look up the object in state.objects.
    // Spells on the stack exist in state.objects with zone == ZoneId::Stack.
    let obj = state
        .objects
        .get(&id)
        .ok_or(GameStateError::ObjectNotFound(id))?;

    // TargetSpell / TargetSpellWithFilter: object must be in the stack zone (CR 601.2c).
    if matches!(
        req,
        TargetRequirement::TargetSpell | TargetRequirement::TargetSpellWithFilter(_)
    ) {
        if obj.zone != ZoneId::Stack {
            return Err(GameStateError::InvalidTarget(format!(
                "object {:?} is not on the stack",
                id
            )));
        }
        // For TargetSpellWithFilter, also check the filter against the spell's characteristics.
        if let TargetRequirement::TargetSpellWithFilter(filter) = req {
            let chars: Characteristics =
                calculate_characteristics(state, id).unwrap_or_else(|| obj.characteristics.clone());
            if !crate::effects::matches_filter(&chars, filter) {
                return Err(GameStateError::InvalidTarget(format!(
                    "spell {:?} does not match the filter for {:?}",
                    id, req
                )));
            }
        }
        return Ok(());
    }

    // Use calculate_characteristics to respect continuous effects (CR 613).
    let chars: Characteristics =
        calculate_characteristics(state, id).unwrap_or_else(|| obj.characteristics.clone());

    let on_battlefield = obj.zone == ZoneId::Battlefield;
    let is_creature = chars.card_types.contains(&CardType::Creature);
    let is_artifact = chars.card_types.contains(&CardType::Artifact);
    let is_enchantment = chars.card_types.contains(&CardType::Enchantment);
    let is_land = chars.card_types.contains(&CardType::Land);
    let is_planeswalker = chars.card_types.contains(&CardType::Planeswalker);

    let valid = match req {
        TargetRequirement::TargetCreature => on_battlefield && is_creature,
        TargetRequirement::TargetPermanent => on_battlefield,
        TargetRequirement::TargetArtifact => on_battlefield && is_artifact,
        TargetRequirement::TargetEnchantment => on_battlefield && is_enchantment,
        TargetRequirement::TargetLand => on_battlefield && is_land,
        TargetRequirement::TargetPlaneswalker => on_battlefield && is_planeswalker,
        // "target creature or player" — object side requires creature on battlefield
        TargetRequirement::TargetCreatureOrPlayer => on_battlefield && is_creature,
        // "any target" = creature, planeswalker, or player (CR 115.4) — object side
        TargetRequirement::TargetAny => on_battlefield && (is_creature || is_planeswalker),
        // "target player or planeswalker" — object side requires planeswalker
        TargetRequirement::TargetPlayerOrPlaneswalker => on_battlefield && is_planeswalker,
        TargetRequirement::TargetCreatureWithFilter(filter) => {
            if !on_battlefield || !is_creature {
                false
            } else {
                let passes_filter = crate::effects::matches_filter(&chars, filter);
                let passes_controller = match filter.controller {
                    TargetController::Any => true,
                    TargetController::You => obj.controller == caster,
                    TargetController::Opponent => obj.controller != caster,
                };
                passes_filter && passes_controller
            }
        }
        TargetRequirement::TargetPermanentWithFilter(filter) => {
            if !on_battlefield {
                false
            } else {
                let passes_filter = crate::effects::matches_filter(&chars, filter);
                let passes_controller = match filter.controller {
                    TargetController::Any => true,
                    TargetController::You => obj.controller == caster,
                    TargetController::Opponent => obj.controller != caster,
                };
                passes_filter && passes_controller
            }
        }
        // Player requirement — object target is illegal
        TargetRequirement::TargetPlayer => false,
        // TargetSpell and TargetSpellWithFilter handled above via early return (zone + filter check).
        TargetRequirement::TargetSpell | TargetRequirement::TargetSpellWithFilter(_) => false,
    };

    if valid {
        Ok(())
    } else {
        Err(GameStateError::InvalidTarget(format!(
            "object {:?} does not satisfy requirement {:?}",
            id, req
        )))
    }
}

/// Returns true if the mana pool can cover the mana cost.
///
/// Colored mana (W/U/B/R/G) must be paid with the matching color.
/// Colorless mana (`{C}`) must be paid with colorless mana specifically (CR 106.1).
/// Generic mana (`{N}`) can be paid with any remaining mana in the pool.
pub fn can_pay_cost(
    pool: &crate::state::player::ManaPool,
    cost: &crate::state::game_object::ManaCost,
) -> bool {
    if pool.white < cost.white {
        return false;
    }
    if pool.blue < cost.blue {
        return false;
    }
    if pool.black < cost.black {
        return false;
    }
    if pool.red < cost.red {
        return false;
    }
    if pool.green < cost.green {
        return false;
    }
    if pool.colorless < cost.colorless {
        return false;
    }

    // Remaining mana after paying colored and colorless requirements.
    let remaining = (pool.white - cost.white)
        + (pool.blue - cost.blue)
        + (pool.black - cost.black)
        + (pool.red - cost.red)
        + (pool.green - cost.green)
        + (pool.colorless - cost.colorless);

    remaining >= cost.generic
}

/// Deduct a mana cost from the mana pool. Caller must verify `can_pay_cost` first.
///
/// For generic mana, mana is taken from remaining colored/colorless in order:
/// colorless, then green, red, black, blue, white. The specific order doesn't
/// affect correctness since generic can use any color.
pub fn pay_cost(
    pool: &mut crate::state::player::ManaPool,
    cost: &crate::state::game_object::ManaCost,
) {
    pool.white -= cost.white;
    pool.blue -= cost.blue;
    pool.black -= cost.black;
    pool.red -= cost.red;
    pool.green -= cost.green;
    pool.colorless -= cost.colorless;

    // Pay generic cost from remaining mana.
    let mut remaining = cost.generic;
    for slot in [
        &mut pool.colorless,
        &mut pool.green,
        &mut pool.red,
        &mut pool.black,
        &mut pool.blue,
        &mut pool.white,
    ] {
        let take = remaining.min(*slot);
        *slot -= take;
        remaining -= take;
        if remaining == 0 {
            break;
        }
    }
}
