//! PB-DX22 — probes for the fuzzer's pregame build path (`mtg_simulator::fuzz_setup`).
//!
//! CR 103.3 (shuffle), CR 103.5 (opening hand — deliberately NOT dealt here),
//! CR 903.6 (commander to the command zone, remainder shuffled into the library),
//! CR 903.8 (commander tax), CR 903.9a (zone-return SBA), CR 903.9b (hand/library
//! redirect replacements).
//!
//! **These gate the binary, not a copy of it.** `mtg-fuzzer`'s `run_single_game` calls
//! `build_fuzz_state` and does nothing else to the state, so a probe here is a probe on
//! `src/bin/fuzzer.rs` — which Cargo compiles as its own crate and no integration test
//! can otherwise reach (plan §B3).
//!
//! SR-9a does not apply: that gate (`crates/engine/tests/no_stray_test_binaries.rs`) is
//! scoped to `CARGO_MANIFEST_DIR = crates/engine`. `crates/simulator/tests/` is a flat
//! directory of integration targets and adding one is the convention here.

use std::sync::Arc;

use mtg_engine::{all_cards, CardDefinition, CardRegistry, PlayerId, ZoneId};
use mtg_simulator::{build_fuzz_state, build_registry, FuzzGameSetup};

const PLAYERS: u32 = 4;

fn pool() -> (Vec<CardDefinition>, Arc<CardRegistry>) {
    (all_cards(), build_registry())
}

fn seats(player_count: u32) -> Vec<PlayerId> {
    (1..=player_count).map(|i| PlayerId(i as u64)).collect()
}

fn built(seed: u64, cards: &[CardDefinition], registry: &Arc<CardRegistry>) -> FuzzGameSetup {
    build_fuzz_state(seed, PLAYERS, cards, registry)
        .unwrap_or_else(|e| panic!("fuzz state for seed {seed} must build: {e:?}"))
}

// ── P1 ───────────────────────────────────────────────────────────────────────────

/// CR 903.6 / CR 103.5 — the extracted build path produces exactly the table
/// `mtg-fuzzer` has always played: commander in the command zone, 99 cards in the
/// library, and **no opening hand**.
///
/// The empty-hand assertion is not incidental: it PINS plan §B2's decision to leave
/// CR 103.5 out of scope (`OOS-DX22-1`). If a successor deals seven cards, this probe
/// is the thing that says so out loud.
#[test]
fn test_dx22_build_fuzz_state_produces_the_fuzzers_table() {
    let (cards, registry) = pool();
    let setup = built(1, &cards, &registry);

    assert_eq!(
        setup.decks.len(),
        PLAYERS as usize,
        "one decklist per seat must be returned"
    );

    for (i, pid) in seats(PLAYERS).into_iter().enumerate() {
        let (deck_pid, deck) = &setup.decks[i];
        assert_eq!(*deck_pid, pid, "decks must be in ascending PlayerId order");
        assert_eq!(
            deck.main_deck.len(),
            99,
            "CR 903.5a: seat {pid:?}'s main deck is 99 cards + the commander"
        );

        let command = setup.state.objects_in_zone(&ZoneId::Command(pid));
        assert_eq!(
            command.len(),
            1,
            "CR 903.6: seat {pid:?} must have exactly its commander in the command zone"
        );
        assert_eq!(
            command[0].card_id.as_ref(),
            Some(&deck.commander),
            "the command-zone object must BE the decklist's commander"
        );

        assert_eq!(
            setup.state.objects_in_zone(&ZoneId::Library(pid)).len(),
            99,
            "seat {pid:?}'s library must hold all 99 main-deck cards"
        );

        assert_eq!(
            setup.state.objects_in_zone(&ZoneId::Hand(pid)).len(),
            0,
            "CR 103.5 is NOT dealt on the fuzz path (plan §B2 / OOS-DX22-1): seat \
             {pid:?} starts with an empty hand"
        );
    }
}
