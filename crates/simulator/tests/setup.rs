//! Acceptance tests for `crates/simulator/src/setup.rs` (M11-local Session 2:
//! deterministic pregame setup and mulligans).
//!
//! CR 103.5 / 402.1 (opening hand of seven), CR 103.5 / 103.5c (mulligan), CR 903.5a (100-card deck) /
//! Architecture Invariant 9 (deck admission), CR 903.6 (commander to the command zone).

use std::collections::BTreeSet;

use mtg_engine::DeckViolation;
use mtg_engine::{
    all_cards, CardDefinition, CardId, CardRegistry, CardType, GameStateBuilder, ObjectSpec,
    PlayerId, ReplacementModification, SuperType, ZoneId, ZoneType,
};
use mtg_simulator::{
    build_initial_state, dealt_decks, redeal, BotKind, DeckConfig, DeckSource, LocalGameConfig,
    LocalGameLimits, SetupError,
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

/// CR 103.5 / 402.1 — the engine deals no opening hand; `build_initial_state` must deal exactly
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
            "seat {i} must open with exactly 7 cards (CR 103.5)"
        );
    }
}

/// The opening hand holds *real, playable* cards — not name-only placeholders.
///
/// `ObjectSpec::card()` sets only `characteristics.name`; every other characteristic
/// (card types, mana cost, subtypes, P/T) comes from `enrich_spec_from_def`, which is the
/// project's documented #1 `ObjectSpec` gotcha. Dropping those calls from
/// `build_initial_state` would leave every hand and library object typeless and
/// costless — uncastable, and invisible to every type filter in the engine — while the
/// hand *counts* stayed a perfect 7 and every other test in this file stayed green. This
/// is the assertion that reddens instead.
///
/// Cross-checked against the real `CardDefinition` rather than just asserting
/// "non-empty", so a hypothetical enrichment that filled in the wrong card also fails.
#[test]
fn test_setup_opening_hand_cards_are_enriched_from_their_definitions() {
    let cards = all_cards();
    let cfg = random_cfg(2, 606);
    let (state, _names) = build_initial_state(&cfg).expect("setup should succeed");

    for i in 1..=2u64 {
        let pid = PlayerId(i);
        for obj in state.objects_in_zone(&ZoneId::Hand(pid)) {
            let def = cards
                .iter()
                .find(|c| c.name == obj.characteristics.name)
                .unwrap_or_else(|| {
                    panic!(
                        "hand object {:?} must correspond to a real CardDefinition",
                        obj.characteristics.name
                    )
                });
            assert!(
                !obj.characteristics.card_types.is_empty(),
                "{}: card_types must be populated by enrich_spec_from_def — an object with \
                 no card types is uncastable and invisible to every type filter",
                def.name
            );
            assert_eq!(
                obj.characteristics.card_types, def.types.card_types,
                "{}: card types must match its CardDefinition",
                def.name
            );
            assert_eq!(
                obj.characteristics.mana_cost, def.mana_cost,
                "{}: mana cost must match its CardDefinition",
                def.name
            );
        }
    }
}

/// Criterion 1's literal "human seat" case: a seat listed in `human_seats` is dealt the
/// same real 7-card opening hand a bot seat gets (dealing is seat-agnostic), and is named
/// as a human rather than a bot.
#[test]
fn test_setup_deals_a_human_seat_the_same_real_opening_hand() {
    let cfg = LocalGameConfig {
        human_seats: [PlayerId(1)].into_iter().collect(),
        ..random_cfg(4, 20_260_731)
    };
    let (state, names) = build_initial_state(&cfg).expect("setup should succeed");

    assert_eq!(
        names.get(&PlayerId(1)).map(String::as_str),
        Some("Human-1"),
        "the human seat must be named as one"
    );
    assert_eq!(names.get(&PlayerId(2)).map(String::as_str), Some("Bot-2"));

    let hand = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)));
    assert_eq!(
        hand.len(),
        7,
        "the human seat opens with 7 cards (CR 103.5)"
    );
    assert_eq!(
        state.objects_in_zone(&ZoneId::Command(PlayerId(1))).len(),
        1,
        "the human seat's commander starts in its command zone (CR 903.6)"
    );
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

