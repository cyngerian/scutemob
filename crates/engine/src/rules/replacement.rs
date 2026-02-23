//! Replacement and prevention effect application (CR 614, 615, 616).
//!
//! Replacement effects intercept events before they occur and modify them inline.
//! They are NOT triggers — they don't use the stack.
//!
//! Key rules:
//! - CR 614.5: A replacement effect can apply to a given event at most once.
//! - CR 614.15: Self-replacement effects apply before other replacement effects.
//! - CR 616.1: When multiple replacements apply, affected player chooses order.
//! - CR 616.1a: Self-replacement effects must be chosen first.
//! - CR 616.1f: After applying one, repeat with remaining applicable effects.

use std::collections::HashSet;

use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::replacement_effect::{
    DamageTargetFilter, ObjectFilter, PendingZoneChange, PlayerFilter, ReplacementId,
    ReplacementModification, ReplacementTrigger,
};
use crate::state::types::CardType;
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::GameEvent;

/// The result of checking for applicable replacement effects.
#[derive(Debug)]
pub enum ReplacementResult {
    /// No replacement effects apply — proceed with the original event.
    NoApplicable,
    /// Exactly one replacement effect applies — auto-apply it.
    AutoApply(ReplacementId),
    /// Multiple replacement effects apply — the player must choose order (CR 616.1).
    NeedsChoice {
        player: PlayerId,
        choices: Vec<ReplacementId>,
        event_description: String,
    },
}

/// Implements CR 614/616: find all active replacement effects matching a trigger.
///
/// Checks duration validity (source still on battlefield for `WhileSourceOnBattlefield`)
/// and excludes effects already applied to this event chain (CR 614.5).
///
/// Returns IDs sorted: self-replacement effects first (CR 614.15), then others.
/// Within each group, order is preserved from `state.replacement_effects` (registration order).
pub fn find_applicable(
    state: &GameState,
    trigger: &ReplacementTrigger,
    already_applied: &HashSet<ReplacementId>,
) -> Vec<ReplacementId> {
    let mut self_replacements = Vec::new();
    let mut other_replacements = Vec::new();

    for effect in state.replacement_effects.iter() {
        // CR 614.5: skip effects already applied to this event chain
        if already_applied.contains(&effect.id) {
            continue;
        }

        // Check duration validity
        if !is_effect_active(state, effect.duration, effect.source) {
            continue;
        }

        // Check trigger match
        if trigger_matches(state, &effect.trigger, trigger) {
            // CR 614.15: partition self-replacement effects
            if effect.is_self_replacement {
                self_replacements.push(effect.id);
            } else {
                other_replacements.push(effect.id);
            }
        }
    }

    // CR 614.15 / 616.1a: self-replacements come first
    self_replacements.extend(other_replacements);
    self_replacements
}

/// Implements CR 616.1: determine what action to take given applicable replacements.
///
/// - 0 applicable: `NoApplicable`
/// - 1 applicable: `AutoApply`
/// - 2+ with exactly 1 self-replacement: auto-apply the self-replacement (CR 616.1a)
/// - 2+ with multiple self-replacements: player chooses among self-replacements (CR 616.1a)
/// - 2+ with no self-replacements: player chooses among all (CR 616.1e)
pub fn determine_action(
    state: &GameState,
    applicable: &[ReplacementId],
    affected_player: PlayerId,
    event_description: &str,
) -> ReplacementResult {
    if applicable.is_empty() {
        return ReplacementResult::NoApplicable;
    }

    if applicable.len() == 1 {
        return ReplacementResult::AutoApply(applicable[0]);
    }

    // CR 616.1a: If any self-replacements exist, they must be chosen first
    let self_ids: Vec<ReplacementId> = applicable
        .iter()
        .copied()
        .filter(|id| {
            state
                .replacement_effects
                .iter()
                .any(|e| e.id == *id && e.is_self_replacement)
        })
        .collect();

    if self_ids.len() == 1 {
        // Exactly one self-replacement: auto-apply it (CR 616.1a)
        return ReplacementResult::AutoApply(self_ids[0]);
    }

    if self_ids.len() > 1 {
        // Multiple self-replacements: player chooses among them (CR 616.1a)
        return ReplacementResult::NeedsChoice {
            player: affected_player,
            choices: self_ids,
            event_description: event_description.to_string(),
        };
    }

    // No self-replacements: player chooses among all (CR 616.1e)
    ReplacementResult::NeedsChoice {
        player: affected_player,
        choices: applicable.to_vec(),
        event_description: event_description.to_string(),
    }
}

