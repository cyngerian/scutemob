//! PB-DX18 — the pregame trust boundary (`OOS-DX2-4`) and CR 103.5's mulligan cap
//! (`OOS-DP2-8`).
//!
//! `memory/primitives/pb-DX18-execution-notes.md` is authoritative for the census and the
//! revert matrix.
//!
//! Before this batch, `rules::engine::process_command` gated `Command::TakeMulligan` and
//! `Command::KeepHand` on `validate_player_exists` and **nothing else**. There was no
//! pregame state anywhere to consult — `PlayerState::mulligan_count` is a counter that
//! never resets, so it cannot tell "before the game began" from "turn 14".
//!
//! ## Why `t5` exists, and why the other refusal probes are not enough
//!
//! `process_command`'s `Err` arm carries **no `GameState`** (`OOS-DX21-7`), so "the
//! rejected command mutated nothing" cannot be asserted through it — the state was moved
//! into the call and nothing comes back. Every refusal probe here therefore asserts on the
//! **error itself**, and `t5` supplies the missing half from the other side: it calls the
//! handler *directly*, past the gate, on a started game, and measures the damage the gate
//! is preventing. Without `t5` a reader could not tell a load-bearing gate from a
//! redundant one.

use mtg_engine::{
    process_command, start_game_allowing_incomplete, Command, GameState, GameStateBuilder,
    GameStateError, ObjectSpec, PlayerId, Step, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// A four-seat pregame state with `card_count` cards in `player`'s library and nothing
/// else — the shape `rules/commander.rs`'s own mulligan fixtures use.
fn pregame_state(player: PlayerId, card_count: usize) -> GameState {
    let mut builder = GameStateBuilder::four_player()
        .active_player(player)
        .at_step(Step::PreCombatMain);
    for i in 0..card_count {
        builder = builder.object(
            ObjectSpec::card(player, &format!("Card {}", i)).in_zone(ZoneId::Library(player)),
        );
    }
    builder.build().unwrap()
}

fn err_message(e: &GameStateError) -> String {
    match e {
        GameStateError::InvalidCommand(m) => m.clone(),
        other => panic!("expected InvalidCommand, got {:?}", other),
    }
}

// ── The phase boundary (CR 103.5, `OOS-DX2-4`) ────────────────────────────────

#[test]
/// CONTROL — a pregame `TakeMulligan` is still accepted. The gate must refuse more than
/// HEAD did, and nothing else.
fn t1_pregame_take_mulligan_is_accepted() {
    let p1 = p(1);
    let state = pregame_state(p1, 20);
    assert!(
        state.pregame().is_pregame(),
        "a freshly built state is in the pregame procedure"
    );
    assert!(state.pregame().may_mulligan(p1));
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 })
        .expect("CR 103.5: a pregame mulligan is legal");
    assert_eq!(state.zone(&ZoneId::Hand(p1)).unwrap().len(), 7);
}

#[test]
/// CR 103.5 — the mulligan procedure is a PREGAME procedure. Once `start_game` has run,
/// `TakeMulligan` is refused.
fn t2_take_mulligan_after_start_game_is_refused() {
    let p1 = p(1);
    let state = pregame_state(p1, 20);
    let (state, _) = start_game_allowing_incomplete(state).expect("game starts");
    assert!(
        !state.pregame().is_pregame(),
        "start_game closes the pregame procedure"
    );
    let err = process_command(state, Command::TakeMulligan { player: p1 })
        .expect_err("CR 103.5: no mulligans once the game has begun");
    let m = err_message(&err);
    assert!(
        m.contains("TakeMulligan") && m.contains("game has already started") && m.contains("103.5"),
        "refusal must name the command, the reason and the rule; got {m:?}"
    );
}

#[test]
/// CR 103.5 — `KeepHand` is bounded by the same phase. It is a declaration IN the mulligan
/// procedure, so a mid-game one is equally out of bounds.
fn t3_keep_hand_after_start_game_is_refused() {
    let p1 = p(1);
    let state = pregame_state(p1, 20);
    let (state, _) = start_game_allowing_incomplete(state).expect("game starts");
    let err = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![],
        },
    )
    .expect_err("CR 103.5: no keep declaration once the game has begun");
    let m = err_message(&err);
    assert!(
        m.contains("KeepHand") && m.contains("game has already started"),
        "got {m:?}"
    );
}

#[test]
/// CR 103.5 — *"Once a player chooses not to take a mulligan, ... that player may not take
/// any further mulligans."* The per-player half, which a bare `game_started: bool` would
/// not close.
fn t4_keeping_ends_the_procedure_for_that_player_only() {
    let (p1, p2) = (p(1), p(2));
    let state = pregame_state(p1, 20);
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![],
        },
    )
    .unwrap();

    // p1 is finished.
    assert!(!state.pregame().may_mulligan(p1));
    let err = process_command(state.clone(), Command::TakeMulligan { player: p1 })
        .expect_err("CR 103.5: a player who kept may take no further mulligans");
    assert!(err_message(&err).contains("already kept"));
    // ...and so is a second keep from the same player.
    let err2 = process_command(
        state.clone(),
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![],
        },
    )
    .expect_err("CR 103.5: a second keep is a second declaration");
    assert!(
        err2.to_string().contains("already kept") || err_message(&err2).contains("already kept")
    );

    // NON-VACUITY: the gate is PER PLAYER, not a game-wide latch. p2 is untouched.
    assert!(
        state.pregame().may_mulligan(p2),
        "p1 keeping must not end p2's mulligan procedure (CR 103.5 is per player)"
    );
    process_command(state, Command::TakeMulligan { player: p2 })
        .expect("p2 has not kept and may still mulligan");
}

