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

use mtg_engine::{all_cards, CardDefinition, CardId, CardRegistry, ObjectId, PlayerId, ZoneId};
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

/// The ORDERED library sequence for a seat, read from the `Zone::Ordered` vector — NOT
/// from `objects_in_zone`, which yields objects in the zone storage's own order and
/// would make an order assertion meaningless.
fn library_card_ids(setup: &FuzzGameSetup, pid: PlayerId) -> Vec<CardId> {
    let zone = setup
        .state
        .zones()
        .get(&ZoneId::Library(pid))
        .unwrap_or_else(|| panic!("seat {pid:?} must have a library zone"));
    zone.object_ids()
        .into_iter()
        .map(|oid: ObjectId| {
            setup
                .state
                .object(oid)
                .expect("a library object id must resolve")
                .card_id
                .clone()
                .expect("every library card must carry a CardId")
        })
        .collect()
}

fn sorted(mut ids: Vec<CardId>) -> Vec<CardId> {
    ids.sort();
    ids
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

// ── P2 ───────────────────────────────────────────────────────────────────────────

/// CR 103.3 / CR 903.6 — every seat's library is a *permutation* of the decklist
/// `random_deck` produced, not that decklist.
///
/// Written as sequence-INEQUALITY plus multiset-EQUALITY on purpose, so it is
/// structure-independent: `deck.rs` pads a coloured commander's deck with ~34 basics but
/// pads a **colourless** one with colourless nonlands (CR 903.5c, the PB-DX4 arm), so a
/// "basics are no longer last" assertion would be false for those seats. Do not
/// "improve" this into a position assertion.
///
/// Non-vacuity floor: both sides are asserted to be exactly 99 long before they are
/// compared, so a build that produced two empty vectors cannot pass.
#[test]
fn test_dx22_libraries_are_shuffled_cr_103_3() {
    let (cards, registry) = pool();
    let setup = built(1, &cards, &registry);

    for (i, pid) in seats(PLAYERS).into_iter().enumerate() {
        let pre_shuffle = setup.decks[i].1.main_deck.clone();
        let library = library_card_ids(&setup, pid);

        assert_eq!(
            pre_shuffle.len(),
            99,
            "non-vacuity: seat {pid:?} pre-shuffle deck"
        );
        assert_eq!(library.len(), 99, "non-vacuity: seat {pid:?} built library");

        assert_ne!(
            library, pre_shuffle,
            "CR 103.3: seat {pid:?}'s library must not be the decklist in its \
             construction order — that is the unshuffled instrument PB-DX22 closes"
        );
        assert_eq!(
            sorted(library),
            sorted(pre_shuffle),
            "CR 103.3: a shuffle is a permutation — seat {pid:?}'s library must hold \
             exactly the same cards, no more and no fewer"
        );
    }
}

// ── P3 ───────────────────────────────────────────────────────────────────────────

/// CR 103.3 + this crate's determinism contract — the shuffle is drawn from the game's
/// own seeded RNG, so one seed reproduces one table.
///
/// Asserts both the per-seat library sequence AND `public_state_hash`, because the
/// sequence alone would not notice a non-determinism that lived anywhere else in the
/// build.
#[test]
fn test_dx22_shuffle_is_seed_deterministic() {
    let (cards, registry) = pool();
    let a = built(1, &cards, &registry);
    let b = built(1, &cards, &registry);

    for pid in seats(PLAYERS) {
        let lib_a = library_card_ids(&a, pid);
        assert_eq!(lib_a.len(), 99, "non-vacuity: seat {pid:?}");
        assert_eq!(
            lib_a,
            library_card_ids(&b, pid),
            "seed 1 must reproduce seat {pid:?}'s library order exactly"
        );
    }

    assert_eq!(
        a.state.public_state_hash(),
        b.state.public_state_hash(),
        "seed 1 must reproduce the whole built state, not just the libraries"
    );
}

// ── P4 ───────────────────────────────────────────────────────────────────────────

/// CR 103.3 — a different seed deals a different table.
///
/// Only the ORDER is asserted to differ. The multiset differs too (a different seed
/// draws a different decklist), and asserting the multisets *equal* would be wrong —
/// that is P2's assertion, about one seed, and it does not generalise across seeds.
#[test]
fn test_dx22_different_seed_different_order() {
    let (cards, registry) = pool();
    let one = built(1, &cards, &registry);
    let two = built(2, &cards, &registry);

    let pid = PlayerId(1);
    let lib_one = library_card_ids(&one, pid);
    let lib_two = library_card_ids(&two, pid);
    assert_eq!(lib_one.len(), 99, "non-vacuity: seed 1 seat {pid:?}");
    assert_eq!(lib_two.len(), 99, "non-vacuity: seed 2 seat {pid:?}");
    assert_ne!(
        lib_one, lib_two,
        "seeds 1 and 2 must not deal seat {pid:?} the same library"
    );
}