/// Handle the `Command::OrderReplacements` command (CR 616.1).
///
/// Validates the player and IDs, then applies the first chosen replacement.
/// If there's a pending zone change, resolves it by applying the replacement
/// and completing the zone move. Otherwise, emits a `ReplacementEffectApplied`
/// event for the first ID.
pub fn handle_order_replacements(
    state: &mut GameState,
    player: PlayerId,
    ids: Vec<ReplacementId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    if ids.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "OrderReplacements requires at least one replacement ID".to_string(),
        ));
    }

    // Validate all IDs exist in the state
    for id in &ids {
        if !state.replacement_effects.iter().any(|e| e.id == *id) {
            return Err(GameStateError::InvalidCommand(format!(
                "replacement effect {:?} not found",
                id
            )));
        }
    }

    // Validate the player is the affected player of a pending zone change
    // or the controller of at least one of the effects.
    let pending_idx = state
        .pending_zone_changes
        .iter()
        .position(|p| p.affected_player == player);

    let player_controls_any = ids.iter().any(|id| {
        state
            .replacement_effects
            .iter()
            .any(|e| e.id == *id && e.controller == player)
    });
    let is_affected_player = pending_idx.is_some();

    if !player_controls_any && !is_affected_player {
        return Err(GameStateError::InvalidCommand(format!(
            "player {:?} does not control any of the specified replacement effects",
            player
        )));
    }

    // If there's a pending zone change for this player, resolve it.
    if let Some(idx) = pending_idx {
        let first_id = ids[0];
        return resolve_pending_zone_change(state, first_id, idx);
    }

    // No pending zone change — emit the applied event (Session 2 fallback).
    let mut events = Vec::new();
    let first_id = ids[0];
    let description = state
        .replacement_effects
        .iter()
        .find(|e| e.id == first_id)
        .map(|e| format!("{:?}", e.modification))
        .unwrap_or_default();

    events.push(GameEvent::ReplacementEffectApplied {
        effect_id: first_id,
        description,
    });

    Ok(events)
}

/// Check whether a replacement effect is currently active based on its duration.
///
/// - `WhileSourceOnBattlefield`: source object must still exist on the battlefield.
/// - `UntilEndOfTurn`: always active (cleanup step handles removal).
/// - `Indefinite`: always active.
fn is_effect_active(
    state: &GameState,
    duration: crate::state::continuous_effect::EffectDuration,
    source: Option<ObjectId>,
) -> bool {
    use crate::state::continuous_effect::EffectDuration;
    match duration {
        EffectDuration::WhileSourceOnBattlefield => {
            if let Some(source_id) = source {
                // Source must exist and be on the battlefield
                state
                    .objects
                    .get(&source_id)
                    .map(|obj| obj.zone == ZoneId::Battlefield)
                    .unwrap_or(false)
            } else {
                // No source — a sourceless WhileSourceOnBattlefield is a configuration error
                false
            }
        }
        EffectDuration::UntilEndOfTurn => true,
        EffectDuration::Indefinite => true,
    }
}