/// Every card a seat owns, by name, sorted — the CR 103.5 multiset: hand ∪ library ∪
/// command zone.
///
/// Sorted rather than compared zone-by-zone precisely because a mulligan is *allowed* to
/// move cards between hand and library and to reorder the library; what it may not do is
/// change which cards are there. Names rather than `CardId`s so a mismatch names the card.
fn deck_multiset(state: &mtg_engine::GameState, pid: PlayerId) -> Vec<String> {
    let mut names: Vec<String> = [
        ZoneId::Hand(pid),
        ZoneId::Library(pid),
        ZoneId::Command(pid),
    ]
    .iter()
    .flat_map(|zone| {
        state
            .objects_in_zone(zone)
            .iter()
            .map(|obj| obj.characteristics.name.clone())
            .collect::<Vec<_>>()
    })
    .collect();
    names.sort();
    names
}

/// The seat's registered commander (`PlayerState::commander_ids`), not merely the card
/// sitting in the command zone — that is the field every commander rule keys off.
fn registered_commanders(state: &mtg_engine::GameState, pid: PlayerId) -> Vec<CardId> {
    state
        .players()
        .get(&pid)
        .unwrap_or_else(|| panic!("seat {pid:?} must exist"))
        .commander_ids
        .iter()
        .cloned()
        .collect()
}

/// Seat `pid`'s opening hand by name, sorted.
fn hand_names(state: &mtg_engine::GameState, pid: PlayerId) -> Vec<String> {
    let mut names: Vec<String> = state
        .objects_in_zone(&ZoneId::Hand(pid))
        .iter()
        .map(|obj| obj.characteristics.name.clone())
        .collect();
    names.sort();
    names
}

