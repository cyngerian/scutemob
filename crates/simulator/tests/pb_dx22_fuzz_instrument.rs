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

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use mtg_engine::{
    all_cards, apply_commander_tax, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameStateBuilder, ObjectFilter, ObjectId, PlayerId, ReplacementModification,
    ReplacementTrigger, SuperType, ZoneId, ZoneType,
};
use mtg_simulator::{
    build_fuzz_state, build_registry, effective_cast_cost, place_registered_deck, Bot, DeckConfig,
    FuzzGameSetup, GameDriver, LocalGame, LocalGameLimits, RandomBot, StubProvider,
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
        setup.decklists.len(),
        PLAYERS as usize,
        "one decklist per seat must be returned"
    );

    for (i, pid) in seats(PLAYERS).into_iter().enumerate() {
        let (deck_pid, deck) = &setup.decklists[i];
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
///
/// # Two seeds, and the second one is the point (review Finding 8)
///
/// Every structural probe in this file builds seed 1, and no seat of seed 1 has a
/// colourless commander — so the CR 903.5c padding arm the paragraph above is *written
/// for* was never actually exercised by any probe. **Seed 8 seat `PlayerId(3)` draws
/// `rograkh-son-of-rohgahh`**, whose colour identity is empty, so its 99 cards are
/// colourless nonlands and lands rather than ~34 basics. The seed was found by
/// enumeration over seeds 1..=120 (hits: 8, 50, 73, 119 — all the same commander, which
/// is the only `Complete` colourless legendary creature in the pool); 8 is used because
/// it is the first.
///
/// The test asserts that this arm was really taken, so the seed silently ceasing to draw
/// a colourless commander reddens rather than quietly reverting the coverage.
#[test]
fn test_dx22_libraries_are_shuffled_cr_103_3() {
    let (cards, registry) = pool();

    // (seed, how many seats must have a colourless commander)
    let mut colourless_seats_seen = 0usize;
    for seed in [1u64, 8] {
        let setup = built(seed, &cards, &registry);

        for (i, pid) in seats(PLAYERS).into_iter().enumerate() {
            let pre_shuffle = setup.decklists[i].1.main_deck.clone();
            let library = library_card_ids(&setup, pid);

            let commander_def = cards
                .iter()
                .find(|c| c.card_id == setup.decklists[i].1.commander)
                .expect("a decklist's commander must be in the pool");
            if mtg_engine::compute_color_identity(commander_def).is_empty() {
                colourless_seats_seen += 1;
            }

            assert_eq!(
                pre_shuffle.len(),
                99,
                "non-vacuity: seed {seed} seat {pid:?} pre-shuffle deck"
            );
            assert_eq!(
                library.len(),
                99,
                "non-vacuity: seed {seed} seat {pid:?} built library"
            );

            assert_ne!(
                library, pre_shuffle,
                "CR 103.3: seed {seed} seat {pid:?}'s library must not be the decklist \
                 in its construction order — that is the unshuffled instrument PB-DX22 \
                 closes"
            );
            assert_eq!(
                sorted(library),
                sorted(pre_shuffle),
                "CR 103.3: a shuffle is a permutation — seed {seed} seat {pid:?}'s \
                 library must hold exactly the same cards, no more and no fewer"
            );
        }
    }

    assert_eq!(
        colourless_seats_seen, 1,
        "non-vacuity: seed 8 seat PlayerId(3) must still draw a colourless commander, \
         or the CR 903.5c padding arm (deck.rs's `basics.is_empty()` branch) is once \
         again unexercised by every probe in this file"
    );
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
/// # The non-vacuity floor, and why the needles are spelled with `concat!`
///
/// As first written this gate **counted itself**: its own `PLACEMENT` and `REGISTRATION`
/// constants are literal occurrences of the needles, so this file entered `matched` and
/// satisfied the rule because of its own declarations. The census read 5 files where only
/// 4 genuinely place, i.e. the stated floor of 4 was an effective floor of 3 — two genuine
/// files could have stopped placing with the gate still green (review Finding 5).
///
/// `concat!` splits both needles at compile time, so the file no longer matches itself.
/// The floor is then re-derived from the genuine census and stated by NAME rather than by
/// count: naming them means a file that stops placing reddens this test, while a *new*
/// placing file (which must obey the rule anyway) does not.
#[test]
fn test_dx22_command_zone_placement_and_registration_are_one_operation() {
    // Split so this file is not its own match — see the doc above.
    const PLACEMENT: &str = concat!("in_zone(ZoneId::", "Command(");
    const REGISTRATION: &str = concat!("player_", "commander");
    /// The genuine placing files as of the PB-DX22 fix cycle, re-derived after the
    /// self-match was removed. A file that stops placing must be removed from this list
    /// deliberately, not silently.
    const EXPECTED_PLACERS: [&str; 4] = [
        "src/fuzz_setup.rs",
        "src/legal_actions.rs",
        "src/setup.rs",
        "tests/commander_cast.rs",
    ];

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
            // Comments do not register a commander. Every one of these files DOCUMENTS
            // the pairing at length -- `fuzz_setup.rs:68`, `setup.rs:19`,
            // `commander_cast.rs:11` and `legal_actions.rs:716` all spell the needle in
            // prose -- so searching the raw body lets a file satisfy the rule with the
            // very sentence explaining the rule. Measured: with the real
            // `builder.player_commander(..)` call deleted from `fuzz_setup.rs:121`, this
            // gate stayed GREEN on the raw body while six behavioural probes reddened
            // (P5/P6/P7/P8/P12/P13). Stripping line comments closes that, and it is safe
            // rather than merely stricter: all four `EXPECTED_PLACERS` carry a real
            // non-comment call (`fuzz_setup.rs:121`, `legal_actions.rs:4148`,
            // `setup.rs:418`, `commander_cast.rs:74`), verified before this was tightened.
            let code: String = body
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            if !code.contains(PLACEMENT) {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            matched.push(rel.clone());
            if !code.contains(REGISTRATION) {
                offenders.push(rel);
            }
        }
    }

    matched.sort();
    offenders.sort();
    // Printed so the census is readable from the run rather than trusted from a comment
    // — the exact failure Finding 5 was.
    println!(
        "P11 genuine command-zone placers ({}): {matched:?}",
        matched.len()
    );

    // Non-vacuity by NAME, not by count. The gate no longer matches itself (`concat!`
    // above), so `matched` is the genuine census; every file that placed at the fix
    // cycle must still be found, or the walk/needle has rotted.
    for expected in EXPECTED_PLACERS {
        assert!(
            matched.iter().any(|m| m.replace('\\', "/") == expected),
            "non-vacuity: `{expected}` places a command-zone object and this gate did \
             not find it — the needle or the walk has rotted. Found: {matched:?}"
        );
    }
    assert!(
        matched.len() >= EXPECTED_PLACERS.len(),
        "non-vacuity: expected at least {} placing files, found {} ({matched:?})",
        EXPECTED_PLACERS.len(),
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
/// `place_registered_deck`, and the second half exercises it with the same fixed
/// 99-Plains deck shape `tests/local_game.rs::build_state` uses.
///
/// # What half (b) is, precisely — it is NOT a gate on `build_state` (review Finding 7)
///
/// `tests/local_game.rs::build_state` is private to a different test crate and cannot be
/// called from here, so half (b) **rebuilds the scaffolding** and duplicates that file's
/// `fixed_deck`. It therefore adds no discrimination over half (a): the same revert
/// (delete `builder.player_commander`) reddens both, because both call the same
/// `place_registered_deck`. It is kept because it exercises the helper against a
/// hand-built deck rather than a `random_deck` one, not because it watches that file.
///
/// `build_state`'s real gates are elsewhere, and they are two:
/// * `test_dx22_command_zone_placement_and_registration_are_one_operation` (P11) — a
///   source walk over `crates/simulator/{src,tests}`; if anyone re-inlines the placement
///   into `build_state`, the literal `in_zone(ZoneId::Command(` reappears in a file with
///   no `player_commander` and P11 reddens by name;
/// * `test_dx22_cr_903_9b_replacements_exist_in_the_fixed_deck_build` (the stage-4b
///   probe) — it asserts on the state `build_state` actually returns, so it is the probe
///   that watches that function's live output.
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
        let expected = &setup.decklists[i].1.commander;
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
        let cmdr = &setup.decklists[i].1.commander;
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

// ── P9 ───────────────────────────────────────────────────────────────────────────

/// CR 103.3 — a spell is cast at an ORDINARY depth, on every seed.
///
/// This is the probe `OOS-UI2-1` and `OOS-SIM3-1` come down to. Before PB-DX22 the floor
/// was arithmetic: `random_deck` appends ~34 basics LAST and `Zone::top()` is the vector's
/// end, so the first non-land sat at personal draw ~35-40 ⇒ game turn ≈136-156 at four
/// seats; the pre-plan measurement observed 143-154 across five seeds. **The turn-30 gate
/// therefore sits more than 4× below the old behaviour and cannot be satisfied by it**,
/// which is what makes it a floor rather than a tuned number.
///
/// # What the margin actually is — MEASURED, because the estimate was wrong
///
/// The plan expected the post-fix band to sit "well above" nothing in particular and
/// certainly under ~15. It does not. Over **20** seeds the first-cast game turn is
/// **min 3 / median 12 / max 29**:
/// `[3,5,5,6,8,9,9,10,10,11,12,17,17,18,18,18,23,25,26,29]`. Against this 30-turn gate
/// that is a margin of **one turn**. Seeds `[1,2,3,4]` land at 17/9/25/23, so this test
/// is not currently at risk — but a successor that widens the seed set will get a
/// failure, and **the correct response is still not to raise the gate**.
///
/// The cause is measured, not guessed: the same run records the first `PlayLand` on turn
/// **1-7 for all 20 seeds**, so land availability is not the limiter. The limiter is that
/// this path deals **no opening hand** (CR 103.5, deliberately out of scope — §B2 /
/// `OOS-DX22-1`): a seat starts with zero cards and draws one per *personal* turn, so by
/// game turn *T* at four seats it has drawn only about *T*/4. §B2 argued seven opening
/// cards would move this "by ≤1-2 personal draws" — true, and that is 4-8 GAME turns,
/// which is the unit this threshold is written in.
///
/// # It is a probe on the LIBRARY, and the predicate says so (review Finding 4)
///
/// As first written this matched `Command::CastSpell(_)` with no zone discrimination.
/// `CastSpellData` carries only `card: ObjectId` — no zone — so a **commander** cast from
/// the command zone satisfied it identically, and the commander is not in the library:
/// this test's own subject, CR 103.3 library order, did not gate it. The batch measured
/// that from one side (shuffle reverted ⇒ 3 of 4 seeds still green at turns 26/25/25) and
/// recorded it, but shipped the weak predicate.
///
/// The predicate now excludes command-zone casts **twice over**, because either half
/// alone leaves a gap:
///
/// 1. by OBJECT — the four command-zone `ObjectId`s are read off the started state and a
///    matching record's `CastSpellData.card` must not be one of them;
/// 2. by EVENT — a command-zone commander cast emits `CommanderCastFromCommandZone`
///    alongside `SpellCast` (see that event's doc), so a record carrying it is rejected
///    regardless of ids. This half also covers a commander that returned to the command
///    zone under CR 903.9a and was recast under a *new* `ObjectId` (CR 400.7), which the
///    id set cannot see.
///
/// Re-run after the strengthening, the observed turns did **not** move: 17/9/25/23, the
/// same four numbers. That is a measurement, not a coincidence to shrug at — it says
/// seeds 1-4's first casts always were library casts, which is what the batch could only
/// call "probably". Independently confirmed by `mtg-fuzzer`'s own census, which puts the
/// first *commander* cast on those seeds at game turn 45-49.
///
/// # What reverting the shuffle alone does
///
/// The plan states this test reddens on every seed if the shuffle is deleted; the
/// pre-strengthening probe did not deliver that (seeds 1/2/3 still cast, via the
/// command zone, at turns 26/25/25). With the predicate above it does, which is the
/// single-variable shuffle probe the plan intended. Reverting **both** fixes (the
/// merge-base behaviour) also fails on the first seed with zero casts in 1,073 commands.
///
/// `max_commands` is set to `30 * 400`, deliberately double `GameDriver`'s `max_turns *
/// 200` ratio, so a failure here can only mean "no cast", never "budget exhausted"
/// (`memory/gotchas-infra.md` on that ratio being the fuzzer's, calibrated on land-only
/// games).
#[test]
fn test_dx22_a_spell_is_cast_at_an_ordinary_depth() {
    const MAX_TURNS: u32 = 30;
    let (cards, registry) = pool();

    let mut observed: Vec<(u64, u32)> = Vec::new();
    for seed in [1u64, 2, 3, 4] {
        let setup = built(seed, &cards, &registry);

        let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
        for (i, pid) in seats(PLAYERS).into_iter().enumerate() {
            // The same per-seat bot seeding `mtg-fuzzer::run_single_game` uses.
            let bot_seed = seed.wrapping_add(100 + i as u64);
            bots.insert(
                pid,
                Box::new(RandomBot::new(bot_seed, format!("Bot-{}", pid.0))),
            );
        }

        let limits = LocalGameLimits {
            max_turns: MAX_TURNS,
            max_commands: MAX_TURNS * 400,
            max_consecutive_passes: 500,
            record_journal: true,
        };

        let (mut game, _) = LocalGame::start(
            setup.state,
            seed,
            StubProvider,
            bots,
            BTreeSet::new(),
            limits,
            false,
        )
        .unwrap_or_else(|e| panic!("seed {seed} must start: {e:?}"));

        // Read off the STARTED state, not off `setup.state`, so no assumption about
        // `start_game` preserving `ObjectId`s is needed.
        let command_zone_ids: BTreeSet<ObjectId> = seats(PLAYERS)
            .into_iter()
            .flat_map(|pid| {
                game.state()
                    .objects_in_zone(&ZoneId::Command(pid))
                    .into_iter()
                    .map(|o| o.id)
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            command_zone_ids.len(),
            PLAYERS as usize,
            "non-vacuity: the exclusion set must name one command-zone object per seat, \
             or this probe is not excluding anything"
        );

        let _outcome = game.advance();

        let first_cast =
            game.journal()
                .iter()
                .find(|rec| match &rec.command {
                    // CR 903.8 casts are excluded by BOTH halves — see this test's doc.
                    Command::CastSpell(data) => {
                        !command_zone_ids.contains(&data.card)
                            && !rec.events.iter().any(|e| {
                                matches!(e, GameEvent::CommanderCastFromCommandZone { .. })
                            })
                    }
                    _ => false,
                })
                .map(|rec| rec.turn);

        let turn = first_cast.unwrap_or_else(|| {
            panic!(
                "CR 103.3: seed {seed} cast no spell from anywhere but the command zone \
                 within {MAX_TURNS} turns ({} commands recorded) — that is the \
                 unshuffled instrument's signature",
                game.command_count()
            )
        });
        println!("P9 seed {seed}: first non-commander CastSpell on game turn {turn}");
        assert!(
            turn <= MAX_TURNS,
            "seed {seed} first cast on turn {turn}, beyond the {MAX_TURNS}-turn floor"
        );
        observed.push((seed, turn));
    }

    println!("P9 observed first-cast turns: {observed:?}");
    assert_eq!(observed.len(), 4, "non-vacuity: all four seeds must be run");
}

// ── P12 ──────────────────────────────────────────────────────────────────────────

/// CR 903.10a — **commander damage is recorded, and its loss condition is reachable,
/// from a fuzz-built state.**
///
/// # Why this probe exists (review Finding 1)
///
/// Acceptance criterion 2 required the commander mechanics to be "exercised **or
/// explicitly probed**". CR 903.6 has P5, CR 903.8 has P8, CR 903.9a has P7, CR 903.9b
/// has P6 — and CR 903.10a had **nothing**. Its only evidence was a scratch
/// `examples/dx22_p10.rs` that was deleted, so the one commander rule with a
/// *lose-the-game* consequence was the one with no committed gate. This is that gate, and
/// it is deterministic rather than statistical: it drives one attack, not a fuzz sample.
///
/// # What it actually gates
///
/// `rules/combat.rs`'s damage loop only attributes damage as *commander* damage when the
/// source's controller has that source's `CardId` in `commander_ids` — the exact field
/// `place_registered_deck`'s `builder.player_commander(..)` populates and which no fuzz
/// game had before PB-DX22. So on a build with the registration deleted, the attack still
/// happens and P2 still loses 21+ life, but `commander_damage_received` stays **empty**
/// and the CR 903.10a state-based action can never fire. That is the revert this probe is
/// proven red by.
///
/// The `test-util` escape hatches (`turn_mut`, `objects_mut`, `move_object_to_zone`) put
/// the fuzz-built state into a declare-attackers step; everything after that is the real
/// engine — `handle_declare_attackers` then `apply_combat_damage` then
/// `rules::sba::check_state_based_actions`.
#[test]
fn test_dx22_cr_903_10a_commander_damage_is_recorded_on_the_fuzz_build() {
    use mtg_engine::state::test_util;
    use mtg_engine::AttackTarget;

    let (cards, registry) = pool();
    let setup = built(1, &cards, &registry);
    let attacker_seat = PlayerId(1);
    let defender_seat = PlayerId(2);

    let commander_card_id = setup.decklists[0].1.commander.clone();
    let (mut state, _events) = mtg_engine::start_game(setup.state).expect("fuzz state must start");

    // CR 400.7: moving the commander to the battlefield mints a new ObjectId.
    let in_command_zone = state.objects_in_zone(&ZoneId::Command(attacker_seat))[0].id;
    let (attacker, _) =
        test_util::move_object_to_zone(&mut state, in_command_zone, ZoneId::Battlefield)
            .expect("the commander must be movable to the battlefield");

    // CR 302.6 / CR 506.4: make it a legal attacker without changing what it IS.
    {
        let obj = test_util::object_mut(&mut state, attacker).expect("the attacker must resolve");
        obj.has_summoning_sickness = false;
        obj.status.tapped = false;
    }
    let power = mtg_engine::rules::layers::calculate_characteristics(&state, attacker)
        .and_then(|c| c.power)
        .expect("a legendary creature commander has a power");
    assert!(
        power > 0,
        "non-vacuity: seed 1's commander must have positive power to deal any damage \
         (got {power})"
    );

    // CR 508.1: put the state in the attacker's declare-attackers step.
    {
        let turn = state.turn_mut();
        turn.active_player = attacker_seat;
        turn.phase = mtg_engine::state::turn::Phase::Combat;
        turn.step = mtg_engine::state::turn::Step::DeclareAttackers;
        turn.priority_holder = Some(attacker_seat);
    }

    assert!(
        state
            .player(defender_seat)
            .expect("seat 2 must exist")
            .commander_damage_received
            .is_empty(),
        "non-vacuity: no commander damage may be recorded before the attack"
    );

    mtg_engine::rules::combat::handle_declare_attackers(
        &mut state,
        attacker_seat,
        vec![(attacker, AttackTarget::Player(defender_seat))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .expect("CR 508.1: the registered commander must be able to attack");

    // No blockers declared, so CR 510.1c sends the whole assignment at the player.
    let _damage_events = mtg_engine::rules::combat::apply_combat_damage(&mut state, false);

    let dealt = state
        .player(defender_seat)
        .expect("seat 2 must exist")
        .commander_damage_received
        .get(&attacker_seat)
        .and_then(|by_card| by_card.get(&commander_card_id))
        .copied();
    assert_eq!(
        dealt,
        Some(power as u32),
        "CR 903.10a: combat damage from a REGISTERED commander must be recorded against \
         the defending player, keyed by (dealing player, commander CardId). \
         `commander_damage_received` = {:?}",
        state
            .player(defender_seat)
            .expect("seat 2 must exist")
            .commander_damage_received
    );

    // CR 903.10a's threshold, through the real SBA rather than a re-derivation: 21 or
    // more from the same commander loses the game. `sba.rs` reads exactly the map the
    // combat loop just wrote.
    assert!(
        !state
            .player(defender_seat)
            .expect("seat 2 must exist")
            .has_lost,
        "non-vacuity: {power} damage is below the 21 threshold, so seat 2 must still be \
         in the game before the total is raised"
    );
    {
        let defender = test_util::player_mut(&mut state, defender_seat).expect("seat 2 must exist");
        let mut by_card = defender
            .commander_damage_received
            .get(&attacker_seat)
            .cloned()
            .unwrap_or_default();
        by_card.insert(commander_card_id.clone(), 21);
        defender
            .commander_damage_received
            .insert(attacker_seat, by_card);
    }
    let sba_events = mtg_engine::rules::sba::check_and_apply_sbas(&mut state);
    assert!(
        state
            .player(defender_seat)
            .expect("seat 2 must exist")
            .has_lost,
        "CR 903.10a: 21 combat damage from the same commander must lose the game; SBA \
         events were {sba_events:?}"
    );
}

// ── P13 ──────────────────────────────────────────────────────────────────────────

/// The fuzzer's own mechanics census is **not vacuous** (review Findings 1 and 2).
///
/// `mtg-fuzzer` now prints a commander-mechanics and first-cast summary, and a
/// violation-by-`check` histogram over **every** game in a run, so the numbers PB-DX22
/// published are re-derivable from committed code instead of from a deleted scratch
/// binary. A printed summary that silently reports zeros is worse than no summary — it is
/// the exact failure this batch exists to remove — so the counters are gated here.
///
/// This runs the same `GameDriver::run_game_with_mechanics` the binary calls, on the same
/// `build_fuzz_state`, with the same per-seat bot seeding. It asserts the census is
/// populated in every dimension the summary prints and that PB-DX22's own claims are
/// consistent with it.
#[test]
fn test_dx22_the_fuzzers_mechanics_census_is_not_vacuous() {
    const SEED: u64 = 1;
    const MAX_TURNS: u32 = 60;
    let (cards, registry) = pool();
    let setup = built(SEED, &cards, &registry);

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for (i, pid) in seats(PLAYERS).into_iter().enumerate() {
        let bot_seed = SEED.wrapping_add(100 + i as u64);
        bots.insert(
            pid,
            Box::new(RandomBot::new(bot_seed, format!("Bot-{}", pid.0))),
        );
    }

    let driver = GameDriver::new(StubProvider, bots, MAX_TURNS, SEED);
    let (result, mechanics) = driver.run_game_with_mechanics(setup.state, SEED);
    println!(
        "P13 seed {SEED} ({} turns): {mechanics:?}",
        result.turn_count
    );

    assert!(
        mechanics.spell_casts > 0,
        "CR 601.2: the census must count the spells a fuzz game casts (got {mechanics:?})"
    );
    assert!(
        mechanics.first_spell_cast_turn.is_some(),
        "CR 601.2: the census must record the first-cast turn — the number `OOS-UI2-1` \
         and `OOS-SIM3-1` are about (got {mechanics:?})"
    );
    assert!(
        mechanics.first_library_spell_cast_turn.is_some(),
        "CR 103.3: the census must separate the first NON-commander cast, which is the \
         only one library order gates (got {mechanics:?})"
    );
    assert!(
        mechanics.lands_played > 0 && mechanics.first_land_played_turn.is_some(),
        "CR 305.1: the census must count lands, which is how the batch showed land \
         availability is not what gates the first cast (got {mechanics:?})"
    );
    assert!(
        mechanics.commander_casts_from_command_zone > 0
            && mechanics.first_commander_cast_turn.is_some(),
        "CR 903.8: the census must count command-zone commander casts. This number was \
         **0 in every fuzz game** before PB-DX22 (`OOS-SIM1-4`), so a zero here means \
         either the registration or the counter has regressed (got {mechanics:?})"
    );
    assert!(
        mechanics.seats_dealt_commander_damage > 0 && mechanics.max_commander_damage > 0,
        "CR 903.10a: commander damage must be recorded in a fuzz game — also 0 before \
         PB-DX22 (got {mechanics:?})"
    );
}