/// Check whether an effect's trigger matches the event trigger.
///
/// For zone-change triggers, checks zone matching (effect's `from: None` is wildcard)
/// and object filter compatibility. For other trigger types, checks the trigger
/// variant matches and the filter/player is compatible.
fn trigger_matches(
    state: &GameState,
    effect_trigger: &ReplacementTrigger,
    event_trigger: &ReplacementTrigger,
) -> bool {
    match (effect_trigger, event_trigger) {
        (
            ReplacementTrigger::WouldChangeZone {
                from: eff_from,
                to: eff_to,
                filter: eff_filter,
            },
            ReplacementTrigger::WouldChangeZone {
                from: evt_from,
                to: evt_to,
                filter: evt_filter,
            },
        ) => {
            // Effect's `from: None` means "from any zone" (wildcard)
            let from_matches = eff_from.is_none() || eff_from == evt_from;
            let to_matches = eff_to == evt_to;
            // Check if the event's specific object matches the effect's filter
            let filter_matches = event_object_matches_filter(state, evt_filter, eff_filter);
            from_matches && to_matches && filter_matches
        }
        (
            ReplacementTrigger::WouldDraw {
                player_filter: eff_filter,
            },
            ReplacementTrigger::WouldDraw {
                player_filter: evt_filter,
            },
        ) => event_player_matches_filter(evt_filter, eff_filter),
        (
            ReplacementTrigger::WouldEnterBattlefield { filter: eff_filter },
            ReplacementTrigger::WouldEnterBattlefield { filter: evt_filter },
        ) => event_object_matches_filter(state, evt_filter, eff_filter),
        (
            ReplacementTrigger::WouldGainLife {
                player_filter: eff_filter,
            },
            ReplacementTrigger::WouldGainLife {
                player_filter: evt_filter,
            },
        ) => event_player_matches_filter(evt_filter, eff_filter),
        (
            ReplacementTrigger::DamageWouldBeDealt {
                target_filter: eff_filter,
            },
            ReplacementTrigger::DamageWouldBeDealt {
                target_filter: evt_filter,
            },
        ) => event_damage_target_matches_filter(evt_filter, eff_filter),
        // Different trigger types never match
        _ => false,
    }
}

/// Check if the object identified by the event filter matches the effect's filter predicate.
///
/// The event filter identifies a specific object (typically `SpecificObject(id)`).
/// The effect filter describes what objects the effect cares about (e.g., `Any`, `AnyCreature`).
fn event_object_matches_filter(
    state: &GameState,
    event_filter: &ObjectFilter,
    effect_filter: &ObjectFilter,
) -> bool {
    // Effect's `Any` matches everything
    if *effect_filter == ObjectFilter::Any {
        return true;
    }

    // Extract the specific object from the event filter
    match event_filter {
        ObjectFilter::SpecificObject(obj_id) => {
            object_matches_filter(state, *obj_id, effect_filter)
        }
        // If event filter is also general (e.g., AnyCreature), check structural overlap
        _ => {
            // General-to-general matching: same variant matches
            event_filter == effect_filter
        }
    }
}

/// Check if a specific game object matches a filter predicate.
pub fn object_matches_filter(state: &GameState, obj_id: ObjectId, filter: &ObjectFilter) -> bool {
    match filter {
        ObjectFilter::Any => true,
        ObjectFilter::SpecificObject(id) => obj_id == *id,
        ObjectFilter::ControlledBy(player) => state
            .objects
            .get(&obj_id)
            .map(|o| o.controller == *player)
            .unwrap_or(false),
        ObjectFilter::AnyCreature => state
            .objects
            .get(&obj_id)
            .map(|o| o.characteristics.card_types.contains(&CardType::Creature))
            .unwrap_or(false),
        ObjectFilter::HasCardType(ct) => state
            .objects
            .get(&obj_id)
            .map(|o| o.characteristics.card_types.contains(ct))
            .unwrap_or(false),
        ObjectFilter::Commander => state
            .objects
            .get(&obj_id)
            .and_then(|o| o.card_id.as_ref())
            .map(|card_id| {
                state
                    .players
                    .values()
                    .any(|p| p.commander_ids.contains(card_id))
            })
            .unwrap_or(false),
        ObjectFilter::HasCardId(target_card_id) => state
            .objects
            .get(&obj_id)
            .and_then(|o| o.card_id.as_ref())
            .map(|cid| cid == target_card_id)
            .unwrap_or(false),
    }
}

/// Check if the player identified by the event filter matches the effect's filter.
fn event_player_matches_filter(event_filter: &PlayerFilter, effect_filter: &PlayerFilter) -> bool {
    if *effect_filter == PlayerFilter::Any {
        return true;
    }

    match event_filter {
        PlayerFilter::Specific(player_id) => player_matches_filter(*player_id, effect_filter),
        _ => event_filter == effect_filter,
    }
}

/// Check if a specific player matches a filter predicate.
pub fn player_matches_filter(player_id: PlayerId, filter: &PlayerFilter) -> bool {
    match filter {
        PlayerFilter::Any => true,
        PlayerFilter::Specific(id) => player_id == *id,
        PlayerFilter::OpponentsOf(id) => player_id != *id,
    }
}

