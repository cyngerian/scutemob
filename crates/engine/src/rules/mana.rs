//! Mana ability activation (CR 605).
//!
//! Mana abilities are activated abilities that produce mana and don't target.
//! They do not use the stack — they activate and resolve immediately.
//! They can be activated any time a player has priority (CR 605.3b).
//!
//! For M3-A, only tap-activated mana abilities are supported.

use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::stubs::GameRestriction;
use crate::state::types::{CardType, KeywordAbility, ManaColor};
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::{CombatDamageTarget, GameEvent};

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

    // 5. Fetch the mana ability.
    let ability = obj
        .characteristics
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

    // 8. Add produced mana to the player's pool.
    //    CR 111.10a: `any_color` produces 1 mana of any color.
    //    Simplified: colorless until interactive color choice is implemented
    //    (consistent with Effect::AddManaAnyColor in effects/mod.rs).
    if ability.any_color {
        // CR 111.10a: "Add one mana of any color."
        let player_state = state.player_mut(player)?;
        player_state.mana_pool.add(ManaColor::Colorless, 1);
        events.push(GameEvent::ManaAdded {
            player,
            color: ManaColor::Colorless,
            amount: 1,
        });
    } else {
        let player_state = state.player_mut(player)?;
        for (color, amount) in &ability.produces {
            player_state.mana_pool.add(*color, *amount);
            events.push(GameEvent::ManaAdded {
                player,
                color: *color,
                amount: *amount,
            });
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

    // 10. Player retains priority. players_passed is unchanged.
    //    (CR 605.5: mana abilities are special actions; they do not reset priority.)

    Ok(events)
}
