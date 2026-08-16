//! PB-DX44 (`OOS-DX29-9`) — the split-card right half, through the OFFER LAYER
//! (`StubProvider`, `params.rs`), not by hand-building a `Command`.
//!
//! `crates/engine/tests/rules/pb_dx44_split_half_cast.rs` (Stage 2a) already
//! proves the ENGINE resolves a right-half-only cast correctly; it drives that
//! by constructing `CastSpellData` directly, which is exactly the channel that
//! did NOT exist before this stage. This file is the missing half: does
//! `StubProvider.legal_actions` actually OFFER `alt_cost:
//! Some(AltCostKind::SplitRightHalf)`, and does a REAL client (a bot via
//! `action_to_command_with_params`, a human via `LocalGame`/`HumanChoice`) get
//! it accepted end to end.
//!
//! CR index: CR 702.102a/709.4 (a right-half-only cast), CR 601.2c (targeting).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, enrich_spec_from_def, AltCostKind, CardDefinition, CardId, CardRegistry, GameState,
    GameStateBuilder, ObjectId, ObjectSpec, PlayerId, SubType, Target, ZoneId,
};
use mtg_simulator::{
    action_to_command_with_params, ActionParams, AdvanceOutcome, Bot, HumanChoice, LegalAction,
    LegalActionProvider, LocalGame, LocalGameLimits, PendingDecision, RandomBot, StubProvider,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn corpus_registry() -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}

fn corpus_card_id(defs: &HashMap<String, CardDefinition>, name: &str) -> CardId {
    defs.get(name)
        .unwrap_or_else(|| panic!("{name:?} is not in `all_cards()`"))
        .card_id
        .clone()
}

fn corpus_object(
    defs: &HashMap<String, CardDefinition>,
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .with_card_id(corpus_card_id(defs, name))
            .in_zone(zone),
        defs,
    )
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?}"))
}

fn is_right_half_cast_of(card: ObjectId) -> impl Fn(&LegalAction) -> bool {
    move |a| {
        matches!(
            a,
            LegalAction::CastSpell {
                card: c,
                alt_cost: Some(AltCostKind::SplitRightHalf),
                ..
            } if *c == card
        )
    }
}

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 3,
        max_commands: 2000,
        max_consecutive_passes: 500,
        record_journal: true,
    }
}

fn expect_decision(game: &mut LocalGame<StubProvider>) -> PendingDecision {
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {other:?}"),
    }
}

fn index_of(actions: &[LegalAction], pred: impl Fn(&LegalAction) -> bool) -> usize {
    actions
        .iter()
        .position(pred)
        .unwrap_or_else(|| panic!("no matching action in {actions:?}"))
}

fn drive_until(
    game: &mut LocalGame<StubProvider>,
    pred: impl Fn(&LegalAction) -> bool,
) -> PendingDecision {
    for _ in 0..400 {
        let decision = expect_decision(game);
        if decision.actions.iter().any(&pred) {
            return decision;
        }
        let pass = index_of(&decision.actions, |a| {
            matches!(a, LegalAction::PassPriority)
        });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: pass,
                params: ActionParams::default(),
            },
        )
        .expect("passing priority is always legal at a priority window");
    }
    panic!("no decision offered a matching action within 400 priority windows");
}

fn drain_stack(game: &mut LocalGame<StubProvider>) {
    for _ in 0..20 {
        if game.state().stack_objects().is_empty() {
            return;
        }
        let decision = expect_decision(game);
        let pass = index_of(&decision.actions, |a| {
            matches!(a, LegalAction::PassPriority)
        });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: pass,
                params: ActionParams::default(),
            },
        )
        .expect("passing to resolve is always legal");
    }
    panic!("stack did not empty within budget");
}

