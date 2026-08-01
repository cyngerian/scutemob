//! Acceptance tests for `crates/simulator/src/setup.rs` (M11-local Session 2:
//! deterministic pregame setup and mulligans).
//!
//! CR 103.4 (opening hand size), CR 103.5 (mulligan), CR 903.5a (100-card deck) /
//! Architecture Invariant 9 (deck admission), CR 903.6 (commander to the command zone).

use std::collections::BTreeSet;

use mtg_engine::DeckViolation;
use mtg_engine::{all_cards, CardDefinition, CardId, CardType, PlayerId, SuperType, ZoneId};
use mtg_simulator::{
    build_initial_state, redeal, BotKind, DeckConfig, DeckSource, LocalGameConfig, LocalGameLimits,
    SetupError,
};

/// Default `LocalGameLimits` for these tests — `build_initial_state`/`redeal` never
/// read them (they only assemble a pregame `GameState`, they do not run a `LocalGame`),
/// but `LocalGameConfig` carries the field for the sessions that do.
fn unused_limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 1,
        max_commands: 1,
        max_consecutive_passes: 1,
        record_journal: false,
    }
}

fn random_cfg(player_count: u32, seed: u64) -> LocalGameConfig {
    LocalGameConfig {
        player_count,
        human_seats: BTreeSet::new(),
        bot_kind: BotKind::Random,
        seed,
        decks: DeckSource::RandomPerSeat,
        limits: unused_limits(),
    }
}

/// The first `Complete` legendary creature in `cards` — a deterministic pick (`all_cards`
/// is a plain, statically-ordered `Vec`), matching the same pattern
/// `crates/simulator/tests/local_game.rs::fixed_deck` uses for its fixed commander.
fn fixed_commander(cards: &[CardDefinition]) -> &CardDefinition {
    cards
        .iter()
        .find(|c| {
            c.completeness.is_complete()
                && c.types.supertypes.contains(&SuperType::Legendary)
                && c.types.card_types.contains(&CardType::Creature)
        })
        .expect("at least one Complete legendary creature must exist in the card pool")
}

/// A real non-`Complete` `CardDefinition` from the live card pool — found by
/// enumerating `all_cards()` and filtering on `completeness`, per the task's
/// instruction not to hardcode an unverified card name (the corpus is ~63% `Complete`
/// per `docs/authoring-status.md`, so plenty of inert/partial/known-wrong defs exist).
fn first_non_complete_card(cards: &[CardDefinition]) -> &CardDefinition {
    cards.iter().find(|c| !c.completeness.is_complete()).expect(
        "at least one non-Complete CardDefinition must exist in the card pool for this \
             test to be meaningful",
    )
}

/// CR 103.4 — the engine deals no opening hand; `build_initial_state` must deal exactly
/// seven cards to every seat's hand.
#[test]
fn test_setup_deals_seven_card_opening_hand_per_seat() {
    let cfg = random_cfg(3, 100);
    let (state, names) = build_initial_state(&cfg).expect("setup should succeed");

    assert_eq!(names.len(), 3);
    for i in 1..=3u64 {
        let pid = PlayerId(i);
        let hand = state.objects_in_zone(&ZoneId::Hand(pid));
        assert_eq!(
            hand.len(),
            7,
            "seat {i} must open with exactly 7 cards (CR 103.4)"
        );
    }
}

/// The remaining 92 main-deck cards (99 - 7 opening hand) land in the library, not
/// dropped or duplicated.
#[test]
fn test_setup_library_holds_the_remainder() {
    let cfg = random_cfg(2, 101);
    let (state, _names) = build_initial_state(&cfg).expect("setup should succeed");

    for i in 1..=2u64 {
        let pid = PlayerId(i);
        let library = state.objects_in_zone(&ZoneId::Library(pid));
        assert_eq!(
            library.len(),
            92,
            "seat {i}'s library must hold the 92 cards not dealt to the opening hand \
             (random_deck's 99-card contract minus the 7-card hand)"
        );
    }
}

/// Determinism: the same seed reproduces byte-identical state. Every random draw
/// (commander pick, deck assembly, shuffle) is taken from one `StdRng` seeded from
/// `cfg.seed`, consumed in ascending `PlayerId` order, so nothing outside the config can
/// perturb the result.
#[test]
fn test_setup_same_seed_same_state_hash() {
    let cfg = random_cfg(4, 4242);
    let (state_a, names_a) = build_initial_state(&cfg).expect("setup should succeed");
    let (state_b, names_b) = build_initial_state(&cfg).expect("setup should succeed");

    assert_eq!(
        state_a.public_state_hash(),
        state_b.public_state_hash(),
        "the same seed must reproduce the same public state hash"
    );
    assert_eq!(names_a, names_b);
}

