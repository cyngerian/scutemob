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
use crate::state::replacement_effect::{
    ChosenColorRef, ReplacementManaSourceFilter, ReplacementModification, ReplacementTrigger,
};
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
            #[allow(clippy::collapsible_match)]
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
    // SR-14 IMPOSSIBLE: `source` was validated present via `state.object(source)?`
    // above with no intervening zone change, so calc cannot be None here.
    let resolved_chars = crate::rules::layers::expect_characteristics(state, source);
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
        // SR-14 IMPOSSIBLE: `source` still present (validated above, tapped only later).
        let tap_chars = crate::rules::layers::expect_characteristics(state, source);
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
    // SR-28 (CR 106.12a / CR 106.12b): Snapshot the source's layer-resolved
    // characteristics BEFORE the sacrifice cost step below. A {T}+Sacrifice mana
    // source (Treasure — CR 111.10a; Crystal Vein; Dwarven Ruins) is a dead ObjectId
    // (CR 400.7) by the time the mana-production replacement filter (step 7b) and the
    // "tapped for mana" trigger filter (step 10) run. Per CR 106.12a/106.12b those
    // filters must still see the source as it last existed on the battlefield, so both
    // read this snapshot rather than live state. The snapshot is taken after the {T}
    // tap (step 6) — tapping does not change type/subtype/color — and before any zone
    // change, so it is the source's last-known-information (CR 603.10a semantics).
    // `calculate_characteristics` returns `None` only if the object is absent, which
    // cannot happen here: `source` was validated present at step 3 with no intervening
    // zone change. Kept as `Option` so `mana_source_matches` / the replacement filter
    // fizzle cleanly (rather than panic) if a future caller path ever taps a gone source.
    let source_pre_cost_chars = crate::rules::layers::calculate_characteristics(state, source);
    // 7. Pay sacrifice cost if required (CR 111.10a: Treasure tokens).
    //    Sacrifice is a cost paid before mana is produced (CR 602.2c).
    //    After the zone move, `source` is a dead ObjectId (CR 400.7).
    if ability.sacrifice_self {
        let (
            is_creature,
            owner,
            pre_death_controller,
            pre_death_counters,
            pre_death_power_mana,
            mana_sac_pre_chars,
        ) = {
            let obj = state.object(source)?;
            // CR 613.1d: Use layer-resolved types for sacrifice creature check
            // (animated artifacts/lands are creatures per layer 4).
            // `pre_chars_opt` is threaded into the death event as an
            // Option<Characteristics> LKI snapshot (CR 603.10a), so it stays an Option.
            let pre_chars_opt = crate::rules::layers::calculate_characteristics(state, source);
            // SR-14 IMPOSSIBLE: `source` validated present via `state.object(source)?`
            // just above; the sacrifice move has not happened yet, so calc is not None.
            let sac_chars = crate::rules::layers::expect_characteristics(state, source);
            let lki_power = sac_chars.power.or(obj.characteristics.power);
            (
                sac_chars.card_types.contains(&CardType::Creature),
                obj.owner,
                obj.controller,
                obj.counters.clone(),
                lki_power,
                pre_chars_opt,
            )
        };
        let (new_id, _) = state.move_object_to_zone(source, ZoneId::Graveyard(owner))?;
        if is_creature {
            events.push(GameEvent::CreatureDied {
                object_id: source,
                new_grave_id: new_id,
                controller: pre_death_controller,
                pre_death_counters,
                pre_death_power: pre_death_power_mana,
                pre_death_characteristics: mana_sac_pre_chars,
            });
        } else {
            events.push(GameEvent::PermanentDestroyed {
                object_id: source,
                new_grave_id: new_id,
                // CR 603.10a: pass LKI counters for WhenLeavesBattlefield triggers.
                pre_lba_counters: pre_death_counters.clone(),
                // CR 603.10a: LKI power for SourcePowerAtLastKnownInformation.
                pre_lba_power: pre_death_power_mana,
            });
        }
    }
    // 7b. Apply mana-production replacement effects (CR 106.12b).
    //     Only applies to mana abilities with {T} in cost (CR 106.12).
    //     Returns (multiplier, additions) where additions is a list of (color, amount)
    //     to append to the mana pool after multiplication (CR 106.6a).
    let (mana_multiplier, mana_additions) = if ability.requires_tap {
        // Compute base mana preview for color-filter checks in apply_mana_production_replacements.
        let mut base_preview: Vec<(ManaColor, u32)> = Vec::new();
        if ability.any_color {
            base_preview.push((ManaColor::Colorless, 1));
        } else {
            for (color, amount) in &ability.produces {
                base_preview.push((*color, *amount));
            }
        }
        apply_mana_production_replacements(state, player, &base_preview, &source_pre_cost_chars)
    } else {
        (1u32, Vec::new())
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
    // 8b. Apply additive mana additions (CR 106.6a — e.g., Caged Sun / Gauntlet of Power).
    //     These are added to the pool after the base mana (and multiplier) has been applied.
    //     Each entry is one mana of the chosen color (replacement source's chosen_color).
    for (add_color, add_amount) in &mana_additions {
        if *add_amount > 0 {
            let player_state = state.player_mut(player)?;
            player_state.mana_pool.add(*add_color, *add_amount);
            events.push(GameEvent::ManaAdded {
                player,
                color: *add_color,
                amount: *add_amount,
                source: Some(source),
            });
            mana_produced.push((*add_color, *add_amount));
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
        fire_mana_triggered_abilities(
            state,
            player,
            source,
            &mana_produced,
            &source_pre_cost_chars,
            &mut events,
        );
    }
    // 11. Player retains priority. players_passed is unchanged.
    //    (CR 605.5: mana abilities are special actions; they do not reset priority.)
    Ok(events)
}
/// CR 106.12b / CR 106.6a: Check for mana production replacement effects.
///
/// Returns `(multiplier, additions)` where:
/// - `multiplier`: product of all `MultiplyMana` replacements active for this player
///   (Multiple Nyxbloom Ancients: 3 * 3 = 9x. Multiple Mana Reflections: 2 * 2 = 4x.)
/// - `additions`: list of `(ManaColor, amount)` to add to the pool (CR 106.6a additivity).
///   Used by Caged Sun / Gauntlet of Power ("add an additional one mana of that color").
///
/// `base_mana` is the mana the ability would produce before replacements (color-filter check).
/// `source_pre_cost_chars` is the tapped source's layer-resolved characteristics snapshotted
/// BEFORE the sacrifice cost (SR-28) — read by source_filter checks so a {T}+Sacrifice
/// land (now a dead ObjectId, CR 400.7) is still recognized per CR 106.12b.
fn apply_mana_production_replacements(
    state: &GameState,
    player: PlayerId,
    base_mana: &[(ManaColor, u32)],
    source_pre_cost_chars: &Option<crate::state::game_object::Characteristics>,
) -> (u32, Vec<(ManaColor, u32)>) {
    let mut multiplier = 1u32;
    let mut additions: Vec<(ManaColor, u32)> = Vec::new();
    for effect in state.replacement_effects.iter() {
        if let ReplacementTrigger::ManaWouldBeProduced {
            controller,
            color_filter,
            source_filter,
        } = &effect.trigger
        {
            if *controller != player {
                continue;
            }
            // Check source_filter: does this replacement apply to the tapped permanent?
            if let Some(sf) = source_filter {
                let passes_source_filter = match sf {
                    ReplacementManaSourceFilter::Any => true,
                    // SR-28 (CR 106.12b): read the pre-cost snapshot, not live state. A
                    // {T}+Sacrifice land (Dwarven Ruins, Crystal Vein) is already a dead
                    // ObjectId (CR 400.7) here — the previous live `state.objects.get`
                    // lookup returned `None` and silently dropped the replacement (e.g.
                    // Caged Sun missed its +1). The snapshot is `Some` for any real tap.
                    ReplacementManaSourceFilter::AnyLand => source_pre_cost_chars
                        .as_ref()
                        .map(|c| c.card_types.contains(&crate::state::types::CardType::Land))
                        .unwrap_or(false),
                };
                if !passes_source_filter {
                    continue;
                }
            }
            // Check color_filter: does this replacement only fire for a specific color?
            if let Some(cf) = color_filter {
                let required_color = match cf {
                    ChosenColorRef::SelfChosen => {
                        // Read chosen_color from the replacement's source (Caged Sun etc.),
                        // not from the tapped land.
                        effect
                            .source
                            .and_then(|sid| state.objects.get(&sid))
                            .and_then(|o| o.chosen_color)
                    }
                    ChosenColorRef::Fixed(c) => Some(*c),
                };
                // Check if any of the base_mana produced matches the required color.
                let mana_color_for_comparison = required_color.map(|c| match c {
                    crate::state::types::Color::White => ManaColor::White,
                    crate::state::types::Color::Blue => ManaColor::Blue,
                    crate::state::types::Color::Black => ManaColor::Black,
                    crate::state::types::Color::Red => ManaColor::Red,
                    crate::state::types::Color::Green => ManaColor::Green,
                });
                let color_matches = mana_color_for_comparison
                    .map(|mc| base_mana.iter().any(|(bc, amt)| *bc == mc && *amt > 0))
                    .unwrap_or(false);
                if !color_matches {
                    continue;
                }
                // This filter passed — apply modification.
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
                if !source_on_bf {
                    continue;
                }
                match &effect.modification {
                    ReplacementModification::MultiplyMana(n) => {
                        multiplier = multiplier.saturating_mul(*n);
                    }
                    ReplacementModification::AddOneManaOfChosenColor => {
                        // Add one mana of the chosen color (from the replacement source).
                        // CR 106.6a: "additional one mana of that color" per trigger event.
                        if let Some(mc) = mana_color_for_comparison {
                            additions.push((mc, 1));
                        }
                    }
                    _ => {}
                }
            } else {
                // No color filter — unconditional replacement (Mana Reflection / Nyxbloom).
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
                    if let ReplacementModification::MultiplyMana(n) = &effect.modification {
                        multiplier = multiplier.saturating_mul(*n);
                    }
                }
            }
        }
    }
    (multiplier, additions)
}
/// CR 605.4a / CR 106.12a: Fire triggered abilities that trigger from tapping a permanent
/// for mana. Called after mana is added to the pool.
///
/// - Triggered mana abilities (no targets, produces mana) resolve immediately per CR 605.4a.
/// - Triggered abilities with targets (e.g., Forbidden Orchard) go on the stack per CR 605.5a.
///
/// The `source` is the permanent that was tapped for mana.
/// `mana_produced` is the list of (color, amount) pairs the ability produced (post-multiplier).
/// `source_pre_cost_chars` is the tapped source's layer-resolved characteristics snapshotted
/// BEFORE the sacrifice cost (SR-28) — threaded into `mana_source_matches` so a
/// {T}+Sacrifice source (a dead ObjectId here, CR 400.7) still matches per CR 106.12a.
fn fire_mana_triggered_abilities(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    mana_produced: &[(ManaColor, u32)],
    source_pre_cost_chars: &Option<crate::state::game_object::Characteristics>,
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
            if !mana_source_matches(
                state,
                source,
                trigger_source_id,
                source_filter,
                source_pre_cost_chars,
            ) {
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
///
/// SR-28 (CR 106.12a): the characteristic-reading arms (Land / LandSubtype / Creature /
/// AnyPermanent) read `source_pre_cost_chars` — the source's layer-resolved characteristics
/// snapshotted before any sacrifice cost — rather than live state, because by the time this
/// fires a {T}+Sacrifice source is a dead ObjectId (CR 400.7) whose live `calculate_characteristics`
/// is `None`. The snapshot is `Some` for any legitimately-tapped permanent. The `EnchantedLand`
/// and `This` arms compare against `trigger_source_id` (still live) and need no snapshot.
fn mana_source_matches(
    state: &GameState,
    source: ObjectId,
    trigger_source_id: ObjectId,
    filter: &ManaSourceFilter,
    source_pre_cost_chars: &Option<crate::state::game_object::Characteristics>,
) -> bool {
    match filter {
        ManaSourceFilter::Land => {
            // Source must be a land controlled by the trigger source's controller.
            source_pre_cost_chars
                .as_ref()
                .map(|c| c.card_types.contains(&CardType::Land))
                .unwrap_or(false)
        }
        ManaSourceFilter::LandSubtype(subtype) => {
            // Source must be a land with the specific subtype.
            source_pre_cost_chars
                .as_ref()
                .map(|c| c.card_types.contains(&CardType::Land) && c.subtypes.contains(subtype))
                .unwrap_or(false)
        }
        ManaSourceFilter::Creature => {
            // Source must be a creature.
            source_pre_cost_chars
                .as_ref()
                .map(|c| c.card_types.contains(&CardType::Creature))
                .unwrap_or(false)
        }
        ManaSourceFilter::AnyPermanent => {
            // Any permanent tapped for mana matches. The snapshot is `Some` iff the source
            // was a real permanent when tapped (CR 106.12a) — this also matches a sacrificed
            // source (e.g. a Treasure), whose live battlefield lookup would now be `None`.
            source_pre_cost_chars.is_some()
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
            | Effect::AddManaOfChosenColor { .. }
    )
}