/// `Turn // Burn` in P1's hand, REAL untapped mana SOURCES (never a floating
/// pool -- `start_game`'s `reset_turn_state` empties any pre-game pool per CR
/// 500.4, so a fixture that relies on one is affordable only until the FIRST
/// `LocalGame::start`, which is exactly the trap this comment exists to name)
/// on P1's battlefield that afford BOTH the ordinary (left-half, `{2}{U}`) and
/// the right-half-only (`{1}{R}`) cast at once -- `can_afford` is a static
/// computation over untapped sources, not a spend, so both showing as legal
/// simultaneously is exactly the point (R1 checks that both are OFFERED
/// together) -- and a bystander creature on P2's battlefield, plus a small
/// library for both players so no draw step ends the game before the offer is
/// even reached (`R3` drives real priority passes across turns). Mirrors the
/// shape of `pb_dx44_split_half_cast.rs::t1`'s fixture, rebuilt here because
/// that file's own helpers are private to its test binary (project
/// convention: a duplicate over a public API, not a shared dependency across
/// test binaries).
fn turn_burn_state() -> GameState {
    let defs = defs_by_name();
    let mut builder = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .object(corpus_object(&defs, P1, "Turn // Burn", ZoneId::Hand(P1)))
        .object(corpus_object(&defs, P1, "Mountain", ZoneId::Battlefield))
        .object(corpus_object(&defs, P1, "Island", ZoneId::Battlefield))
        .object(corpus_object(&defs, P1, "Island", ZoneId::Battlefield))
        .object(
            ObjectSpec::creature(P2, "Bystander Bear", 3, 3)
                .with_subtypes(vec![SubType("Bear".to_string())]),
        );
    for i in 0..5 {
        builder = builder
            .object(ObjectSpec::card(P1, &format!("P1 Filler {i}")).in_zone(ZoneId::Library(P1)));
        builder = builder
            .object(ObjectSpec::card(P2, &format!("P2 Filler {i}")).in_zone(ZoneId::Library(P2)));
    }
    builder.build().expect("state builds")
}

// ═══════════════════════════════════════════════════════════════════════════
// R1 — the OFFER itself: SR-38 membership.
// ═══════════════════════════════════════════════════════════════════════════

