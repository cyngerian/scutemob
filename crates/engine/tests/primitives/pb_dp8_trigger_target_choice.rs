//! PB-DP8 (DP-6 / OOS-M11-4) — triggered-ability targets become a player choice.
//!
//! CR 603.3d: "The remainder of the process for putting a triggered ability on the
//! stack is identical to the process for casting a spell listed in rules 601.2c-d."
//! CR 601.2c: "The player announces their choice of an appropriate object or player
//! for each target the spell requires."
//! CR 603.3b: the CR 603.3b batch is placed in APNAP order, one ability at a time.

use mtg_engine::rules::abilities::default_trigger_targets;
use mtg_engine::{process_command, Command, GameEvent, GameState, TriggerTargetOption};

/// Answer any outstanding CR 603.3d target choice with the engine's own default,
/// through `process_command`.
///
/// **Panics if nothing is pending** -- so it can never mask a missing block (the
/// `answer_pending_cleanup_discard` precedent from PB-DP7).
#[allow(dead_code)] // TEMP-DP8
pub fn answer_pending_trigger_targets(state: GameState) -> (GameState, Vec<GameEvent>) {
    let entry = state
        .pending_trigger_targets()
        .expect("no CR 603.3d trigger-target choice is pending");
    let player = entry.player;
    let choice_id = entry.choice_id;
    let slots: Vec<TriggerTargetOption> = entry.slots.iter().cloned().collect();
    let targets = default_trigger_targets(&slots);
    process_command(
        state,
        Command::ChooseTriggerTargets {
            player,
            choice_id,
            targets,
        },
    )
    .expect("the engine must accept its own default answer (SR-38)")
}

/// The in-place twin, for the tests that drive `flush_pending_triggers` directly
/// (a `&mut GameState` API) rather than through `process_command`.
///
/// Answers with the engine's own default, which is byte-identical to the
/// pre-PB-DP8 first-match auto-pick, so a test written before this batch keeps
/// pinning exactly what it was written to pin. Returns how many questions were
/// answered; `0` means the flush never suspended.
pub fn answer_pending_trigger_targets_in_place(state: &mut GameState) -> usize {
    let mut n = 0;
    while let Some(entry) = state.pending_trigger_targets() {
        let player = entry.player;
        let choice_id = entry.choice_id;
        let slots: Vec<TriggerTargetOption> = entry.slots.iter().cloned().collect();
        let targets = default_trigger_targets(&slots);
        mtg_engine::rules::abilities::handle_choose_trigger_targets(
            state, player, choice_id, targets,
        )
        .expect("the engine must accept its own default answer (SR-38)");
        n += 1;
        assert!(n < 256, "trigger-target answers did not converge");
    }
    n
}

/// Answer every outstanding CR 603.3d target choice with the engine's default,
/// looping until the CR 603.3b batch completes. Returns the number answered.
#[allow(dead_code)] // TEMP-DP8
pub fn answer_all_pending_trigger_targets(state: GameState) -> (GameState, usize) {
    let mut state = state;
    let mut n = 0;
    while state.pending_trigger_targets().is_some() {
        let (s, _) = answer_pending_trigger_targets(state);
        state = s;
        n += 1;
        assert!(n < 256, "trigger-target answers did not converge");
    }
    (state, n)
}
use mtg_card_defs::all_cards;
use mtg_card_types::cards::card_definition::{AbilityDefinition, Completeness};

/// CR 603.3d / SR-36 — the PB-DP8 roster, derived by enumerating `all_cards()`
/// rather than by grepping source.
///
/// A def is in the roster iff some `AbilityDefinition::Triggered` on **any** of its
/// faces (front, `back_face`, `adventure_face`) declares a non-empty `targets`, and
/// the def is `Completeness::Complete` (i.e. legal in a deck, per SR-2). Those are
/// exactly the defs whose trigger reaches
/// `rules::abilities::flush_pending_triggers`'s CR 603.3d announcement.
///
/// The assertion is `>=` on purpose: the authoring campaign adds cards continuously
/// and an `==` pin would redden on unrelated work.
#[test]
fn test_dp8_roster_enumeration() {
    fn has_targeted_trigger(abilities: &[AbilityDefinition]) -> bool {
        abilities.iter().any(|a| {
            matches!(a, AbilityDefinition::Triggered { targets, .. } if !targets.is_empty())
        })
    }

    let mut roster: Vec<String> = Vec::new();
    let mut incomplete = 0usize;
    for def in all_cards() {
        let mut hit = has_targeted_trigger(&def.abilities);
        if let Some(face) = def.back_face.as_ref() {
            hit |= has_targeted_trigger(&face.abilities);
        }
        if let Some(face) = def.adventure_face.as_ref() {
            hit |= has_targeted_trigger(&face.abilities);
        }
        if !hit {
            continue;
        }
        if def.completeness == Completeness::Complete {
            roster.push(def.name.clone());
        } else {
            incomplete += 1;
        }
    }
    roster.sort();
    println!(
        "PB-DP8 roster: {} effectively-Complete defs with a targeted triggered ability \
         ({} more carry a non-Complete marker)",
        roster.len(),
        incomplete
    );
    for name in &roster {
        println!("  {name}");
    }
    assert!(
        roster.len() >= 60,
        "PB-DP8 roster collapsed to {} defs (expected >= 60)",
        roster.len()
    );
}
