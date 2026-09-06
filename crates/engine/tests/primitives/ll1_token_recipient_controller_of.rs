//! LL-1 (`scutemob-255`) — "Destroy target permanent. **Its controller** creates a 3/3."
//!
//! CR 701.7a: the token is created by the DESTROYED permanent's controller, not by
//! the caster. At a two-player table nobody notices which of the two got the 3/3
//! unless they look; at a four-player pod the difference decides games, and
//! `beast_within.rs` / `generous_gift.rs` shipped deck-legal handing it to the caster
//! (`docs/mtg-engine-landscape-assessment.md` §2).
//!
//! The card-side fix is `TokenSpec.recipient =
//! PlayerTarget::ControllerOf(DeclaredTarget 0)`. On its own that is not enough, and
//! the reason is the point of this file:
//!
//! 1. `Effect::Sequence` runs the destroy FIRST. `move_object_to_zone` retires the
//!    target's `ObjectId` (CR 400.7, `state/mod.rs`).
//! 2. `resolve_effect_target_list_indexed`'s `DeclaredTarget` arm then returns
//!    nothing for that id — CR 608.2b's partial-fizzle skip, correct for every
//!    object-target consumer.
//! 3. So `ControllerOf` resolved to an EMPTY recipient list and
//!    `Effect::CreateToken`'s `for recipient in recipients` loop ran zero times.
//!    **No token was created for anybody** — strictly worse than the original bug.
//!
//! The engine half is CR 608.2h ("if an effect requires information about a
//! permanent that has left the battlefield, use its last known information"):
//! `Effect::DestroyPermanent` records the departed permanent's `(controller, owner)`
//! into `EffectContext::departed_permanent_players`, and the `ControllerOf` / `OwnerOf`
//! arms fall back to it when the live lookup finds nothing.
//!
//! `t2` is the pin that makes the LKI reading load-bearing rather than incidental.
//! The obvious cheap alternative — read the GRAVEYARD object the destroy created —
//! passes `t1` and FAILS `t2`, because `move_object_to_zone` resets a new object's
//! controller to its OWNER. A permanent you have gained control of would send its
//! Beast to the player who owns the card instead of the player who controlled it.
//!
//! ## The two defeats, executed (change-class row 1, revert-proven probe)
//!
//! Both run against this file as shipped; the numbers are the runs, not a plan.
//!
//! | revert applied to `crates/engine/src/effects/mod.rs` | t1 | t2 |
//! |---|---|---|
//! | none (shipped) | ok | ok |
//! | the `ControllerOf` / `OwnerOf` CR 608.2h fallback returns `Vec::new()` | **FAILED** — "NO Beast token was created for anyone" | **FAILED** |
//! | the fallback returns the recorded `owner` instead of the `controller` (what reading the graveyard object would give) | ok | **FAILED** — got p4, expected p1 |
//!
//! The third row is why `t2` exists as a separate test: the cheap
//! implementation passes `t1` and is wrong, and only a fixture where owner and
//! controller are different seats can tell the two apart.
//!
//! Golden-script companion: `test-data/generated-scripts/tokens/002_beast_within_creates_beast.json`
//! (re-approved by this batch) covers the same shape through the JSON harness at
//! N=2. This file is the direct-`Command` half at N=4, per SR-9b.

use std::collections::HashMap;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameState, GameStateBuilder, ManaColor, ObjectId, ObjectSpec, PlayerId,
    Step, Target,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn all_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

/// The controller of the (single) Beast token on the battlefield, or `None` if no
/// token was created at all. The two are DIFFERENT failures and the tests below
/// distinguish them: "created for the wrong player" is the original bug, "created
/// for nobody" is what the naive card-only fix produced.
fn beast_controller(state: &GameState) -> Option<PlayerId> {
    let beasts: Vec<PlayerId> = state
        .objects()
        .iter()
        .filter(|(_, o)| {
            o.characteristics.name == "Beast" && o.is_token && o.zone == ZoneId::Battlefield
        })
        .map(|(_, o)| o.controller)
        .collect();
    assert!(
        beasts.len() <= 1,
        "expected at most one Beast token, found {}",
        beasts.len()
    );
    beasts.first().copied()
}

/// Everyone passes once, in seat order starting from `first`.
fn pass_round(state: GameState, first: PlayerId) -> GameState {
    let order = [1u64, 2, 3, 4];
    let start = order.iter().position(|n| *n == first.0).expect("seat");
    let mut current = state;
    for i in 0..4 {
        let pl = p(order[(start + i) % 4]);
        let (s, _) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
    }
    current
}