/// **CR 103.5 — a mulligan permutes a FIXED library-plus-hand multiset.** *"To take a
/// mulligan, a player shuffles the cards in their hand back into their library, draws a new
/// hand of cards equal to their starting hand size, then puts a number of those cards ... on
/// the bottom of their library."* Nothing in that rule replaces a card; the deck a player
/// mulligans is the deck they registered (CR 903.5), and their commander stays in the
/// public command zone (CR 903.6) where the other three players can see it.
///
/// **This is the gate whose absence let G2 ship** (`memory/playtest-triage-2026-08-02b.md`;
/// fixed in `scutemob-187`). `test_redeal_produces_a_different_hand` below asserts only
/// that the *hand* changes — which stayed true while every seat's 99 **and commander** were
/// being re-rolled from the perturbed seed, because `DeckSource::RandomPerSeat` makes every
/// card a function of `cfg.seed`. A test that watches only the thing that is supposed to
/// change cannot see the thing that is not.
///
/// Asserted for **every** seat, not just the mulliganing one: the defect's most visible
/// symptom was the three *opponents'* commanders changing.
#[test]
fn test_redeal_preserves_every_seats_deck_and_commander() {
    // The shape a caller must hand `redeal`: build once from the recipe, then hold the
    // decklists that build actually dealt. This is exactly what `play-server`'s
    // `session::new_game` does.
    let recipe = random_cfg(4, 20_260_802);
    let (dealt_state, _) = build_initial_state(&recipe).expect("setup should succeed");
    let cfg = LocalGameConfig {
        decks: DeckSource::Fixed(
            dealt_decks(&dealt_state, &recipe).expect("a freshly dealt table must be readable"),
        ),
        ..recipe
    };

    let (before, _) = build_initial_state(&cfg).expect("setup should succeed");
    let (after, _) =
        redeal(&cfg, PlayerId(1), 1).expect("redeal should succeed for seat 1's first mulligan");

    for i in 1..=4u64 {
        let pid = PlayerId(i);

        let deck_before = deck_multiset(&before, pid);
        // Non-vacuity floor: an empty or short multiset would compare equal to itself and
        // assert nothing. CR 903.5a — 99 main-deck cards plus the commander.
        assert_eq!(
            deck_before.len(),
            100,
            "seat {i} must own exactly 100 cards before the mulligan (CR 903.5a) — \
             otherwise this test's equality check is vacuous"
        );
        assert_eq!(
            deck_before,
            deck_multiset(&after, pid),
            "CR 103.5: seat {i}'s card multiset must be identical across a mulligan — a \
             mulligan permutes the deck, it does not replace it"
        );

        let commander_before = registered_commanders(&before, pid);
        assert_eq!(
            commander_before.len(),
            1,
            "seat {i} must have exactly one registered commander before the mulligan"
        );
        assert_eq!(
            commander_before,
            registered_commanders(&after, pid),
            "CR 903.6: seat {i}'s commander is public in the command zone and must not \
             change because another seat mulliganed"
        );
        // ... and the object in the public zone must still be that commander, not just the
        // registration: the defect was visible to the other players precisely there.
        let command_zone = after.objects_in_zone(&ZoneId::Command(pid));
        assert_eq!(
            command_zone.len(),
            1,
            "seat {i} keeps one command-zone card"
        );
        let cards = all_cards();
        let def = cards
            .iter()
            .find(|c| c.card_id == commander_before[0])
            .expect("the registered commander must resolve to a CardDefinition");
        assert_eq!(
            &def.name, &command_zone[0].characteristics.name,
            "seat {i}'s command-zone card must still be its registered commander"
        );
    }

    // The other half of CR 103.5: it is still a mulligan. Same cards, new order, new hand.
    assert_ne!(
        hand_names(&before, PlayerId(1)),
        hand_names(&after, PlayerId(1)),
        "the mulliganing seat must actually get a different hand"
    );
    assert_eq!(
        after.objects_in_zone(&ZoneId::Hand(PlayerId(1))).len(),
        7,
        "CR 103.5 — the redeal still draws a fresh 7 (bottoming is the caller's, per the \
         module doc)"
    );
}

/// The caller's obligation, pinned rather than assumed: `redeal` given a config that still
/// holds the **recipe** (`DeckSource::RandomPerSeat`) re-rolls every seat's decklist, which
/// is not a CR 103.5 mulligan.
///
/// `redeal` cannot reject such a config — it is also the plain pregame-rebuild primitive
/// for callers that have not dealt anything yet, where re-rolling is harmless. So the
/// obligation lives with the caller (`resolve_decks` once, then hold the result), and this
/// test is what makes it visible: if `redeal` is ever changed to resolve internally, this
/// test reddens and whoever changed it must say so, rather than the behaviour drifting
/// silently in either direction.
#[test]
fn test_redeal_on_an_unresolved_recipe_still_rerolls_the_decks() {
    let cfg = random_cfg(2, 20_260_802);
    let (before, _) = build_initial_state(&cfg).expect("setup should succeed");
    let (after, _) = redeal(&cfg, PlayerId(1), 1).expect("redeal should succeed");

    assert_ne!(
        deck_multiset(&before, PlayerId(1)),
        deck_multiset(&after, PlayerId(1)),
        "an unresolved RandomPerSeat config re-rolls the decklist — this is the state G2 \
         found in the browser, kept here as the reason callers must resolve first"
    );
}