/// Check if the damage target identified by the event matches the effect's filter.
fn event_damage_target_matches_filter(
    event_filter: &DamageTargetFilter,
    effect_filter: &DamageTargetFilter,
) -> bool {
    if *effect_filter == DamageTargetFilter::Any {
        return true;
    }
    // For specific filters, exact match (the interception site constructs
    // the event filter to match the actual damage event)
    event_filter == effect_filter
}

// ── Zone-change interception helpers (Session 3) ──────────────────────────

/// The result of checking replacement effects for a zone change.
#[derive(Debug)]
pub enum ZoneChangeAction {
    /// No replacement effects apply — proceed with the original zone change.
    Proceed,
    /// A single replacement was auto-applied — redirect to a different zone.
    Redirect {
        /// The zone to move the object to instead.
        to: ZoneId,
        /// Events to emit (ReplacementEffectApplied).
        events: Vec<GameEvent>,
        /// The ID of the replacement that was applied (for CR 614.5 tracking).
        applied_id: ReplacementId,
    },
    /// Multiple replacement effects apply — the player must choose (CR 616.1).
    ChoiceRequired {
        player: PlayerId,
        choices: Vec<ReplacementId>,
        event_description: String,
    },
}

/// Check whether replacement effects apply to a zone change and return the action to take.
///
/// Called by interception sites (SBAs, effects) before moving an object between zones.
/// Constructs the appropriate trigger, finds applicable effects, and returns one of:
/// - `Proceed`: no replacements, move normally
/// - `Redirect`: single replacement auto-applied, move to a different zone
/// - `ChoiceRequired`: multiple replacements, player must choose
pub fn check_zone_change_replacement(
    state: &GameState,
    object_id: ObjectId,
    from: crate::state::zone::ZoneType,
    to: crate::state::zone::ZoneType,
    owner: PlayerId,
    already_applied: &HashSet<ReplacementId>,
) -> ZoneChangeAction {
    let trigger = ReplacementTrigger::WouldChangeZone {
        from: Some(from),
        to,
        filter: ObjectFilter::SpecificObject(object_id),
    };

    let applicable = find_applicable(state, &trigger, already_applied);

    let description = format!("{:?} would move from {:?} to {:?}", object_id, from, to);

    match determine_action(state, &applicable, owner, &description) {
        ReplacementResult::NoApplicable => ZoneChangeAction::Proceed,
        ReplacementResult::AutoApply(id) => {
            // Look up the modification
            let modification = state
                .replacement_effects
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.modification.clone());

            match modification {
                Some(ReplacementModification::RedirectToZone(zone_type)) => {
                    let dest = resolve_zone_type_to_zone_id(zone_type, owner);
                    let events = vec![GameEvent::ReplacementEffectApplied {
                        effect_id: id,
                        description: format!("Redirected to {:?}", zone_type),
                    }];
                    ZoneChangeAction::Redirect {
                        to: dest,
                        events,
                        applied_id: id,
                    }
                }
                _ => {
                    // Non-redirect modifications (EntersTapped, etc.) don't change the zone.
                    // For zone-change interception, only RedirectToZone is relevant.
                    ZoneChangeAction::Proceed
                }
            }
        }
        ReplacementResult::NeedsChoice {
            player,
            choices,
            event_description,
        } => ZoneChangeAction::ChoiceRequired {
            player,
            choices,
            event_description,
        },
    }
}

/// Resolve a `ZoneType` to a concrete `ZoneId`, using the object owner for
/// per-player zones (graveyard, hand, library, command zone).
pub fn resolve_zone_type_to_zone_id(
    zone_type: crate::state::zone::ZoneType,
    owner: PlayerId,
) -> ZoneId {
    use crate::state::zone::ZoneType;
    match zone_type {
        ZoneType::Battlefield => ZoneId::Battlefield,
        ZoneType::Graveyard => ZoneId::Graveyard(owner),
        ZoneType::Hand => ZoneId::Hand(owner),
        ZoneType::Library => ZoneId::Library(owner),
        ZoneType::Stack => ZoneId::Stack,
        ZoneType::Exile => ZoneId::Exile,
        ZoneType::Command => ZoneId::Command(owner),
    }
}

