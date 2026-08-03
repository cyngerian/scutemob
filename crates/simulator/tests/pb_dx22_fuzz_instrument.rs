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

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::{
    all_cards, apply_commander_tax, CardDefinition, CardId, CardRegistry, CardType,
    GameStateBuilder, ObjectFilter, ObjectId, PlayerId, ReplacementModification,
    ReplacementTrigger, SuperType, ZoneId, ZoneType,
};
use mtg_simulator::{
    build_fuzz_state, build_registry, effective_cast_cost, place_registered_deck, DeckConfig,
    FuzzGameSetup,
};

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

/// The same fixed, low-complexity deck `crates/simulator/tests/local_game.rs::fixed_deck`
/// builds: 99 Plains plus the first `Complete` legendary creature in `all_cards()` as
/// commander. Duplicated here rather than shared because it belongs to that file's
/// fixture story; what P5 needs from it is only that it is a `DeckConfig` naming a
/// commander.
fn fixed_plains_deck(cards: &[CardDefinition]) -> DeckConfig {
    let commander = cards
        .iter()
        .find(|c| {
            c.completeness.is_complete()
                && c.types.supertypes.contains(&SuperType::Legendary)
                && c.types.card_types.contains(&CardType::Creature)
        })
        .expect("at least one Complete legendary creature must exist in the card pool");
    DeckConfig {
        commander: commander.card_id.clone(),
        main_deck: (0..99).map(|_| CardId("plains".to_string())).collect(),
    }
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

// ── P11 ──────────────────────────────────────────────────────────────────────────

/// CR 903.6 — a source gate: **placing an object in a command zone and registering it
/// as a commander are ONE operation**, and no file in this crate may do half of it.
///
/// Placing the object records nothing. `PlayerState::commander_ids` is what every
/// commander rule keys off — CR 903.8 tax (`rules/casting.rs`, mirrored by
/// `legal_actions::effective_cast_cost`), the CR 903.9a/704.6d zone-return SBA
/// (`rules/commander.rs`), CR 903.10a commander damage, and the CR 903.9b replacements
/// `register_commander_zone_replacements` derives *from* `commander_ids`. A game with
/// the object but not the registration is not a Commander game: the commander is
/// recastable for free forever and deals no commander damage.
///
/// This gate exists because the convention was stated in three places
/// (`setup.rs:381-393`, `commander_cast.rs`'s module doc, `fuzz_setup.rs`) and obeyed in
/// two of the five files that needed it. **It was red on the tree that introduced it** —
/// `fuzz_setup.rs` (the lift of `bin/fuzzer.rs`) and `tests/local_game.rs` both violated
/// it, which is the strongest available proof that it discriminates.
///
/// Non-vacuity floor: the matched-file set must be non-empty and at least 4, so a
/// renamed API or a broken walk cannot make this pass by finding nothing.
#[test]
fn test_dx22_command_zone_placement_and_registration_are_one_operation() {
    const PLACEMENT: &str = "in_zone(ZoneId::Command(";
    const REGISTRATION: &str = "player_commander";

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut matched: Vec<String> = Vec::new();
    let mut offenders: Vec<String> = Vec::new();

    let mut stack = vec![root.join("src"), root.join("tests")];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("walk must reach {}: {e}", dir.display()));
        for entry in entries {
            let path = entry.expect("a readable dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let body = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("must read {}: {e}", path.display()));
            if !body.contains(PLACEMENT) {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            matched.push(rel.clone());
            if !body.contains(REGISTRATION) {
                offenders.push(rel);
            }
        }
    }

    matched.sort();
    offenders.sort();

    assert!(
        matched.len() >= 4,
        "non-vacuity: this gate must find at least 4 files placing a command-zone \
         object, found {} ({matched:?}) — if the API was renamed, rename the needle",
        matched.len()
    );
    assert!(
        offenders.is_empty(),
        "CR 903.6: these files place an object in a command zone without ever calling \
         `player_commander`, so `commander_ids` stays empty and every commander rule \
         is silently inert there: {offenders:?}"
    );
}

// ── P5 ───────────────────────────────────────────────────────────────────────────

/// CR 903.6 / CR 903.8 — `PlayerState::commander_ids` is populated by **both** build
/// paths in this crate.
///
/// Both halves live in one test deliberately. `commander_ids` is the field every
/// commander rule keys off, and before PB-DX22 the fuzzer's path and
/// `tests/local_game.rs`'s path were two independent copies that both left it empty
/// (`OOS-SIM1-4`); a probe on one would have proven half the fix. They now share
/// `place_registered_deck`, and the second half exercises it with the fixed 99-Plains
/// deck `tests/local_game.rs::build_state` uses, which is that function's live path.
///
/// CR 903.5b/702.124 partner: `random_deck` never builds a partner pair and `DeckConfig`
/// cannot represent one (`OOS-SIM4-3`), so exactly one registered commander per seat is
/// the correct assertion here, not a lower bound.
#[test]
fn test_dx22_commander_ids_are_registered_by_both_build_paths() {
    let (cards, registry) = pool();

    // (a) the fuzzer's own path.
    let setup = built(1, &cards, &registry);
    for (i, pid) in seats(PLAYERS).into_iter().enumerate() {
        let expected = &setup.decks[i].1.commander;
        let ids: Vec<CardId> = setup
            .state
            .player(pid)
            .expect("seat must exist")
            .commander_ids
            .iter()
            .cloned()
            .collect();
        assert_eq!(
            ids,
            vec![expected.clone()],
            "CR 903.6: seat {pid:?}'s commander must be REGISTERED by build_fuzz_state, \
             not merely placed in the command zone"
        );
    }

    // (b) `tests/local_game.rs::build_state`'s path — the same helper, a fixed deck.
    let deck = fixed_plains_deck(&cards);
    let card_defs: HashMap<String, CardDefinition> =
        cards.iter().map(|c| (c.name.clone(), c.clone())).collect();
    let mut builder = GameStateBuilder::new().with_registry(registry.clone());
    for pid in seats(PLAYERS) {
        builder = builder.add_player(pid);
    }
    for pid in seats(PLAYERS) {
        builder = place_registered_deck(builder, pid, &deck, &cards, &card_defs);
    }
    let state = builder
        .first_turn_of_game()
        .build()
        .expect("fixed-deck state must build");

    for pid in seats(PLAYERS) {
        let ids: Vec<CardId> = state
            .player(pid)
            .expect("seat must exist")
            .commander_ids
            .iter()
            .cloned()
            .collect();
        assert_eq!(
            ids,
            vec![deck.commander.clone()],
            "CR 903.6: seat {pid:?}'s commander must be REGISTERED on the fixed-deck \
             path too"
        );
    }
}

// ── P6 ───────────────────────────────────────────────────────────────────────────

/// CR 903.9b — the hand/library → command-zone redirects exist on the fuzz-built state.
///
/// These are replacement effects, and `GameStateBuilder` does **not** derive them from
/// `commander_ids`: something has to call `register_commander_zone_replacements`. If
/// nobody does, CR 903.9b silently does not exist — a bounced or shuffled-away commander
/// stays where it went and no `CommanderZoneRedirect` is ever emitted, so a measurement
/// counting that event reads zero VACUOUSLY rather than failing.
#[test]
fn test_dx22_cr_903_9b_replacements_are_registered() {
    let (cards, registry) = pool();
    let setup = built(1, &cards, &registry);

    let effects = setup.state.replacement_effects();
    assert_eq!(
        effects.len(),
        2 * PLAYERS as usize,
        "CR 903.9b: exactly two redirects (hand, library) per registered commander"
    );

    for (i, pid) in seats(PLAYERS).into_iter().enumerate() {
        let cmdr = &setup.decks[i].1.commander;
        for want in [ZoneType::Hand, ZoneType::Library] {
            let hits = effects
                .iter()
                .filter(|e| {
                    e.controller == pid
                        && matches!(
                            &e.trigger,
                            ReplacementTrigger::WouldChangeZone { to, filter, .. }
                                if *to == want
                                    && matches!(filter, ObjectFilter::HasCardId(c) if c == cmdr)
                        )
                        && matches!(
                            e.modification,
                            ReplacementModification::RedirectToZone(ZoneType::Command)
                        )
                })
                .count();
            assert_eq!(
                hits, 1,
                "CR 903.9b: seat {pid:?} must have exactly one {want:?} → Command \
                 redirect for its commander"
            );
        }
    }
}

// ── P7 ───────────────────────────────────────────────────────────────────────────

/// CR 903.9a / CR 704.6d — the command-zone-return state-based action is REACHABLE from
/// a fuzz-built state.
///
/// The SBA is keyed on `commander_ids`, not on the zone, so before PB-DX22 it could
/// never fire in a fuzz game no matter how many commanders died. Note this half of
/// CR 903.9 is independent of P6's replacements: it would work from the registration
/// alone.
#[test]
fn test_dx22_cr_903_9a_zone_return_sba_is_reachable_from_the_fuzz_build() {
    let (cards, registry) = pool();
    let setup = built(1, &cards, &registry);
    let p1 = PlayerId(1);

    let (mut state, _events) = mtg_engine::start_game(setup.state).expect("fuzz state must start");

    let cmdr_obj = state.objects_in_zone(&ZoneId::Command(p1))[0].id;
    let (new_id, _) = mtg_engine::state::test_util::move_object_to_zone(
        &mut state,
        cmdr_obj,
        ZoneId::Graveyard(p1),
    )
    .expect("the commander must be movable to its owner's graveyard");

    assert!(
        state.pending_commander_zone_choices().is_empty(),
        "non-vacuity: no choice may be pending before the SBA runs"
    );

    mtg_engine::rules::commander::check_commander_zone_return_sba(&mut state);

    let pending: Vec<(PlayerId, mtg_engine::ObjectId)> = state
        .pending_commander_zone_choices()
        .iter()
        .copied()
        .collect();
    assert!(
        pending.contains(&(p1, new_id)),
        "CR 903.9a: the SBA must offer P1 the choice for its own commander in the \
         graveyard; pending = {pending:?}"
    );
}

// ── P8 ───────────────────────────────────────────────────────────────────────────

/// CR 903.8 — the commander tax applies on the fuzz-built state.
///
/// `effective_cast_cost` gates on `commander_ids` (it returns the printed cost unchanged
/// for a command-zone object that is not a registered commander), so before PB-DX22 a
/// fuzz commander would have been recastable for the printed cost forever.
#[test]
fn test_dx22_cr_903_8_tax_applies_on_the_fuzz_build() {
    let (cards, registry) = pool();
    let mut setup = built(1, &cards, &registry);
    let p1 = PlayerId(1);

    let cmdr = setup.state.objects_in_zone(&ZoneId::Command(p1))[0];
    let cmdr_obj = cmdr.id;
    let cid = cmdr.card_id.clone().expect("a commander carries a CardId");
    let printed = cmdr
        .characteristics
        .mana_cost
        .clone()
        .expect("a legendary creature commander has a printed mana cost");

    assert_eq!(
        effective_cast_cost(&setup.state, p1, cmdr_obj),
        Some(printed.clone()),
        "CR 903.8: with tax 0 the effective cost is the printed cost"
    );

    setup
        .state
        .players_mut()
        .get_mut(&p1)
        .expect("seat 1 must exist")
        .commander_tax
        .insert(cid, 1);

    let taxed = apply_commander_tax(&printed, 1);
    assert_ne!(
        taxed, printed,
        "non-vacuity: one prior cast must actually change the cost (CR 903.8: +{{2}})"
    );
    assert_eq!(
        effective_cast_cost(&setup.state, p1, cmdr_obj),
        Some(taxed),
        "CR 903.8: after one prior command-zone cast the commander costs {{2}} more"
    );
}
