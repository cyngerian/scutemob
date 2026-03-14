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
    DamageTargetFilter, ObjectFilter, PendingZoneChange, PlayerFilter, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger,
};
use crate::state::types::CardType;
use crate::state::zone::ZoneId;
use crate::state::GameState;

use super::events::{CombatDamageTarget, GameEvent};

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
        // CR 702.95a: Active as long as both creatures are on the battlefield and paired.
        EffectDuration::WhilePaired(a, b) => {
            let a_ok = state
                .objects
                .get(&a)
                .map(|o| {
                    o.zone == ZoneId::Battlefield && o.is_phased_in() && o.paired_with == Some(b)
                })
                .unwrap_or(false);
            let b_ok = state
                .objects
                .get(&b)
                .map(|o| {
                    o.zone == ZoneId::Battlefield && o.is_phased_in() && o.paired_with == Some(a)
                })
                .unwrap_or(false);
            a_ok && b_ok
        }
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
        (
            ReplacementTrigger::WouldBeDestroyed { filter: eff_filter },
            ReplacementTrigger::WouldBeDestroyed { filter: evt_filter },
        ) => event_object_matches_filter(state, evt_filter, eff_filter),
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
        ObjectFilter::OwnedByOpponentsOf(player_id) => state
            .objects
            .get(&obj_id)
            .map(|o| o.owner != *player_id)
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

// ── Draw interception helpers (Session 4) ─────────────────────────────────

/// The result of checking WouldDraw replacement effects for a draw event.
#[derive(Debug)]
pub enum DrawAction {
    /// No replacement effects apply — perform the draw normally.
    Proceed,
    /// A SkipDraw replacement was auto-applied — skip the draw entirely.
    /// Contains the `ReplacementEffectApplied` event to emit.
    Skip(GameEvent),
    /// Multiple replacements apply — the player must choose (CR 616.1).
    /// Emit the returned `ReplacementChoiceRequired` event and defer the draw.
    NeedsChoice(GameEvent),
    /// CR 702.52: One or more dredge cards in the player's graveyard can replace
    /// this draw. Contains the `DredgeChoiceRequired` event to emit.
    /// The engine pauses until a `Command::ChooseDredge` is received.
    DredgeAvailable(GameEvent),
}

