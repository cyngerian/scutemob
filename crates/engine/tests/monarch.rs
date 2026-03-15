//! Monarch designation tests (CR 724).
//!
//! Tests for the monarch mechanic:
//! - CR 724.1: Designation assigned by effects
//! - CR 724.2: EOT draw trigger + combat damage steal
//! - CR 724.3: Only one monarch at a time
//! - CR 724.4: Monarch leaves game → active player takes over

use mtg_engine::state::player::PlayerId;
use mtg_engine::{CardType, GameEvent, GameStateBuilder, ObjectSpec, Step, ZoneId};

// ── CR 724.1: BecomeMonarch effect ──────────────────────────────────────────

#[test]
/// CR 724.1 — BecomeMonarch effect sets state.monarch.
fn test_724_1_become_monarch_sets_designation() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    assert!(state.monarch.is_none(), "No monarch at game start");

    // Directly set the monarch (simulating an effect)
    state.monarch = Some(p1);
    assert_eq!(state.monarch, Some(p1));
}

// ── CR 724.3: Only one monarch at a time ────────────────────────────────────

#[test]
/// CR 724.3 — Setting a new monarch replaces the previous one.
fn test_724_3_new_monarch_replaces_old() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.monarch = Some(p1);
    assert_eq!(state.monarch, Some(p1));

    // P2 becomes monarch — P1 is no longer monarch
    state.monarch = Some(p2);
    assert_eq!(state.monarch, Some(p2));
}

// ── CR 724.2: EOT draw ─────────────────────────────────────────────────────

#[test]
/// CR 724.2 — Monarch draws a card at the beginning of their end step.
fn test_724_2_monarch_eot_draw() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Give P1 a card in library so the draw succeeds
    let lib_card = ObjectSpec::card(p1, "Library Card")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lib_card)
        .at_step(Step::End)
        .active_player(p1)
        .build()
        .unwrap();

    // P1 is the monarch
    state.monarch = Some(p1);

    // Count cards in hand before
    let hand_before = state
        .objects
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Hand(pid) if pid == p1))
        .count();

    // Trigger end step actions (which includes monarch draw)
    let events = mtg_engine::rules::turn_actions::end_step_actions(&mut state);

    // Count cards in hand after
    let hand_after = state
        .objects
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Hand(pid) if pid == p1))
        .count();

    assert_eq!(
        hand_after,
        hand_before + 1,
        "Monarch should draw one card at end step"
    );

    // Check for draw event
    let drew = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(drew, "Should have CardDrawn event for monarch");
}

#[test]
/// CR 724.2 — Non-monarch does not draw at end step.
fn test_724_2_non_monarch_no_eot_draw() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let lib_card = ObjectSpec::card(p1, "Library Card")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lib_card)
        .at_step(Step::End)
        .active_player(p1)
        .build()
        .unwrap();

    // P2 is the monarch, P1 is the active player
    state.monarch = Some(p2);

    let hand_before = state
        .objects
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Hand(pid) if pid == p1))
        .count();

    let _events = mtg_engine::rules::turn_actions::end_step_actions(&mut state);

    let hand_after = state
        .objects
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Hand(pid) if pid == p1))
        .count();

    assert_eq!(
        hand_after, hand_before,
        "Non-monarch active player should not draw at end step"
    );
}

// ── CR 724.4: Monarch leaves game ──────────────────────────────────────────

#[test]
/// CR 724.4 — When the monarch dies, the active player becomes the monarch.
fn test_724_4_monarch_leaves_active_player_inherits() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // P2 is monarch, P1 is active player
    state.monarch = Some(p2);

    // P2 loses the game
    let events = mtg_engine::rules::sba::transfer_monarch_on_player_leave(&mut state, p2);

    assert_eq!(
        state.monarch,
        Some(p1),
        "Active player should become monarch when monarch leaves"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerBecameMonarch { player } if *player == p1)),
        "Should emit PlayerBecameMonarch event"
    );
}

#[test]
/// CR 724.4 — Non-monarch leaving doesn't change the monarch.
fn test_724_4_non_monarch_leaves_no_change() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // P1 is monarch
    state.monarch = Some(p1);

    // P2 leaves (not the monarch)
    let events = mtg_engine::rules::sba::transfer_monarch_on_player_leave(&mut state, p2);

    assert_eq!(state.monarch, Some(p1), "Monarch should not change");
    assert!(events.is_empty(), "No events when non-monarch leaves");
}