/// Build a four-player board, put `beast_within_caster`'s Beast Within in hand with
/// the mana to cast it, and stand up one Sol Ring whose OWNER and CONTROLLER are
/// given separately. Returns the built state with priority on the caster.
fn four_player_board(
    caster: PlayerId,
    sol_ring_owner: PlayerId,
    sol_ring_controller: PlayerId,
) -> GameState {
    let defs = all_defs_by_name();
    let registry = CardRegistry::new(all_cards());

    let beast_within = enrich_spec_from_def(
        ObjectSpec::card(caster, "Beast Within")
            .in_zone(ZoneId::Hand(caster))
            .with_card_id(card_name_to_id("Beast Within")),
        &defs,
    );
    let sol_ring = enrich_spec_from_def(
        ObjectSpec::card(sol_ring_owner, "Sol Ring")
            .in_zone(ZoneId::Battlefield)
            .controlled_by(sol_ring_controller)
            .with_card_id(card_name_to_id("Sol Ring")),
        &defs,
    );

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .object(beast_within)
        .object(sol_ring)
        .active_player(caster)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("four-player board");

    // Beast Within is {2}{G}.
    {
        let pool = &mut state.players_mut().get_mut(&caster).unwrap().mana_pool;
        pool.add(ManaColor::Green, 1);
        pool.add(ManaColor::Colorless, 2);
    }
    state.turn_mut().priority_holder = Some(caster);
    state
}