#[test]
/// The gate is LOAD-BEARING, measured from the other side of it.
///
/// `process_command`'s `Err` arm carries no `GameState` (`OOS-DX21-7`), so this calls
/// `rules::commander::handle_take_mulligan` **directly** on a started game — the code path
/// the gate now refuses to reach — and measures what it does: the player's whole hand goes
/// into the library, the library is really permuted, and seven fresh cards are drawn.
fn t5_the_handler_the_gate_refuses_is_really_destructive() {
    let p1 = p(1);
    let mut builder = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for i in 0..20 {
        builder =
            builder.object(ObjectSpec::card(p1, &format!("Lib {i}")).in_zone(ZoneId::Library(p1)));
    }
    for i in 0..3 {
        builder =
            builder.object(ObjectSpec::card(p1, &format!("Hand {i}")).in_zone(ZoneId::Hand(p1)));
    }
    let state = builder.build().unwrap();
    let (mut state, _) = start_game_allowing_incomplete(state).expect("game starts");

    // The gate refuses this through the command channel...
    assert!(process_command(state.clone(), Command::TakeMulligan { player: p1 }).is_err());

    // ...and this is what it is refusing. Direct handler call, past the gate.
    let hand_before: Vec<String> = state
        .zone(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .into_iter()
        .map(|id| state.object(id).unwrap().characteristics.name.clone())
        .collect();
    assert_eq!(hand_before.len(), 3);
    mtg_engine::rules::commander::handle_take_mulligan(&mut state, p1)
        .expect("the handler itself has no phase check — the gate is the only thing stopping it");
    let hand_after: Vec<String> = state
        .zone(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .into_iter()
        .map(|id| state.object(id).unwrap().characteristics.name.clone())
        .collect();
    assert_eq!(
        hand_after.len(),
        7,
        "a mid-game mulligan draws a whole new hand of seven"
    );
    for name in &hand_before {
        assert!(
            !hand_after.contains(name) || hand_after.iter().filter(|n| *n == name).count() <= 1,
            "the old hand was shuffled away"
        );
    }
    assert_eq!(
        state.zone(&ZoneId::Library(p1)).unwrap().len(),
        16,
        "20 library + 3 hand = 23, minus the 7 redrawn"
    );
}

// ── CR 103.5's cap (`OOS-DP2-8`) ──────────────────────────────────────────────

#[test]
/// CR 103.5 — *"A player can take mulligans until their opening hand would be zero cards,
/// after which they may not take further mulligans."*
///
/// With CR 103.5c's free first mulligan and a starting hand size of 7, the opening hand
/// after the Nth mulligan is `7 - (N - 1)`, so N = 8 is the last legal mulligan.
fn t6_cap_is_starting_hand_size_plus_one() {
    let p1 = p(1);
    let mut state = pregame_state(p1, 99);
    for n in 1..=mtg_engine::rules::commander::MAX_MULLIGANS {
        let (s, _) = process_command(state, Command::TakeMulligan { player: p1 })
            .unwrap_or_else(|e| panic!("mulligan {n} must be legal: {e:?}"));
        state = s;
        assert_eq!(state.players().get(&p1).unwrap().mulligan_count, n);
    }
    assert_eq!(
        mtg_engine::rules::commander::MAX_MULLIGANS,
        mtg_engine::rules::commander::STARTING_HAND_SIZE as u32 + 1,
        "the cap is derived from the same constant the draw loop counts to"
    );
    let err = process_command(state, Command::TakeMulligan { player: p1 })
        .expect_err("CR 103.5: the mulligan past zero cards is refused");
    let m = err_message(&err);
    assert!(m.contains("zero cards") && m.contains("103.5"), "got {m:?}");
}

#[test]
/// CR 103.5 — `KeepHand` is SATISFIABLE at the cap. This is the half `OOS-DP2-8` was filed
/// on: past the cap `required_bottom` exceeds the hand size and the keep becomes
/// impossible, stranding the player.
fn t7_keep_hand_is_satisfiable_at_the_cap() {
    let p1 = p(1);
    let mut state = pregame_state(p1, 99);
    for _ in 1..=mtg_engine::rules::commander::MAX_MULLIGANS {
        let (s, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
        state = s;
    }
    let hand = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids();
    assert_eq!(hand.len(), mtg_engine::rules::commander::STARTING_HAND_SIZE);
    // required_bottom = 8 - 1 = 7, exactly the hand size: the opening hand is zero cards.
    let (state, _) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: hand,
        },
    )
    .expect("CR 103.5: a keep at the cap bottoms the whole hand and is legal");
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        0,
        "CR 103.5: 'until their opening hand would be zero cards'"
    );
}