/// **R1** — `StubProvider.legal_actions` offers the right-half cast for `Turn //
/// Burn` alongside the ordinary (left-half) cast, as TWO separate actions.
#[test]
fn r1_right_half_is_offered_alongside_the_ordinary_cast() {
    let state = turn_burn_state();
    let card_id = id_of(&state, "Turn // Burn");
    let actions = StubProvider.legal_actions(&state, P1);

    assert!(
        actions.iter().any(|a| matches!(
            a,
            LegalAction::CastSpell {
                card, alt_cost: None, ..
            } if *card == card_id
        )),
        "the ordinary (left-half) cast must still be offered: {actions:?}"
    );
    assert!(
        actions.iter().any(is_right_half_cast_of(card_id)),
        "the right-half-only cast must ALSO be offered: {actions:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// R2 — the BOT channel: `action_to_command_with_params` + a real `RandomBot`.
// ═══════════════════════════════════════════════════════════════════════════

/// **R2** — a bot presented with ONLY the right-half action builds a `Command`
/// through the real mapping table (`action_to_command_with_params`, the same
/// function `RandomBot::choose_action` and `LocalGame::advance` both call),
/// and the engine ACCEPTS it — proving the bot channel this stage opened, not
/// just the offer.
///
/// A ONE-element action slice (mirrors `local_game.rs`'s own precedent for
/// forcing a deterministic bot choice) rather than a longer list with
/// `PassPriority` alongside it: `RandomBot` picks by index into whatever it is
/// given, so a single-action slice makes WHICH action gets built
/// deterministic, and this test's subject is the mapping, not the bot's
/// preference.
#[test]
fn r2_bot_channel_builds_and_the_engine_accepts_a_right_half_cast() {
    let state = turn_burn_state();
    let card_id = id_of(&state, "Turn // Burn");
    let target_id = id_of(&state, "Bystander Bear");

    let offer = LegalAction::CastSpell {
        card: card_id,
        from_zone: ZoneId::Hand(P1),
        additional_costs: Default::default(),
        alt_cost: Some(AltCostKind::SplitRightHalf),
    };

    let mut bot = RandomBot::new(7, "Bot-1".to_string());
    let command = bot.choose_action(&state, P1, std::slice::from_ref(&offer));
    let mtg_engine::Command::CastSpell(cast) = &command else {
        panic!("a one-element list of CastSpell must yield a CastSpell, got {command:?}");
    };
    assert_eq!(
        cast.alt_cost,
        Some(AltCostKind::SplitRightHalf),
        "the bot-built command must carry the OFFER's own alt_cost, forwarded \
         verbatim by `action_to_command_with_params`, not `None`"
    );

    // Confirm the SAME mapping function (not a bot-internal shortcut) produces
    // an identical `alt_cost`, then let the ENGINE be the arbiter: fund Burn's
    // own cost ({1}{R}, already on the fixture) and apply the built command
    // for real.
    let params = ActionParams {
        targets: vec![Target::Object(target_id)],
        auto_tap: true,
        ..Default::default()
    };
    let real_command = action_to_command_with_params(&state, P1, &offer, &params)
        .expect("the mapping table must build a Command from the offer's own alt_cost");
    let mtg_engine::Command::CastSpell(real_cast) = &real_command else {
        panic!("expected CastSpell, got {real_command:?}");
    };
    assert_eq!(real_cast.alt_cost, Some(AltCostKind::SplitRightHalf));
    assert_eq!(real_cast.targets, vec![Target::Object(target_id)]);

    // `auto_tap: true` on `ActionParams` is a `LocalGame::submit`-only channel
    // (`local_game.rs::auto_tap_commands_for`); this is a raw `process_command`
    // chain, so Burn's own cost ({1}{R}) is paid explicitly here: tap the
    // Mountain for {R} and one Island for the generic pip.
    let mountain_id = id_of(&state, "Mountain");
    let island_id = id_of(&state, "Island");
    let (state, _) = mtg_engine::process_command(
        state,
        mtg_engine::Command::TapForMana {
            player: P1,
            source: mountain_id,
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("tapping the Mountain must be accepted");
    let (state, _) = mtg_engine::process_command(
        state,
        mtg_engine::Command::TapForMana {
            player: P1,
            source: island_id,
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("tapping an Island must be accepted");

    mtg_engine::process_command(state, real_command)
        .unwrap_or_else(|e| panic!("the engine must accept the bot-built right-half cast: {e:?}"));
}

// ═══════════════════════════════════════════════════════════════════════════
// R3 — the HUMAN channel: `LocalGame` + `HumanChoice`, driven through real
// priority passes exactly as `pb_dx44_spree_mode_costs.rs::e1` does for Spree.
// ═══════════════════════════════════════════════════════════════════════════

/// **R3** — a human seat reaches the right-half offer through
/// `StubProvider`/`LocalGame::advance`, submits it via `HumanChoice`, and the
/// engine resolves Burn's own effect on the announced target — end to end,
/// through the exact channel a browser client uses (`params.rs`'s `CastSpell`
/// arm forwards `alt_cost` verbatim from the chosen `LegalAction`).
#[test]
fn r3_human_channel_offers_and_resolves_a_right_half_cast() {
    let state = turn_burn_state();
    let card_id = id_of(&state, "Turn // Burn");
    let target_id = id_of(&state, "Bystander Bear");

    let human_seats: BTreeSet<PlayerId> = [P1].into_iter().collect();
    let (mut game, _events) = LocalGame::start(
        state,
        3,
        StubProvider,
        HashMap::new(),
        human_seats,
        limits(),
        true,
    )
    .expect("game starts");

    let decision = drive_until(&mut game, is_right_half_cast_of(card_id));
    let idx = index_of(&decision.actions, is_right_half_cast_of(card_id));

    let bear_power_before = mtg_engine::calculate_characteristics(game.state(), target_id)
        .and_then(|c| c.power)
        .expect("the bear has a power before anything resolves");

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![Target::Object(target_id)],
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .unwrap_or_else(|e| {
        panic!("a right-half-only cast, offered and funded, must be accepted: {e:?}")
    });

    assert!(
        game.state()
            .player(P1)
            .expect("p1 exists")
            .mana_pool
            .is_empty(),
        "Burn's own cost ({{1}}{{R}}) must be charged exactly"
    );

    drain_stack(&mut game);

    // Burn deals 2 damage to any target -- observed as a change in the marked
    // damage a layer-resolved characteristics read cannot see directly, so read
    // it from the object's own damage counter via the public accessor instead:
    // the bear's OWN power is unaffected by Burn (Burn deals damage, it is not
    // a P/T-modifying effect), so the index-hazard discriminator here is that
    // Turn's (left half) SetPower/SetToughness/SetCreatureTypes/SetColors did
    // NOT run -- the bear's power must be unchanged from its printed value.
    let bear_power_after = mtg_engine::calculate_characteristics(game.state(), target_id)
        .and_then(|c| c.power)
        .expect("the bear must still exist (2 damage on a 3-toughness creature is not lethal)");
    assert_eq!(
        bear_power_after, bear_power_before,
        "Turn (left half) must NOT have run -- its SetPower/SetToughness effect \
         would change this"
    );
    assert!(
        game.state()
            .object(target_id)
            .map(|o| o.damage_marked > 0)
            .unwrap_or(false),
        "Burn (right half) must have marked damage on the announced target"
    );
}
