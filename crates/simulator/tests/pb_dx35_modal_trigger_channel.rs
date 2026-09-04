//! PB-DX35 Half A (`OOS-DX4-2`) — the channel probe.
//!
//! `crates/engine/tests/primitives/pb_dx35_modal_trigger_targets.rs`'s `t1` proves the
//! engine-level primitive (`process_command` directly). This file drives the SAME
//! interaction (`retreat_to_kazandu`'s landfall trigger, no creature on the board) end
//! to end through `LocalGame`/`HumanChoice` -- the same channel the browser and a real
//! player go through -- because existence at the engine layer is never sufficiency
//! (the `kaito_shizuki` lesson, PB-DX43): an offer layer or a human decision loop
//! could still lose the fix between the engine and the player.
//!
//! Both seats are human here (no bot), so the drive loop passes priority for whichever
//! seat `advance()` asks, submitting `PlayLand` only for `p1`'s real land drop. This
//! is deliberately NOT a bot game: `StubProvider`'s bot path never needed to change for
//! this fix (the modal choice is `AutoChosen` inside the engine, execution-notes §0.3),
//! so a bot-driven probe would not exercise anything a bot-agnostic engine test does
//! not already cover more directly. Driving it through two human seats instead proves
//! the fix survives the FULL command round-trip a real player's client makes.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, CardRegistry, GameState,
    GameStateBuilder, ObjectId, ObjectSpec, PlayerId, ZoneId,
};
use mtg_simulator::{
    ActionParams, AdvanceOutcome, Bot, HumanChoice, LegalAction, LocalGame, LocalGameLimits,
    StubProvider,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == zone)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found in {zone:?}"))
}

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 3,
        max_commands: 200,
        max_consecutive_passes: 100,
        record_journal: true,
    }
}

/// CR 700.2b / `OOS-DX4-2` through the real channel: `retreat_to_kazandu`'s landfall
/// trigger, with NO creature on the board, resolves and gains 2 life -- the whole
/// scenario `crates/engine/tests/primitives/pb_dx35_modal_trigger_targets.rs::t1`
/// proves at the engine layer, driven here through `LocalGame::submit`.
#[test]
fn c1_retreat_to_kazandu_gains_life_through_local_game_with_no_creature() {
    let defs = load_defs();
    let retreat_id = card_name_to_id("Retreat to Kazandu");
    let retreat = enrich_spec_from_def(
        ObjectSpec::card(P1, "Retreat to Kazandu")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(retreat_id),
        &defs,
    );
    let land = ObjectSpec::land(P1, "Other Land").in_zone(ZoneId::Hand(P1));
    let mut builder = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .active_player(P1)
        .with_registry(CardRegistry::new(vec![defs["Retreat to Kazandu"].clone()]))
        .object(retreat)
        .object(land);
    // `LocalGame::start` -> `start_game` resets the turn to `Step::Untap`
    // (`local_game_human_actions.rs`'s own documented caveat), so this fixture
    // must survive a REAL Untap -> Upkeep -> Draw sequence for both players
    // before ever reaching the main phase where `PlayLand` is offered. An
    // empty library makes CR 104.3c's failed-draw loss fire on turn 1's draw
    // step, which would make this probe pass VACUOUSLY (life unchanged because
    // the game ended, not because the fix worked) -- filler cards close that
    // hole.
    for i in 0..10 {
        builder = builder
            .object(ObjectSpec::card(P1, &format!("Filler {i}")).in_zone(ZoneId::Library(P1)));
        builder = builder
            .object(ObjectSpec::card(P2, &format!("Filler {i} (P2)")).in_zone(ZoneId::Library(P2)));
    }
    let state = builder.build().expect("channel fixture must build");

    assert!(
        state.objects().values().all(|o| !o
            .characteristics
            .card_types
            .contains(&mtg_engine::CardType::Creature)),
        "non-vacuity premise: NO creature must exist on this board"
    );

    let land_id = find_in_zone(&state, "Other Land", ZoneId::Hand(P1));
    let life_before = state.players()[&P1].life_total;

    let human_seats: BTreeSet<PlayerId> = [P1, P2].into_iter().collect();
    let bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    let (mut game, _start_events) =
        LocalGame::start(state, 1, StubProvider, bots, human_seats, limits(), true)
            .expect("game should start");

    let mut land_played = false;
    let mut rounds = 0;
    loop {
        rounds += 1;
        assert!(
            rounds < 50,
            "the game did not settle within 50 priority windows"
        );
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(decision) => {
                if !land_played {
                    if let Some(idx) = decision.actions.iter().position(
                        |a| matches!(a, LegalAction::PlayLand { card } if *card == land_id),
                    ) {
                        game.submit(
                            decision.seq,
                            HumanChoice {
                                action_index: idx,
                                params: ActionParams::default(),
                            },
                        )
                        .expect("PlayLand must be accepted");
                        land_played = true;
                        continue;
                    }
                }
                let pass_idx = decision
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!(
                            "no PassPriority offered to {:?}: {:?}",
                            decision.player, decision.actions
                        )
                    });
                game.submit(
                    decision.seq,
                    HumanChoice {
                        action_index: pass_idx,
                        params: ActionParams::default(),
                    },
                )
                .expect("PassPriority must always be accepted at a priority window");
            }
            AdvanceOutcome::GameOver(_) => break,
            AdvanceOutcome::Halted(reason) => {
                panic!("game halted unexpectedly: {reason:?}");
            }
        }
        if land_played && game.state().stack_objects().is_empty() {
            break;
        }
    }

    assert!(land_played, "the land must actually have been played");
    let life_after = game.state().players()[&P1].life_total;
    assert_eq!(
        life_after,
        life_before + 2,
        "CR 700.2b through the real LocalGame/HumanChoice channel: with no legal \
         target for mode 0, mode 1 (\"You gain 2 life\") must be chosen and resolve"
    );
}