/// `dealt_decks` reads back exactly what was dealt: one decklist per seat, 99 main-deck
/// cards plus the registered commander (CR 903.5a), and rebuilding from it reproduces every
/// seat's 100-card multiset.
///
/// The round trip is the property `session::new_game` depends on. It does **not** claim the
/// rebuilt table is the same table — the library order and the opening hands change,
/// because a rebuild reshuffles, which is exactly what a mulligan is for.
#[test]
fn test_dealt_decks_round_trip_preserves_every_seats_multiset() {
    let recipe = random_cfg(4, 8_675_309);
    let (dealt_state, _) = build_initial_state(&recipe).expect("setup should succeed");
    let dealt = dealt_decks(&dealt_state, &recipe).expect("a freshly dealt table must be readable");

    assert_eq!(dealt.len(), 4, "one decklist per seat");
    assert_eq!(
        dealt.iter().map(|(pid, _)| *pid).collect::<Vec<_>>(),
        vec![PlayerId(1), PlayerId(2), PlayerId(3), PlayerId(4)],
        "in ascending seat order"
    );
    for (pid, deck) in &dealt {
        assert_eq!(
            deck.main_deck.len(),
            99,
            "seat {pid:?}: 99 main-deck cards (CR 903.5a — the commander is the 100th and \
             is carried separately)"
        );
        assert!(
            !deck.main_deck.contains(&deck.commander),
            "seat {pid:?}: the commander must not also be in the main deck (CR 903.6 put \
             it in the command zone)"
        );
    }

    let rebuilt_cfg = LocalGameConfig {
        decks: DeckSource::Fixed(dealt),
        ..recipe
    };
    let (rebuilt, _) = build_initial_state(&rebuilt_cfg).expect("the dealt decks must be legal");
    for i in 1..=4u64 {
        assert_eq!(
            deck_multiset(&dealt_state, PlayerId(i)),
            deck_multiset(&rebuilt, PlayerId(i)),
            "seat {i}'s 100 cards must survive the round trip"
        );
    }
}

/// `dealt_decks` is pure and total over a well-formed pregame state, and refuses a state it
/// cannot read rather than returning a short decklist.
///
/// The refusal case is what matters: a seat the config names but the state does not hold is
/// `NoDeckForSeat`, not a silently missing entry that would surface much later as a
/// mysteriously invalid deck.
#[test]
fn test_dealt_decks_is_deterministic_and_refuses_an_unreadable_seat() {
    let recipe = random_cfg(3, 4242);
    let (state, _) = build_initial_state(&recipe).expect("setup should succeed");

    let a = dealt_decks(&state, &recipe).expect("read should succeed");
    let b = dealt_decks(&state, &recipe).expect("read should succeed");
    for ((pid_a, deck_a), (pid_b, deck_b)) in a.iter().zip(b.iter()) {
        assert_eq!(pid_a, pid_b);
        assert_eq!(deck_a.commander, deck_b.commander);
        assert_eq!(deck_a.main_deck, deck_b.main_deck);
    }

    // A config naming a seat the state does not have.
    let wider = LocalGameConfig {
        player_count: 4,
        ..random_cfg(3, 4242)
    };
    match dealt_decks(&state, &wider) {
        Err(SetupError::NoDeckForSeat { seat }) => assert_eq!(seat, PlayerId(4)),
        other => panic!("expected NoDeckForSeat for the absent seat 4, got {other:?}"),
    }
}

