//! SR-21: the Architecture Invariant 9 completeness gate reaches the script/replay path.
//!
//! Before SR-21, the only completeness choke point was
//! [`mtg_engine::start_game`], which the simulator, fuzzer, and TUI all funnel
//! through. The script/replay regime does not: it builds a `GameState` with
//! `build_initial_state` and steps it with `process_command`, never calling
//! `start_game`. So a script placing an inert / partial / knowingly-wrong
//! `CardDefinition` produced a game that ran ungated — the replay-viewer served
//! whole games out of incomplete defs, and SR-12's "no silent bypass" doc claim
//! was false for this path.
//!
//! `build_initial_state_checked` closes that hole: it is the checked entry (the
//! replay-viewer's production path), while the plain `build_initial_state` is the
//! greppable opt-out that test harnesses and retired scripts use on purpose.
//!
//! These tests prove:
//! 1. the checked builder **refuses** a script placing a known-but-non-`Complete`
//!    card, and the opt-out builder **accepts** the identical script (the opt-out
//!    is real, not decorative);
//! 2. an **unknown** card_id (no registry def) is *not* refused — the gate's scope
//!    is exactly `validate_deck`/`start_game`'s (known-but-marked only), so it
//!    cannot redden the naked-object tests.
//!
//! Note deliberately *not* asserted: "no approved script names a non-`Complete`
//! card." The corpus does not satisfy it — ~20 approved golden scripts place
//! Partial/KnownWrong-marked cards (Darksteel Colossus, Arcane Signet, Terminus,
//! Leyline of the Void, …) to exercise one interaction while an unrelated clause
//! of the def is unfinished. That is exactly why the replay-viewer uses the
//! opt-out [`build_initial_state`] rather than the checked builder.

use std::collections::HashMap;

use mtg_engine::testing::script_schema::{
    InitialState, PermanentInitState, PlayerInitState, ZonesInitState,
};
use mtg_engine::{all_cards, build_initial_state, build_initial_state_checked, GameStateError};

/// A minimal single-player `InitialState` with `card_name` as the sole
/// battlefield permanent under `"p1"`.
fn state_with_battlefield_card(card_name: &str) -> InitialState {
    let mut players = HashMap::new();
    players.insert(
        "p1".to_string(),
        PlayerInitState {
            life: 40,
            mana_pool: HashMap::new(),
            land_plays_remaining: 1,
            poison_counters: 0,
            commander_damage_received: HashMap::new(),
            commander: None,
            partner_commander: None,
        },
    );

    let mut battlefield = HashMap::new();
    battlefield.insert(
        "p1".to_string(),
        vec![PermanentInitState {
            card: card_name.to_string(),
            tapped: false,
            summoning_sick: false,
            counters: HashMap::new(),
            attached: vec![],
            damage_marked: 0,
            is_commander: false,
            subtypes: None,
            is_basic: None,
        }],
    );

    InitialState {
        format: "commander".to_string(),
        turn_number: 1,
        active_player: "p1".to_string(),
        phase: "precombat_main".to_string(),
        step: None,
        priority: "p1".to_string(),
        players,
        zones: ZonesInitState {
            battlefield,
            hand: HashMap::new(),
            graveyard: HashMap::new(),
            exile: vec![],
            command_zone: HashMap::new(),
            library: HashMap::new(),
            stack: vec![],
        },
        continuous_effects: vec![],
    }
}

/// The name of some card whose registry def is *not* `Complete`. Panics if the
/// registry has no such card — which would itself be a signal (the corpus of
/// inert/partial/known-wrong markers is nonzero and this test would be vacuous).
fn a_known_incomplete_card_name() -> String {
    all_cards()
        .iter()
        .find(|d| !d.completeness.is_complete())
        .map(|d| d.name.clone())
        .expect(
            "registry has no non-Complete card def — this gate would be vacuous; \
             if the campaign ever reaches 100% Complete, retire this test",
        )
}

#[test]
/// The checked builder refuses a known-but-non-`Complete` card; the opt-out builds it.
fn checked_builder_refuses_known_incomplete_card_and_opt_out_accepts_it() {
    let card = a_known_incomplete_card_name();
    let init = state_with_battlefield_card(&card);

    // Checked path: refused with the same error `start_game` raises.
    match build_initial_state_checked(&init) {
        Err(GameStateError::IncompleteCardsInGame {
            count, first_name, ..
        }) => {
            assert!(count >= 1, "at least one offender expected");
            assert_eq!(
                first_name, card,
                "the offender should be the incomplete card we placed"
            );
        }
        Err(other) => panic!("expected IncompleteCardsInGame, got {other:?}"),
        Ok(_) => panic!(
            "build_initial_state_checked accepted a state containing non-Complete card {card:?} \
             — the SR-21 gate is not firing"
        ),
    }

    // Opt-out path: the identical script builds without complaint. This is what
    // makes the opt-out load-bearing rather than decorative — if this ever fails,
    // the completeness check leaked into the unchecked builder.
    let (state, _players) = build_initial_state(&init);
    assert!(
        state.objects().values().any(|o| o
            .card_id
            .as_ref()
            .map(|c| c.0 == mtg_engine::card_name_to_id(&card).0)
            .unwrap_or(false)),
        "the opt-out builder should place the incomplete card as an object"
    );
}

#[test]
/// An unknown card_id (no registry def) is *not* an offender — the gate scope is
/// known-but-marked only, matching `start_game` / `validate_deck`. A name the
/// registry has never heard of enters typeless and passes, exactly as the
/// hundreds of naked-object tests rely on.
fn checked_builder_does_not_refuse_unknown_card() {
    let bogus = "Zzz Nonexistent Card That Has No Definition Zzz";
    assert!(
        all_cards().iter().all(|d| d.name != bogus),
        "test premise broken: {bogus:?} unexpectedly has a definition"
    );
    let init = state_with_battlefield_card(bogus);
    build_initial_state_checked(&init)
        .expect("an unknown card_id must pass the gate — it is the UnknownCard axis, out of scope");
}