/// Complete a pending zone change after a player has chosen the replacement order.
///
/// Called from `handle_order_replacements` when a `Command::OrderReplacements`
/// resolves a pending zone change. Applies the chosen replacement, does the
/// zone move, and re-checks for remaining applicable replacements (CR 616.1f).
pub fn resolve_pending_zone_change(
    state: &mut GameState,
    chosen_id: ReplacementId,
    pending_idx: usize,
) -> Result<Vec<GameEvent>, GameStateError> {
    let pending = state.pending_zone_changes[pending_idx].clone();
    let mut events = Vec::new();
    let mut already_applied: HashSet<ReplacementId> =
        pending.already_applied.iter().copied().collect();

    // Apply the chosen replacement
    let modification = state
        .replacement_effects
        .iter()
        .find(|e| e.id == chosen_id)
        .map(|e| e.modification.clone())
        .ok_or_else(|| {
            GameStateError::InvalidCommand(format!("replacement effect {:?} not found", chosen_id))
        })?;

    already_applied.insert(chosen_id);

    events.push(GameEvent::ReplacementEffectApplied {
        effect_id: chosen_id,
        description: format!("{:?}", modification),
    });

    // Determine the final destination
    let dest = match &modification {
        ReplacementModification::RedirectToZone(zone_type) => {
            resolve_zone_type_to_zone_id(*zone_type, pending.affected_player)
        }
        _ => {
            // Non-redirect: use original destination
            resolve_zone_type_to_zone_id(pending.original_destination, pending.affected_player)
        }
    };

    // Check for additional applicable replacements on the modified event (CR 616.1f)
    let new_to = match &modification {
        ReplacementModification::RedirectToZone(zt) => *zt,
        _ => pending.original_destination,
    };

    // Re-check with the modified destination
    let action = check_zone_change_replacement(
        state,
        pending.object_id,
        crate::state::zone::ZoneType::Battlefield, // still on battlefield
        new_to,
        pending.affected_player,
        &already_applied,
    );

    // Remove the pending entry
    state.pending_zone_changes.remove(pending_idx);

    match action {
        ZoneChangeAction::Proceed | ZoneChangeAction::Redirect { .. } => {
            // Determine final destination (may have been further redirected)
            let final_dest = match action {
                ZoneChangeAction::Redirect {
                    to: redirect_dest,
                    events: redirect_events,
                    ..
                } => {
                    events.extend(redirect_events);
                    redirect_dest
                }
                _ => dest,
            };

            // Do the zone move
            if let Ok((new_id, _old)) = state.move_object_to_zone(pending.object_id, final_dest) {
                events.extend(zone_change_events(pending.object_id, new_id, final_dest));
            }
        }
        ZoneChangeAction::ChoiceRequired {
            player,
            choices,
            event_description,
        } => {
            // Another choice needed — re-add as pending
            state.pending_zone_changes.push_back(PendingZoneChange {
                object_id: pending.object_id,
                original_destination: new_to,
                affected_player: player,
                already_applied: already_applied.into_iter().collect(),
            });
            events.push(GameEvent::ReplacementChoiceRequired {
                player,
                event_description,
                choices,
            });
        }
    }

    Ok(events)
}

// ── ETB replacement interception (Session 4) ──────────────────────────────

/// CR 614.12 / 614.15: Apply self-ETB replacement abilities from a card definition.
///
/// Called immediately after a permanent enters the battlefield (before emitting
/// the ETB event). Looks up the card definition and applies any
/// `AbilityDefinition::Replacement` abilities with a `WouldEnterBattlefield` trigger
/// and `is_self: true`. Self-replacements apply before global replacement effects (CR 614.15).
///
/// Unlike global ETB replacements, self-ETB replacements are not registered in
/// `state.replacement_effects` — they are applied inline from the card definition.
/// No `ReplacementEffectApplied` event is emitted.
pub fn apply_self_etb_from_definition(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    card_id: Option<&crate::state::player::CardId>,
    registry: &crate::cards::registry::CardRegistry,
) -> Vec<GameEvent> {
    use crate::cards::card_definition::AbilityDefinition;

    let Some(cid) = card_id else {
        return Vec::new();
    };
    let Some(def) = registry.get(cid.clone()) else {
        return Vec::new();
    };

    let mut evts = Vec::new();
    for ability in &def.abilities {
        if let AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldEnterBattlefield { .. },
            modification,
            is_self: true,
        } = ability
        {
            evts.extend(apply_self_etb_modification(state, new_id, controller, modification));
        }
    }
    evts
}