/// CR 614.11: Check WouldDraw replacement effects before performing a draw.
///
/// Finds applicable replacements for `player` drawing a card, determines the
/// action per CR 616.1, and returns a `DrawAction` indicating how the draw
/// should proceed.
///
/// Also checks for dredge-eligible cards in the player's graveyard (CR 702.52).
/// Dredge takes priority as a "may" replacement — if dredge options are available,
/// the engine pauses for the player's choice before checking other WouldDraw
/// replacements. If the player declines dredge, the normal draw path re-checks
/// other WouldDraw replacements.
///
/// Called from both `draw_card` (turn_actions.rs) and `draw_one_card`
/// (effects/mod.rs) to keep the two draw paths consistent.
pub fn check_would_draw_replacement(state: &GameState, player: PlayerId) -> DrawAction {
    use crate::state::replacement_effect::{
        PlayerFilter, ReplacementModification, ReplacementTrigger,
    };
    use crate::state::types::KeywordAbility;

    // CR 702.52a: Scan the player's graveyard for dredge-eligible cards.
    // A card is eligible if:
    //   1. It has KeywordAbility::Dredge(n) in its keywords.
    //   2. The player has >= n cards in their library (CR 702.52b).
    let graveyard_zone = ZoneId::Graveyard(player);
    let library_zone = ZoneId::Library(player);
    let library_count = state.zones.get(&library_zone).map(|z| z.len()).unwrap_or(0);

    let mut dredge_options: Vec<(ObjectId, u32)> = state
        .objects
        .values()
        .filter(|obj| obj.zone == graveyard_zone)
        .filter_map(|obj| {
            obj.characteristics.keywords.iter().find_map(|kw| {
                if let KeywordAbility::Dredge(n) = kw {
                    if (*n as usize) <= library_count {
                        Some((obj.id, *n))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
        .collect();

    // Sort for determinism (by ObjectId).
    dredge_options.sort_by_key(|(id, _)| *id);

    if !dredge_options.is_empty() {
        // CR 702.52a: Dredge options available — pause for player choice.
        return DrawAction::DredgeAvailable(GameEvent::DredgeChoiceRequired {
            player,
            options: dredge_options,
        });
    }

    let trigger = ReplacementTrigger::WouldDraw {
        player_filter: PlayerFilter::Specific(player),
    };
    let applicable = find_applicable(state, &trigger, &std::collections::HashSet::new());
    let action = determine_action(state, &applicable, player, "draw a card");

    match action {
        ReplacementResult::NoApplicable => DrawAction::Proceed,
        ReplacementResult::AutoApply(id) => {
            let modification = state
                .replacement_effects
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.modification.clone());
            if matches!(modification, Some(ReplacementModification::SkipDraw)) {
                // CR 614.10: Replace the draw with nothing — no card moved, no CardDrawn.
                DrawAction::Skip(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: "skip that draw".to_string(),
                })
            } else {
                // Other modifications are not applicable to draws — proceed normally.
                DrawAction::Proceed
            }
        }
        ReplacementResult::NeedsChoice {
            player,
            choices,
            event_description,
        } => {
            // CR 616.1: Multiple WouldDraw replacements apply — player must choose order.
            DrawAction::NeedsChoice(GameEvent::ReplacementChoiceRequired {
                player,
                event_description,
                choices,
            })
        }
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
    use crate::state::zone::ZoneType;

    // CR 702.84a: Unearth replacement effect -- "If it would leave the battlefield,
    // exile it instead of putting it anywhere else."
    //
    // This is NOT an ability on the creature -- it persists even if the creature
    // loses all abilities (Humility, Sudden Spoiling, etc.). The was_unearthed flag
    // on the object is the tracking mechanism (independent of creature abilities).
    //
    // Per ruling: "If the spell or ability is actually trying to exile it, it
    // succeeds at exiling it." -- only redirect if destination is not already exile.
    if from == ZoneType::Battlefield && to != ZoneType::Exile {
        if let Some(obj) = state.objects.get(&object_id) {
            if obj.was_unearthed {
                // Redirect to exile (CR 702.84a).
                return ZoneChangeAction::Redirect {
                    to: ZoneId::Exile,
                    events: vec![GameEvent::ReplacementEffectApplied {
                        effect_id: crate::state::replacement_effect::ReplacementId(u64::MAX),
                        description: "Unearth: exiled instead of leaving the battlefield"
                            .to_string(),
                    }],
                    applied_id: crate::state::replacement_effect::ReplacementId(u64::MAX),
                };
            }
        }
    }

    // CR 702.146b: Disturb replacement effect -- "If a permanent with disturb would be
    // put into a graveyard from the battlefield, exile it instead."
    //
    // This replacement uses the was_cast_disturbed flag set when the permanent entered
    // the battlefield via a disturb cast. It persists regardless of ability loss.
    // Only applies when moving from battlefield to graveyard (not other zones).
    if from == ZoneType::Battlefield && to == ZoneType::Graveyard {
        if let Some(obj) = state.objects.get(&object_id) {
            if obj.was_cast_disturbed {
                return ZoneChangeAction::Redirect {
                    to: ZoneId::Exile,
                    events: vec![GameEvent::ReplacementEffectApplied {
                        effect_id: crate::state::replacement_effect::ReplacementId(u64::MAX - 1),
                        description: "Disturb: exiled instead of going to graveyard (CR 702.146b)"
                            .to_string(),
                    }],
                    applied_id: crate::state::replacement_effect::ReplacementId(u64::MAX - 1),
                };
            }
        }
    }

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
                Some(ReplacementModification::ShuffleIntoOwnerLibrary) => {
                    // CR 701.20: Redirect to library AND shuffle the library.
                    // The shuffle event is included in the redirect events so the
                    // interception site can emit both the redirect and the shuffle.
                    let dest =
                        resolve_zone_type_to_zone_id(crate::state::zone::ZoneType::Library, owner);
                    let events = vec![
                        GameEvent::ReplacementEffectApplied {
                            effect_id: id,
                            description: "Shuffled into owner's library".to_string(),
                        },
                        GameEvent::LibraryShuffled { player: owner },
                    ];
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
        ReplacementModification::ShuffleIntoOwnerLibrary => {
            // CR 701.20: redirect to library and shuffle
            resolve_zone_type_to_zone_id(
                crate::state::zone::ZoneType::Library,
                pending.affected_player,
            )
        }
        _ => {
            // Non-redirect: use original destination
            resolve_zone_type_to_zone_id(pending.original_destination, pending.affected_player)
        }
    };

    // Check for additional applicable replacements on the modified event (CR 616.1f)
    let new_to = match &modification {
        ReplacementModification::RedirectToZone(zt) => *zt,
        ReplacementModification::ShuffleIntoOwnerLibrary => crate::state::zone::ZoneType::Library,
        _ => pending.original_destination,
    };

    // If shuffling into library, emit shuffle event.
    if matches!(
        &modification,
        ReplacementModification::ShuffleIntoOwnerLibrary
    ) {
        events.push(GameEvent::LibraryShuffled {
            player: pending.affected_player,
        });
    }

    // Re-check with the modified destination, using the stored original_from zone
    // so non-battlefield zone changes use the correct "from" zone (MR-M8-01).
    let action = check_zone_change_replacement(
        state,
        pending.object_id,
        pending.original_from, // use stored from-zone, not hardcoded Battlefield
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

            // CR 603.3a: capture controller before move_object_to_zone resets it to owner.
            // CR 702.79a: capture counters before move_object_to_zone resets them.
            let (pre_move_controller, pre_death_counters) = state
                .objects
                .get(&pending.object_id)
                .map(|o| (o.controller, o.counters.clone()))
                .unwrap_or((pending.affected_player, Default::default()));
            // Do the zone move
            if let Ok((new_id, _old)) = state.move_object_to_zone(pending.object_id, final_dest) {
                events.extend(zone_change_events(
                    state,
                    pending.object_id,
                    new_id,
                    final_dest,
                    pending.affected_player,
                    pre_move_controller,
                    &pre_death_counters,
                ));
            }
        }
        ZoneChangeAction::ChoiceRequired {
            player,
            choices,
            event_description,
        } => {
            // Another choice needed — re-add as pending, preserving original_from
            state.pending_zone_changes.push_back(PendingZoneChange {
                object_id: pending.object_id,
                original_from: pending.original_from,
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
            unless_condition,
        } = ability
        {
            // CR 614.1c: "enters tapped unless [condition]" — if the condition
            // is met, skip the replacement (permanent enters untapped).
            if let Some(condition) = unless_condition {
                let ctx = crate::effects::EffectContext::new(controller, new_id, vec![]);
                if crate::effects::check_condition(state, condition, &ctx) {
                    continue;
                }
            }
            evts.extend(apply_self_etb_modification(
                state,
                new_id,
                controller,
                modification,
            ));
        }
    }
    evts
}

/// CR 603.3, 603.6a: Queue "When ~ enters the battlefield" triggered abilities from a
/// card definition as `PendingTrigger` entries so they go on the stack the next time a
/// player would receive priority (CR 603.3).
///
/// `queue_carddef_etb_triggers` supersedes the old inline-execution approach.
/// `AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield }` entries
/// and `TributeNotPaid` entries are queued as `PendingTrigger`. Fabricate stays inline
/// (bot approximation, TODO). The existing `flush_pending_triggers` + `TriggeredAbility`
/// SOK resolution path (with CardDef registry fallback from B14) handles resolution.
///
/// CR 708.3: Face-down permanents have no triggered abilities — checked at entry.
/// CR 603.2, 613 (Layer 6): If a continuous effect removes all abilities from this
/// permanent, no ETB triggers are queued (IG-1).
/// CR 614.16a: If a Torpor Orb-style ETB suppressor applies to this permanent,
/// no ETB triggers are queued (IG-2).
///
/// Returns `Vec<GameEvent>` for Fabricate inline events only (bot approximation).
/// All other ETB triggers are queued — no events returned for them.
pub fn queue_carddef_etb_triggers(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    card_id: Option<&crate::state::player::CardId>,
    registry: &crate::cards::registry::CardRegistry,
) -> Vec<GameEvent> {
    use crate::cards::card_definition::{AbilityDefinition, Effect, TokenSpec, TriggerCondition};
    use crate::effects::{execute_effect, EffectContext};
    use crate::state::stubs::{ETBSuppressFilter, PendingTrigger, PendingTriggerKind};
    use crate::state::types::{CounterType, KeywordAbility, SubType};

    // CR 708.3: Face-down permanents have no triggered abilities.
    if let Some(obj) = state.objects.get(&new_id) {
        if obj.status.face_down && obj.face_down_as.is_some() {
            return Vec::new();
        }
    }

    // IG-1 (CR 603.2, 613 Layer 6): If any active continuous effect applies
    // RemoveAllAbilities (Layer 6) to the entering permanent, its CardDef triggered
    // abilities are suppressed — do not queue any ETB triggers.
    //
    // We check this by calling calculate_characteristics and examining whether any
    // Layer 6 RemoveAllAbilities effect applies. Using the layer-resolved chars
    // directly: if RemoveAllAbilities was applied, the keywords will reflect that.
    // However, CardDef triggers are not in chars.triggered_abilities, so we must
    // check the active effects directly for RemoveAllAbilities targeting new_id.
    {
        use crate::rules::layers;
        use crate::state::continuous_effect::{EffectLayer, LayerModification};

        let abilities_removed = state
            .continuous_effects
            .iter()
            .filter(|e| layers::is_effect_active(state, e))
            .filter(|e| e.layer == EffectLayer::Ability)
            .filter(|e| matches!(e.modification, LayerModification::RemoveAllAbilities))
            .any(|e| {
                // Check if this effect's filter applies to new_id.
                // We need base characteristics to evaluate filter predicates.
                // Use the object's stored characteristics as the filter basis.
                let obj_zone = state
                    .objects
                    .get(&new_id)
                    .map(|o| o.zone)
                    .unwrap_or(crate::state::zone::ZoneId::Exile);
                let chars = state
                    .objects
                    .get(&new_id)
                    .map(|o| o.characteristics.clone())
                    .unwrap_or_default();
                layers::effect_applies_to_object(state, e, new_id, obj_zone, &chars)
            });

        if abilities_removed {
            return Vec::new();
        }
    }

    // IG-2 (CR 614.16a): If any active ETB suppressor on the battlefield applies
    // to this entering permanent, its CardDef ETB triggered abilities are suppressed.
    //
    // Lazily remove stale suppressors whose source left the battlefield.
    state.etb_suppressors.retain(|s| {
        state
            .objects
            .get(&s.source)
            .map(|o| o.zone == crate::state::zone::ZoneId::Battlefield)
            .unwrap_or(false)
    });
    {
        let entering_is_creature = state
            .objects
            .get(&new_id)
            .map(|o| {
                o.characteristics
                    .card_types
                    .contains(&crate::state::types::CardType::Creature)
            })
            .unwrap_or(false);

        let etb_suppressed = state.etb_suppressors.iter().any(|s| match &s.filter {
            ETBSuppressFilter::CreaturesOnly => entering_is_creature,
            ETBSuppressFilter::AllPermanents => true,
        });

        if etb_suppressed {
            return Vec::new();
        }
    }

    let Some(cid) = card_id else {
        return Vec::new();
    };
    let Some(def) = registry.get(cid.clone()) else {
        return Vec::new();
    };

    // CR 702.104b: Retrieve tribute_was_paid status from the permanent for trigger condition check.
    let tribute_was_paid = state
        .objects
        .get(&new_id)
        .map(|o| o.tribute_was_paid)
        .unwrap_or(false);

    let mut evts = Vec::new();
    for (idx, ability) in def.abilities.iter().enumerate() {
        match ability {
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                intervening_if,
                ..
            } => {
                // CR 603.4: Check intervening-if condition at trigger time.
                // CR 207.2c (Corrupted): "if an opponent has N or more poison counters."
                if let Some(cond) = intervening_if {
                    use crate::cards::card_definition::Condition;
                    let condition_met = match cond {
                        Condition::OpponentHasPoisonCounters(n) => {
                            state.players.iter().any(|(pid, ps)| {
                                *pid != controller && !ps.has_lost && ps.poison_counters >= *n
                            })
                        }
                        // Other conditions: treat as satisfied at trigger time (safe default).
                        _ => true,
                    };
                    if !condition_met {
                        continue;
                    }
                }
                // CR 603.3: Queue as PendingTrigger; flush_pending_triggers places it on the stack.
                // Use PendingTriggerKind::CardDefETB so resolution looks up the effect from
                // the card registry (ability_index is into CardDef::abilities, NOT into
                // runtime triggered_abilities). This avoids index collisions with triggers
                // added by enrich_spec_from_def for attack/dies/etc. triggers.
                state.pending_triggers.push_back(PendingTrigger {
                    source: new_id,
                    ability_index: idx,
                    controller,
                    kind: PendingTriggerKind::CardDefETB,
                    triggering_event: Some(
                        crate::state::game_object::TriggerEvent::SelfEntersBattlefield,
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
                    cipher_encoded_card_id: None,
                    cipher_encoded_object_id: None,
                    haunt_source_object_id: None,
                    haunt_source_card_id: None,
                });
            }
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::TributeNotPaid,
                ..
            } => {
                // CR 702.104b: "When ~ enters, if tribute wasn't paid, ..."
                // CR 603.4: Intervening-if — only queue trigger if tribute was not paid.
                if !tribute_was_paid {
                    state.pending_triggers.push_back(PendingTrigger {
                        source: new_id,
                        ability_index: idx,
                        controller,
                        kind: PendingTriggerKind::CardDefETB,
                        triggering_event: Some(
                            crate::state::game_object::TriggerEvent::SelfEntersBattlefield,
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
                        cipher_encoded_card_id: None,
                        cipher_encoded_object_id: None,
                        haunt_source_object_id: None,
                        haunt_source_card_id: None,
                    });
                }
            }
            _ => {}
        }
    }

    // CR 702.123a: Fabricate N -- "When this permanent enters, you may put N
    // +1/+1 counters on it. If you don't, create N 1/1 colorless Servo
    // artifact creature tokens."
    // CR 702.123b: Multiple instances trigger separately.
    //
    // NOTE: Fires inline for bot play rather than going on the stack. In
    // interactive play, Fabricate is a triggered ability that uses the stack
    // (CR 702.123a: "When this permanent enters" is triggered ability language).
    // This inline approximation must be replaced with proper stack-based
    // resolution before adding human player support.
    //
    // Bot play: always choose counters if the permanent is still on the battlefield.
    // Ruling 2016-09-20: if the permanent is no longer on the battlefield, create tokens.
    {
        let fabricate_instances: Vec<u32> = def
            .abilities
            .iter()
            .filter_map(|a| match a {
                AbilityDefinition::Keyword(KeywordAbility::Fabricate(n)) => Some(*n),
                _ => None,
            })
            .collect();

        for n in fabricate_instances {
            let permanent_on_bf = state
                .objects
                .get(&new_id)
                .map(|o| o.zone == ZoneId::Battlefield)
                .unwrap_or(false);

            if permanent_on_bf {
                // Bot choice: put N +1/+1 counters on it (CR 702.123a).
                if n > 0 {
                    if let Some(obj) = state.objects.get_mut(&new_id) {
                        let current = obj
                            .counters
                            .get(&CounterType::PlusOnePlusOne)
                            .copied()
                            .unwrap_or(0);
                        obj.counters = obj
                            .counters
                            .update(CounterType::PlusOnePlusOne, current + n);
                    }
                    evts.push(super::events::GameEvent::CounterAdded {
                        object_id: new_id,
                        counter: CounterType::PlusOnePlusOne,
                        count: n,
                    });
                }
            } else {
                // Ruling 2016-09-20: if permanent left the battlefield, create Servo tokens.
                if n > 0 {
                    let servo_spec = TokenSpec {
                        name: "Servo".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: im::OrdSet::new(),
                        supertypes: im::OrdSet::new(),
                        card_types: [CardType::Artifact, CardType::Creature]
                            .into_iter()
                            .collect(),
                        subtypes: [SubType("Servo".to_string())].into_iter().collect(),
                        keywords: im::OrdSet::new(),
                        count: n,
                        tapped: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    };
                    let mut ctx = EffectContext::new(controller, new_id, vec![]);
                    evts.extend(execute_effect(
                        state,
                        &Effect::CreateToken { spec: servo_spec },
                        &mut ctx,
                    ));
                }
            }
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
        Some(ReplacementModification::EntersTapped)
        | Some(ReplacementModification::EntersTappedUnlessPayLife(_)) => {
            // EntersTappedUnlessPayLife: deterministic fallback (pre-M10) — always
            // enters tapped. Interactive "may pay N life" choice deferred to M10.
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
///
/// `owner` is used for the `ObjectExiled` player field and `CommanderZoneRedirect`.
/// `pre_move_controller` is the controller captured before `move_object_to_zone` reset it to
/// owner — required for CR 603.3a correctness in `CreatureDied`.
/// `pre_death_counters` is the counter state captured before `move_object_to_zone` reset it —
/// required for CR 702.79a persist/undying intervening-if check in `check_triggers`.
/// `card_types` is used to choose `CreatureDied` vs `PermanentDestroyed` for graveyard moves.
fn zone_change_events(
    state: &GameState,
    old_id: ObjectId,
    new_id: ObjectId,
    dest: ZoneId,
    owner: PlayerId,
    pre_move_controller: PlayerId,
    pre_death_counters: &im::OrdMap<crate::state::types::CounterType, u32>,
) -> Vec<GameEvent> {
    match dest {
        ZoneId::Graveyard(_) => {
            // MR-M8-06: check card types before choosing event variant.
            let is_creature = state
                .objects
                .get(&new_id)
                .map(|o| o.characteristics.card_types.contains(&CardType::Creature))
                .unwrap_or(false);
            if is_creature {
                vec![GameEvent::CreatureDied {
                    object_id: old_id,
                    new_grave_id: new_id,
                    // CR 603.3a: use pre-move controller, not owner (which is what
                    // move_object_to_zone resets controller to).
                    controller: pre_move_controller,
                    // CR 702.79a: last-known counter state for persist/undying check.
                    pre_death_counters: pre_death_counters.clone(),
                }]
            } else {
                vec![GameEvent::PermanentDestroyed {
                    object_id: old_id,
                    new_grave_id: new_id,
                }]
            }
        }
        ZoneId::Exile => vec![GameEvent::ObjectExiled {
            player: owner, // MR-M8-04: use real owner instead of hardcoded PlayerId(0)
            object_id: old_id,
            new_exile_id: new_id,
        }],
        ZoneId::Command(_) => vec![GameEvent::CommanderZoneRedirect {
            // MR-M8-05: proper variant instead of ReplacementId(u64::MAX) sentinel
            object_id: old_id,
            new_command_id: new_id,
            owner,
        }],
        _ => vec![],
    }
}

// ── Global replacement registration (Session 6) ───────────────────────────

/// Register global replacement abilities from a card definition when a permanent
/// enters the battlefield (CR 614, 615).
///
/// Called at every ETB site (resolution.rs, lands.rs) immediately after
/// `apply_etb_replacements`. Reads each `AbilityDefinition::Replacement` ability
/// from the card definition and creates a `ReplacementEffect` entry in
/// `state.replacement_effects` with:
///
/// - `source: Some(new_id)` — `is_effect_active` deactivates it when source leaves.
/// - `duration: WhileSourceOnBattlefield`.
/// - `is_self_replacement`, `trigger`, and `modification` from the definition.
///
/// **Skips** `WouldEnterBattlefield + is_self: true` abilities — those are applied
/// inline during ETB via `apply_self_etb_from_definition` and must not be
/// registered in state (they would fire again on the next ETB event).
pub fn register_permanent_replacement_abilities(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    card_id: Option<&crate::state::player::CardId>,
    registry: &crate::cards::registry::CardRegistry,
) {
    use crate::cards::card_definition::AbilityDefinition;
    use crate::state::continuous_effect::EffectDuration;

    let Some(cid) = card_id else {
        return;
    };
    let Some(def) = registry.get(cid.clone()) else {
        return;
    };

    for ability in &def.abilities {
        if let AbilityDefinition::Replacement {
            trigger,
            modification,
            is_self,
            unless_condition: _,
        } = ability
        {
            // Self-ETB replacements are applied inline — do not register.
            if *is_self {
                if let ReplacementTrigger::WouldEnterBattlefield { .. } = trigger {
                    continue;
                }
            }

            // For self-replacement zone-change effects, bind the filter to this
            // specific object at registration time. The card definition uses
            // `ObjectFilter::Any` as a placeholder meaning "this object," but
            // we must narrow it at runtime so the effect doesn't fire for other objects.
            //
            // For non-self WouldChangeZone effects with `OwnedByOpponentsOf`, bind the
            // controller's PlayerId at registration time so "opponents" is computed
            // relative to the Leyline controller (MR-M8-09).
            let resolved_trigger = if *is_self {
                match trigger {
                    ReplacementTrigger::WouldChangeZone { from, to, .. } => {
                        ReplacementTrigger::WouldChangeZone {
                            from: *from,
                            to: *to,
                            filter: ObjectFilter::SpecificObject(new_id),
                        }
                    }
                    other => other.clone(),
                }
            } else {
                match trigger {
                    ReplacementTrigger::WouldChangeZone {
                        from,
                        to,
                        filter: ObjectFilter::OwnedByOpponentsOf(_),
                    } => ReplacementTrigger::WouldChangeZone {
                        from: *from,
                        to: *to,
                        filter: ObjectFilter::OwnedByOpponentsOf(controller),
                    },
                    other => other.clone(),
                }
            };

            let id = state.next_replacement_id();
            state.replacement_effects.push_back(ReplacementEffect {
                id,
                source: Some(new_id),
                controller,
                duration: EffectDuration::WhileSourceOnBattlefield,
                is_self_replacement: *is_self,
                trigger: resolved_trigger,
                modification: modification.clone(),
            });
        }
    }
}

// ── Static continuous effect registration (Session 2, M9.4) ──────────────

/// Register static continuous effects from a card definition when a permanent
/// enters the battlefield (CR 604, CR 613).
///
/// Called at every ETB site (resolution.rs, lands.rs) immediately after
/// `register_permanent_replacement_abilities`. Reads each
/// `AbilityDefinition::Static` from the card definition and creates a
/// `ContinuousEffect` entry in `state.continuous_effects` with:
///
/// - `source: Some(new_id)` — `is_effect_active` deactivates it when source leaves.
/// - `duration: WhileSourceOnBattlefield`.
/// - `layer`, `filter`, and `modification` from the definition.
///
/// The `filter` field is used as-is; `EffectFilter::AttachedCreature` will resolve
/// correctly at characteristic-calculation time via the source's `attached_to` field.
pub fn register_static_continuous_effects(
    state: &mut GameState,
    new_id: ObjectId,
    card_id: Option<&crate::state::player::CardId>,
    registry: &crate::cards::registry::CardRegistry,
) {
    use crate::cards::card_definition::AbilityDefinition;
    use crate::state::continuous_effect::{ContinuousEffect, EffectId};

    let Some(cid) = card_id else {
        return;
    };
    let Some(def) = registry.get(cid.clone()) else {
        return;
    };

    // Get the controller of the entering permanent for TriggerDoubler registration.
    let controller = state
        .objects
        .get(&new_id)
        .map(|obj| obj.controller)
        .unwrap_or_else(|| crate::state::player::PlayerId(0));

    for ability in &def.abilities {
        match ability {
            AbilityDefinition::Static { continuous_effect } => {
                let eff_id = state.next_object_id().0;
                let ts = state.timestamp_counter;
                state.timestamp_counter += 1;
                state.continuous_effects.push_back(ContinuousEffect {
                    id: EffectId(eff_id),
                    source: Some(new_id),
                    timestamp: ts,
                    layer: continuous_effect.layer,
                    duration: continuous_effect.duration,
                    filter: continuous_effect.filter.clone(),
                    modification: continuous_effect.modification.clone(),
                    is_cda: false,
                });
            }
            // CR 603.2d: Register a Panharmonicon-style trigger-doubling effect.
            AbilityDefinition::TriggerDoubling {
                filter,
                additional_triggers,
            } => {
                state
                    .trigger_doublers
                    .push_back(crate::state::stubs::TriggerDoubler {
                        source: new_id,
                        controller,
                        filter: filter.clone(),
                        additional_triggers: *additional_triggers,
                    });
            }
            // CR 614.16a: Register a Torpor Orb-style ETB trigger suppressor.
            AbilityDefinition::SuppressCreatureETBTriggers { filter } => {
                state
                    .etb_suppressors
                    .push_back(crate::state::stubs::ETBSuppressor {
                        source: new_id,
                        filter: filter.clone(),
                    });
            }
            _ => {}
        }
    }
}

// ── Damage prevention interception (Session 5) ────────────────────────────

/// CR 615 + CR 702.16e: Check and apply damage prevention effects to a damage event.
///
/// Called by damage interception sites (`DealDamage` effect, `apply_combat_damage`)
/// before applying damage to a target.
///
/// Step 1 (CR 702.16e): check protection — if the target is a permanent with
/// protection from a quality the source matches, all damage is prevented immediately
/// (no events emitted, amount returns 0).
///
/// Step 2 (CR 615.7): apply dynamic prevention shields in registration order.
/// Decrements shields, removes exhausted shields, emits `DamagePrevented` and
/// `ReplacementEffectApplied` events.
///
/// Returns `(final_amount, events)`. If `final_amount == 0`, all damage was prevented.
pub fn apply_damage_prevention(
    state: &mut GameState,
    source: ObjectId,
    target: &CombatDamageTarget,
    amount: u32,
) -> (u32, Vec<GameEvent>) {
    // CR 702.16e: protection is a static prevention — checked BEFORE dynamic shields.
    match target {
        CombatDamageTarget::Creature(target_id) | CombatDamageTarget::Planeswalker(target_id) => {
            let target_keywords =
                crate::rules::layers::calculate_characteristics(state, *target_id)
                    .map(|c| c.keywords)
                    .unwrap_or_default();
            let source_chars = crate::rules::protection::source_characteristics(state, source);
            if let Some(sc) = &source_chars {
                if crate::rules::protection::protection_prevents_damage(&target_keywords, sc) {
                    return (0, Vec::new());
                }
            }
        }
        CombatDamageTarget::Player(player_id) => {
            // CR 702.16e: damage from a source with the stated quality to a player
            // with protection from that quality is prevented.
            let source_chars = crate::rules::protection::source_characteristics(state, source);
            if let Some(sc) = &source_chars {
                if let Some(player) = state.players.get(player_id) {
                    let qualities = player.protection_qualities.clone();
                    for quality in &qualities {
                        if crate::rules::protection::has_protection_from_source_quality(quality, sc)
                        {
                            return (0, Vec::new());
                        }
                    }
                }
            }
        }
    }

    // Build the event trigger for this specific damage target.
    let target_filter = match target {
        CombatDamageTarget::Player(p) => DamageTargetFilter::Player(*p),
        CombatDamageTarget::Creature(id) | CombatDamageTarget::Planeswalker(id) => {
            DamageTargetFilter::Permanent(*id)
        }
    };
    let trigger = ReplacementTrigger::DamageWouldBeDealt { target_filter };

    let applicable = find_applicable(state, &trigger, &HashSet::new());
    if applicable.is_empty() {
        return (amount, Vec::new());
    }

    let mut remaining = amount;
    let mut events = Vec::new();
    let mut exhausted: Vec<ReplacementId> = Vec::new();

    for id in applicable {
        if remaining == 0 {
            break;
        }

        let modification = state
            .replacement_effects
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.modification.clone());

        match modification {
            Some(ReplacementModification::PreventDamage(shield_max)) => {
                // Use the live counter if present; initialise from the modification otherwise.
                let counter = state
                    .prevention_counters
                    .get(&id)
                    .copied()
                    .unwrap_or(shield_max);
                let prevented = counter.min(remaining);
                let new_counter = counter - prevented;
                remaining -= prevented;

                events.push(GameEvent::DamagePrevented {
                    source,
                    target: target.clone(),
                    prevented,
                    remaining,
                });
                events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: format!(
                        "prevented {} damage ({} remaining on shield)",
                        prevented, new_counter
                    ),
                });

                if new_counter == 0 {
                    // Shield exhausted — remove the counter and queue the effect for removal.
                    state.prevention_counters.remove(&id);
                    exhausted.push(id);
                } else {
                    state.prevention_counters.insert(id, new_counter);
                }
            }
            Some(ReplacementModification::PreventAllDamage) => {
                let prevented = remaining;
                remaining = 0;

                events.push(GameEvent::DamagePrevented {
                    source,
                    target: target.clone(),
                    prevented,
                    remaining: 0,
                });
                events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: "prevented all damage".to_string(),
                });
                // PreventAllDamage is not consumed — it lasts until its duration expires.
            }
            _ => {
                // Other modifications on a DamageWouldBeDealt trigger (future use) are
                // not handled here. Zone redirects and other replacements are separate.
            }
        }
    }

    // Remove exhausted prevention shields.
    for id in exhausted {
        if let Some(pos) = state.replacement_effects.iter().position(|e| e.id == id) {
            state.replacement_effects.remove(pos);
        }
    }

    (remaining, events)
}

// ── Dredge command handler (CR 702.52) ────────────────────────────────────

/// CR 702.52: Handle the player's choice to dredge or draw normally.
///
/// Called from `engine.rs::process_command` when a `Command::ChooseDredge` is received.
///
/// If `card` is `Some(id)`:
///   1. Validate the card is in the player's graveyard with `KeywordAbility::Dredge(n)`.
///   2. Validate the player has >= n cards in library (CR 702.52b).
///   3. Mill n cards from the top of the library (emitting `CardMilled` events).
///   4. Move the dredge card from graveyard to hand (CR 400.7: new ObjectId).
///   5. Emit `Dredged` event.
///   6. Do NOT increment `cards_drawn_this_turn` (dredge is NOT drawing — CR 702.52a).
///
/// If `card` is `None`:
///   The player declined to dredge — perform the normal draw (re-checks other
///   WouldDraw replacements including any registered replacement effects).
pub fn handle_choose_dredge(
    state: &mut GameState,
    player: PlayerId,
    card: Option<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::types::KeywordAbility;

    match card {
        None => {
            // Player declined dredge — perform normal draw.
            // CR 702.52a: "you may instead" — declining is always legal.
            // Call draw_card which will re-check WouldDraw replacement effects
            // (but NOT dredge — the player just declined).
            // We temporarily call draw_card_without_dredge to avoid re-offering dredge.
            draw_card_skipping_dredge(state, player)
        }
        Some(card_id) => {
            // Player chose to dredge card_id.
            // Step 1: Validate the card is in the player's graveyard.
            let graveyard_zone = ZoneId::Graveyard(player);
            let dredge_n = {
                let obj = state.objects.get(&card_id).ok_or_else(|| {
                    GameStateError::InvalidCommand(format!("dredge card {:?} not found", card_id))
                })?;
                if obj.zone != graveyard_zone {
                    return Err(GameStateError::InvalidCommand(format!(
                        "dredge card {:?} is not in {:?}'s graveyard (zone: {:?})",
                        card_id, player, obj.zone
                    )));
                }
                // Find Dredge(n) in keywords.
                obj.characteristics
                    .keywords
                    .iter()
                    .find_map(|kw| {
                        if let KeywordAbility::Dredge(n) = kw {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| {
                        GameStateError::InvalidCommand(format!(
                            "card {:?} does not have Dredge keyword",
                            card_id
                        ))
                    })?
            };

            // Step 2: Validate library has >= n cards (CR 702.52b).
            let library_zone = ZoneId::Library(player);
            let library_count = state.zones.get(&library_zone).map(|z| z.len()).unwrap_or(0);
            if (dredge_n as usize) > library_count {
                return Err(GameStateError::InvalidCommand(format!(
                    "cannot dredge {}: library has only {} cards (need {})",
                    card_id.0, library_count, dredge_n
                )));
            }

            let mut events = Vec::new();

            // Step 3: Mill n cards from the top of library.
            for _ in 0..dredge_n {
                let top = state.zones.get(&library_zone).and_then(|z| z.top());
                if let Some(top_id) = top {
                    if let Ok((new_id, _)) =
                        state.move_object_to_zone(top_id, ZoneId::Graveyard(player))
                    {
                        events.push(GameEvent::CardMilled { player, new_id });
                    }
                }
            }

            // Step 4: Move the dredge card from graveyard to hand (CR 400.7: new ObjectId).
            let (new_hand_id, _) = state
                .move_object_to_zone(card_id, ZoneId::Hand(player))
                .map_err(|e| {
                    GameStateError::InvalidCommand(format!(
                        "failed to move dredge card to hand: {:?}",
                        e
                    ))
                })?;

            // Step 5: Emit Dredged event.
            // Step 6: Do NOT increment cards_drawn_this_turn (CR 702.52a).
            events.push(GameEvent::Dredged {
                player,
                card_new_id: new_hand_id,
                milled: dredge_n,
            });

            Ok(events)
        }
    }
}

/// Perform a normal draw for `player`, bypassing the dredge check.
///
/// Called when a player declines dredge (`ChooseDredge { card: None }`). We
/// re-check other WouldDraw replacement effects (SkipDraw etc.) but do NOT
/// re-offer dredge (the player just chose not to dredge this draw).
///
/// Mirrors the logic of `turn_actions::draw_card` but skips the dredge
/// portion of `check_would_draw_replacement`.
fn draw_card_skipping_dredge(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::rules::events::LossReason;

    // Eliminated / conceded players cannot draw.
    if let Some(p) = state.players.get(&player) {
        if p.has_lost || p.has_conceded {
            return Ok(vec![]);
        }
    }

    // Check non-dredge WouldDraw replacement effects.
    use crate::state::replacement_effect::{
        PlayerFilter, ReplacementModification, ReplacementTrigger,
    };
    let trigger = ReplacementTrigger::WouldDraw {
        player_filter: PlayerFilter::Specific(player),
    };
    let applicable = find_applicable(state, &trigger, &HashSet::new());
    let action = determine_action(state, &applicable, player, "draw a card");

    match action {
        ReplacementResult::AutoApply(id) => {
            let modification = state
                .replacement_effects
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.modification.clone());
            if matches!(modification, Some(ReplacementModification::SkipDraw)) {
                return Ok(vec![GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: "skip that draw".to_string(),
                }]);
            }
        }
        ReplacementResult::NeedsChoice {
            player,
            choices,
            event_description,
        } => {
            return Ok(vec![GameEvent::ReplacementChoiceRequired {
                player,
                event_description,
                choices,
            }]);
        }
        ReplacementResult::NoApplicable => {}
    }

    // Perform the actual draw.
    let library_zone = ZoneId::Library(player);
    let top_id = match state.zones.get(&library_zone).and_then(|z| z.top()) {
        Some(id) => id,
        None => {
            // Library empty — player loses (CR 104.3b).
            if let Some(p) = state.players.get_mut(&player) {
                p.has_lost = true;
            }
            return Ok(vec![GameEvent::PlayerLost {
                player,
                reason: LossReason::LibraryEmpty,
            }]);
        }
    };

    let (new_id, _) = state.move_object_to_zone(top_id, ZoneId::Hand(player))?;
    if let Some(p) = state.players.get_mut(&player) {
        p.has_drawn_for_turn = true;
        p.cards_drawn_this_turn += 1;
    }

    let mut events = vec![GameEvent::CardDrawn {
        player,
        new_object_id: new_id,
    }];

    // CR 702.94a: Check if the just-drawn card has miracle and is the first draw.
    // (After the player declined dredge, this is a normal draw and miracle applies.)
    if let Some(miracle_event) =
        crate::rules::miracle::check_miracle_eligible(state, player, new_id)
    {
        events.push(miracle_event);
    }

    Ok(events)
}

// ── Regeneration helpers (CR 701.19) ─────────────────────────────────────

/// CR 701.19a/614.8: Check if a regeneration shield can replace destruction.
///
/// Returns `Some(shield_id)` if a regeneration shield exists for this permanent,
/// or `None` if no shield applies.
pub fn check_regeneration_shield(state: &GameState, object_id: ObjectId) -> Option<ReplacementId> {
    let trigger = ReplacementTrigger::WouldBeDestroyed {
        filter: ObjectFilter::SpecificObject(object_id),
    };
    let applicable = find_applicable(state, &trigger, &std::collections::HashSet::new());
    // Find the first applicable regeneration modification
    applicable.into_iter().find(|id| {
        state
            .replacement_effects
            .iter()
            .any(|e| e.id == *id && e.modification == ReplacementModification::Regenerate)
    })
}

/// CR 701.19a: Apply a regeneration shield to a permanent that would be destroyed.
///
/// Performs the regeneration replacement:
/// 1. Remove all damage marked on the permanent (CR 701.19a).
/// 2. Tap the permanent (CR 701.19a).
/// 3. If it's an attacking or blocking creature, remove it from combat (CR 701.19a).
/// 4. Remove the one-shot regeneration shield (consumed).
///
/// Returns the events to emit.
pub fn apply_regeneration(
    state: &mut GameState,
    object_id: ObjectId,
    shield_id: ReplacementId,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // 1. Remove all damage
    if let Some(obj) = state.objects.get_mut(&object_id) {
        obj.damage_marked = 0;
        obj.deathtouch_damage = false;
    }

    // 2. Tap the permanent
    if let Some(obj) = state.objects.get_mut(&object_id) {
        obj.status.tapped = true;
    }

    // 3. Remove from combat (if attacking or blocking)
    if let Some(combat) = &mut state.combat {
        combat.attackers.remove(&object_id);
        combat.blockers.remove(&object_id);
        // Also remove from damage_assignment_order as an attacker
        combat.damage_assignment_order.remove(&object_id);
        // Remove as a blocker from all damage assignment orders.
        // im::OrdMap has no iter_mut, so rebuild.
        let updated: im::OrdMap<_, _> = combat
            .damage_assignment_order
            .iter()
            .map(|(attacker_id, order)| {
                let filtered: Vec<_> = order
                    .iter()
                    .filter(|&&blocker| blocker != object_id)
                    .copied()
                    .collect();
                (*attacker_id, filtered)
            })
            .collect();
        combat.damage_assignment_order = updated;
    }

    // 4. Remove the one-shot shield (consumed)
    let keep: im::Vector<_> = state
        .replacement_effects
        .iter()
        .filter(|e| e.id != shield_id)
        .cloned()
        .collect();
    state.replacement_effects = keep;

    events.push(GameEvent::Regenerated {
        object_id,
        shield_id,
    });

    events
}

/// CR 702.89a: Check if an Aura with umbra armor can replace destruction.
///
/// Scans the battlefield for Auras with the `UmbraArmor` keyword that are
/// attached to the target permanent. Returns the `ObjectId`(s) of matching Auras.
///
/// If exactly one Aura matches it is auto-selected. If multiple match, the
/// enchanted permanent's controller must choose (CR 616.1) -- callers should
/// auto-select the first for now (TODO: add full CR 616.1 choice path).
///
/// Unlike regeneration (CR 701.19a), umbra armor is NOT a one-shot shield. It
/// does not need to be registered in `state.replacement_effects`. The Aura simply
/// needs to be on the battlefield with the keyword; when the Aura is destroyed by
/// this replacement the protection ends automatically.
pub fn check_umbra_armor(state: &GameState, object_id: ObjectId) -> Vec<ObjectId> {
    use crate::state::types::KeywordAbility;
    use crate::state::zone::ZoneId;

    let mut auras: Vec<ObjectId> = state
        .objects
        .iter()
        .filter_map(|(aura_id, aura_obj)| {
            // Must be on the battlefield.
            if !matches!(aura_obj.zone, ZoneId::Battlefield) {
                return None;
            }
            // CR 702.26b: phased-out permanents are treated as though they do not exist.
            // Exclude phased-out Auras so they cannot trigger umbra armor.
            if !aura_obj.is_phased_in() {
                return None;
            }
            // Must be attached to the target permanent.
            if aura_obj.attached_to != Some(object_id) {
                return None;
            }
            // Use layer-resolved characteristics to check for UmbraArmor
            // (respects Humility / Dress Down ability removal -- CR 702.89a).
            let chars = crate::rules::layers::calculate_characteristics(state, *aura_id)?;
            if chars.keywords.contains(&KeywordAbility::UmbraArmor) {
                Some(*aura_id)
            } else {
                None
            }
        })
        .collect();
    // Sort by ObjectId for deterministic selection when multiple Auras match
    // (im::HashMap iteration order is non-deterministic; replay correctness requires
    // stable ordering so the same Aura is always selected first -- CR 616.1 TODO).
    auras.sort();
    auras
}

/// CR 702.89a: Apply umbra armor replacement -- destroy the Aura instead of the enchanted permanent.
///
/// Instead of destroying the enchanted permanent:
/// 1. Remove all damage marked on the permanent (CR 702.89a).
/// 2. Clear the `deathtouch_damage` flag.
/// 3. Destroy the Aura (move to its owner's graveyard via `move_object_to_zone`).
///    Standard zone-change replacement effects on the Aura DO apply (e.g., commander redirect).
///
/// Unlike regeneration (CR 701.19a): the permanent is NOT tapped and NOT removed from combat.
/// Effects that say "can't be regenerated" do NOT prevent umbra armor (separate mechanics).
///
/// Returns the events to emit.
pub fn apply_umbra_armor(
    state: &mut GameState,
    protected_id: ObjectId,
    aura_id: ObjectId,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // 1. Remove all damage from the protected permanent and clear deathtouch flag.
    if let Some(obj) = state.objects.get_mut(&protected_id) {
        obj.damage_marked = 0;
        obj.deathtouch_damage = false;
    }

    // 2. Destroy the Aura -- move to its owner's graveyard.
    let aura_owner = match state.objects.get(&aura_id) {
        Some(obj) => obj.owner,
        None => return events, // Aura already gone -- nothing to do.
    };
    // Note: standard zone-change replacements on the Aura (e.g., commander redirect)
    // are handled by the existing pending_zone_changes / SBA flow, not here.
    // We simply move it directly (701.8a: destroy = move to graveyard).
    if state
        .move_object_to_zone(aura_id, ZoneId::Graveyard(aura_owner))
        .is_ok()
    {
        events.push(GameEvent::UmbraArmorApplied {
            protected_id,
            aura_id,
        });
    }

    events
}