/// Two different seeds must not produce the same table.
///
/// This cannot flake: `RandomPerSeat` draws a commander from dozens of `Complete`
/// legendary creatures and a ~60-card non-land pool from well over a thousand `Complete`
/// cards per seat, all from one shared `StdRng` sequence keyed on `cfg.seed`. For two
/// different seeds to land on the same public state hash would require either an actual
/// hash collision or an implausible run of identical independent draws across every
/// seat — not a scenario a finite retry budget needs to defend against.
#[test]
fn test_setup_different_seed_different_opening_hand() {
    let cfg_a = random_cfg(2, 7);
    let cfg_b = random_cfg(2, 70_000);
    let (state_a, _) = build_initial_state(&cfg_a).expect("setup should succeed");
    let (state_b, _) = build_initial_state(&cfg_b).expect("setup should succeed");

    assert_ne!(
        state_a.public_state_hash(),
        state_b.public_state_hash(),
        "different seeds must not produce the same table"
    );

    // A more targeted check on the specific claim the test name makes: seat 1's actual
    // opening hand (by card name, since the library the deck is drawn from is shared) —
    // this is the part of the state a "different opening hand" claim is actually about.
    let hand_names = |state: &mtg_engine::GameState, pid: PlayerId| -> Vec<String> {
        let mut names: Vec<String> = state
            .objects_in_zone(&ZoneId::Hand(pid))
            .iter()
            .map(|obj| obj.characteristics.name.clone())
            .collect();
        names.sort();
        names
    };
    assert_ne!(
        hand_names(&state_a, PlayerId(1)),
        hand_names(&state_b, PlayerId(1)),
        "seat 1's opening hand must differ between the two seeds"
    );
}

/// Architecture Invariant 9: a deck naming a non-`Complete` `CardDefinition` is refused
/// by `validate_deck`, and `build_initial_state` propagates that as
/// `SetupError::InvalidDeck` rather than building a corrupting game.
#[test]
fn test_setup_rejects_deck_with_non_complete_card() {
    let cards = all_cards();
    let bad_card = first_non_complete_card(&cards);
    let commander = fixed_commander(&cards);

    let mut main_deck: Vec<CardId> = (0..98).map(|_| CardId("plains".to_string())).collect();
    main_deck.push(bad_card.card_id.clone());
    let deck = DeckConfig {
        commander: commander.card_id.clone(),
        main_deck,
    };

    let cfg = LocalGameConfig {
        player_count: 1,
        human_seats: BTreeSet::new(),
        bot_kind: BotKind::Random,
        seed: 1,
        decks: DeckSource::Fixed(vec![(PlayerId(1), deck)]),
        limits: unused_limits(),
    };

    let result = build_initial_state(&cfg);
    match result {
        Err(SetupError::InvalidDeck { seat, violations }) => {
            assert_eq!(seat, PlayerId(1));
            assert!(
                violations.iter().any(|v| matches!(
                    v,
                    DeckViolation::IncompleteCard { card_id, .. }
                        if card_id == &bad_card.card_id.0
                )),
                "expected an IncompleteCard violation naming {:?}, got {:?}",
                bad_card.card_id,
                violations
            );
        }
        other => panic!("expected InvalidDeck, got {:?}", other),
    }
}

/// CR 103.5 — a re-deal produces a genuinely different hand for the mulliganing seat,
/// not the same 7 cards back (the failure mode a naive seed perturbation — e.g. XOR'ing
/// two terms that can cancel to zero — would silently reproduce; see `redeal_seed`'s doc
/// comment in `setup.rs`). Not flake-prone for the same combinatorial reason as
/// `test_setup_different_seed_different_opening_hand`.
#[test]
fn test_redeal_produces_a_different_hand() {
    let cfg = random_cfg(2, 55);
    let (state_before, _) = build_initial_state(&cfg).expect("setup should succeed");
    let (state_after, _) =
        redeal(&cfg, PlayerId(1), 1).expect("redeal should succeed for seat 1's first mulligan");

    let hand_names = |state: &mtg_engine::GameState, pid: PlayerId| -> Vec<String> {
        let mut names: Vec<String> = state
            .objects_in_zone(&ZoneId::Hand(pid))
            .iter()
            .map(|obj| obj.characteristics.name.clone())
            .collect();
        names.sort();
        names
    };

    assert_eq!(
        state_after
            .objects_in_zone(&ZoneId::Hand(PlayerId(1)))
            .len(),
        7,
        "CR 103.5: a mulligan still draws a fresh 7-card hand (the London mulligan — \
         bottoming happens after the draw, and is left to the caller per this module's \
         doc comment)"
    );
    assert_ne!(
        hand_names(&state_before, PlayerId(1)),
        hand_names(&state_after, PlayerId(1)),
        "seat 1's redealt hand must differ from its original opening hand"
    );
}

/// CR 903.6 — the commander starts face up in the command zone, not the library or hand.
#[test]
fn test_setup_commander_starts_in_command_zone() {
    let cards = all_cards();
    let cfg = random_cfg(2, 9001);
    let (state, _names) = build_initial_state(&cfg).expect("setup should succeed");

    for i in 1..=2u64 {
        let pid = PlayerId(i);
        let command_zone = state.objects_in_zone(&ZoneId::Command(pid));
        assert_eq!(
            command_zone.len(),
            1,
            "seat {i} must have exactly one card in the command zone"
        );
        let commander_obj = &command_zone[0];
        // The commander is neither in the opening hand nor the library.
        let in_hand = state
            .objects_in_zone(&ZoneId::Hand(pid))
            .iter()
            .any(|o| o.characteristics.name == commander_obj.characteristics.name);
        let in_library = state
            .objects_in_zone(&ZoneId::Library(pid))
            .iter()
            .any(|o| o.characteristics.name == commander_obj.characteristics.name);
        assert!(
            !in_hand,
            "the commander must not also appear in the opening hand"
        );
        assert!(
            !in_library,
            "the commander must not also appear in the library"
        );
        // Sanity: the object actually corresponds to a real, known CardDefinition (i.e.
        // this isn't just an empty placeholder zone entry).
        assert!(
            cards
                .iter()
                .any(|c| c.name == commander_obj.characteristics.name),
            "the command-zone object must be a real CardDefinition"
        );
    }
}
