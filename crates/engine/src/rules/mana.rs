//! Mana ability activation (CR 605).
//!
//! Mana abilities are activated abilities that produce mana and don't target.
//! They do not use the stack — they activate and resolve immediately.
//! They can be activated any time a player has priority (CR 605.3b).
//!
//! For M3-A, only tap-activated mana abilities are supported.
use super::events::{CombatDamageTarget, GameEvent};
use crate::cards::card_definition::{
    AbilityDefinition, Effect, ManaSourceFilter, TriggerCondition,
};
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::replacement_effect::{ReplacementModification, ReplacementTrigger};
use crate::state::stubs::{GameRestriction, PendingTrigger, PendingTriggerKind};
use crate::state::types::{CardType, KeywordAbility, ManaColor};
use crate::state::zone::ZoneId;
use crate::state::GameState;
/// Handle a TapForMana command: activate a mana ability by tapping a permanent.
///
/// Validates priority, battlefield presence, controller, ability existence,
/// and tap status. Taps the permanent (if required), adds mana to the pool.
///
/// Per CR 605.5, activating a mana ability is a special action. The player
/// retains priority and `players_passed` is not reset.
pub fn handle_tap_for_mana(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Validate player has priority (CR 605.3b).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // 1b. PB-18 review Finding 2: Check restrictions that block mana ability activation.
    //
    // CR 605.3: "Activating an activated mana ability follows the rules for activating
    // any other activated ability." Therefore Stony Silence / Collector Ouphe block
    // mana abilities of artifacts (per ruling: "including mana abilities").
    // Grand Abolisher prevents opponents from activating mana abilities of artifacts,
    // creatures, or enchantments during the controller's turn.
    //
    // Per Finding 3 (zone scope): only artifacts ON THE BATTLEFIELD are affected.
    // "Stony Silence's ability affects only artifacts on the battlefield."
    {
        let active_player = state.turn.active_player;
        // Determine source types (battlefield-only check per Finding 3).
        let source_zone = state.objects.get(&source).map(|o| o.zone);
        let source_on_bf = matches!(source_zone, Some(ZoneId::Battlefield));
        let source_is_artifact = source_on_bf
            && crate::rules::layers::calculate_characteristics(state, source)
                .map(|chars| chars.card_types.contains(&CardType::Artifact))
                .unwrap_or(false);
        let source_is_restricted_type = source_on_bf
            && crate::rules::layers::calculate_characteristics(state, source)
                .map(|chars| {
                    chars.card_types.contains(&CardType::Artifact)
                        || chars.card_types.contains(&CardType::Creature)
                        || chars.card_types.contains(&CardType::Enchantment)
                })
                .unwrap_or(false);
        for restriction in state.restrictions.iter() {
            // Skip restrictions whose source is no longer on the battlefield.
            let restriction_source_on_bf = state
                .objects
                .get(&restriction.source)
                .map(|o| matches!(o.zone, ZoneId::Battlefield))
                .unwrap_or(false);
            if !restriction_source_on_bf {
                continue;
            }
            let controller = restriction.controller;
            match &restriction.restriction {
                // Collector Ouphe / Stony Silence: blocks ALL activated abilities of artifacts
                // including mana abilities (CR 605.3 + ruling).
                GameRestriction::ArtifactAbilitiesCantBeActivated => {
                    if source_is_artifact {
                        return Err(GameStateError::InvalidCommand(
                            "restriction: activated abilities of artifacts can't be activated, \
                             including mana abilities (CR 605.3, Stony Silence)"
                                .into(),
                        ));
                    }
                }
                // Grand Abolisher / Myrel: opponents can't activate mana abilities of
                // artifact/creature/enchantment permanents during controller's turn.
                GameRestriction::OpponentsCantCastOrActivateDuringYourTurn => {
                    if active_player == controller
                        && player != controller
                        && source_is_restricted_type
                    {
                        return Err(GameStateError::InvalidCommand(
                            "restriction: opponents can't activate abilities of artifacts, \
                             creatures, or enchantments during your turn, including mana abilities \
                             (CR 605.3, Grand Abolisher)"
                                .into(),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    // 2. Fetch a clone of the source object to avoid borrow conflicts.
    let obj = state.object(source)?.clone();
    // 3. Validate source is on the battlefield.
    if obj.zone != ZoneId::Battlefield {
        return Err(GameStateError::ObjectNotOnBattlefield(source));
    }
    // 4. Validate player controls the source.
    if obj.controller != player {
        return Err(GameStateError::NotController {
            player,
            object_id: source,
        });
    }
    // 5. Fetch the mana ability via layer-resolved characteristics.
    // CR 613.1f: Use calc'd chars so granted abilities (Cryptolith Rite, Chromatic Lantern)
    // and ability-removal (Humility) both apply. W3-LC audit fix.
    let resolved_chars = crate::rules::layers::calculate_characteristics(state, source)
        .unwrap_or_else(|| obj.characteristics.clone());
    let ability = resolved_chars
        .mana_abilities
        .get(ability_index)
        .ok_or(GameStateError::InvalidAbilityIndex {
            object_id: source,
            index: ability_index,
        })?
        .clone();
    let mut events = Vec::new();
    // 6. If the ability requires tapping: validate not already tapped, then tap.
    if ability.requires_tap {
        if obj.status.tapped {
            return Err(GameStateError::PermanentAlreadyTapped(source));
        }
        // CR 302.6 / CR 702.10: Summoning sickness prevents using {T} mana abilities
        // on creatures unless they have haste.
        // CR 613.1d/613.1f: Use layer-resolved types and keywords so animated
        // permanents (e.g., Nissa-animated lands) respect summoning sickness and
        // layer-granted haste (e.g., Fervor) is recognized.
        let tap_chars = crate::rules::layers::calculate_characteristics(state, source)
            .unwrap_or_else(|| obj.characteristics.clone());
        if tap_chars.card_types.contains(&CardType::Creature)
            && obj.has_summoning_sickness
            && !tap_chars.keywords.contains(&KeywordAbility::Haste)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "object {:?} has summoning sickness and cannot tap for mana (no haste)",
                source
            )));
        }
        let obj_mut = state.object_mut(source)?;
        obj_mut.status.tapped = true;
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: source,
        });
    }
    // 7. Pay sacrifice cost if required (CR 111.10a: Treasure tokens).
    //    Sacrifice is a cost paid before mana is produced (CR 602.2c).
    //    After the zone move, `source` is a dead ObjectId (CR 400.7).
    if ability.sacrifice_self {
        let (is_creature, owner, pre_death_controller, pre_death_counters) = {
            let obj = state.object(source)?;
            // CR 613.1d: Use layer-resolved types for sacrifice creature check
            // (animated artifacts/lands are creatures per layer 4).
            let sac_chars = crate::rules::layers::calculate_characteristics(state, source)
                .unwrap_or_else(|| obj.characteristics.clone());
            (
                sac_chars.card_types.contains(&CardType::Creature),
                obj.owner,
                obj.controller,
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
    // 7b. Apply mana-production replacement effects (CR 106.12b).
    //     Only applies to mana abilities with {T} in cost (CR 106.12).
    let mana_multiplier = if ability.requires_tap {
        apply_mana_production_replacements(state, player)
    } else {
        1u32
    };
    // 8. Add produced mana to the player's pool (multiplied by replacement effects).
    //    CR 111.10a: `any_color` produces 1 mana of any color.
    //    Simplified: colorless until interactive color choice is implemented
    //    (consistent with Effect::AddManaAnyColor in effects/mod.rs).
    let mut mana_produced: Vec<(ManaColor, u32)> = Vec::new();
    if ability.any_color {
        // CR 111.10a: "Add one mana of any color."
        let amount = mana_multiplier;
        let player_state = state.player_mut(player)?;
        player_state.mana_pool.add(ManaColor::Colorless, amount);
        events.push(GameEvent::ManaAdded {
            player,
            color: ManaColor::Colorless,
            amount,
            source: Some(source),
        });
        mana_produced.push((ManaColor::Colorless, amount));
    } else {
        let player_state = state.player_mut(player)?;
        for (color, base_amount) in &ability.produces {
            let amount = base_amount * mana_multiplier;
            player_state.mana_pool.add(*color, amount);
            events.push(GameEvent::ManaAdded {
                player,
                color: *color,
                amount,
                source: Some(source),
            });
            mana_produced.push((*color, amount));
        }
    }
    // 9. Pain land damage: deal damage to controller as part of the mana ability.
    //    CR 605: this is part of the mana ability resolution, not a separate trigger.
    if ability.damage_to_controller > 0 {
        let player_state = state.player_mut(player)?;
        player_state.life_total -= ability.damage_to_controller as i32;
        events.push(GameEvent::DamageDealt {
            source,
            target: CombatDamageTarget::Player(player),
            amount: ability.damage_to_controller,
        });
    }
    // 10. Fire triggered mana abilities (CR 605.4a / CR 106.12a).
    //     Only fires for tap-cost mana abilities (CR 106.12: "tap for mana").
    //     Triggered mana abilities (no target) resolve immediately.
    //     Normal triggered abilities (has targets, e.g., Forbidden Orchard) go on the stack.
    if ability.requires_tap {
        fire_mana_triggered_abilities(state, player, source, &mana_produced, &mut events);
    }
    // 11. Player retains priority. players_passed is unchanged.
    //    (CR 605.5: mana abilities are special actions; they do not reset priority.)
    Ok(events)
}
/// CR 106.12b: Check for mana multiplication replacement effects.
/// Returns the product of all matching multipliers for the given player.
/// Multiple Nyxbloom Ancients: 3 * 3 = 9x. Multiple Mana Reflections: 2 * 2 = 4x.
fn apply_mana_production_replacements(state: &GameState, player: PlayerId) -> u32 {
    let mut multiplier = 1u32;
    for effect in state.replacement_effects.iter() {
        if let ReplacementTrigger::ManaWouldBeProduced { controller } = &effect.trigger {
            if *controller == player {
                if let ReplacementModification::MultiplyMana(n) = &effect.modification {
                    // Skip inactive sources (source no longer on battlefield).
                    let source_on_bf = effect
                        .source
                        .map(|src| {
                            state
                                .objects
                                .get(&src)
                                .map(|o| matches!(o.zone, ZoneId::Battlefield))
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    if source_on_bf {
                        multiplier = multiplier.saturating_mul(*n);
                    }
                }
            }
        }
    }
    multiplier
}
/// CR 605.4a / CR 106.12a: Fire triggered abilities that trigger from tapping a permanent
/// for mana. Called after mana is added to the pool.
///
/// - Triggered mana abilities (no targets, produces mana) resolve immediately per CR 605.4a.
/// - Triggered abilities with targets (e.g., Forbidden Orchard) go on the stack per CR 605.5a.
///
/// The `source` is the permanent that was tapped for mana.
/// `mana_produced` is the list of (color, amount) pairs the ability produced (post-multiplier).
fn fire_mana_triggered_abilities(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    mana_produced: &[(ManaColor, u32)],
    events: &mut Vec<GameEvent>,
) {
    // Collect permanents on the battlefield with WhenTappedForMana triggered abilities.
    // We snapshot IDs first to avoid borrow conflicts.
    let battlefield_ids: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Battlefield) && o.controller == player)
        .map(|o| o.id)
        .collect();
    for trigger_source_id in battlefield_ids {
        // Get the card_id for registry lookup.
        let card_id = match state.objects.get(&trigger_source_id) {
            Some(o) => o.card_id.clone(),
            None => continue,
        };
        let card_id = match card_id {
            Some(cid) => cid,
            None => continue,
        };
        // Look up the card definition.
        let registry = state.card_registry.clone();
        let def = match registry.get(card_id) {
            Some(d) => d.clone(),
            None => continue,
        };
        for (ability_idx, ability) in def.abilities.iter().enumerate() {
            let (source_filter, effect, targets) = match ability {
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenTappedForMana { source_filter },
                    effect,
                    targets,
                    ..
                } => (source_filter, effect, targets),
                _ => continue,
            };
            // Check if the tapped source matches the filter.
            if !mana_source_matches(state, source, trigger_source_id, source_filter) {
                continue;
            }
            // Determine if this is a triggered mana ability (CR 605.1b):
            // no targets + could add mana → resolves immediately (CR 605.4a).
            // Has targets → goes on the stack as a normal triggered ability (CR 605.5a).
            if targets.is_empty() && is_mana_producing_effect(effect) {
                // Triggered mana ability: resolve immediately (CR 605.4a).
                use crate::effects::{execute_effect, EffectContext};
                let dummy_source = trigger_source_id;
                let mut ctx = EffectContext::new(player, dummy_source, vec![]);
                ctx.mana_produced = Some(mana_produced.to_vec());
                let mut mana_events = execute_effect(state, effect, &mut ctx);
                // Tag ManaAdded events with no source (triggered mana is not the original tap).
                // Per Nyxbloom ruling: triggered mana abilities are NOT multiplied.
                events.append(&mut mana_events);
            } else {
                // Normal triggered ability with targets or non-mana effect: push to stack.
                // CR 605.5a: this trigger is NOT a mana ability; goes on the stack normally.
                let mut trigger =
                    PendingTrigger::blank(trigger_source_id, player, PendingTriggerKind::Normal);
                trigger.ability_index = ability_idx;
                state.pending_triggers.push_back(trigger);
            }
        }
    }
}
/// Check if the tapped permanent (`source`) matches the `ManaSourceFilter` on the
/// trigger source (`trigger_source_id`). The trigger source is the permanent whose
/// ability is firing (e.g., Mirari's Wake, Wild Growth, Forbidden Orchard).
fn mana_source_matches(
    state: &GameState,
    source: ObjectId,
    trigger_source_id: ObjectId,
    filter: &ManaSourceFilter,
) -> bool {
    match filter {
        ManaSourceFilter::Land => {
            // Source must be a land controlled by the trigger source's controller.
            let chars = crate::rules::layers::calculate_characteristics(state, source);
            chars
                .map(|c| c.card_types.contains(&CardType::Land))
                .unwrap_or(false)
        }
        ManaSourceFilter::LandSubtype(subtype) => {
            // Source must be a land with the specific subtype.
            let chars = crate::rules::layers::calculate_characteristics(state, source);
            chars
                .map(|c| c.card_types.contains(&CardType::Land) && c.subtypes.contains(subtype))
                .unwrap_or(false)
        }
        ManaSourceFilter::Creature => {
            // Source must be a creature.
            let chars = crate::rules::layers::calculate_characteristics(state, source);
            chars
                .map(|c| c.card_types.contains(&CardType::Creature))
                .unwrap_or(false)
        }
        ManaSourceFilter::AnyPermanent => {
            // Any permanent matches.
            state
                .objects
                .get(&source)
                .map(|o| matches!(o.zone, ZoneId::Battlefield))
                .unwrap_or(false)
        }
        ManaSourceFilter::EnchantedLand => {
            // The trigger source (Aura) must be attached to the tapped permanent.
            state
                .objects
                .get(&trigger_source_id)
                .and_then(|o| o.attached_to)
                .map(|attached_id| attached_id == source)
                .unwrap_or(false)
        }
        ManaSourceFilter::This => {
            // The trigger source IS the tapped permanent.
            trigger_source_id == source
        }
    }
}
/// Returns true if the effect can produce mana (making this ability a triggered mana ability
/// per CR 605.1b when it also has no targets). Only checks top-level mana-producing effects.
fn is_mana_producing_effect(effect: &Effect) -> bool {
    matches!(
        effect,
        Effect::AddMana { .. }
            | Effect::AddManaAnyColor { .. }
            | Effect::AddManaMatchingType { .. }
            | Effect::AddManaChoice { .. }
            | Effect::AddManaFilterChoice { .. }
            | Effect::AddManaScaled { .. }
            | Effect::AddManaRestricted { .. }
            | Effect::AddManaAnyColorRestricted { .. }
            | Effect::AddManaOfAnyColorAmount { .. }
    )
}
