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
use super::events::{CombatDamageTarget, GameEvent};
use crate::state::error::GameStateError;
use crate::state::game_object::ObjectId;
use crate::state::player::PlayerId;
use crate::state::replacement_effect::{
    DamageTargetFilter, ObjectFilter, PendingDraw, PendingZoneChange, PlayerFilter,
    ReplacementEffect, ReplacementId, ReplacementModification, ReplacementTrigger,
};
use crate::state::types::{CardType, CounterType};
use crate::state::zone::ZoneId;
use crate::state::GameState;
use std::collections::HashSet;
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
/// This is a networked player command and therefore a **trust boundary** (invariant
/// #3): the sender is untrusted. It is rejected unless
///
/// 1. a pending zone change OR a pending draw (PB-DP5) exists whose affected
///    player (the CR 616.1 chooser) is `player`, and
/// 2. every id in `ids` is a replacement that is **currently applicable** to that
///    pending event (checked via [`find_applicable`], not by mere existence in
///    `state.replacement_effects`).
///
/// Without (1) a player could order another player's choice; without (2) a hostile
/// or buggy client could apply an arbitrary registered replacement's modification to
/// an event it does not apply to (e.g. redirect an unrelated dies event).
///
/// # Routing between a pending zone change and a pending draw (PB-DP5)
///
/// Two kinds of pending event can be outstanding for the same player at the same
/// time — a zone change (`PendingZoneChange`) and a draw (`PendingDraw`) — and
/// `Command::OrderReplacements` carries no discriminator naming which one an
/// answer is for. Routing is by **applicability**, which is total:
/// [`trigger_matches`] requires the effect's trigger and the event's trigger to
/// be the SAME [`ReplacementTrigger`] variant, so a `WouldChangeZone`
/// replacement can never be applicable to a draw and vice versa — the two
/// candidate sets are provably disjoint. A well-formed answer therefore names
/// exactly one of the two, and the check that decides "is this a legal answer"
/// is the same check that decides "which question is this answering" — no new
/// trust surface.
///
/// # A `PendingDraw` entry can be either a CR 616.1 deferral OR a dredge
/// offer (PB-DX2, `pb-plan-DX2.md` §3.3)
///
/// The disjointness argument above is about `PendingZoneChange` vs
/// `PendingDraw` — it does **not** extend to the two things that can now
/// populate a `PendingDraw` slot, because both a `NeedsChoice` deferral and a
/// dredge (`DredgeAvailable`) offer register as the SAME `ReplacementTrigger::
/// WouldDraw` variant. `Command::OrderReplacements` and `Command::ChooseDredge`
/// therefore share one undiscriminated queue, and both can legally land on
/// EITHER origin. Enumerated (all four cells are reachable and all four are
/// CR-legal, so none of them is "fixed" here — they are documented):
///
/// | answer | entry origin | outcome |
/// |---|---|---|
/// | `OrderReplacements` | `NeedsChoice` | unchanged, the original design |
/// | `OrderReplacements` | dredge (`DredgeAvailable`) | every ordered id must pass [`find_applicable`] first, so this can only name a genuinely applicable non-dredge `WouldDraw` replacement — "declined dredge, applied a legal replacement instead" (CR 616.1e) |
/// | `ChooseDredge { None }` | `NeedsChoice` | declines dredge for THIS draw and resumes it with the entry's own bookkeeping (re-checking other replacements) — see `resolve_declined_pending_draw` |
/// | `ChooseDredge { Some }` | `NeedsChoice` | the `Some` arm validates only that the named card is dredge-eligible against the player's OWN graveyard/library — byte-for-byte `check_would_draw_replacement`'s own predicate — so this is CR 616.1e's "any of the applicable replacements may be chosen", not a bypass |
///
/// `position(|p| p.player == player)` here and in `handle_choose_dredge` both
/// take the FIFO (oldest) entry for the player. **This is real, not
/// vacuous** (re-review Finding R1, `pb-review-DX2.md` — corrects a prior
/// version of this note that claimed `perform_one_draw`'s discharge made a
/// second entry structurally impossible): a `NeedsChoice`-origin entry
/// re-raised INSIDE a discharge can coexist with the entry the discharge's
/// own caller then pushes, so `player` CAN have 2+ outstanding entries —
/// see `perform_one_draw`'s "Per-player invariant" doc and `OOS-DX2-3`
/// (reopened). Both this function and `handle_choose_dredge` deliberately
/// answer the OLDEST one first.
///
/// Order of evaluation: zone change first, then draw. This is pure preservation
/// of pre-PB-DP5 behavior — byte-for-byte for any existing test — and, because
/// the two candidate sets are disjoint, can never actually misroute a
/// well-formed draw answer: it simply fails the zone-change applicability check
/// and falls through to the draw arm below.
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
    // Validate all IDs exist in the state (a cheap, precise error before the
    // applicability check below).
    for id in &ids {
        if !state.replacement_effects.iter().any(|e| e.id == *id) {
            return Err(GameStateError::InvalidCommand(format!(
                "replacement effect {:?} not found",
                id
            )));
        }
    }
    // ── 1. Try a pending zone change (byte-for-byte pre-PB-DP5 behavior). ──
    let zone_change_idx = state
        .pending_zone_changes
        .iter()
        .position(|p| p.affected_player == player);
    if let Some(pending_idx) = zone_change_idx {
        // CR 616.1/614.5: every ordered id must be applicable to THIS pending
        // event, taking into account replacements already applied in this
        // chain. Reconstruct the pending event's trigger and consult
        // `find_applicable`.
        let pending = &state.pending_zone_changes[pending_idx];
        let already_applied: HashSet<ReplacementId> =
            pending.already_applied.iter().copied().collect();
        let event_trigger = ReplacementTrigger::WouldChangeZone {
            from: Some(pending.original_from),
            to: pending.original_destination,
            filter: ObjectFilter::SpecificObject(pending.object_id),
        };
        let applicable = find_applicable(state, &event_trigger, &already_applied);
        if ids.iter().all(|id| applicable.contains(id)) {
            // All checks passed — resolve the pending zone change with the
            // chosen order.
            let first_id = ids[0];
            return resolve_pending_zone_change(state, first_id, pending_idx);
        }
    }
    // ── 2. Try a pending draw (PB-DP5, CR 616.1 / 614.11). ──
    let draw_idx = state.pending_draws.iter().position(|p| p.player == player);
    if let Some(pending_idx) = draw_idx {
        let pending = &state.pending_draws[pending_idx];
        let already_applied: HashSet<ReplacementId> =
            pending.already_applied.iter().copied().collect();
        let event_trigger = ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(pending.player),
        };
        let applicable = find_applicable(state, &event_trigger, &already_applied);
        if ids.iter().all(|id| applicable.contains(id)) {
            let first_id = ids[0];
            return resolve_pending_draw(state, first_id, pending_idx);
        }
    }
    // ── 3. Neither pending kind accepted every ordered id. ──
    if zone_change_idx.is_none() && draw_idx.is_none() {
        return Err(GameStateError::InvalidCommand(format!(
            "player {:?} is not the affected player of any pending replacement choice",
            player
        )));
    }
    Err(GameStateError::InvalidCommand(format!(
        "none of the ordered replacement ids {:?} are applicable to player {:?}'s pending \
         replacement choice (zone change pending: {}, draw pending: {})",
        ids,
        player,
        zone_change_idx.is_some(),
        draw_idx.is_some()
    )))
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
        // CR 611.2b: Active until the specified player's next turn begins.
        EffectDuration::UntilYourNextTurn(_) => true,
        // CR 611.2b/c (PB-EF9): no card today authors a *replacement* effect with this
        // duration (only continuous effects, via GainControl/ApplyContinuousEffect) —
        // this arm exists solely for exhaustiveness. Mirror the continuous-effect arm
        // in layers.rs::is_effect_active: always "active" here; if a future replacement
        // effect ever needs this duration, expiry must be added as its own one-shot
        // pass (never a live control check), the same way
        // `expire_while_you_control_source_effects` does for continuous effects.
        EffectDuration::WhileYouControlSource(_) => true,
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
        // CR 122.6/614.1: Counter placement replacement matching.
        // placer_filter, receiver_filter, and (PB-CD) counter_filter must all match.
        (
            ReplacementTrigger::WouldPlaceCounters {
                placer_filter: eff_placer,
                receiver_filter: eff_receiver,
                counter_filter: eff_counter,
            },
            ReplacementTrigger::WouldPlaceCounters {
                placer_filter: evt_placer,
                receiver_filter: evt_receiver,
                counter_filter: evt_counter,
            },
        ) => {
            event_player_matches_filter(evt_placer, eff_placer)
                && event_object_matches_filter(state, evt_receiver, eff_receiver)
                && event_counter_matches_filter(evt_counter, eff_counter)
        }
        // CR 111.1/614.1: Token creation replacement matching.
        (
            ReplacementTrigger::WouldCreateTokens {
                controller_filter: eff_filter,
            },
            ReplacementTrigger::WouldCreateTokens {
                controller_filter: evt_filter,
            },
        ) => event_player_matches_filter(evt_filter, eff_filter),
        // CR 701.23/614.1: Library search replacement matching.
        (
            ReplacementTrigger::WouldSearchLibrary {
                searcher_filter: eff_filter,
            },
            ReplacementTrigger::WouldSearchLibrary {
                searcher_filter: evt_filter,
            },
        ) => event_player_matches_filter(evt_filter, eff_filter),
        // CR 614.1: Life loss replacement matching.
        (
            ReplacementTrigger::WouldLoseLife {
                player_filter: eff_filter,
            },
            ReplacementTrigger::WouldLoseLife {
                player_filter: evt_filter,
            },
        ) => event_player_matches_filter(evt_filter, eff_filter),
        // CR 701.34: Proliferate replacement matching.
        (
            ReplacementTrigger::WouldProliferate {
                player_filter: eff_filter,
            },
            ReplacementTrigger::WouldProliferate {
                player_filter: evt_filter,
            },
        ) => event_player_matches_filter(evt_filter, eff_filter),
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
        // CR 613.1d: Use layer-resolved types for replacement effect applicability.
        ObjectFilter::AnyCreature => state
            .objects
            .get(&obj_id)
            .map(|_| {
                // SR-14: obj_id is proven present by the enclosing `.map`, so
                // calculate_characteristics is total here (CR 613.1d).
                crate::rules::layers::expect_characteristics(state, obj_id)
                    .card_types
                    .contains(&CardType::Creature)
            })
            .unwrap_or(false),
        ObjectFilter::HasCardType(ct) => state
            .objects
            .get(&obj_id)
            .map(|_| {
                // SR-14: obj_id is proven present by the enclosing `.map` (CR 613.1d).
                crate::rules::layers::expect_characteristics(state, obj_id)
                    .card_types
                    .contains(ct)
            })
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
        // PB-CD: layer-resolved creature-type check + controller equality.
        // CR 613.1d: use layer-resolved types for replacement applicability.
        ObjectFilter::CreatureControlledBy(player_id) => state
            .objects
            .get(&obj_id)
            .map(|o| {
                // SR-14: obj_id is proven present by the enclosing `.map` (CR 613.1d).
                let is_creature = crate::rules::layers::expect_characteristics(state, obj_id)
                    .card_types
                    .contains(&CardType::Creature);
                is_creature && o.controller == *player_id
            })
            .unwrap_or(false),
        // PB-EWC-D: layer-resolved subtype + controller equality (CR 613.1d).
        // Used for "Each [Subtype] you control" receiver filters (Dragonstorm Globe).
        ObjectFilter::CreatureControlledByOfSubtype {
            controller: player_id,
            subtype,
        } => state
            .objects
            .get(&obj_id)
            .map(|o| {
                // SR-14: obj_id is proven present by the enclosing `.map` (CR 613.1d).
                let chars = crate::rules::layers::expect_characteristics(state, obj_id);
                let is_creature = chars.card_types.contains(&CardType::Creature);
                let has_subtype = chars.subtypes.contains(subtype);
                is_creature && has_subtype && o.controller == *player_id
            })
            .unwrap_or(false),
    }
}
/// PB-CD: Check if the event's counter type matches the effect's counter filter.
///
/// Effect `None` = no counter-type restriction (matches any counter type — Vorinclex,
/// Pir, Lae'zel — "one or more counters").
/// Effect `Some(t)` = only matches when the event also has `Some(t)` with equal type
/// (Hardened Scales, Conclave Mentor, Corpsejack Menace — "+1/+1 counters" only).
///
/// The event side is always `Some(t)` because `apply_counter_replacement` constructs
/// the event trigger from a concrete counter type. We still handle the `None` event
/// case defensively (returns false for typed effects).
fn event_counter_matches_filter(
    event_filter: &Option<crate::state::types::CounterType>,
    effect_filter: &Option<crate::state::types::CounterType>,
) -> bool {
    match (event_filter, effect_filter) {
        (_, None) => true,
        (Some(evt), Some(eff)) => evt == eff,
        (None, Some(_)) => false,
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
/// Bind a PlayerFilter placeholder to the actual controller's PlayerId.
///
/// Card definitions use `Specific(PlayerId(0))` as a placeholder for "the controller"
/// and `OpponentsOf(PlayerId(0))` for "opponents of the controller". At registration
/// time, replace these with the actual controller's PlayerId.
fn bind_player_filter(filter: &PlayerFilter, controller: PlayerId) -> PlayerFilter {
    match filter {
        PlayerFilter::Specific(PlayerId(0)) => PlayerFilter::Specific(controller),
        PlayerFilter::OpponentsOf(PlayerId(0)) => PlayerFilter::OpponentsOf(controller),
        other => other.clone(),
    }
}
/// Bind an ObjectFilter placeholder to the actual controller's PlayerId.
///
/// Card definitions use `ControlledBy(PlayerId(0))` as a placeholder for
/// "controlled by the controller". At registration time, replace with the
/// actual controller's PlayerId.
///
/// Public for test access (see `tests/primitive_pb_ewcd.rs`); not part of the
/// engine's runtime API.
pub fn bind_object_filter(filter: &ObjectFilter, controller: PlayerId) -> ObjectFilter {
    match filter {
        ObjectFilter::ControlledBy(PlayerId(0)) => ObjectFilter::ControlledBy(controller),
        // PB-CD: bind CreatureControlledBy placeholder to the actual controller.
        ObjectFilter::CreatureControlledBy(PlayerId(0)) => {
            ObjectFilter::CreatureControlledBy(controller)
        }
        // PB-EWC-D (E2 fix from pb-review-EWC.md): bind OwnedByOpponentsOf placeholder.
        // Symmetric to the direct pattern-match in register_permanent_replacement_abilities
        // for WouldChangeZone — now WouldEnterBattlefield (and any other site that routes
        // through bind_object_filter) handles the same rebind.
        ObjectFilter::OwnedByOpponentsOf(PlayerId(0)) => {
            ObjectFilter::OwnedByOpponentsOf(controller)
        }
        // PB-EWC-D: bind CreatureControlledByOfSubtype placeholder to the actual controller.
        ObjectFilter::CreatureControlledByOfSubtype {
            controller: PlayerId(0),
            subtype,
        } => ObjectFilter::CreatureControlledByOfSubtype {
            controller,
            subtype: subtype.clone(),
        },
        other => other.clone(),
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
    ///
    /// **The engine does NOT block on this** (PB-DX2, closing OOS-DP7-2's half
    /// of the claim). The draw does not occur; the caller (`perform_one_draw`)
    /// records a `PendingDraw` entry for the player and the draw SEQUENCE stops
    /// (CR 614.11a). `Command::ChooseDredge` is legal ONLY while that entry
    /// stands and CONSUMES it (`handle_choose_dredge`) — priority, SBAs and
    /// step advancement all continue in the meantime.
    DredgeAvailable(GameEvent),
}
/// CR 614.11: Check WouldDraw replacement effects before performing a draw.
///
/// Finds applicable replacements for `player` drawing a card, determines the
/// action per CR 616.1, and returns a `DrawAction` indicating how the draw
/// should proceed.
///
/// Also checks for dredge-eligible cards in the player's graveyard (CR 702.52).
/// Dredge takes priority as a "may" replacement — if dredge options are
/// available, this draw is REPLACED by an outstanding `PendingDraw`
/// obligation (see `perform_one_draw`'s `DredgeAvailable` arm) rather than
/// performed immediately. **The engine does not pause or block** (fix-cycle
/// Finding 1, `pb-review-DX2.md`: this comment used to say "pauses", which
/// was never true of the code — see `GameEvent::DredgeChoiceRequired`'s doc
/// for the full deadline semantics). If the player declines dredge, the
/// normal draw path re-checks other WouldDraw replacements.
///
/// Called (via `perform_one_draw`) from `turn_actions::draw_card`,
/// `effects::draw_cards_for_player` (renamed from `draw_one_card` in PB-DP5),
/// and `replacement::handle_choose_dredge`'s decline arm (PB-DX2 removed the
/// standalone helper this call used to go through and folded the call
/// directly into the gated handler) to keep all draw paths consistent.
///
/// `already_applied` (CR 614.5) is threaded in so a CR 616.1f re-check (from
/// `resolve_pending_draw`) does not re-offer an effect already applied to this
/// draw event. `offer_dredge` is `false` on a resume (PB-DP5 §3.3): re-offering
/// dredge mid-chain would restart a CR 616.1 application the player already
/// began, and there is nowhere to record a second pause.
pub fn check_would_draw_replacement(
    state: &GameState,
    player: PlayerId,
    already_applied: &HashSet<ReplacementId>,
    offer_dredge: bool,
) -> DrawAction {
    use crate::state::replacement_effect::{
        PlayerFilter, ReplacementModification, ReplacementTrigger,
    };
    use crate::state::types::KeywordAbility;
    // CR 702.52a: Scan the player's graveyard for dredge-eligible cards.
    // A card is eligible if:
    //   1. It has KeywordAbility::Dredge(n) in its keywords.
    //   2. The player has >= n cards in their library (CR 702.52b).
    if offer_dredge {
        let graveyard_zone = ZoneId::Graveyard(player);
        let library_zone = ZoneId::Library(player);
        // SR-14: the library zone is built before turn 1 and never removed (ground truth 2).
        let library_count = state
            .expect_zone(&library_zone)
            .map(|z| z.len())
            .unwrap_or(0);
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
    }
    // CR 616.1f: "Once the chosen effect has been applied, this process is
    // repeated ... until there are no more left to apply." A single
    // `determine_action` dispatch is NOT always terminal: CR 616.1a forces an
    // `AutoApply` when exactly one applicable replacement is a
    // self-replacement, even if 2+ replacements are applicable overall (PB-DP5
    // review Finding 1). If that auto-applied replacement is not `SkipDraw`
    // (the only modification this path honours), the pre-fix code returned
    // `Proceed` immediately and silently dropped every other applicable
    // replacement — including a `SkipDraw` that CR 616.1f says must then be
    // applied, which would mean no card is drawn at all. Mirror the
    // `check_zone_change_replacement` loop (`:984-1032`) so the same class of
    // effect is repeated and excluded (`applied.insert`) rather than dispatched
    // once. Bounded: `applied` strictly grows each iteration and
    // `find_applicable` excludes its members, so this runs at most
    // `state.replacement_effects.len()` times.
    let mut applied: HashSet<ReplacementId> = already_applied.clone();
    loop {
        let trigger = ReplacementTrigger::WouldDraw {
            player_filter: PlayerFilter::Specific(player),
        };
        let applicable = find_applicable(state, &trigger, &applied);
        let action = determine_action(state, &applicable, player, "draw a card");
        match action {
            ReplacementResult::NoApplicable => return DrawAction::Proceed,
            ReplacementResult::AutoApply(id) => {
                let modification = state
                    .replacement_effects
                    .iter()
                    .find(|e| e.id == id)
                    .map(|e| e.modification.clone());
                if matches!(modification, Some(ReplacementModification::SkipDraw)) {
                    // CR 614.10: Replace the draw with nothing — no card moved, no CardDrawn.
                    return DrawAction::Skip(GameEvent::ReplacementEffectApplied {
                        effect_id: id,
                        description: "skip that draw".to_string(),
                    });
                } else {
                    // CR 614.5 / 616.1f: this replacement has no draw-level
                    // effect today (only `SkipDraw` is honoured, OOS-DP5-6/8),
                    // but it still applied — mark it so it is excluded from the
                    // re-check, then repeat with only what is NOW applicable.
                    applied.insert(id);
                    continue;
                }
            }
            ReplacementResult::NeedsChoice {
                player,
                choices,
                event_description,
            } => {
                // CR 616.1: Multiple WouldDraw replacements apply — player must choose order.
                return DrawAction::NeedsChoice(GameEvent::ReplacementChoiceRequired {
                    player,
                    event_description,
                    choices,
                });
            }
        }
    }
}
/// What happened to one attempted draw (CR 121.1 / 614.11 / 616.1).
pub(crate) enum DrawStepOutcome {
    /// A card moved to hand.
    Completed,
    /// A replacement (CR 614.10 `SkipDraw`) consumed the draw. No card moved.
    Replaced,
    /// CR 616.1: 2+ replacements applied; a `PendingDraw` was pushed and a
    /// `ReplacementChoiceRequired` emitted. The caller MUST stop the sequence
    /// (CR 614.11a) — see `PendingDraw.remaining`.
    Deferred,
    /// CR 702.52a: a `DredgeChoiceRequired` was emitted and a `PendingDraw`
    /// entry was recorded (PB-DX2, closing OOS-DP5-7). The caller MUST STOP
    /// the sequence (CR 614.11a) — this reverses the pre-PB-DX2 behaviour,
    /// under which the caller did NOT stop and a multi-draw sequence
    /// destroyed every draw but the last-answered one. Dredge is only ever
    /// offered with `offer_dredge: true`, i.e. never mid-resume (PB-DP5 plan
    /// §3.3) — the resume paths pass `false` and thread the entry's own
    /// `already_applied`/`remaining` instead.
    DredgeOffered,
    /// CR 104.3b: the library was empty; `PlayerLost` emitted.
    LostToEmptyLibrary,
}
/// CR 121.1 / 121.2 / 614.11 / 616.1: perform ONE card draw for `player`.
///
/// This is the single completion body shared by `turn_actions::draw_card`,
/// `effects::draw_cards_for_player` (the sequence loop, formerly `draw_one_card`),
/// `replacement::handle_choose_dredge`'s decline arm, and
/// `resolve_pending_draw`'s resume path — replacing three near-duplicate
/// bodies (PB-DP5).
///
/// `already_applied` (CR 614.5) and `remaining_after` (CR 614.11a / 121.2, the
/// count of further draws in this player's current sequence) are threaded
/// through to `PendingDraw` if a `NeedsChoice` is hit.
///
/// **Known gap (OOS-DP5-9), inherited from the sibling zone-change path.** What is
/// threaded is the `already_applied` this function was *called* with, not any id
/// that `check_would_draw_replacement`'s own CR 616.1f re-check auto-applied on
/// the way to the `NeedsChoice`. So for an applicable set of
/// `{S: a non-SkipDraw self-replacement, X, Y}` — `determine_action` returns
/// `AutoApply(S)` under CR 616.1a, the re-check then yields `NeedsChoice` on
/// `{X, Y}` — the pushed `PendingDraw.already_applied` is empty, and on resume
/// `find_applicable` re-offers `S`. A client can therefore submit an id that was
/// never in the offered `choices` and have `S` applied twice. Unobservable today:
/// every non-`SkipDraw` draw modification is a game-state no-op (OOS-DP5-6 /
/// OOS-DP5-8), so "applied twice" and "applied once" are indistinguishable.
/// `check_zone_change_replacement` has the identical gap, documented there as the
/// registered M10 follow-up. Closing it means threading the re-check's local
/// `applied` set out of `check_would_draw_replacement` — folded into OOS-DP5-6,
/// which is where a draw modification first becomes observable.
///
/// `sets_has_drawn_for_turn` preserves a pre-existing, currently-unobservable
/// divergence between the three original bodies rather than silently unifying
/// it: `PlayerState::has_drawn_for_turn` is write-only dead state (never read by
/// any engine logic) but IS fed to the public state hash, so unifying the write
/// would move real game hashes for zero behavioural gain. `true` for
/// `turn_actions::draw_card` and `handle_choose_dredge`'s decline arm (which
/// both set it today); `false` for the effect-draw path (which never has).
///
/// # Per-player invariant — CORRECTED (re-review Finding R1, `pb-review-DX2.md`)
///
/// **The queue is NOT bounded to one entry per player.** An earlier version of
/// this doc claimed the discharge below made a second entry "structurally
/// impossible", reasoning from where the two `push_back` calls live rather
/// than when they run relative to the discharge — the fallacy R1 identifies.
/// Before this fix (fix-cycle Finding 1), a second offer for a player who
/// already owed an answer FOLDED into the existing entry
/// (`remaining += 1 + remaining_after`), which conserved the draw count but
/// let the obligation accumulate WITHOUT BOUND across turns and be cashed in
/// a single `ChooseDredge` at an arbitrary later moment, out of priority
/// (the review's concrete scenario: seven cards drawn during another
/// player's declare-blockers step). This function now DISCHARGES — not
/// folds — any stale entry for `player` as its very first action, before
/// even checking what this new draw requires; see the top of the body below
/// for why the discharge is unconditional rather than nested inside the
/// `DredgeAvailable` arm, and `resolve_declined_pending_draw` for how a
/// discharge plays out (identically to an explicit `ChooseDredge { None }`,
/// so the draw is never destroyed — only completed at a different moment
/// than a human answer would have chosen).
///
/// That discharge, however, re-enters this very function
/// (`resolve_declined_pending_draw` calls `perform_one_draw` with
/// `offer_dredge: false`), and the re-entrant call's OWN
/// `check_would_draw_replacement` can independently return `NeedsChoice` and
/// push a FRESH entry — CR 616.1f only excludes replacements that were
/// *applied*, not merely offered, so 2+ still-applicable `WouldDraw`
/// replacements stay applicable across the discharge. Control then returns
/// to the *outer* call, which pushes its own entry for the draw it was
/// originally asked to perform. Each `draw_card` (or resumed dredge answer)
/// for a player whose prior entries are all `NeedsChoice`-originated and
/// whose replacements remain applicable therefore GROWS the queue by exactly
/// one entry, without bound — pinned by
/// `pb_dx2_command_gates.rs::test_dx2_needschoice_redefer_grows_the_queue`.
/// **What the discharge DOES bound**: at most one **dredge-originated**
/// (`DredgeAvailable` arm) entry can exist per player, because that arm only
/// runs when `offer_dredge` is true, which is never the case on a re-entrant
/// discharge call — so a discharge can never itself mint a second dredge
/// offer. This is **OOS-DX2-3**, REOPENED (it was closed on the false
/// "structurally impossible" argument above; see the audit row for the
/// corrected statement).
///
/// What the discharge DOES close: an outstanding entry can no longer be
/// destroyed (folded/overwritten) by a later draw, and — corpus-permitting —
/// cannot be cashed at a later moment covering many intervening turns, since
/// the entry itself is resolved (not merely re-stamped) the instant another
/// draw arrives for the same player. It does NOT close the single-entry
/// version of the timing gap: an outstanding entry can still be answered, or
/// now auto-discharged, at an arbitrary later moment with no priority/step
/// check on `Command::ChooseDredge`. That residual is **OOS-DP5-2**'s
/// pre-existing "no deadline for `pending_draws`" finding, not a new one,
/// and stays out of scope for a wire-neutral fix.
///
/// **Corpus exposure today is zero**: no card definition registers a
/// `ReplacementTrigger::WouldDraw` replacement effect (the only source hit is
/// an `inert`-completeness note in `out_of_the_tombs.rs`), so a
/// `NeedsChoice`-originated `PendingDraw` cannot arise from any legal deck —
/// this growth path is latent, not live-wrong, and is not being engine-fixed
/// for that reason (see the audit row for the argument against a defensive
/// engine change here).
///
/// No internal loop *here*: like `resolve_pending_zone_change`'s single call to
/// `check_zone_change_replacement`, a `NeedsChoice` here defers to a *future*
/// `Command::OrderReplacements` round-trip rather than looping in-process — the
/// termination argument is the same one that function relies on: each round
/// strictly grows `already_applied`, `find_applicable` excludes elements of it,
/// so the total number of rounds is bounded by `state.replacement_effects.len()`.
/// The CR 616.1f re-check *within* one `NeedsChoice`-free dispatch is not this
/// function's job either: `check_would_draw_replacement` runs its own internal
/// loop (mirroring `check_zone_change_replacement`'s) so that a CR 616.1a
/// self-replacement `AutoApply` — which is not always terminal, since 2+
/// replacements can still be applicable overall — is followed by a re-check of
/// the remainder before this function ever sees a final `DrawAction` (PB-DP5
/// review Finding 1; fixed in the fix cycle, not left as a documented gap).
pub(crate) fn perform_one_draw(
    state: &mut GameState,
    player: PlayerId,
    offer_dredge: bool,
    sets_has_drawn_for_turn: bool,
    already_applied: HashSet<ReplacementId>,
    remaining_after: u32,
) -> (Vec<GameEvent>, DrawStepOutcome) {
    // Fix-cycle Finding 1 (pb-review-DX2.md, HIGH): discharge -- never fold
    // -- any STALE `PendingDraw` for `player` before this new draw is even
    // examined. This is unconditional (not nested inside the
    // `DredgeAvailable` arm below) so it fires regardless of what THIS draw
    // turns out to need -- gating it on the current draw also being a dredge
    // offer would leave a gap if the dredge card left the graveyard between
    // offers (e.g. exiled by another effect): the code would then never
    // re-enter that arm and the stale entry would sit forever. See this
    // function's doc for the resulting per-player invariant and the
    // discharge's relationship to `OOS-DX2-3` / `OOS-DP5-2`.
    let mut events = Vec::new();
    if let Some(i) = state.pending_draws.iter().position(|p| p.player == player) {
        let stale = state.pending_draws[i].clone();
        state.pending_draws.remove(i);
        events.extend(resolve_declined_pending_draw(state, player, stale));
    }
    let (draw_events, outcome) =
        match check_would_draw_replacement(state, player, &already_applied, offer_dredge) {
            DrawAction::DredgeAvailable(event) => {
                // CR 702.52a + 614.11a (PB-DX2, OOS-DP5-7): the offer
                // REPLACES this draw, so the draw is now an outstanding
                // obligation. Record it in the same `pending_draws` queue the
                // CR 616.1 deferral uses — see `handle_choose_dredge`, which
                // requires and CONSUMES an entry, and pb-plan-DX2.md §3.3 for
                // why one undiscriminated queue is sound. This push is always
                // into an EMPTY slot for `player` — the discharge above
                // guarantees it, so there is no fold/accumulate case here
                // anymore (fix-cycle Finding 1).
                //
                // Determinism (SR-9b): sort `already_applied` by
                // ReplacementId before storing — `HashSet` iteration order is
                // not stable and this field is hashed. Same reasoning as the
                // `NeedsChoice` arm below.
                let mut sorted: Vec<ReplacementId> = already_applied.into_iter().collect();
                sorted.sort_by_key(|id| id.0);
                state.pending_draws.push_back(PendingDraw {
                    player,
                    already_applied: sorted,
                    remaining: remaining_after,
                    sets_has_drawn_for_turn,
                });
                (vec![event], DrawStepOutcome::DredgeOffered)
            }
            DrawAction::Skip(event) => (vec![event], DrawStepOutcome::Replaced),
            DrawAction::NeedsChoice(event) => {
                // CR 616.1e: 2+ replacements apply — record the pending state
                // so a future `Command::OrderReplacements` (routed by
                // `handle_order_replacements` to `resolve_pending_draw`) can
                // resume this exact draw. As above, this push is always into
                // an empty slot for `player` (fix-cycle Finding 1).
                //
                // Determinism (SR-9b): sort by ReplacementId before storing.
                // `HashSet` iteration order is not stable and this field is
                // hashed — this is load-bearing, not cosmetic.
                let mut sorted: Vec<ReplacementId> = already_applied.into_iter().collect();
                sorted.sort_by_key(|id| id.0);
                state.pending_draws.push_back(PendingDraw {
                    player,
                    already_applied: sorted,
                    remaining: remaining_after,
                    sets_has_drawn_for_turn,
                });
                (vec![event], DrawStepOutcome::Deferred)
            }
            DrawAction::Proceed => {
                // CR 121.1: perform the draw. The eliminated/conceded guard runs
                // before this is reached at three of this function's four
                // callers: `turn_actions::draw_card`, `handle_choose_dredge`'s
                // decline arm (PB-DX2 step 0 discharges a dead player's entry
                // before the gate is even consulted), and `resolve_pending_draw`
                // (indirectly — `engine.rs` runs
                // `validate_player_active` on the `OrderReplacements` sender, and
                // the draw arm only routes to the player named by the pending
                // entry). `effects::draw_cards_for_player` has no such guard —
                // not a regression (the pre-PB-DP5 `draw_one_card` had none
                // either, review Finding 6) but worth flagging rather than
                // asserting a blanket guarantee that does not hold everywhere.
                //
                // NOTE (fix-cycle): the two early exits below are expressed as
                // the tail value of a nested `match`, not `return`, precisely
                // because a bare `return` here would skip the
                // `events.extend(draw_events)` step after this outer `match`
                // and silently drop any discharge events accumulated above.
                let library_zone = ZoneId::Library(player);
                // SR-14: the library zone is built pre-turn-1 and never removed
                // (ground truth 2); `top()` returning `None` is the legal CR 104.3c
                // empty case, not an absence.
                match state.expect_zone(&library_zone).and_then(|z| z.top()) {
                    None => {
                        // CR 104.3c: being required to draw more cards than remain
                        // in the library causes loss (R6 re-review: this cite was
                        // 104.3b, which is the SEPARATE life-total-<=0 loss rule).
                        // R5 re-review: if the discharge above already lost this
                        // player (its own empty-library arm ran first and set
                        // `has_lost`), do NOT emit a second `PlayerLost` for the
                        // same condition — Architecture Invariant 4, no
                        // phantom/duplicate events. This is a tail value, not a
                        // `return`, for the same reason as the NOTE above: a bare
                        // `return` here would skip `events.extend(draw_events)`
                        // and silently drop the discharge's own events.
                        if state.expect_player(player).is_some_and(|p| p.has_lost) {
                            (vec![], DrawStepOutcome::LostToEmptyLibrary)
                        } else {
                            // SR-14: players are never removed from state.players
                            // (ground truth 1).
                            if let Some(p) = state.expect_player_mut(player) {
                                p.has_lost = true;
                            }
                            (
                                vec![GameEvent::PlayerLost {
                                    player,
                                    reason: crate::rules::events::LossReason::LibraryEmpty,
                                }],
                                DrawStepOutcome::LostToEmptyLibrary,
                            )
                        }
                    }
                    Some(top_id) => {
                        // SR-14: `top_id` was just read from the live library top
                        // — the move cannot fail.
                        match state.expect_move_object_to_zone(top_id, ZoneId::Hand(player)) {
                            None => (vec![], DrawStepOutcome::Completed),
                            Some((new_id, _)) => {
                                // SR-14: players are never removed (ground truth 1).
                                if let Some(p) = state.expect_player_mut(player) {
                                    // CR 121.1: track draws-per-turn for Sylvan
                                    // Library and similar effects (CC#33).
                                    p.cards_drawn_this_turn += 1;
                                    if sets_has_drawn_for_turn {
                                        p.has_drawn_for_turn = true;
                                    }
                                }
                                let mut proceed_events = vec![GameEvent::CardDrawn {
                                    player,
                                    new_object_id: new_id,
                                }];
                                // CR 702.94a: check if the just-drawn card has
                                // miracle and is the first draw.
                                if let Some(miracle_event) =
                                    crate::rules::miracle::check_miracle_eligible(
                                        state, player, new_id,
                                    )
                                {
                                    proceed_events.push(miracle_event);
                                }
                                (proceed_events, DrawStepOutcome::Completed)
                            }
                        }
                    }
                }
            }
        };
    events.extend(draw_events);
    (events, outcome)
}
/// CR 702.52a: discharge a `PendingDraw` as though `player` declined dredge
/// for it — resume the replaced draw (re-checking other WouldDraw
/// replacements, but not dredge itself: a decline suppresses the automatic
/// re-offer for THIS draw, see `dredge.rs` test 10 and the "decline is not
/// sticky" note on `handle_choose_dredge`'s `None` arm), then perform the
/// rest of the sequence it belonged to (CR 614.11a).
///
/// Shared by two callers (fix-cycle Finding 1, `pb-review-DX2.md`):
/// `handle_choose_dredge`'s `None` arm, an EXPLICIT decline, and
/// `perform_one_draw`'s unconditional stale-entry discharge, an IMPLICIT one
/// — forced because a second, unrelated draw arrived before the player
/// answered the first offer. Both play out identically: the draw is never
/// destroyed, only completed at a different moment than a human answer would
/// have chosen.
///
/// Terminates, but NOT for the reason a prior version of this doc claimed
/// (re-review Finding R1, `pb-review-DX2.md`): `perform_one_draw`'s
/// discharge does NOT guarantee `pending_draws` is empty for `player` when
/// this function's recursive call re-enters it — if the player had `k > 1`
/// STALE entries queued (itself only reachable via the growth path documented
/// on `perform_one_draw`'s "Per-player invariant" section), the re-entrant
/// call finds and discharges the NEXT one, recursing again. The true bound:
/// each recursive level removes exactly one entry from `pending_draws` before
/// calling this function again, `pending_draws` only shrinks (never grows)
/// *within* one discharge chain — the growth described above happens only on
/// the UNWIND, via `push_back`, after the deepest call returns — so recursion
/// depth is bounded by `k`, the number of entries queued for `player` at the
/// moment the chain begins. `k` is itself unbounded ACROSS separate draws
/// (see `perform_one_draw`'s doc), so this is a real, not cosmetic, bound —
/// it is finite for any single call but grows with prior queue depth.
/// `perform_remaining_draws` below bounds its own loop by `pending.remaining`,
/// a `u32` captured up front, independently of this recursion.
fn resolve_declined_pending_draw(
    state: &mut GameState,
    player: PlayerId,
    pending: PendingDraw,
) -> Vec<GameEvent> {
    let (mut events, outcome) = perform_one_draw(
        state,
        player,
        false, // CR 702.52a: declining suppresses the automatic re-offer for
        // THIS draw (see the sticky-decline note referenced above).
        pending.sets_has_drawn_for_turn,
        pending.already_applied.iter().copied().collect(),
        pending.remaining,
    );
    // CR 614.11a: if this draw completed (not itself deferred again), perform
    // the rest of the sequence it belonged to.
    if !matches!(
        outcome,
        DrawStepOutcome::Deferred
            | DrawStepOutcome::LostToEmptyLibrary
            | DrawStepOutcome::DredgeOffered
    ) && pending.remaining > 0
    {
        events.extend(perform_remaining_draws(
            state,
            player,
            pending.remaining,
            pending.sets_has_drawn_for_turn,
        ));
    }
    events
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
        // SR-14: object_id is the subject of the current would-move event — the
        // caller is about to move it, so it is live here (not LKI).
        if let Some(obj) = state.expect_object(object_id) {
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
        // SR-14: object_id is the subject of the current would-move event (live).
        if let Some(obj) = state.expect_object(object_id) {
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
    // CR 616.1: the affected object's *controller* chooses which replacement to
    // apply (its owner only if it has no controller). `owner` is retained for
    // resolving per-player destination zones (graveyard/library/command are the
    // *owner's* zones, CR 400.6/404.2), but must not be used as the chooser: after
    // a control change (Act of Treason class) owner != controller. A zone change is
    // evaluated before the object moves, so `object_id` is still live here and its
    // `controller` field is the battlefield controller; for an object with no
    // controller (any non-battlefield zone) `move_object_to_zone` resets the field
    // to `owner`, so reading it yields the "owner fallback" automatically.
    let chooser = state
        .expect_object(object_id)
        .map(|o| o.controller)
        .unwrap_or(owner);
    // CR 616.1f: after applying one replacement, repeat with only the effects that
    // would now be applicable (to the *modified* event) until none apply. Redirects
    // are chained here without moving the object — the object moves once, to the
    // final destination the loop settles on. `applied` grows across hops so CR 614.5
    // (an effect applies at most once per event) holds. Interactive ordering among
    // 2+ simultaneously-applicable effects (a `NeedsChoice` reached mid-chain) is
    // M10 scope: it is returned as `ChoiceRequired` for the current modified event.
    let mut applied: HashSet<ReplacementId> = already_applied.clone();
    let mut current_to = to;
    let mut acc_events: Vec<GameEvent> = Vec::new();
    let mut first_applied: Option<ReplacementId> = None;
    loop {
        let trigger = ReplacementTrigger::WouldChangeZone {
            from: Some(from),
            to: current_to,
            filter: ObjectFilter::SpecificObject(object_id),
        };
        let applicable = find_applicable(state, &trigger, &applied);
        let description = format!(
            "{:?} would move from {:?} to {:?}",
            object_id, from, current_to
        );
        let id = match determine_action(state, &applicable, chooser, &description) {
            ReplacementResult::NoApplicable => {
                return finish_zone_redirect(current_to, owner, acc_events, first_applied);
            }
            ReplacementResult::AutoApply(id) => id,
            ReplacementResult::NeedsChoice {
                player,
                choices,
                event_description,
            } => {
                // First hop: unchanged behavior — defer immediately. Mid-chain
                // (first_applied.is_some()): interactive ordering of the effects
                // applicable to the modified event is M10 scope; hand back the
                // choice for that event. (already_applied threading through
                // ChoiceRequired is the registered M10 follow-up.)
                return ZoneChangeAction::ChoiceRequired {
                    player,
                    choices,
                    event_description,
                };
            }
        };
        let modification = state
            .replacement_effects
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.modification.clone());
        first_applied.get_or_insert(id);
        applied.insert(id);
        match modification {
            Some(ReplacementModification::RedirectToZone(zone_type)) => {
                acc_events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: format!("Redirected to {:?}", zone_type),
                });
                current_to = zone_type;
                // Loop: re-check the modified event (CR 616.1f).
            }
            Some(ReplacementModification::ShuffleIntoOwnerLibrary) => {
                // CR 701.20: Redirect to library AND shuffle the library.
                acc_events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: "Shuffled into owner's library".to_string(),
                });
                acc_events.push(GameEvent::LibraryShuffled { player: owner });
                current_to = crate::state::zone::ZoneType::Library;
                // Loop: re-check the modified event (CR 616.1f).
            }
            _ => {
                // Non-redirect modifications (EntersTapped, etc.) don't change the
                // zone. Terminal for zone-change interception.
                return finish_zone_redirect(current_to, owner, acc_events, first_applied);
            }
        }
    }
}
/// Assemble the terminal `ZoneChangeAction` for [`check_zone_change_replacement`]'s
/// CR 616.1f loop: `Proceed` if no replacement redirected the object, otherwise a
/// single `Redirect` to the settled destination carrying every chained
/// `ReplacementEffectApplied`/`LibraryShuffled` event.
fn finish_zone_redirect(
    current_to: crate::state::zone::ZoneType,
    owner: PlayerId,
    acc_events: Vec<GameEvent>,
    first_applied: Option<ReplacementId>,
) -> ZoneChangeAction {
    match first_applied {
        None => ZoneChangeAction::Proceed,
        Some(applied_id) => ZoneChangeAction::Redirect {
            to: resolve_zone_type_to_zone_id(current_to, owner),
            events: acc_events,
            applied_id,
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
    // CR 616.1: `pending.affected_player` is the *chooser* (the object's controller,
    // owner-fallback) recorded when the choice was raised. Destination zones and
    // owner-scoped events must use the true *owner* instead — after a control change
    // (Act of Treason class) the two differ, and a permanent always goes to its
    // owner's graveyard/library/command zone (CR 400.6/404.2/903.9). The object has
    // not moved yet, so it is still live and `owner` is stable.
    let owner = state
        .expect_object(pending.object_id)
        .map(|o| o.owner)
        .unwrap_or(pending.affected_player);
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
    // Determine the final destination (owner-scoped per CR 400.6/404.2).
    let dest = match &modification {
        ReplacementModification::RedirectToZone(zone_type) => {
            resolve_zone_type_to_zone_id(*zone_type, owner)
        }
        ReplacementModification::ShuffleIntoOwnerLibrary => {
            // CR 701.20: redirect to library and shuffle
            resolve_zone_type_to_zone_id(crate::state::zone::ZoneType::Library, owner)
        }
        _ => {
            // Non-redirect: use original destination
            resolve_zone_type_to_zone_id(pending.original_destination, owner)
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
        events.push(GameEvent::LibraryShuffled { player: owner });
    }
    // Re-check with the modified destination, using the stored original_from zone
    // so non-battlefield zone changes use the correct "from" zone (MR-M8-01).
    // Pass `owner` for destination resolution; the chooser is re-derived from the
    // object's controller inside the call (CR 616.1).
    let action = check_zone_change_replacement(
        state,
        pending.object_id,
        pending.original_from, // use stored from-zone, not hardcoded Battlefield
        new_to,
        owner,
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
            // CR 603.10a: capture LKI power before move_object_to_zone for SourcePowerAtLKI.
            // CR 603.10a / CR 613.1d: capture full characteristics for filtered death triggers.
            let oid = pending.object_id;
            // SR-14: oid is the pending object about to be moved just below — it is
            // still live at this point (the move has not happened yet). The inner
            // `calculate_characteristics` stays: `pre_chars` is deliberately kept as an
            // Option and threaded through as the LKI snapshot, so its `None` is not a
            // swallowed lookup.
            let (pre_move_controller, pre_death_counters, pre_death_power_repl, repl_pre_chars) =
                state
                    .expect_object(oid)
                    .map(|o| {
                        let pre_chars = crate::rules::layers::calculate_characteristics(state, oid);
                        let lki_power = pre_chars
                            .as_ref()
                            .and_then(|c| c.power)
                            .or(o.characteristics.power);
                        (o.controller, o.counters.clone(), lki_power, pre_chars)
                    })
                    .unwrap_or((owner, Default::default(), None, None));
            // Do the zone move
            if let Some((new_id, _old)) =
                state.expect_move_object_to_zone(pending.object_id, final_dest)
            {
                events.extend(zone_change_events(
                    state,
                    pending.object_id,
                    new_id,
                    final_dest,
                    owner,
                    pre_move_controller,
                    &pre_death_counters,
                    pre_death_power_repl,
                    repl_pre_chars,
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
                // Determinism (SR-9b), PB-DP9 fix-cycle Finding 4's widened
                // audit: this field is a `Vec` fed element-by-element into
                // `HashInto` and it is built from a `HashSet`, whose iteration
                // order is not stable. Sort, exactly as the sibling site in
                // `pending_draws` (see the "load-bearing, not cosmetic" note
                // there) already does -- this one had been missed.
                already_applied: {
                    let mut v: Vec<ReplacementId> = already_applied.into_iter().collect();
                    v.sort_by_key(|id| id.0);
                    v
                },
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
/// CR 614.11a / 121.2: perform the `remaining` further draws of the sequence a
/// deferred draw belonged to, stopping on a further deferral or an empty library.
///
/// Extracted from `resolve_pending_draw` by PB-DX2 so `handle_choose_dredge` and
/// `resolve_declined_pending_draw` can discharge the same obligation without
/// duplicating it. Behaviour is byte-for-byte the pre-PB-DX2 loop, including
/// `offer_dredge: false` (see OOS-DX2-2: each draw of a sequence is separately
/// replaceable under CR 702.52a, and suppressing dredge for the whole tail is a
/// pre-existing simplification this batch deliberately does not change).
///
/// Terminates in at most `remaining` iterations: `remaining` is a `u32` captured
/// before the loop and `perform_one_draw` never calls back into this function.
///
/// (Fix-cycle Finding 6, `pb-review-DX2.md`: this function is placed ABOVE
/// `resolve_pending_draw`'s own doc block below, not between it and `fn
/// resolve_pending_draw`, specifically because a doc comment attaches to the
/// item immediately following it — inserting an undocumented item in between
/// silently reassigns the preceding doc to the wrong function. That is
/// exactly what happened here at implement time and is why this note exists.)
fn perform_remaining_draws(
    state: &mut GameState,
    player: PlayerId,
    remaining: u32,
    sets_has_drawn_for_turn: bool,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for i in 0..remaining {
        let remaining_after = remaining - 1 - i;
        let (evts, out) = perform_one_draw(
            state,
            player,
            false,
            sets_has_drawn_for_turn,
            HashSet::new(),
            remaining_after,
        );
        events.extend(evts);
        if matches!(
            out,
            DrawStepOutcome::Deferred
                | DrawStepOutcome::LostToEmptyLibrary
                | DrawStepOutcome::DredgeOffered
        ) {
            break;
        }
    }
    events
}
/// Complete a pending draw after a player has chosen the replacement order
/// (PB-DP5, CR 616.1 / 614.11). Modelled on [`resolve_pending_zone_change`].
///
/// Applies the chosen replacement (emitting `ReplacementEffectApplied` for it
/// **before anything else** — this is the order discriminator: with two
/// `SkipDraw` replacements the resulting game state is identical either way,
/// but the event stream names the effect the player actually chose, so a test
/// can prove the chosen order was honoured rather than an arbitrary one). Then
/// re-checks for remaining applicable replacements (CR 616.1f) via a single
/// call to [`perform_one_draw`], and if the sequence this draw belonged to has
/// further draws (CR 614.11a, `PendingDraw.remaining`), resumes it.
///
/// # Termination
///
/// No unbounded loop: like `resolve_pending_zone_change`, a further
/// `NeedsChoice` here defers to a *future* `Command::OrderReplacements`
/// round-trip rather than looping in-process. `already_applied` strictly grows
/// by `chosen_id` on entry, and `find_applicable` excludes every id already in
/// it, so the number of rounds across the whole chain is bounded by
/// `state.replacement_effects.len()`. The `remaining` resume loop (step 2
/// below) is a `for i in 0..pending.remaining` over a `u32` captured before the
/// loop starts, so it terminates in exactly `pending.remaining` iterations or
/// fewer (it `break`s early on a further deferral or an empty library). There
/// is no mutual recursion: this function calls `perform_one_draw`, never the
/// reverse.
pub fn resolve_pending_draw(
    state: &mut GameState,
    chosen_id: ReplacementId,
    pending_idx: usize,
) -> Result<Vec<GameEvent>, GameStateError> {
    let pending = state.pending_draws[pending_idx].clone();
    let mut events = Vec::new();
    // Apply the chosen replacement.
    let modification = state
        .replacement_effects
        .iter()
        .find(|e| e.id == chosen_id)
        .map(|e| e.modification.clone())
        .ok_or_else(|| {
            GameStateError::InvalidCommand(format!("replacement effect {:?} not found", chosen_id))
        })?;
    let mut already_applied: HashSet<ReplacementId> =
        pending.already_applied.iter().copied().collect();
    already_applied.insert(chosen_id);
    events.push(GameEvent::ReplacementEffectApplied {
        effect_id: chosen_id,
        description: format!("{:?}", modification),
    });
    // Remove the pending entry now — a re-defer below (CR 616.1f finding 2+
    // still applicable) pushes a FRESH entry via `perform_one_draw` rather than
    // mutating this one in place, mirroring `resolve_pending_zone_change`.
    state.pending_draws.remove(pending_idx);
    let outcome = if matches!(modification, ReplacementModification::SkipDraw) {
        // CR 614.10 + CR 616.1f: the draw event has been replaced by nothing,
        // so there is no longer an event for a remaining replacement to
        // modify — "taking into account only replacement effects that would
        // NOW be applicable" yields the empty set. The chain ends here. No
        // card moves; `cards_drawn_this_turn` is NOT incremented (a replaced
        // draw is not a draw, CR 121.1).
        DrawStepOutcome::Replaced
    } else {
        // CR 616.1f re-check: this single call to `perform_one_draw` (which
        // calls `check_would_draw_replacement` with the grown
        // `already_applied`) IS the re-check — not because a non-`SkipDraw`
        // modification is a no-op (it is, but that alone would not close the
        // CR 616.1a self-replacement hole, PB-DP5 review Finding 1),
        // but because `check_would_draw_replacement` itself now runs its own
        // internal CR 616.1f loop over `determine_action` before returning a
        // final `DrawAction`, so whatever it hands back here already reflects
        // every applicable self-replacement having been excluded in turn.
        // Outcomes: `NoApplicable`/exhausted `AutoApply` chain → the draw is
        // performed (`Completed`); the chain ends on `AutoApply(SkipDraw)` →
        // stop, no draw (`Replaced`); `NeedsChoice` → a NEW `PendingDraw` is
        // pushed carrying the grown `already_applied` and the SAME
        // `remaining`, a second `ReplacementChoiceRequired` is emitted, and
        // this returns `Deferred`.
        let (evts, outcome) = perform_one_draw(
            state,
            pending.player,
            false, // offer_dredge: never re-offer mid-resume, PB-DP5 plan §3.3.
            pending.sets_has_drawn_for_turn,
            already_applied,
            pending.remaining,
        );
        events.extend(evts);
        outcome
    };
    // CR 614.11a: "all actions required by the replacement are completed, if
    // possible, before resuming the sequence." The replacement is complete
    // (not itself deferred) — if the sequence this draw belonged to has
    // further draws, resume it now. `LostToEmptyLibrary` must also stop the
    // resume (review Finding 5): `draw_cards_for_player`'s sequence loop
    // (`effects/mod.rs`) breaks on it too, and without this guard a second
    // iteration would hit the same empty library and emit a second
    // `GameEvent::PlayerLost` (CR 104.3b already resolved the loss once).
    if !matches!(
        outcome,
        DrawStepOutcome::Deferred
            | DrawStepOutcome::LostToEmptyLibrary
            | DrawStepOutcome::DredgeOffered // PB-DX2 / P3: a dredge offer now
                                             // records its own entry with the
                                             // correct `remaining`; resuming here
                                             // would double-count it.
    ) && pending.remaining > 0
    {
        events.extend(perform_remaining_draws(
            state,
            pending.player,
            pending.remaining,
            pending.sets_has_drawn_for_turn,
        ));
    }
    Ok(events)
}
// ── ETB replacement interception (Session 4) ──────────────────────────────
/// Which subset of `WouldEnterBattlefield` replacements an application pass covers.
///
/// CR 614.15 requires self-replacement effects to be applied before other
/// replacement effects. The two-pass split (self pass, then global pass)
/// guarantees that ordering: each pass is a full CR 614/616 application loop,
/// but `apply_self_etb_from_definition` runs the self pass before any call site
/// reaches `apply_etb_replacements` (the global pass).
#[derive(Clone, Copy, PartialEq, Eq)]
enum EtbPass {
    /// CR 614.15: self-replacement effects (`is_self_replacement == true`).
    SelfReplacements,
    /// Global (non-self) replacement effects registered by other permanents.
    Global,
}
/// CR 614/616: shared application loop for `WouldEnterBattlefield` replacements.
///
/// Repeatedly calls [`find_applicable`] (which orders self-replacements first per
/// CR 614.15) and [`determine_action`], applying one effect per iteration and
/// recording it in `already_applied` so CR 614.5 loop prevention holds — no
/// replacement is applied twice to the same enter-the-battlefield event.
///
/// `pass` restricts the loop to one subset (`SelfReplacements` or `Global`) so the
/// caller can guarantee self-replacements resolve before global ones (CR 614.15).
fn apply_etb_replacement_pass(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    pass: EtbPass,
) -> Vec<GameEvent> {
    let trigger = ReplacementTrigger::WouldEnterBattlefield {
        filter: ObjectFilter::SpecificObject(new_id),
    };
    let mut already_applied: HashSet<ReplacementId> = HashSet::new();
    let mut events = Vec::new();
    loop {
        // CR 614.5: `find_applicable` excludes already-applied effects; CR 614.15:
        // it returns self-replacements first. Restrict to this pass's subset.
        let applicable: Vec<ReplacementId> = find_applicable(state, &trigger, &already_applied)
            .into_iter()
            .filter(|id| {
                state
                    .replacement_effects
                    .iter()
                    .find(|e| e.id == *id)
                    .map(|e| match pass {
                        EtbPass::SelfReplacements => e.is_self_replacement,
                        EtbPass::Global => !e.is_self_replacement,
                    })
                    .unwrap_or(false)
            })
            .collect();
        let description = format!("{new_id:?} would enter the battlefield");
        let chosen = match determine_action(state, &applicable, controller, &description) {
            ReplacementResult::NoApplicable => break,
            ReplacementResult::AutoApply(id) => id,
            // Pre-M10: no interactive replacement ordering. ETB modifications
            // (EntersTapped, EntersWithCounters, ...) are commutative, so apply
            // the first choice deterministically; the CR 614.5 `already_applied`
            // loop applies the remaining effects on later iterations.
            ReplacementResult::NeedsChoice { choices, .. } => match choices.first() {
                Some(id) => *id,
                None => break,
            },
        };
        let modification_source = state
            .replacement_effects
            .iter()
            .find(|e| e.id == chosen)
            .map(|e| (e.modification.clone(), e.source));
        already_applied.insert(chosen);
        if let Some((modification, replacement_source)) = modification_source {
            events.extend(emit_etb_modification(
                state,
                new_id,
                controller,
                Some(chosen),
                Some(modification),
                replacement_source,
            ));
        }
    }
    events
}
/// CR 614.12 / 614.15: Register and apply self-ETB replacement abilities from a card definition.
///
/// Called immediately after a permanent enters the battlefield (before emitting
/// the ETB event). Looks up the card definition and, for each
/// `AbilityDefinition::Replacement` ability with a `WouldEnterBattlefield` trigger
/// and `is_self: true`, registers a `ReplacementEffect` in `state.replacement_effects`,
/// then applies it through the replacement-effect framework
/// ([`find_applicable`] / [`determine_action`] via [`apply_etb_replacement_pass`]).
///
/// MR-M8-12: self-ETB replacements previously bypassed the framework (applied
/// inline). They now route through it, so they participate in CR 614.15 self-first
/// ordering and CR 614.5 loop prevention exactly like global ETB replacements.
/// Self-replacements are applied here (the self pass); global ETB replacements are
/// applied by the immediately-following `apply_etb_replacements` call (the global
/// pass) — preserving CR 614.15's "self-replacements first" rule by call order.
///
/// The registered effects use `duration: WhileSourceOnBattlefield`; once the
/// permanent leaves the battlefield they are garbage-collected (see MR-M8-16).
/// While the permanent remains, the effect is inert — its `SpecificObject` filter
/// can never match a second enter event (CR 400.7: re-entry creates a new object).
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
    // PB-RS4 (CR 614.12 / 712.8e): gather from the face that is actually showing.
    // CR 400.7: an earlier same-batch ETB replacement (or a harness call) may leave
    // no live object for this id -- a departed permanent has no face, which is a
    // legal fizzle, not an engine bug. Mirrors `queue_carddef_etb_triggers`'s guard.
    let entering_is_transformed = state
        .fizzle_object(new_id)
        .map(|o| o.is_transformed)
        .unwrap_or(false);
    use crate::state::continuous_effect::EffectDuration;
    let mut evts = Vec::new();
    // MR-M8-12: register each self-ETB WouldEnterBattlefield replacement into
    // `state.replacement_effects` so it is applied through the framework below
    // (rather than inline) — participating in CR 614.15 ordering / CR 614.5.
    //
    // PB-RS4 (CR 614.12 / 712.8d/e, OOS-RS-3): gather from the face that is
    // actually showing. A permanent entering back-face-up (disturb --
    // resolution.rs:665; stack craft -- resolution.rs:7276) has only its back
    // face's characteristics, so only that face's self-ETB replacements apply.
    // Closes the PB-OS4b-era limitation this comment used to describe.
    for ability in def.effective_abilities(entering_is_transformed) {
        if let AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldEnterBattlefield { .. },
            modification,
            is_self: true,
            unless_condition,
        } = ability
        {
            // CR 614.1c: "enters tapped unless [condition]" — if the condition
            // is met, do not register the replacement (permanent enters untapped).
            if let Some(condition) = unless_condition {
                let ctx = crate::effects::EffectContext::new(controller, new_id, vec![]);
                if crate::effects::check_condition(state, condition, &ctx) {
                    continue;
                }
            }
            let id = state.next_replacement_id();
            state.replacement_effects.push_back(ReplacementEffect {
                id,
                source: Some(new_id),
                controller,
                duration: EffectDuration::WhileSourceOnBattlefield,
                is_self_replacement: true,
                // Bind the placeholder filter to this specific object so the
                // effect only matches this permanent's enter event.
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::SpecificObject(new_id),
                },
                modification: modification.clone(),
            });
        }
    }
    // CR 614.15: apply the self-replacement pass now (before the caller's
    // `apply_etb_replacements` global pass).
    evts.extend(apply_etb_replacement_pass(
        state,
        new_id,
        controller,
        EtbPass::SelfReplacements,
    ));
    // CR 306.5b: "This permanent enters with a number of loyalty counters on it
    // equal to its printed loyalty number." This is an intrinsic replacement effect.
    // CR 306.5b: back-face starting loyalty is OOS-OS4-1 / rider-seed queue item R10
    // -- deliberately front-only here (PB-RS4 does not widen into it).
    if let Some(loyalty) = def.starting_loyalty {
        if loyalty > 0 {
            // SR-14: new_id is the permanent that just entered — live here.
            if let Some(obj) = state.expect_object_mut(new_id) {
                let current = obj
                    .counters
                    .get(&CounterType::Loyalty)
                    .copied()
                    .unwrap_or(0);
                obj.counters.insert(CounterType::Loyalty, current + loyalty);
            }
        }
    }
    // CR 714.3a: As a Saga enters the battlefield, its controller puts a lore counter on it.
    // This is a turn-based action that happens as part of the ETB event.
    // PB-RS4 (CR 714.3a / 712.8d/e): only the face that is actually showing can make
    // this permanent a Saga.
    let has_saga_chapters = def
        .effective_abilities(entering_is_transformed)
        .iter()
        .any(|a| matches!(a, AbilityDefinition::SagaChapter { .. }));
    if has_saga_chapters {
        // SR-14: new_id is the permanent that just entered — live here.
        if let Some(obj) = state.expect_object_mut(new_id) {
            let current = obj.counters.get(&CounterType::Lore).copied().unwrap_or(0);
            obj.counters.insert(CounterType::Lore, current + 1);
        }
        // Fire chapter triggers for the initial lore counter (counter went from 0 to 1).
        #[allow(clippy::needless_borrow)]
        let chapter_evts = fire_saga_chapter_triggers(state, new_id, controller, 0, 1, &def);
        evts.extend(chapter_evts);
    }
    // CR 716.2d: When a Class enters the battlefield, set its level to 1.
    // PB-RS4: face-aware for internal consistency; Classes are not DFCs today, so
    // this swap is a no-op in practice (entering_is_transformed is always false).
    let has_class_levels = def
        .effective_abilities(entering_is_transformed)
        .iter()
        .any(|a| matches!(a, AbilityDefinition::ClassLevel { .. }));
    if has_class_levels {
        // SR-14: new_id is the permanent that just entered — live here.
        if let Some(obj) = state.expect_object_mut(new_id) {
            obj.class_level = 1;
        }
    }
    evts
}
/// CR 714.2b: Fire chapter ability triggers when lore counters are added to a Saga.
///
/// "{rN}—[Effect]" means "When one or more lore counters are put onto this Saga, if the
/// number of lore counters on it was less than N and became at least N, [effect]."
///
/// Chapters that trigger are those where `old_count < chapter && new_count >= chapter`.
///
/// CR 712.8d/e (PB-RS4): `ability_index` is a dense index into the currently-visible
/// face's *effective* ability list — the same namespace every consumer that resolves
/// a CardDef ability index resolves it against (`effective_abilities(obj.is_transformed)`).
/// That is eight sites in the tree, not just the SagaChapter-specific ones: e.g.
/// `resolution.rs:1996`/`:2028` (SagaChapter effect lookup), `resolution.rs:2066`
/// (modal-trigger `modes` lookup), `sba.rs:889` (CR 714.4 "chapter still on the
/// stack" guard), `abilities.rs:7004`/`:7082`/`:7210`/`:8379` (`once_per_turn`,
/// `has_ability_targets`, `ability_targets`, flush-time lookup). The face signal is
/// read live from `saga_id`'s `is_transformed` flag (not threaded as a parameter):
/// this fn is `pub` and called from `turn_actions.rs` and directly from tests, and
/// every existing caller already holds a live, up-to-date object at call time.
pub fn fire_saga_chapter_triggers(
    state: &mut GameState,
    saga_id: ObjectId,
    controller: PlayerId,
    old_count: u32,
    new_count: u32,
    def: &crate::cards::card_definition::CardDefinition,
) -> Vec<GameEvent> {
    use crate::cards::card_definition::AbilityDefinition;
    use crate::state::stubs::{PendingTrigger, PendingTriggerKind};
    // CR 400.7: `saga_id` may have departed its zone since the caller looked it up
    // (a legal fizzle, not an engine bug) -- default to front-face indexing.
    let is_transformed = state
        .fizzle_object(saga_id)
        .map(|o| o.is_transformed)
        .unwrap_or(false);
    let evts = Vec::new();
    for (ability_index, ability) in def.effective_abilities(is_transformed).iter().enumerate() {
        if let AbilityDefinition::SagaChapter { chapter, .. } = ability {
            // CR 714.2b: Trigger fires if count crossed the chapter threshold.
            if old_count < *chapter && new_count >= *chapter {
                state.pending_triggers.push_back(PendingTrigger {
                    ability_index,
                    ..PendingTrigger::blank(saga_id, controller, PendingTriggerKind::Normal)
                });
            }
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
    use crate::cards::card_definition::{
        AbilityDefinition, Effect, EffectAmount, TokenSpec, TriggerCondition,
    };
    use crate::effects::{execute_effect, EffectContext};
    use crate::state::stubs::{ETBSuppressFilter, PendingTrigger, PendingTriggerKind};
    use crate::state::types::{CounterType, KeywordAbility, SubType};
    // CR 708.3: Face-down permanents have no triggered abilities.
    // CR 400.7: `new_id` is passed in by the caller; an earlier ETB replacement in the
    // same batch (or a harness) may leave no live object for this id, so its absence is
    // a legal fizzle (no permanent here means no face-down suppression to apply).
    if let Some(obj) = state.fizzle_object(new_id) {
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
                // CR 400.7: `new_id` may name a permanent that already left (an earlier
                // same-batch ETB replacement, or a harness call); absence defaults to
                // Exile/empty chars, i.e. the suppressor simply does not apply.
                let obj_zone = state
                    .fizzle_object(new_id)
                    .map(|o| o.zone)
                    .unwrap_or(crate::state::zone::ZoneId::Exile);
                let chars = state
                    .fizzle_object(new_id)
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
    // CR 702.104b: Retrieve tribute_was_paid status from the permanent for trigger
    // condition check. PB-OS4b (CR 712.8d/e): also read the entering object's live
    // `is_transformed` here (single lookup, SR-25 ratchet) so a permanent that
    // enters already showing its back face (craft, disturb,
    // ExileSourceAndReturnTransformed) queues that face's ETB triggers, not the
    // front face's. `ability_index` below is a dense index into the *effective*
    // list; the CardDefETB consumers (rules/abilities.rs) re-derive against
    // `effective_abilities(obj.is_transformed)` at resolution time using the same
    // contract (see the Index-Stability discussion in the PB-OS4b plan).
    let (tribute_was_paid, entering_is_transformed) = state
        .objects
        .get(&new_id)
        .map(|o| (o.tribute_was_paid, o.is_transformed))
        .unwrap_or((false, false));
    let mut evts = Vec::new();
    for (idx, ability) in def
        .effective_abilities(entering_is_transformed)
        .iter()
        .enumerate()
    {
        match ability {
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                intervening_if,
                ..
            } => {
                // CR 603.4 (PB-DP6, scutemob-154): check intervening-if at trigger
                // time via the shared queue-time helper (MR-B12-07/08 fixed the
                // original inline duplicate that only handled OpponentHasPoisonCounters
                // and silently passed all other Condition variants via `_ => true`;
                // PB-DP6 additionally repairs the `EffectContext` this site built --
                // `EffectContext::new` zero-filled `kicker_times_paid`/`x_value`, so
                // `WasKicked`/`XValueAtLeast` were unconditionally false here even
                // though the entering object's fields were set well before this
                // point in resolution -- see the helper's doc comment).
                if !crate::rules::abilities::carddef_intervening_if_holds_at_queue_time(
                    state,
                    intervening_if.as_ref(),
                    controller,
                    new_id,
                ) {
                    continue;
                }
                // CR 603.3: Queue as PendingTrigger; flush_pending_triggers places it on the stack.
                // Use PendingTriggerKind::CardDefETB so resolution looks up the effect from
                // the card registry (ability_index is into CardDef::abilities, NOT into
                // runtime triggered_abilities). This avoids index collisions with triggers
                // added by enrich_spec_from_def for attack/dies/etc. triggers.
                state.pending_triggers.push_back(PendingTrigger {
                    ability_index: idx,
                    triggering_event: Some(
                        crate::state::game_object::TriggerEvent::SelfEntersBattlefield,
                    ),
                    // CR 603.2d: Set entering_object_id so doubler_applies_to_trigger can
                    // verify the entering permanent's card types (artifact/creature for
                    // Panharmonicon, land for Ancient Greenwarden, etc.).
                    entering_object_id: Some(new_id),
                    ..PendingTrigger::blank(new_id, controller, PendingTriggerKind::CardDefETB)
                });
            }
            // CR 702.104b: "When ~ enters, if tribute wasn't paid, ..."
            // CR 603.4: Intervening-if — only queue the trigger if tribute was not
            // paid AND (PB-DP6) the def's own `intervening_if`, if any, also holds
            // at queue time. Neither check alone is sufficient; both must pass for
            // the trigger to be queued.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::TributeNotPaid,
                intervening_if,
                ..
            } if !tribute_was_paid
                && crate::rules::abilities::carddef_intervening_if_holds_at_queue_time(
                    state,
                    intervening_if.as_ref(),
                    controller,
                    new_id,
                ) =>
            {
                state.pending_triggers.push_back(PendingTrigger {
                    ability_index: idx,
                    triggering_event: Some(
                        crate::state::game_object::TriggerEvent::SelfEntersBattlefield,
                    ),
                    // CR 603.2d: Set entering_object_id for trigger doubling type checks.
                    entering_object_id: Some(new_id),
                    ..PendingTrigger::blank(new_id, controller, PendingTriggerKind::CardDefETB)
                });
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
                    // SR-14: guarded by `permanent_on_bf` above — new_id is live here.
                    if let Some(obj) = state.expect_object_mut(new_id) {
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
                        colors: imbl::OrdSet::new(),
                        supertypes: imbl::OrdSet::new(),
                        card_types: [CardType::Artifact, CardType::Creature]
                            .into_iter()
                            .collect(),
                        subtypes: [SubType("Servo".to_string())].into_iter().collect(),
                        keywords: imbl::OrdSet::new(),
                        count: EffectAmount::Fixed(n as i32),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
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
/// battlefield (before emitting the ETB event). Runs the CR 614/616 application
/// loop ([`apply_etb_replacement_pass`]) over the global (non-self) replacement
/// effects in `state.replacement_effects` — applying `EntersTapped`,
/// `EntersWithCounters`, and similar modifications with CR 614.5 loop prevention.
///
/// Self-ETB replacements from card definitions are registered and applied BEFORE
/// this call by `apply_self_etb_from_definition` (CR 614.15: self-replacement
/// first). Self-replacements still registered in `state.replacement_effects` are
/// skipped here — `apply_etb_replacement_pass(.., Global)` filters them out.
pub fn apply_etb_replacements(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
) -> Vec<GameEvent> {
    apply_etb_replacement_pass(state, new_id, controller, EtbPass::Global)
}
/// Internal: set state and produce events for one ETB modification.
///
/// If `effect_id` is Some, emits `ReplacementEffectApplied` (for global effects with a
/// registered ID). If None, skips that event (for inline self-ETB replacements).
///
/// PB-EWC: `replacement_source` is the source permanent of the replacement
/// effect (for `EntersWithCounters` with a dynamic `EffectAmount`). For non-self
/// global replacements (Master Biomancer), this is the replacement source from
/// `ReplacementEffect.source`. For self-ETB (Ingenious Prodigy), this is the
/// entering permanent itself. Falls back to `new_id` when `None` (defensive).
fn emit_etb_modification(
    state: &mut GameState,
    new_id: ObjectId,
    controller: PlayerId,
    effect_id: Option<ReplacementId>,
    modification: Option<ReplacementModification>,
    replacement_source: Option<ObjectId>,
) -> Vec<GameEvent> {
    let mut evts: Vec<GameEvent> = Vec::new();
    match modification {
        Some(ReplacementModification::EntersTapped)
        | Some(ReplacementModification::EntersTappedUnlessPayLife(_)) => {
            // EntersTappedUnlessPayLife: deterministic fallback (pre-M10) — always
            // enters tapped. Interactive "may pay N life" choice deferred to M10.
            // SR-14: new_id is the entering permanent whose ETB modification is being
            // applied — live here.
            if let Some(obj) = state.expect_object_mut(new_id) {
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
            // PB-EWC: resolve the EffectAmount against the replacement's source.
            //
            // CR 614.12: replacement effects modifying how a permanent enters check
            // the source's characteristics "as it would exist on the battlefield."
            // The replacement source (Master Biomancer for non-self; the entering
            // permanent itself for self-ETB) is ALIVE on the battlefield when its
            // replacement fires, so `EffectAmount::PowerOf(EffectTarget::Source)`
            // resolves via the live arm of `resolve_amount` (layer-resolved P/T
            // from `calculate_characteristics`).
            //
            // For `EffectAmount::XValue`, ctx.x_value is read from the source's
            // `x_value` field — set on the entering permanent during permanent-spell
            // resolution (before ETB processing) for self-ETB (Ingenious Prodigy),
            // or 0 for non-X non-self sources.
            let source_id = replacement_source.unwrap_or(new_id);
            let mut ctx = crate::effects::EffectContext::new(controller, source_id, vec![]);
            ctx.x_value = state
                .objects
                .get(&source_id)
                .map(|o| o.x_value)
                .unwrap_or(0);
            let raw_count = crate::effects::resolve_amount(state, &count, &ctx).max(0) as u32;
            // CR 122.6: Apply counter-placement replacements to ETB counters too.
            let (modified_count, repl_events) =
                apply_counter_replacement(state, controller, new_id, &counter, raw_count);
            evts.extend(repl_events);
            if modified_count > 0 {
                // SR-14: new_id is the entering permanent — live here.
                if let Some(obj) = state.expect_object_mut(new_id) {
                    let cur = obj.counters.get(&counter).copied().unwrap_or(0);
                    obj.counters.insert(counter.clone(), cur + modified_count);
                }
            }
            if let Some(id) = effect_id {
                evts.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: format!("enters with {} {:?} counters", modified_count, counter),
                });
            }
            if modified_count > 0 {
                evts.push(GameEvent::CounterAdded {
                    object_id: new_id,
                    counter,
                    count: modified_count,
                });
            }
        }
        Some(ReplacementModification::ChooseCreatureType(default_type)) => {
            // CR 106.12 support: "As this enters, choose a creature type."
            // Deterministic fallback: pick the most common creature subtype
            // among creatures the controller controls, or the default.
            //
            // `BTreeMap`, not `HashMap` — see `Effect::ChooseCreatureType` in
            // `effects/mod.rs` for the full argument (PB-DP9 fix-cycle
            // Finding 4): `max_by_key` breaks ties by iteration order, and
            // `HashMap` iteration order varies between two maps in the same
            // process, which PB-DP9's abort-and-replay cannot tolerate.
            let chosen = {
                let mut type_counts: std::collections::BTreeMap<
                    crate::state::types::SubType,
                    usize,
                > = std::collections::BTreeMap::new();
                // CR 613.1d: Use layer-resolved types/subtypes for creature scan.
                for obj in state.objects.values() {
                    if obj.controller == controller
                        && matches!(obj.zone, crate::state::zone::ZoneId::Battlefield)
                    {
                        // SR-14: obj is a live `state.objects.values()` loop var, so
                        // calculate_characteristics is total (CR 613.1d).
                        let chars = crate::rules::layers::expect_characteristics(state, obj.id);
                        if chars
                            .card_types
                            .contains(&crate::state::types::CardType::Creature)
                        {
                            for st in &chars.subtypes {
                                *type_counts.entry(st.clone()).or_insert(0usize) += 1;
                            }
                        }
                    }
                }
                type_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(st, _)| st)
                    .unwrap_or(default_type)
            };
            // SR-14: new_id is the entering permanent — live here.
            if let Some(obj) = state.expect_object_mut(new_id) {
                obj.chosen_creature_type = Some(chosen);
            }
        }
        Some(ReplacementModification::ChooseColor(default_color)) => {
            // CR 614.12a: "As this enters, choose a color." Choice committed before the
            // permanent enters. Deterministic fallback (M10 deferred): scan battlefield
            // permanents controlled by this controller, count their layer-resolved colors
            // (CR 613.1e), pick the most common. Fall back to default_color if none.
            //
            // Unlike the two `ChooseCreatureType` sites, this one was ALREADY
            // deterministic before PB-DP9's fix cycle: `max_count` comes from
            // `.values().max()`, and the tie-break below picks the unique
            // highest colour discriminant, so iteration order never reached the
            // outcome. (The fix-cycle review listed it with the other two; that
            // part of the finding was wrong.) It is a `BTreeMap` anyway, so the
            // "`HashMap` iteration reaching an outcome" audit has no residue to
            // re-examine here.
            let chosen = {
                let mut color_counts: std::collections::BTreeMap<
                    crate::state::types::Color,
                    usize,
                > = std::collections::BTreeMap::new();
                // CR 613.1d/e: Use layer-resolved characteristics for color scan.
                for obj in state.objects.values() {
                    if obj.controller == controller
                        && matches!(obj.zone, crate::state::zone::ZoneId::Battlefield)
                    {
                        // SR-14: obj is a live `state.objects.values()` loop var (CR 613.1d/e).
                        let chars = crate::rules::layers::expect_characteristics(state, obj.id);
                        for c in &chars.colors {
                            *color_counts.entry(*c).or_insert(0usize) += 1;
                        }
                    }
                }
                // Tie-break: prefer default_color if it appears with the max count,
                // otherwise pick the Color with the highest discriminant (deterministic).
                let max_count = color_counts.values().copied().max().unwrap_or(0);
                if max_count == 0 {
                    default_color
                } else if color_counts.get(&default_color).copied().unwrap_or(0) == max_count {
                    // Default color tied for first — prefer it (deterministic).
                    default_color
                } else {
                    color_counts
                        .into_iter()
                        .filter(|(_, count)| *count == max_count)
                        .max_by_key(|(c, _)| *c as u8)
                        .map(|(c, _)| c)
                        .unwrap_or(default_color)
                }
            };
            // SR-14: new_id is the entering permanent — live here.
            if let Some(obj) = state.expect_object_mut(new_id) {
                obj.chosen_color = Some(chosen);
            }
        }
        Some(ReplacementModification::EntersAsAdditionalType { subtype }) => {
            // PB-EAT: CR 614.1c — "...enters as a [Type] in addition to its other
            // types." This is an entry modification, not a Layer 4 continuous
            // type-adding effect. The subtype is pushed into the entering
            // permanent's `characteristics.subtypes` BEFORE `PermanentEnteredBattlefield`
            // is emitted (the caller — `apply_self_etb_from_definition` /
            // `apply_etb_replacements` — runs before the ETB event in
            // `resolution.rs` / `lands.rs` / `effects/mod.rs`), so ETB triggers
            // and SBAs observe the augmented type set on the very turn it enters.
            //
            // OrdSet semantics: idempotent insert. If the printed type set already
            // contains the subtype (or this replacement was somehow applied twice),
            // the second insert is a no-op (CR 614.5 also forbids double-application).
            // SR-14: new_id is the entering permanent — live here.
            if let Some(obj) = state.expect_object_mut(new_id) {
                obj.characteristics.subtypes.insert(subtype.clone());
            }
            if let Some(id) = effect_id {
                evts.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: format!(
                        "enters as a {} in addition to its other types",
                        subtype.0
                    ),
                });
            }
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
/// `pre_death_characteristics` is the full layer-resolved characteristics snapshot (CR 603.10a /
/// CR 613.1d) captured before the zone move for filtered death trigger evaluation.
#[allow(clippy::too_many_arguments)]
fn zone_change_events(
    state: &GameState,
    old_id: ObjectId,
    new_id: ObjectId,
    dest: ZoneId,
    owner: PlayerId,
    pre_move_controller: PlayerId,
    pre_death_counters: &imbl::OrdMap<crate::state::types::CounterType, u32>,
    pre_death_power: Option<i32>,
    pre_death_characteristics: Option<crate::state::game_object::Characteristics>,
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
                    // CR 603.10a: LKI power for SourcePowerAtLastKnownInformation.
                    pre_death_power,
                    // CR 603.10a / CR 613.1d: full LKI characteristics for filtered death triggers.
                    pre_death_characteristics,
                }]
            } else {
                vec![GameEvent::PermanentDestroyed {
                    object_id: old_id,
                    new_grave_id: new_id,
                    // CR 603.10a: pass LKI counters for WhenLeavesBattlefield triggers.
                    pre_lba_counters: pre_death_counters.clone(),
                    // CR 603.10a: LKI power for SourcePowerAtLastKnownInformation.
                    pre_lba_power: pre_death_power,
                }]
            }
        }
        ZoneId::Exile => vec![GameEvent::ObjectExiled {
            player: owner, // MR-M8-04: use real owner instead of hardcoded PlayerId(0)
            object_id: old_id,
            new_exile_id: new_id,
            // CR 603.10a: pass LKI counters for WhenLeavesBattlefield triggers.
            // pre_death_counters captured before move_object_to_zone resets them.
            pre_lba_counters: pre_death_counters.clone(),
            // CR 603.10a: LKI power for SourcePowerAtLastKnownInformation.
            pre_lba_power: pre_death_power,
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
    // PB-RS4 (CR 614 / 712.8d/e, OOS-RS-3): gather from the face that is actually
    // showing. A permanent entering back-face-up (craft/disturb/exile-return)
    // registers that face's permanent replacement abilities, not the front's.
    // Closes the PB-OS4b-era limitation this comment used to describe.
    let entering_is_transformed = state
        .fizzle_object(new_id)
        .map(|o| o.is_transformed)
        .unwrap_or(false);
    for ability in def.effective_abilities(entering_is_transformed) {
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
            //
            // For WouldPlaceCounters/WouldCreateTokens/WouldSearchLibrary, bind the
            // controller's PlayerId at registration time so player filters resolve
            // correctly (Vorinclex, Pir, Adrix and Nev, Aven Mindcensor).
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
                    // PB-EWC: bind WouldEnterBattlefield filter placeholders to the
                    // controller. Master Biomancer registers with
                    // `CreatureControlledBy(PlayerId(0))` to mean "each (other) creature
                    // you control"; rebind to the actual controller so future entries
                    // match correctly. Other ObjectFilter variants (Any, AnyCreature,
                    // HasCardType, ...) pass through unchanged.
                    ReplacementTrigger::WouldEnterBattlefield { filter } => {
                        ReplacementTrigger::WouldEnterBattlefield {
                            filter: bind_object_filter(filter, controller),
                        }
                    }
                    // Bind PlayerFilter placeholders at registration time.
                    // Card defs use Specific(PlayerId(0)) as a placeholder for "controller".
                    ReplacementTrigger::WouldPlaceCounters {
                        placer_filter,
                        receiver_filter,
                        counter_filter,
                    } => ReplacementTrigger::WouldPlaceCounters {
                        placer_filter: bind_player_filter(placer_filter, controller),
                        receiver_filter: bind_object_filter(receiver_filter, controller),
                        // PB-CD: counter_filter does not bind to controller; pass through.
                        counter_filter: counter_filter.clone(),
                    },
                    ReplacementTrigger::WouldCreateTokens { controller_filter } => {
                        ReplacementTrigger::WouldCreateTokens {
                            controller_filter: bind_player_filter(controller_filter, controller),
                        }
                    }
                    ReplacementTrigger::WouldSearchLibrary { searcher_filter } => {
                        ReplacementTrigger::WouldSearchLibrary {
                            searcher_filter: bind_player_filter(searcher_filter, controller),
                        }
                    }
                    ReplacementTrigger::WouldLoseLife { player_filter } => {
                        ReplacementTrigger::WouldLoseLife {
                            player_filter: bind_player_filter(player_filter, controller),
                        }
                    }
                    ReplacementTrigger::DamageWouldBeDealt {
                        target_filter: DamageTargetFilter::FromControllerSources(PlayerId(0)),
                    } => ReplacementTrigger::DamageWouldBeDealt {
                        target_filter: DamageTargetFilter::FromControllerSources(controller),
                    },
                    ReplacementTrigger::DamageWouldBeDealt {
                        target_filter: DamageTargetFilter::ToOpponentOrTheirPermanent(PlayerId(0)),
                    } => ReplacementTrigger::DamageWouldBeDealt {
                        target_filter: DamageTargetFilter::ToOpponentOrTheirPermanent(controller),
                    },
                    // Bind the "entered this turn" source filter to the controller.
                    // Used by Neriv: "a creature you control that entered this turn".
                    ReplacementTrigger::DamageWouldBeDealt {
                        target_filter:
                            DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(PlayerId(0)),
                    } => ReplacementTrigger::DamageWouldBeDealt {
                        target_filter: DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(
                            controller,
                        ),
                    },
                    ReplacementTrigger::WouldProliferate { player_filter } => {
                        ReplacementTrigger::WouldProliferate {
                            player_filter: bind_player_filter(player_filter, controller),
                        }
                    }
                    // CR 106.12b: Bind the controller PlayerId at registration time.
                    // Card defs use PlayerId(0) as placeholder; resolved here.
                    // PB-Q: preserve color_filter and source_filter fields.
                    ReplacementTrigger::ManaWouldBeProduced {
                        color_filter,
                        source_filter,
                        ..
                    } => ReplacementTrigger::ManaWouldBeProduced {
                        controller,
                        color_filter: color_filter.clone(),
                        source_filter: source_filter.clone(),
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
///
/// `is_transformed` (PB-OS4b, CR 712.8d/e): selects which face's abilities are
/// registered -- pass the entering/current object's `is_transformed` value. `false`
/// for a normal ETB (front face); `true` for a permanent entering already
/// transformed (e.g. craft return, `ExileSourceAndReturnTransformed`) or when this
/// is called from [`super::face::apply_face_change`] to register the newly-visible
/// face at an in-place transform boundary.
pub fn register_static_continuous_effects(
    state: &mut GameState,
    new_id: ObjectId,
    card_id: Option<&crate::state::player::CardId>,
    registry: &crate::cards::registry::CardRegistry,
    is_transformed: bool,
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
    // SR-14: new_id is the entering permanent — live here.
    let controller = state
        .expect_object(new_id)
        .map(|obj| obj.controller)
        .unwrap_or_else(|| crate::state::player::PlayerId(0));
    for ability in def.effective_abilities(is_transformed) {
        match ability {
            AbilityDefinition::Static { continuous_effect } => {
                let eff_id = state.next_object_id().0;
                let ts = state.timestamp_counter;
                state.timestamp_counter += 1;
                // Resolve EffectFilter::Source to a concrete ObjectId at registration time
                // so the filter is stable across zone changes (CR 400.7).
                let resolved_filter = match &continuous_effect.filter {
                    crate::state::continuous_effect::EffectFilter::Source => {
                        crate::state::continuous_effect::EffectFilter::SingleObject(new_id)
                    }
                    other => other.clone(),
                };
                state.continuous_effects.push_back(ContinuousEffect {
                    id: EffectId(eff_id),
                    source: Some(new_id),
                    timestamp: ts,
                    layer: continuous_effect.layer,
                    duration: continuous_effect.duration,
                    filter: resolved_filter,
                    modification: continuous_effect.modification.clone(),
                    is_cda: false,
                    condition: continuous_effect.condition.clone(),
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
            // CR 604.1 / 603.2: Register a Torpor Orb-style ETB trigger suppressor
            // (no dedicated CR subrule for this pattern; CR 614.16 governs
            // token/counter-creation replacement effects, not ETB-trigger
            // suppression -- do not cite it, see face.rs's deregistration arm).
            AbilityDefinition::SuppressCreatureETBTriggers { filter } => {
                state
                    .etb_suppressors
                    .push_back(crate::state::stubs::ETBSuppressor {
                        source: new_id,
                        filter: filter.clone(),
                    });
            }
            // PB-18: Register a stax/action restriction (Rule of Law, Propaganda, etc.).
            AbilityDefinition::StaticRestriction { restriction } => {
                state
                    .restrictions
                    .push_back(crate::state::stubs::ActiveRestriction {
                        source: new_id,
                        controller,
                        restriction: restriction.clone(),
                    });
            }
            // PB-28: Register a CDA Layer 7a continuous effect for dynamic P/T evaluation.
            // CR 604.3: CDAs function in all zones; the layer system evaluates for all objects.
            // CR 613.4a: CDA P/T effects apply in Layer 7a.
            AbilityDefinition::CdaPowerToughness { power, toughness } => {
                let eff_id = state.next_object_id().0;
                let ts = state.timestamp_counter;
                state.timestamp_counter += 1;
                state
                    .continuous_effects
                    .push_back(crate::state::continuous_effect::ContinuousEffect {
                    id: crate::state::continuous_effect::EffectId(eff_id),
                    source: Some(new_id),
                    timestamp: ts,
                    layer: crate::state::continuous_effect::EffectLayer::PtCda,
                    duration:
                        crate::state::continuous_effect::EffectDuration::WhileSourceOnBattlefield,
                    filter: crate::state::continuous_effect::EffectFilter::SingleObject(new_id),
                    modification:
                        crate::state::continuous_effect::LayerModification::SetPtDynamic {
                            power: Box::new(power.clone()),
                            toughness: Box::new(toughness.clone()),
                        },
                    is_cda: true,
                    condition: None, // CR 604.3a(5): CDAs are unconditional
                });
            }
            // PB-CC-C-followup: Register a CDA Layer 7c continuous effect for dynamic P/T
            // modification with continuous re-evaluation (CR 611.3a).
            //
            // Contrast with `CdaPowerToughness` (Layer 7a, SetPtDynamic — *sets* base P/T):
            // this variant *modifies* (adds/subtracts from) P/T after any Layer-7b set-effects.
            //
            // CR 611.3a: "A continuous effect generated by a static ability isn't 'locked in';
            // it applies at any given moment to whatever its text indicates."
            // CR 613.4c: Layer 7c — effects and counters that modify P/T (but don't set it).
            // CR 604.3a(5): CDAs are unconditional — condition is always None.
            //
            // The stored `ModifyPowerDynamic` / `ModifyToughnessDynamic` / `ModifyBothDynamic`
            // variant with `is_cda: true` is NOT substituted at registration time. Instead,
            // `apply_layer_modification` in layers.rs calls `resolve_cda_amount` live on every
            // `calculate_characteristics` invocation.
            AbilityDefinition::CdaModifyPowerToughness { power, toughness } => {
                let ts = state.timestamp_counter;
                state.timestamp_counter += 1;
                // Choose the appropriate LayerModification variant(s):
                // - Both Some → two separate effects: one ModifyPowerDynamic + one
                //   ModifyToughnessDynamic. Registering two independent effects correctly
                //   supports asymmetric amounts (e.g. power: Fixed(3), toughness: Fixed(2))
                //   and avoids silently discarding the toughness amount when the two amounts
                //   differ. For symmetric amounts (e.g. Vishgraz: same PlayerCounterCount on
                //   both axes), the two effects produce the same observable P/T as a single
                //   ModifyBothDynamic would — just with two effect registrations.
                // - Only power Some → single ModifyPowerDynamic.
                // - Only toughness Some → single ModifyToughnessDynamic.
                // - Both None → no-op (no effect registered).
                let mut modifications: Vec<crate::state::continuous_effect::LayerModification> =
                    Vec::new();
                if let Some(p) = power {
                    modifications.push(
                        crate::state::continuous_effect::LayerModification::ModifyPowerDynamic {
                            amount: Box::new(p.clone()),
                            negate: false,
                        },
                    );
                }
                if let Some(t) = toughness {
                    modifications.push(
                        crate::state::continuous_effect::LayerModification::ModifyToughnessDynamic {
                            amount: Box::new(t.clone()),
                            negate: false,
                        },
                    );
                }
                for modification in modifications {
                    let eff_id = state.next_object_id().0;
                    state
                        .continuous_effects
                        .push_back(crate::state::continuous_effect::ContinuousEffect {
                        id: crate::state::continuous_effect::EffectId(eff_id),
                        source: Some(new_id),
                        timestamp: ts,
                        layer: crate::state::continuous_effect::EffectLayer::PtModify,
                        duration:
                            crate::state::continuous_effect::EffectDuration::WhileSourceOnBattlefield,
                        filter: crate::state::continuous_effect::EffectFilter::SingleObject(new_id),
                        modification,
                        is_cda: true,
                        condition: None, // CR 604.3a(5): CDAs are unconditional
                    });
                }
            }
            // CR 305.2: Register a static additional land play source.
            // At the start of each of the controller's turns, land_plays_remaining is
            // incremented by `count` in `reset_turn_state`.
            AbilityDefinition::AdditionalLandPlays { count } => {
                state.additional_land_play_sources.push_back(
                    crate::state::stubs::AdditionalLandPlaySource {
                        source: new_id,
                        controller,
                        count: *count,
                    },
                );
            }
            // PB-I: Register a static flash grant (Yeva-style).
            // CR 601.3b: "You may cast [X] spells as though they had flash."
            AbilityDefinition::StaticFlashGrant { filter } => {
                state
                    .flash_grants
                    .push_back(crate::state::stubs::FlashGrant {
                    source: Some(new_id),
                    player: controller,
                    filter: filter.clone(),
                    duration:
                        crate::state::continuous_effect::EffectDuration::WhileSourceOnBattlefield,
                });
            }
            // PB-B: Register a static play-from-graveyard permission (CR 601.3, 305.1).
            AbilityDefinition::StaticPlayFromGraveyard { filter, condition } => {
                state.play_from_graveyard_permissions.push_back(
                    crate::state::stubs::PlayFromGraveyardPermission {
                        source: new_id,
                        controller,
                        filter: filter.clone(),
                        condition: condition.as_ref().map(|c| *c.clone()),
                    },
                );
            }
            // PB-A: Register a static play-from-top-of-library permission (CR 601.3, 305.1).
            AbilityDefinition::StaticPlayFromTop {
                filter,
                look_at_top,
                reveal_top,
                pay_life_instead,
                condition,
                on_cast_effect,
            } => {
                state.play_from_top_permissions.push_back(
                    crate::state::stubs::PlayFromTopPermission {
                        source: new_id,
                        controller,
                        filter: filter.clone(),
                        look_at_top: *look_at_top,
                        reveal_top: *reveal_top,
                        pay_life_instead: *pay_life_instead,
                        condition: condition.as_ref().map(|c| *c.clone()),
                        on_cast_effect: on_cast_effect.clone(),
                    },
                );
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
    // The controller of the damage source is needed for `FromPlayer` protection
    // (CR 702.16k).
    let source_controller = state.objects.get(&source).map(|o| o.controller);
    match target {
        CombatDamageTarget::Creature(target_id) | CombatDamageTarget::Planeswalker(target_id) => {
            // SR-14 FIZZLE (CR 608.2b): the damage target is a resolved target that may
            // have left its zone before damage is dealt; `None` means no keywords to read
            // for protection, and the (now-absent) target's damage does nothing downstream.
            let target_keywords =
                crate::rules::layers::calculate_characteristics(state, *target_id)
                    .map(|c| c.keywords)
                    .unwrap_or_default();
            let source_chars = crate::rules::protection::source_characteristics(state, source);
            if let Some(sc) = &source_chars {
                if crate::rules::protection::protection_prevents_damage(
                    &target_keywords,
                    sc,
                    source_controller,
                ) {
                    return (0, Vec::new());
                }
            }
        }
        CombatDamageTarget::Player(player_id) => {
            // CR 702.16e: damage from a source with the stated quality to a player
            // with protection from that quality is prevented. Check both permanent and
            // temporary protection qualities (CR 611.2b).
            let source_chars = crate::rules::protection::source_characteristics(state, source);
            if let Some(sc) = &source_chars {
                // SR-14: players are never removed from state.players (ground truth 1).
                if let Some(player) = state.expect_player(*player_id) {
                    let qualities: Vec<_> = player
                        .protection_qualities
                        .iter()
                        .chain(player.temporary_protection_qualities.iter())
                        .cloned()
                        .collect();
                    for quality in &qualities {
                        if crate::rules::protection::has_protection_from_source_quality(
                            quality,
                            sc,
                            source_controller,
                        ) {
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
            Some(ReplacementModification::DoubleDamage) => {
                remaining *= 2;
                events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: format!("doubled damage to {}", remaining),
                });
            }
            Some(ReplacementModification::TripleDamage) => {
                // CR 614.1 / CR 701.10g: Triple the damage instead of doubling.
                // Used by Fiery Emancipation.
                remaining *= 3;
                events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: id,
                    description: format!("tripled damage to {}", remaining),
                });
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
/// **The gate (PB-DX2 / OOS-DP5-7):** `Command::ChooseDredge` is legal ONLY
/// while `player` has an outstanding `PendingDraw` entry (pushed by
/// `perform_one_draw`'s `DredgeAvailable` arm at the offer site) and it
/// CONSUMES that entry. Before PB-DX2 nothing was recorded anywhere on
/// `GameState`, so `card: None` drew a free card for any player at any time
/// and `card: Some(x)` dredged at will regardless of whether a draw was ever
/// offered — see `pb-plan-DX2.md` §1 P1.
///
/// `Command::ChooseDredge` carries no discriminator between a dredge-offer
/// entry and a CR 616.1 `NeedsChoice` entry (both are `WouldDraw` `PendingDraw`
/// rows), and cannot gain one without a PROTOCOL bump. §3.3 of the plan argues
/// this is sound: every possible pairing of {`OrderReplacements`,
/// `ChooseDredge`} x {dredge entry, `NeedsChoice` entry} is a CR-legal
/// outcome — so `ChooseDredge { Some }` succeeding against a `NeedsChoice`
/// entry is a FEATURE (CR 616.1e lets the player pick dredge from the
/// applicable set), not a hole. `position()` (FIFO, oldest entry for this
/// player) matches `handle_order_replacements` — and FIFO is a real choice
/// here, not a formality (re-review Finding R1, `pb-review-DX2.md`): a
/// player CAN have 2+ outstanding entries (see `perform_one_draw`'s
/// "Per-player invariant" doc and `OOS-DX2-3`, reopened), so this always
/// answers the oldest.
///
/// **The decline is not sticky (fix-cycle Finding 10).** The `None` arm below
/// passes `offer_dredge: false` so the SAME draw is not re-offered dredge
/// (`dredge.rs` test 10) — but if that resume itself hits a fresh
/// `NeedsChoice` (other `WouldDraw` replacements still applicable), the
/// FRESH entry it pushes carries no memory of the decline, and the player
/// may immediately send `ChooseDredge { Some(the_same_card) }` and dredge the
/// very draw they just declined. This is intentional, not a bug: CR 616.1f
/// says the replacement-choice process repeats "taking into account only
/// replacement effects that would now be applicable", nothing consumed
/// dredge on the decline, and CR 616.1e still permits choosing it. See
/// `pb_dx2_command_gates.rs::test_dx2_choose_dredge_some_can_answer_a_needschoice_originated_entry`.
///
/// If `card` is `Some(id)`:
///   1. Validate the card is in the player's graveyard with `KeywordAbility::Dredge(n)`.
///   2. Validate the player has >= n cards in library (CR 702.52b).
///      (These two checks are byte-for-byte `check_would_draw_replacement`'s own
///      eligibility predicate, so a gated `Some` answer can only ever name a
///      card dredge law would itself have offered.)
///   3. Mill n cards from the top of the library (emitting `CardMilled` events).
///   4. Move the dredge card from graveyard to hand (CR 400.7: new ObjectId).
///   5. Emit `Dredged` event.
///   6. Do NOT increment `cards_drawn_this_turn` (dredge is NOT drawing — CR 702.52a).
///   7. CR 614.11a: perform the entry's `remaining` further draws of the
///      sequence this draw belonged to.
///
/// If `card` is `None`:
///   The player declined to dredge — resume the replaced draw with the
///   entry's own bookkeeping (re-checking other WouldDraw replacements, but
///   NOT dredge again — CR 702.52a, the player just declined), then perform
///   the entry's remaining draws (CR 614.11a).
pub fn handle_choose_dredge(
    state: &mut GameState,
    player: PlayerId,
    card: Option<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    use crate::state::types::KeywordAbility;
    // Step 0: dead-player discharge. Preserves the pre-PB-DX2
    // has_lost/has_conceded guard, and additionally clears any outstanding
    // entry so a departed player's obligation cannot sit in the hash forever
    // (the OOS-DP9-14 lesson, applied prophylactically here — SR-14 ground
    // truth 1: players are never removed from `state.players`).
    if let Some(p) = state.expect_player(player) {
        if p.has_lost || p.has_conceded {
            if let Some(idx) = state
                .pending_draws
                .iter()
                .position(|pd| pd.player == player)
            {
                state.pending_draws.remove(idx);
            }
            return Ok(vec![]);
        }
    }
    // THE GATE (CR 702.52a): `ChooseDredge` is legal only while an
    // outstanding draw stands for `player`. FIFO — takes the OLDEST
    // outstanding entry, matching `handle_order_replacements`'s `position()`.
    let idx = state
        .pending_draws
        .iter()
        .position(|pd| pd.player == player)
        .ok_or_else(|| {
            GameStateError::InvalidCommand(format!(
                "ChooseDredge from player {:?} with no draw outstanding — CR 702.52a: dredge \
                 replaces a draw, and the engine records the offer as a PendingDraw entry \
                 (GameEvent::DredgeChoiceRequired). PB-DX2 / OOS-DP5-7.",
                player
            ))
        })?;
    match card {
        None => {
            // CONSUME the entry before resuming, then discharge it exactly
            // like `perform_one_draw`'s own stale-entry discharge does — this
            // IS an explicit decline, the sibling of that function's implicit
            // one (fix-cycle Finding 1: both now route through
            // `resolve_declined_pending_draw` so the bookkeeping cannot drift
            // apart).
            let pending = state.pending_draws[idx].clone();
            state.pending_draws.remove(idx);
            Ok(resolve_declined_pending_draw(state, player, pending))
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
            // SR-14: the library zone is never removed (ground truth 2).
            let library_zone = ZoneId::Library(player);
            let library_count = state
                .expect_zone(&library_zone)
                .map(|z| z.len())
                .unwrap_or(0);
            if (dredge_n as usize) > library_count {
                return Err(GameStateError::InvalidCommand(format!(
                    "cannot dredge {}: library has only {} cards (need {})",
                    card_id.0, library_count, dredge_n
                )));
            }
            // CONSUME the entry now that both validations passed (all
            // validation precedes all mutation).
            let pending = state.pending_draws[idx].clone();
            state.pending_draws.remove(idx);
            let mut events = Vec::new();
            // Step 3: Mill n cards from the top of library.
            for _ in 0..dredge_n {
                // SR-14: the library zone is never removed (ground truth 2); `z.top()`
                // returning None is the legal empty-library case.
                let top = state.expect_zone(&library_zone).and_then(|z| z.top());
                if let Some(top_id) = top {
                    // SR-14: top_id was just read from the live library top and the
                    // graveyard always exists — the move cannot fail.
                    if let Some((new_id, _)) =
                        state.expect_move_object_to_zone(top_id, ZoneId::Graveyard(player))
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
            // Step 7 (CR 614.11a): `Dredged` does not set `has_drawn_for_turn`
            // -- only the tail draws do, per the entry's own flag.
            if pending.remaining > 0 {
                events.extend(perform_remaining_draws(
                    state,
                    player,
                    pending.remaining,
                    pending.sets_has_drawn_for_turn,
                ));
            }
            Ok(events)
        }
    }
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
    // SR-14: object_id is the permanent being regenerated — live at replacement time.
    if let Some(obj) = state.expect_object_mut(object_id) {
        obj.damage_marked = 0;
        obj.deathtouch_damage = false;
    }
    // 2. Tap the permanent
    if let Some(obj) = state.expect_object_mut(object_id) {
        obj.status.tapped = true;
    }
    // 3. Remove from combat (if attacking or blocking) -- CR 506.4/701.19a.
    // PB-OS6(g): factored into a shared helper, reused by Effect::RemoveFromCombat.
    crate::rules::combat::remove_from_combat(state, object_id);
    // 4. Remove the one-shot shield (consumed)
    let keep: imbl::Vector<_> = state
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
            // SR-14: aura_id is a live `state.objects.iter()` loop key (CR 702.89a).
            let chars = crate::rules::layers::expect_characteristics(state, *aura_id);
            if chars.keywords.contains(&KeywordAbility::UmbraArmor) {
                Some(*aura_id)
            } else {
                None
            }
        })
        .collect();
    // Sort by ObjectId for deterministic selection when multiple Auras match
    // (imbl::HashMap iteration order is non-deterministic; replay correctness requires
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
    // SR-14: protected_id is the permanent being protected from destruction — live here.
    if let Some(obj) = state.expect_object_mut(protected_id) {
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
    // SR-14: aura_id was confirmed present at the `aura_owner` match above, and the
    // graveyard always exists — the move cannot fail.
    if state
        .expect_move_object_to_zone(aura_id, ZoneId::Graveyard(aura_owner))
        .is_some()
    {
        events.push(GameEvent::UmbraArmorApplied {
            protected_id,
            aura_id,
        });
    }
    events
}
// ── Counter placement replacement helpers (CR 122.6, 614.1) ──────────────
/// CR 122.6 / CR 614.1: Apply counter-placement replacement effects.
///
/// Before placing `count` counters of type `counter` on a permanent, check
/// for registered WouldPlaceCounters replacement effects. Returns the
/// modified count and any ReplacementEffectApplied events to emit.
///
/// `placer` is the player whose effect is placing the counters (typically
/// the controller of the source that places them).
/// `receiver_id` is the permanent receiving the counters.
///
/// NOTE: Player receivers (poison counters, experience counters, energy counters)
/// are NOT handled here. See `apply_counter_replacement_player` below.
/// Vorinclex's oracle says "permanent or player" but only the permanent path is
/// implemented. Fixing this requires adding `ObjectFilter::Player(PlayerId)` or
/// a parallel `WouldPlaceCountersOnPlayer` trigger variant (PB-12 deferred item).
///
/// Multiple replacement effects are applied in controller order per CR 616.1
/// (deterministic for now — interactive choice deferred to M10+).
/// Each replacement applies at most once per event (CR 614.5).
pub fn apply_counter_replacement(
    state: &GameState,
    placer: PlayerId,
    receiver_id: ObjectId,
    counter: &crate::state::types::CounterType,
    count: u32,
) -> (u32, Vec<GameEvent>) {
    let mut events = Vec::new();
    if count == 0 {
        return (0, events);
    }
    // PB-CD: thread the concrete counter type into the event trigger so that
    // typed effects (Hardened Scales: +1/+1 only) gate on counter type at match time.
    let event_trigger = ReplacementTrigger::WouldPlaceCounters {
        placer_filter: PlayerFilter::Specific(placer),
        receiver_filter: ObjectFilter::SpecificObject(receiver_id),
        counter_filter: Some(counter.clone()),
    };
    let applicable = find_applicable(state, &event_trigger, &std::collections::HashSet::new());
    let mut modified_count = count;
    for effect_id in &applicable {
        if let Some(effect) = state
            .replacement_effects
            .iter()
            .find(|e| e.id == *effect_id)
        {
            match &effect.modification {
                ReplacementModification::DoubleCounters => {
                    modified_count *= 2;
                    events.push(GameEvent::ReplacementEffectApplied {
                        effect_id: *effect_id,
                        description: format!("doubled counters: {} → {}", count, modified_count),
                    });
                }
                ReplacementModification::HalveCounters => {
                    modified_count /= 2;
                    events.push(GameEvent::ReplacementEffectApplied {
                        effect_id: *effect_id,
                        description: format!("halved counters: {} → {}", count, modified_count),
                    });
                }
                ReplacementModification::AddExtraCounter => {
                    modified_count += 1;
                    events.push(GameEvent::ReplacementEffectApplied {
                        effect_id: *effect_id,
                        description: format!("added extra counter: {} → {}", count, modified_count),
                    });
                }
                _ => {}
            }
        }
    }
    (modified_count, events)
}
/// CR 122.6 / CR 614.1: Apply counter-placement replacement effects for player receivers.
///
/// Vorinclex, Monstrous Raider oracle: "If you would put one or more counters on a
/// **permanent or player**, put twice that many ... instead."
///
/// TODO (PB-12 deferred): Implement this function and call it from the infect/poison counter
/// paths in effects/mod.rs (~line 215) and combat.rs (~line 1595). Requires either:
///
///   - Adding `ObjectFilter::Player(PlayerId)` and updating `event_object_matches_filter`
///     to match player receivers, OR
///
///   - Adding a new `WouldPlaceCountersOnPlayer` trigger variant parallel to
///     `WouldPlaceCounters` and updating registration in `register_permanent_replacements`.
///
/// Until this is fixed, Vorinclex does not double poison/experience/energy counters
/// placed on players.
#[allow(dead_code)]
pub fn apply_counter_replacement_player(
    _state: &GameState,
    _placer: PlayerId,
    _receiver_player: PlayerId,
    _counter: &crate::state::types::CounterType,
    count: u32,
) -> (u32, Vec<GameEvent>) {
    // TODO: implement player-receiver counter replacement (PB-12 deferred).
    (count, Vec::new())
}
/// CR 111.1 / CR 614.1: Apply token-creation replacement effects.
///
/// Before creating `count` tokens, check for registered WouldCreateTokens
/// replacement effects. Returns the modified count and events.
pub fn apply_token_creation_replacement(
    state: &GameState,
    controller: PlayerId,
    count: u32,
) -> (u32, Vec<GameEvent>) {
    let mut events = Vec::new();
    if count == 0 {
        return (0, events);
    }
    let event_trigger = ReplacementTrigger::WouldCreateTokens {
        controller_filter: PlayerFilter::Specific(controller),
    };
    let applicable = find_applicable(state, &event_trigger, &std::collections::HashSet::new());
    let mut modified_count = count;
    for effect_id in &applicable {
        if let Some(effect) = state
            .replacement_effects
            .iter()
            .find(|e| e.id == *effect_id)
        {
            if matches!(effect.modification, ReplacementModification::DoubleTokens) {
                modified_count *= 2;
                events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: *effect_id,
                    description: format!("doubled tokens: {} → {}", count, modified_count),
                });
            }
        }
    }
    (modified_count, events)
}
/// CR 701.23 / CR 614.1: Apply library-search replacement effects.
///
/// Before searching a library, check for registered WouldSearchLibrary
/// replacement effects. Returns `Some(top_n)` if search should be restricted
/// to the top N cards, or `None` for unrestricted search.
pub fn apply_search_library_replacement(
    state: &GameState,
    searcher: PlayerId,
) -> (Option<u32>, Vec<GameEvent>) {
    let mut events = Vec::new();
    let event_trigger = ReplacementTrigger::WouldSearchLibrary {
        searcher_filter: PlayerFilter::Specific(searcher),
    };
    let applicable = find_applicable(state, &event_trigger, &std::collections::HashSet::new());
    let mut restriction: Option<u32> = None;
    for effect_id in &applicable {
        if let Some(effect) = state
            .replacement_effects
            .iter()
            .find(|e| e.id == *effect_id)
        {
            if let ReplacementModification::RestrictSearchTopN(n) = &effect.modification {
                // Take the most restrictive (smallest) restriction
                restriction = Some(match restriction {
                    Some(existing) => existing.min(*n),
                    None => *n,
                });
                events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: *effect_id,
                    description: format!("search restricted to top {} cards", n),
                });
            }
        }
    }
    (restriction, events)
}
/// CR 614.1: Apply damage-doubling replacement effects.
///
/// Checks for registered DamageWouldBeDealt replacements with DoubleDamage or TripleDamage
/// modification where the source is controlled by the matching player and
/// (optionally) the target matches the effect's target filter.
/// Called before `apply_damage_prevention` in the damage path.
///
/// `damage_target`: the target of the damage event, used to match
/// `DamageTargetFilter::ToOpponentOrTheirPermanent` and `ToPlayerOrTheirPermanents`.
/// Pass `None` to skip target-side filtering (applies multiplier regardless of target).
///
/// Returns `(modified_amount, events)`.
pub fn apply_damage_doubling(
    state: &GameState,
    source: ObjectId,
    amount: u32,
    damage_target: Option<&CombatDamageTarget>,
) -> (u32, Vec<GameEvent>) {
    use crate::rules::layers::calculate_characteristics;
    use crate::state::types::CardType;
    let mut events = Vec::new();
    if amount == 0 {
        return (0, events);
    }
    let source_controller = state.objects.get(&source).map(|o| o.controller);
    let mut modified = amount;
    for effect in state.replacement_effects.iter() {
        let multiplier = match &effect.modification {
            ReplacementModification::DoubleDamage => 2u32,
            ReplacementModification::TripleDamage => 3u32,
            _ => continue,
        };
        if let ReplacementTrigger::DamageWouldBeDealt { target_filter } = &effect.trigger {
            let applies = match target_filter {
                DamageTargetFilter::FromControllerSources(pid) => {
                    // Source-side only: multiplies damage from controller's sources to any target.
                    source_controller == Some(*pid)
                }
                DamageTargetFilter::ToOpponentOrTheirPermanent(controller_pid) => {
                    // CR 614.1 / Twinflame Tyrant: "If a source you control would deal
                    // damage to an opponent or a permanent an opponent controls."
                    // Checks BOTH: source is controlled by controller_pid, AND target is
                    // an opponent of controller_pid or a permanent they control.
                    if source_controller != Some(*controller_pid) {
                        false
                    } else {
                        match damage_target {
                            Some(dt) => damage_target_is_opponent_or_their_permanent(
                                state,
                                dt,
                                *controller_pid,
                            ),
                            None => true, // No target info — apply conservatively.
                        }
                    }
                }
                DamageTargetFilter::ToPlayerOrTheirPermanents(pid) => {
                    // CR 614.1 / Lightning Stagger: "damage to that player or a permanent
                    // that player controls". Targets a specific player by ID.
                    match damage_target {
                        Some(CombatDamageTarget::Player(p)) => p == pid,
                        Some(CombatDamageTarget::Creature(id))
                        | Some(CombatDamageTarget::Planeswalker(id)) => state
                            .objects
                            .get(id)
                            .map(|o| o.controller == *pid)
                            .unwrap_or(false),
                        None => true, // No target info — apply conservatively.
                    }
                }
                DamageTargetFilter::FromControllerCreaturesEnteredThisTurn(pid) => {
                    // CR 614.1 / Neriv: "a creature you control that entered this turn".
                    // Source must be: controlled by pid, a creature, entered this turn.
                    if source_controller != Some(*pid) {
                        false
                    } else {
                        state
                            .objects
                            .get(&source)
                            .map(|o| {
                                o.entered_turn == Some(state.turn.turn_number)
                                    && calculate_characteristics(state, source)
                                        .map(|c| c.card_types.contains(&CardType::Creature))
                                        .unwrap_or(false)
                            })
                            .unwrap_or(false)
                    }
                }
                DamageTargetFilter::Any => true,
                _ => false,
            };
            if !applies {
                continue;
            }
            let before = modified;
            modified *= multiplier;
            events.push(GameEvent::ReplacementEffectApplied {
                effect_id: effect.id,
                description: format!("{}x damage: {} → {}", multiplier, before, modified),
            });
        }
    }
    (modified, events)
}
/// Helper: check if a damage target is an opponent of `controller_pid` or a permanent
/// they control. Used by `DamageTargetFilter::ToOpponentOrTheirPermanent`.
fn damage_target_is_opponent_or_their_permanent(
    state: &GameState,
    target: &CombatDamageTarget,
    controller_pid: PlayerId,
) -> bool {
    match target {
        CombatDamageTarget::Player(p) => *p != controller_pid,
        CombatDamageTarget::Creature(id) | CombatDamageTarget::Planeswalker(id) => state
            .objects
            .get(id)
            .map(|o| o.controller != controller_pid)
            .unwrap_or(false),
    }
}
/// CR 614.1: Apply life-loss doubling replacement effects.
///
/// Checks for registered WouldLoseLife replacements with DoubleLifeLoss
/// modification. Called before applying life loss.
///
/// Returns `(modified_amount, events)`.
pub fn apply_life_loss_doubling(
    state: &GameState,
    player: PlayerId,
    amount: u32,
) -> (u32, Vec<GameEvent>) {
    let mut events = Vec::new();
    if amount == 0 {
        return (0, events);
    }
    let event_trigger = ReplacementTrigger::WouldLoseLife {
        player_filter: PlayerFilter::Specific(player),
    };
    let applicable = find_applicable(state, &event_trigger, &std::collections::HashSet::new());
    let mut modified = amount;
    for effect_id in &applicable {
        if let Some(effect) = state
            .replacement_effects
            .iter()
            .find(|e| e.id == *effect_id)
        {
            if matches!(effect.modification, ReplacementModification::DoubleLifeLoss) {
                // CR 614.1 / Bloodletter of Aclazotz: "during your turn" condition.
                // The doubling only applies when the effect's controller is the active player.
                if state.turn.active_player != effect.controller {
                    continue;
                }
                modified *= 2;
                events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: *effect_id,
                    description: format!("doubled life loss: {} → {}", amount, modified),
                });
            }
        }
    }
    (modified, events)
}
/// CR 701.34 / CR 614.1: Check if proliferate should be doubled.
///
/// Returns the number of times to proliferate (1 normally, 2 with Tekuthal, etc.)
/// and any ReplacementEffectApplied events.
pub fn apply_proliferate_replacement(
    state: &GameState,
    controller: PlayerId,
) -> (u32, Vec<GameEvent>) {
    let mut events = Vec::new();
    let event_trigger = ReplacementTrigger::WouldProliferate {
        player_filter: PlayerFilter::Specific(controller),
    };
    let applicable = find_applicable(state, &event_trigger, &std::collections::HashSet::new());
    let mut times = 1u32;
    for effect_id in &applicable {
        if let Some(effect) = state
            .replacement_effects
            .iter()
            .find(|e| e.id == *effect_id)
        {
            if matches!(
                effect.modification,
                ReplacementModification::DoubleProliferate
            ) {
                times *= 2;
                events.push(GameEvent::ReplacementEffectApplied {
                    effect_id: *effect_id,
                    description: format!("proliferate doubled: {} times", times),
                });
            }
        }
    }
    (times, events)
}