/// CR 614.12: Apply global ETB replacement effects to a just-entered permanent.
///
/// Called in resolution.rs and lands.rs immediately after a permanent enters the
/// battlefield (before emitting the ETB event). Checks `state.replacement_effects` for
/// `WouldEnterBattlefield` effects matching `new_id` and applies `EntersTapped` and
/// `EntersWithCounters` modifications.
///
/// Self-ETB replacements from card definitions must be applied BEFORE calling this
/// function via `apply_self_etb_from_definition` (CR 614.15: self-replacement first).
pub fn apply_etb_replacements(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
) -> Vec<GameEvent> {
    let trigger = ReplacementTrigger::WouldEnterBattlefield {
        filter: ObjectFilter::SpecificObject(new_id),
    };
    let applicable = find_applicable(state, &trigger, &std::collections::HashSet::new());
    if applicable.is_empty() {
        return Vec::new();
    }

    let mut etb_events = Vec::new();
    for id in applicable {
        let modification = state
            .replacement_effects
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.modification.clone());

        etb_events.extend(emit_etb_modification(
            state,
            new_id,
            controller,
            Some(id),
            modification,
        ));
    }
    etb_events
}

/// Apply a single ETB modification directly (used for self-ETB replacements from card
/// definitions, which are not registered in `state.replacement_effects`).
///
/// Does not emit `ReplacementEffectApplied` since there is no registered effect ID.
pub fn apply_self_etb_modification(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    modification: &ReplacementModification,
) -> Vec<GameEvent> {
    emit_etb_modification(state, new_id, controller, None, Some(modification.clone()))
}

/// Internal: set state and produce events for one ETB modification.
///
/// If `effect_id` is Some, emits `ReplacementEffectApplied` (for global effects with a
/// registered ID). If None, skips that event (for inline self-ETB replacements).
fn emit_etb_modification(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    effect_id: Option<ReplacementId>,
    modification: Option<ReplacementModification>,
) -> Vec<GameEvent> {
    let mut evts: Vec<GameEvent> = Vec::new();
    match modification {
        Some(ReplacementModification::EntersTapped) => {
            if let Some(obj) = state.objects.get_mut(&new_id) {
                obj.status.tapped = true;
            }
            if let Some(id) = effect_id {
                evts.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: "enters the battlefield tapped".to_string(),
                });
            }
            // CR 614.1c: permanent was never untapped — emit PermanentTapped, not an
            // untap-then-tap sequence. Corner case 19.
            evts.push(GameEvent::PermanentTapped {
                player: controller,
                object_id: new_id,
            });
        }
        Some(ReplacementModification::EntersWithCounters { counter, count }) => {
            if let Some(obj) = state.objects.get_mut(&new_id) {
                let cur = obj.counters.get(&counter).copied().unwrap_or(0);
                obj.counters.insert(counter.clone(), cur + count);
            }
            if let Some(id) = effect_id {
                evts.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: format!("enters with {} {:?} counters", count, counter),
                });
            }
            evts.push(GameEvent::CounterAdded {
                object_id: new_id,
                counter,
                count,
            });
        }
        _ => {
            // RedirectToZone and other modifications are not applicable to ETB
            // modification interception. Zone redirections are handled at zone-change
            // interception sites in sba.rs and effects/mod.rs.
        }
    }
    evts
}

/// Produce appropriate GameEvents for a zone change based on the destination.
fn zone_change_events(old_id: ObjectId, new_id: ObjectId, dest: ZoneId) -> Vec<GameEvent> {
    match dest {
        ZoneId::Graveyard(_) => vec![GameEvent::CreatureDied {
            object_id: old_id,
            new_grave_id: new_id,
        }],
        ZoneId::Exile => vec![GameEvent::ObjectExiled {
            player: PlayerId(0), // caller should override if needed
            object_id: old_id,
            new_exile_id: new_id,
        }],
        ZoneId::Command(_) => vec![GameEvent::ReplacementEffectApplied {
            effect_id: ReplacementId(u64::MAX), // sentinel for commander redirect
            description: format!("Commander {:?} sent to command zone", old_id),
        }],
        _ => vec![],
    }
}