/// Cast Beast Within at the Sol Ring and let it resolve (all four players pass).
fn cast_and_resolve(state: GameState, caster: PlayerId) -> GameState {
    let beast_within_id = find_object(&state, "Beast Within");
    let sol_ring_id = find_object(&state, "Sol Ring");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: caster,
            card: beast_within_id,
            targets: vec![Target::Object(sol_ring_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell(Beast Within) failed: {e:?}"));

    assert_eq!(
        state.stack_objects().len(),
        1,
        "Beast Within should be the only thing on the stack"
    );

    let state = pass_round(state, caster);
    assert!(
        state.stack_objects().is_empty(),
        "Beast Within should have resolved after a full pass round"
    );
    state
}

/// CR 701.7a at a four-player table: p1 casts Beast Within on a Sol Ring **p3**
/// controls. The 3/3 Beast belongs to p3.
///
/// This is the revert-proven probe for the CR 608.2h fallback in
/// `resolve_player_target_list`. Reverting that arm makes the recipient list empty
/// and this test fails on `Some(p3)` vs `None` — "no Beast token was created at
/// all". Reverting only the card's `recipient` line makes it fail on `Some(p3)` vs
/// `Some(p1)` — the original deck-legal bug. Both halves are load-bearing and each
/// is named separately in the assertion messages below.
#[test]
fn t1_beast_lands_with_the_destroyed_permanents_controller_at_four_players() {
    let (p1, p3) = (p(1), p(3));
    let state = four_player_board(p1, p3, p3);

    assert_eq!(
        beast_controller(&state),
        None,
        "no Beast token before the spell resolves"
    );

    let state = cast_and_resolve(state, p1);

    match beast_controller(&state) {
        None => panic!(
            "CR 701.7a: NO Beast token was created for anyone. The destroy retired the \
             target's ObjectId (CR 400.7) and `PlayerTarget::ControllerOf` resolved to an \
             empty recipient list, so `Effect::CreateToken`'s recipient loop ran zero \
             times. The CR 608.2h fallback in `resolve_player_target_list` \
             (`EffectContext::departed_permanent_players`) is what makes this answerable."
        ),
        Some(owner) => assert_eq!(
            owner, p3,
            "CR 701.7a: 'ITS controller creates a 3/3 green Beast' — the Beast belongs to \
             p3, who controlled the destroyed Sol Ring, not to p1 who cast the spell. \
             Getting p1 here means `beast_within.rs`'s `TokenSpec.recipient` has gone back \
             to the default `PlayerTarget::Controller`."
        ),
    }

    // And the destroy itself still happened, so a green assertion above cannot be
    // the spell having quietly fizzled.
    assert!(
        state
            .objects()
            .iter()
            .all(|(_, o)| !(o.characteristics.name == "Sol Ring" && o.zone == ZoneId::Battlefield)),
        "CR 701.8a: the Sol Ring should have been destroyed"
    );
}

/// CR 608.2h vs CR 400.7: controller and owner are DIFFERENT players.
///
/// p1 has gained control of a Sol Ring **p4 owns**; p2 casts Beast Within on it.
/// "Its controller" is p1 — the player who controlled the permanent while it was on
/// the battlefield — not p4 who owns the card, and not p2 who cast the spell.
///
/// This is the test that forces the fallback to read the PRE-MOVE controller.
/// `move_object_to_zone` builds the graveyard object with `controller:
/// old_object.owner`, so an implementation that reads the destroyed permanent's
/// graveyard object instead of the recorded pre-move pair answers p4 here while
/// still passing `t1` (where owner and controller coincide).
#[test]
fn t2_controller_not_owner_when_the_permanent_was_under_someone_elses_control() {
    let (p1, p2, p4) = (p(1), p(2), p(4));
    // Sol Ring owned by p4, controlled by p1; p2 is the caster. Three distinct seats.
    let state = four_player_board(p2, p4, p1);

    {
        let sol_ring_id = find_object(&state, "Sol Ring");
        let obj = state.objects().get(&sol_ring_id).expect("Sol Ring");
        assert_eq!(obj.owner, p4, "fixture: Sol Ring is owned by p4");
        assert_eq!(obj.controller, p1, "fixture: Sol Ring is controlled by p1");
    }

    let state = cast_and_resolve(state, p2);

    assert_eq!(
        beast_controller(&state),
        Some(p1),
        "CR 701.7a / CR 608.2h: 'its controller' is the permanent's last known \
         CONTROLLER (p1), not its owner (p4) and not the caster (p2). Getting p4 means \
         the fallback is reading the graveyard object — `move_object_to_zone` resets a \
         new object's controller to its owner (CR 400.7), which is exactly why the \
         pre-move pair is recorded in `EffectContext::departed_permanent_players` \
         instead."
    );
}

// ── Emergency Eject: the Lander, and why its marker says `Complete` ───────────

/// `emergency_eject.rs` shipped `Completeness::partial` with a note saying the ONLY
/// blocker was the missing token recipient and that a Lander itself "IS expressible
/// today". LL-1 took the note at its word and authored the token, which moves the
/// def to `Complete` — i.e. **deck-legal**. CLAUDE.md calls a guessed `completeness`
/// marker "the single most prohibited pattern" in this repo, so this test executes
/// the claim rather than trusting it.
///
/// Oracle (verified 2026-09-05 via `mtg-rules` `lookup_card`): "Destroy target
/// nonland permanent. **Its controller** creates a Lander token. (It's an artifact
/// with "{2}, {T}, Sacrifice this token: Search your library for a basic land card,
/// put it onto the battlefield tapped, then shuffle.")"
///
/// Four players, p1 casts at a permanent **p3** controls, so both halves are pinned
/// at once: the Lander must reach p3 (CR 701.7a, the recipient), and p3 must then be
/// able to actually USE it — {2} + tap + sacrifice-self paid, a basic land onto the
/// battlefield TAPPED, the Lander gone. SR-36's lesson is that an activation cost is
/// only paid where code pays it; a token ability authored into a `Complete` def and
/// never executed is that hazard with a deck-legality stamp on it.
#[test]
fn t3_emergency_eject_lander_reaches_the_targets_controller_and_actually_works() {
    let (p1, p3) = (p(1), p(3));
    let defs = all_defs_by_name();
    let registry = CardRegistry::new(all_cards());

    let eject = enrich_spec_from_def(
        ObjectSpec::card(p1, "Emergency Eject")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Emergency Eject")),
        &defs,
    );
    let sol_ring = enrich_spec_from_def(
        ObjectSpec::card(p3, "Sol Ring")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Sol Ring")),
        &defs,
    );
    // A basic land in p3's library for the Lander to find.
    let forest = enrich_spec_from_def(
        ObjectSpec::card(p3, "Forest")
            .in_zone(ZoneId::Library(p3))
            .with_card_id(card_name_to_id("Forest")),
        &defs,
    );

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .object(eject)
        .object(sol_ring)
        .object(forest)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("four-player board");
    {
        // Emergency Eject is {2}{W}.
        let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::White, 1);
        pool.add(ManaColor::Colorless, 2);
    }
    {
        // The Lander's own ability costs {2}.
        let pool = &mut state.players_mut().get_mut(&p3).unwrap().mana_pool;
        pool.add(ManaColor::Colorless, 2);
    }
    state.turn_mut().priority_holder = Some(p1);

    let eject_id = find_object(&state, "Emergency Eject");
    let sol_ring_id = find_object(&state, "Sol Ring");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: eject_id,
            targets: vec![Target::Object(sol_ring_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell(Emergency Eject) failed: {e:?}"));

    let state = pass_round(state, p1);
    assert!(
        state.stack_objects().is_empty(),
        "Emergency Eject should have resolved"
    );

    // Half one: the Lander belongs to p3, the destroyed permanent's controller.
    let landers: Vec<(ObjectId, PlayerId)> = state
        .objects()
        .iter()
        .filter(|(_, o)| {
            o.characteristics.name == "Lander" && o.is_token && o.zone == ZoneId::Battlefield
        })
        .map(|(id, o)| (*id, o.controller))
        .collect();
    assert_eq!(
        landers.len(),
        1,
        "CR 701.7a: exactly one Lander token should exist after resolution"
    );
    let (lander_id, lander_controller) = landers[0];
    assert_eq!(
        lander_controller, p3,
        "CR 701.7a: 'ITS controller creates a Lander token' — p3 controlled the \
         destroyed Sol Ring; p1 merely cast the spell"
    );

    // Half two: the Lander's printed ability is not decoration. p3 activates it.
    state_activates_lander(state, p3, lander_id);
}

/// Split out only to keep `t3` readable. Activates the Lander, answers the CR 701.19a
/// search choice, resolves, and asserts the printed outcome: a basic land onto the
/// battlefield TAPPED, and the Lander sacrificed as part of the cost (CR 601.2h — the
/// cost is paid on activation, so the token is gone before the ability resolves).
fn state_activates_lander(mut state: GameState, controller: PlayerId, lander_id: ObjectId) {
    // Hand priority to the Lander's controller. CR 117.3c gives it back to the actor
    // after Emergency Eject resolves (p1); an instant-speed activation by p3 in the
    // same main phase is legal, and this is the fixture doing what a priority round
    // would, without four more PassPriority calls that prove nothing here.
    state.turn_mut().priority_holder = Some(controller);
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: controller,
            source: lander_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "ActivateAbility on the Lander failed: {e:?}. The def claims \
             `Completeness::Complete`, which makes Emergency Eject deck-legal — a token \
             whose printed ability cannot be activated makes that marker a lie."
        )
    });

    assert!(
        !state.stack_objects().is_empty(),
        "CR 602.2: the Lander's ability should be on the stack after activation"
    );
    // CR 601.2h: costs are paid on activation. The Lander sacrificed itself, so its
    // battlefield object is already gone.
    assert!(
        state.objects().get(&lander_id).is_none()
            || state.objects().get(&lander_id).map(|o| o.zone) != Some(ZoneId::Battlefield),
        "CR 601.2h / 701.17a: `sacrifice_self` should have moved the Lander off the \
         battlefield when the cost was paid"
    );

    let state = pass_round(state, controller);

    // CR 701.19a: the search is a real player decision (PB-DP suite). Answer it with
    // the engine's own default, which finds the only legal candidate.
    let state = if let Some(entry) = state.pending_effect_choice() {
        let (player, choice_id) = (entry.player, entry.choice_id);
        let answer = mtg_engine::effects::default_effect_choice_answer(&entry.question);
        let (s, _) = process_command(
            state,
            Command::AnswerEffectChoice {
                player,
                choice_id,
                answer,
            },
        )
        .expect("the engine must accept its own default search answer");
        s
    } else {
        state
    };

    let forest = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|(_, o)| (o.controller, o.status.tapped));
    let (forest_controller, forest_tapped) = forest.unwrap_or_else(|| {
        panic!(
            "the Lander's ability did not put a basic land onto the battlefield. Its \
             printed text is \"Search your library for a basic land card, put it onto the \
             battlefield tapped, then shuffle\" and `emergency_eject.rs` is marked \
             `Complete` on the strength of that ability being modelled."
        )
    });
    assert_eq!(
        forest_controller, controller,
        "the searched-up land enters under the Lander controller's control"
    );
    assert!(
        forest_tapped,
        "the printed ability says the land enters TAPPED — an untapped one means the \
         `ZoneTarget::Battlefield {{ tapped: true }}` destination is not being honoured"
    );
}