/// The two shape floors inside `dealt_decks`, exercised rather than asserted in a comment
/// (review LOW 5): a seat whose hand ∪ library is empty, and a seat whose registered
/// commander is sitting in the library instead of the command zone (CR 903.6).
///
/// Both would otherwise produce a decklist that only fails on the *next* rebuild, inside
/// `validate_deck`, with a violation naming the wrong cause — a 0-card deck or a 101-card
/// one. Built from `GameStateBuilder` directly, because `build_initial_state` cannot
/// produce either shape, which is the whole reason the floors are cheap insurance.
#[test]
fn test_dealt_decks_refuses_a_state_it_cannot_read_as_a_deck() {
    let cards = all_cards();
    let commander = fixed_commander(&cards);
    let cfg = random_cfg(1, 7);

    let seat = PlayerId(1);
    let build = |place_commander_in_library: bool| {
        let mut builder = GameStateBuilder::new()
            .with_registry(CardRegistry::new(cards.clone()))
            .add_player(seat)
            .player_commander(seat, commander.card_id.clone());
        if place_commander_in_library {
            builder = builder.object(
                ObjectSpec::card(seat, &commander.name)
                    .in_zone(ZoneId::Library(seat))
                    .with_card_id(commander.card_id.clone()),
            );
        }
        builder
            .first_turn_of_game()
            .build()
            .expect("a one-seat state must build")
    };

    // Empty hand and library: a registered commander but no deck behind it.
    match dealt_decks(&build(false), &cfg) {
        Err(SetupError::NoDeckForSeat { seat: s }) => assert_eq!(s, seat),
        other => panic!("expected NoDeckForSeat for an empty deck, got {other:?}"),
    }

    // Commander in the library: rebuilding would hand `validate_deck` 101 cards.
    match dealt_decks(&build(true), &cfg) {
        Err(SetupError::NoDeckForSeat { seat: s }) => assert_eq!(s, seat),
        other => panic!("expected NoDeckForSeat for a commander in the library, got {other:?}"),
    }
}

/// CR 903.6 / 903.9b — the commander is not merely *placed* in the command zone, it is
/// **registered** as a commander.
///
/// `PlayerState::commander_ids` is the field every commander rule keys off: commander tax
/// (`rules/casting.rs`), the CR 903.9a/704.6d command-zone-return SBA and CR 903.10a
/// commander damage (`rules/commander.rs`, `rules/combat.rs`), and the CR 903.9b
/// hand/library redirects. A game with the object in the command zone but an empty
/// `commander_ids` is legal-looking and silently not a Commander game — the commander is
/// free to recast forever and deals no commander damage. The pre-Session-2 TUI setup this
/// module was lifted from had exactly that gap, so this test is the regression pin.
#[test]
fn test_setup_registers_commanders_not_just_places_them() {
    let cfg = random_cfg(3, 31_337);
    let (state, _names) = build_initial_state(&cfg).expect("setup should succeed");

    for i in 1..=3u64 {
        let pid = PlayerId(i);
        let player = state
            .players()
            .get(&pid)
            .unwrap_or_else(|| panic!("seat {i} must exist"));
        assert_eq!(
            player.commander_ids.len(),
            1,
            "seat {i} must have exactly one registered commander, not just a card sitting \
             in the command zone (CR 903.6)"
        );
        // The registered CardId must be the card actually in the command zone — a
        // registration naming a different card would be worse than none.
        let command_zone = state.objects_in_zone(&ZoneId::Command(pid));
        assert_eq!(command_zone.len(), 1);
        let registered = &player.commander_ids[0];
        let placed_name = &command_zone[0].characteristics.name;
        let cards = all_cards();
        let registered_def = cards
            .iter()
            .find(|c| &c.card_id == registered)
            .expect("the registered commander CardId must resolve to a CardDefinition");
        assert_eq!(
            &registered_def.name, placed_name,
            "seat {i}'s registered commander must be the card in its command zone"
        );
    }

    // CR 903.9b: two replacement effects per commander (would-go-to-hand and
    // would-go-to-library, each redirecting to the command zone) must be registered
    // before the game starts — they are replacements, not triggers, so nothing can add
    // them later.
    //
    // Counts only the redirect-to-command-zone effects rather than
    // `replacement_effects().len()`: a global count would redden for the wrong reason the
    // first time any unrelated pregame replacement is registered.
    let to_command = state
        .replacement_effects()
        .iter()
        .filter(|r| {
            matches!(
                r.modification,
                ReplacementModification::RedirectToZone(ZoneType::Command)
            )
        })
        .count();
    assert_eq!(
        to_command, 6,
        "CR 903.9b: 2 zone-change replacements per commander × 3 seats"
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
