//! PB-DX29 (simulator half) — the loyalty answer channel
//! (`OOS-M11-10(loyalty)`).
//!
//! `Command::ActivateLoyaltyAbility` has carried `targets` and `x_value` since
//! M11-local S5. Nothing above the engine could name either: `params.rs`'s
//! `ActivateLoyaltyAbility` arm sat OUTSIDE its own parameterization allowlist (so a
//! client announcing anything got `ParamError::UnsupportedParam` — a 400) and then
//! hard-coded `targets: Vec::new(), x_value: None` (so a client announcing nothing got
//! a silently untargeted, X = 0 activation). The bot path was worse than the browser's:
//! `targeting::target_query_source` had no loyalty arm at all, so `plan_targets`
//! returned `NotTargeted` and no bot could ever announce a loyalty target.
//!
//! Five sections, one per shipped site plus the two end-to-end drives:
//!
//! * **S1/S2/S3** — `params.rs`: targets forwarded, `x_value` forwarded with `0 -> None`,
//!   and `ActionParams::default()` still producing the byte-identical pre-batch command.
//! * **S4** — `targeting.rs`: the BOT half, `plan_targets` -> `Announce(..)`.
//! * **S5/S6** — `legal_actions.rs`: the SR-38 offer suppression, and its non-over-firing.
//! * **S7** — `LocalGame`: a human seat activating with a NON-DEFAULT target, verified
//!   by NAME (CR 400.7).
//!
//! CR references: CR 606.3 (sorcery-speed, once per turn), CR 606.4 (the loyalty cost
//! is paid on activation), CR 601.2c (targets are announced and validated),
//! CR 107.3m (the activating player chooses X), CR 400.7 (an id dies on a zone change).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, legal_targets_per_slot,
    loyalty_ability_target_requirements, CardDefinition, Command, CounterType, GameState,
    GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, Target, TargetRequirement, ZoneId,
};
use mtg_simulator::{
    action_to_command_with_params, build_registry, plan_targets, ActionParams, AdvanceOutcome, Bot,
    HumanChoice, LegalAction, LegalActionProvider, LocalGame, LocalGameLimits, ParamError,
    StubProvider, TargetPlan,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// Sarkhan Vol's loyalty index of the `-2` ("Gain control of target creature until end
/// of turn. Untap that creature. It gains haste until end of turn."). Pinned as a named
/// constant because every section below depends on it, and it is the FILTERED index —
/// the def's raw `abilities` vector happens to agree here only because Sarkhan Vol
/// carries no non-loyalty abilities.
const SARKHAN_MINUS_2: usize = 1;
/// Sarkhan Vol's `+1` — untargeted, and the control case for S5/S6.
const SARKHAN_PLUS_1: usize = 0;

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?}"))
}

fn name_of(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .unwrap_or_else(|| panic!("no live object {id:?}"))
        .characteristics
        .name
        .clone()
}

fn object_by_name<'a>(state: &'a GameState, name: &str) -> &'a mtg_engine::GameObject {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name)
        .unwrap_or_else(|| panic!("no live object named {name:?}"))
}

/// A real, `Complete` Sarkhan Vol on `P1`'s battlefield with four loyalty counters —
/// enriched from its own `CardDefinition` (the standing `ObjectSpec::card()`-is-naked
/// gotcha) and given the counters explicitly, because `enrich_spec_from_def` writes
/// `characteristics.loyalty` and CR 606.4's payment reads `counters[Loyalty]`.
fn sarkhan_spec(defs: &HashMap<String, CardDefinition>) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(P1, "Sarkhan Vol")
            .with_card_id(card_name_to_id("Sarkhan Vol"))
            .in_zone(ZoneId::Battlefield)
            .with_counter(CounterType::Loyalty, 4),
        defs,
    )
}

/// The S5 board, parameterised on **one** thing: which zone the single creature sits
/// in. Everything else — the same Sarkhan Vol, the same two players, the same step, the
/// same object set — is identical, so any difference in the offered action list is
/// attributable to the candidate's presence on the battlefield and to nothing else.
fn sarkhan_board(candidate_zone: ZoneId) -> GameState {
    let defs = card_defs_by_name();
    let mut state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(build_registry())
        .active_player(P1)
        .at_step(Step::PreCombatMain)
        .object(sarkhan_spec(&defs))
        .object(ObjectSpec::creature(P2, "Opp Alpha", 2, 2).in_zone(candidate_zone))
        .build()
        .expect("PB-DX29 S5 fixture must build");
    state.turn_mut().priority_holder = Some(P1);
    state
}

/// A two-creature board for the parameterisation probes (S1-S4) and the `LocalGame`
/// drive (S7): two opponent creatures, so "the second candidate" is meaningful.
fn two_candidate_board() -> GameState {
    let defs = card_defs_by_name();
    let mut state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(build_registry())
        .active_player(P1)
        .at_step(Step::PreCombatMain)
        .object(sarkhan_spec(&defs))
        .object(
            ObjectSpec::creature(P2, "Opp Alpha", 2, 2)
                .in_zone(ZoneId::Battlefield)
                .tapped(),
        )
        .object(
            ObjectSpec::creature(P2, "Opp Beta", 3, 3)
                .in_zone(ZoneId::Battlefield)
                .tapped(),
        )
        .build()
        .expect("PB-DX29 fixture must build");
    state.turn_mut().priority_holder = Some(P1);
    state
}

/// The `-2`'s candidate list, in the engine's own deterministic enumeration order.
fn minus_2_candidates(state: &GameState, sarkhan: ObjectId) -> Vec<Target> {
    let reqs = loyalty_ability_target_requirements(state, sarkhan, SARKHAN_MINUS_2);
    assert_eq!(
        reqs,
        vec![TargetRequirement::TargetCreature],
        "precondition: Sarkhan Vol's -2 must still be one mandatory TargetCreature"
    );
    let per_slot = legal_targets_per_slot(state, P1, sarkhan, &reqs);
    assert_eq!(per_slot.len(), 1, "one slot, parallel to one requirement");
    per_slot[0].clone()
}

fn loyalty_action(source: ObjectId, ability_index: usize) -> LegalAction {
    LegalAction::ActivateLoyaltyAbility {
        source,
        ability_index,
    }
}

/// The loyalty `ability_index`es `actions` offers for `source`, in offer order.
/// `LegalAction` derives only `Clone, Debug` (no `PartialEq`), so membership is
/// decided by a `matches!` predicate rather than by `Vec::contains`.
fn loyalty_offers(actions: &[LegalAction], source: ObjectId) -> Vec<usize> {
    actions
        .iter()
        .filter_map(|a| match a {
            LegalAction::ActivateLoyaltyAbility {
                source: s,
                ability_index,
            } if *s == source => Some(*ability_index),
            _ => None,
        })
        .collect()
}

// ── S1: params.rs forwards announced targets ──────────────────────────────────

/// CR 601.2c — S1. `action_to_command_with_params` forwards a human's announced
/// `targets` onto `Command::ActivateLoyaltyAbility`. Before this batch the same call
/// returned `Err(ParamError::UnsupportedParam("targets"))`, because the action was
/// missing from the allowlist that guards `first_announced_field()`.
///
/// **Revert to watch red**: remove the `| LegalAction::ActivateLoyaltyAbility { .. }`
/// line from the allowlist in `params.rs` (the `Err` half), or restore
/// `targets: Vec::new()` on its arm (the forwarding half).
#[test]
fn test_dx29_s1_params_forwards_announced_targets_on_a_loyalty_action() {
    let state = two_candidate_board();
    let sarkhan = id_of(&state, "Sarkhan Vol");
    let alpha = id_of(&state, "Opp Alpha");

    let action = loyalty_action(sarkhan, SARKHAN_MINUS_2);
    let params = ActionParams {
        targets: vec![Target::Object(alpha)],
        ..Default::default()
    };

    let command = action_to_command_with_params(&state, P1, &action, &params)
        .expect("CR 601.2c: announcing a target on a loyalty ability must be accepted");

    match command {
        Command::ActivateLoyaltyAbility {
            player,
            source,
            ability_index,
            targets,
            x_value,
        } => {
            assert_eq!(player, P1);
            assert_eq!(source, sarkhan);
            assert_eq!(ability_index, SARKHAN_MINUS_2);
            assert_eq!(
                targets,
                vec![Target::Object(alpha)],
                "CR 601.2c: the announced target must reach the Command verbatim"
            );
            assert_eq!(x_value, None, "no X was announced");
        }
        other => panic!("expected ActivateLoyaltyAbility, got {other:?}"),
    }

    // The engine really does accept what the param layer built — the SR-38 standard,
    // checked rather than assumed.
    let command = action_to_command_with_params(&state, P1, &action, &params).unwrap();
    let (after, _events) =
        mtg_engine::process_command(state, command).expect("the engine must accept it");
    assert_eq!(
        after
            .stack_objects()
            .iter()
            .filter(|so| !so.targets.is_empty())
            .count(),
        1,
        "CR 601.2c: the loyalty ability is on the stack carrying its announced target"
    );
}

// ── S2: params.rs forwards x_value, and 0 maps to None ────────────────────────

/// CR 107.3m / CR 606.4 — S2. `x_value: 3` becomes `Some(3)`; `x_value: 0` becomes
/// `None`, **not** `Some(0)`.
///
/// The `None` half is the load-bearing one and is asserted explicitly:
/// `ActionParams::x_value` is a bare `u32` whose serde default is 0, so 0 means "the
/// caller announced nothing". `Command` is serialized into the replay log and the
/// journal, so mapping 0 to `Some(0)` would change every bot-driven loyalty
/// activation's recorded bytes for no behavioural gain (the engine reads
/// `x_value.unwrap_or(0)` either way).
///
/// **Revert to watch red**: replace `(params.x_value > 0).then_some(params.x_value)`
/// with `Some(params.x_value)` (the `None` half reddens) or with `None` (the `Some(3)`
/// half reddens).
#[test]
fn test_dx29_s2_params_forwards_x_value_and_maps_zero_to_none() {
    let defs = card_defs_by_name();
    let mut state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(build_registry())
        .active_player(P1)
        .at_step(Step::PreCombatMain)
        .object(enrich_spec_from_def(
            ObjectSpec::card(P1, "Chandra, Flamecaller")
                .with_card_id(card_name_to_id("Chandra, Flamecaller"))
                .in_zone(ZoneId::Battlefield)
                .with_counter(CounterType::Loyalty, 4),
            &defs,
        ))
        .build()
        .expect("PB-DX29 S2 fixture must build");
    state.turn_mut().priority_holder = Some(P1);

    let chandra = id_of(&state, "Chandra, Flamecaller");
    // Chandra's `-X` is loyalty index 2 (+1, 0, -X). Non-vacuity: the engine agrees.
    assert!(
        mtg_engine::loyalty_ability_needs_x(&state, chandra, 2),
        "precondition: Chandra's loyalty index 2 is the -X"
    );
    let action = loyalty_action(chandra, 2);

    let announced = action_to_command_with_params(
        &state,
        P1,
        &action,
        &ActionParams {
            x_value: 3,
            ..Default::default()
        },
    )
    .expect("CR 107.3m: announcing X must be accepted");
    match announced {
        Command::ActivateLoyaltyAbility { x_value, .. } => assert_eq!(
            x_value,
            Some(3),
            "CR 107.3m: an announced X must reach the Command"
        ),
        other => panic!("expected ActivateLoyaltyAbility, got {other:?}"),
    }

    let unannounced = action_to_command_with_params(
        &state,
        P1,
        &action,
        &ActionParams {
            x_value: 0,
            ..Default::default()
        },
    )
    .expect("x_value 0 is 'announced nothing' and is always accepted");
    match unannounced {
        Command::ActivateLoyaltyAbility { x_value, .. } => assert_eq!(
            x_value, None,
            "x_value 0 must map to None, not Some(0) -- a default-params bot's recorded \
             Command bytes must not move"
        ),
        other => panic!("expected ActivateLoyaltyAbility, got {other:?}"),
    }

    // And the engine spends what was announced (CR 606.4): 4 - 3 = 1.
    let command = action_to_command_with_params(
        &state,
        P1,
        &action,
        &ActionParams {
            x_value: 3,
            ..Default::default()
        },
    )
    .unwrap();
    let (after, _events) =
        mtg_engine::process_command(state, command).expect("the engine must accept X = 3");
    assert_eq!(
        object_by_name(&after, "Chandra, Flamecaller")
            .counters
            .get(&CounterType::Loyalty)
            .copied()
            .unwrap_or(0),
        1,
        "CR 606.4: -X with X = 3 spends three loyalty counters"
    );
}

// ── S3: default params reproduce the pre-batch command exactly ────────────────

/// CR 606.3 — S3. With `ActionParams::default()` the produced `Command` is EXACTLY the
/// pre-batch one: `targets: vec![]`, `x_value: None`. Every recorded fuzz seed and
/// every seeded play-server fixture drives bots through default params, so this is the
/// assertion that says PB-DX29 moved no recorded bytes.
///
/// **Revert to watch red**: map `x_value` to `Some(params.x_value)` in `params.rs`
/// (`x_value` becomes `Some(0)`).
#[test]
fn test_dx29_s3_default_params_produce_the_pre_batch_command() {
    let state = two_candidate_board();
    let sarkhan = id_of(&state, "Sarkhan Vol");

    for ability_index in [SARKHAN_PLUS_1, SARKHAN_MINUS_2] {
        let command = action_to_command_with_params(
            &state,
            P1,
            &loyalty_action(sarkhan, ability_index),
            &ActionParams::default(),
        )
        .expect("default params must always be accepted");
        assert_eq!(
            command,
            Command::ActivateLoyaltyAbility {
                player: P1,
                source: sarkhan,
                ability_index,
                targets: vec![],
                x_value: None,
            },
            "default params must reproduce the pre-PB-DX29 command byte for byte"
        );
    }
}

/// CR 601.2c — S3B, **pinned WRONG-WAY-ROUND**. Joining the `params.rs` allowlist is
/// not free: the allowlist is what makes `first_announced_field()` run at all, so an
/// arm inside it stops refusing *every* param and starts silently ignoring the ones it
/// does not read. Announcing `attackers` on a loyalty activation was
/// `Err(ParamError::UnsupportedParam("attackers"))` before PB-DX29 and is now `Ok(..)`
/// with the field dropped.
///
/// This is exactly the class `params.rs`' own doc calls out — *"Residual, deliberately
/// not guarded: a param announced on a consuming arm that that arm does not read (e.g.
/// `attackers` alongside a `CastSpell`) is still ignored. The nine consuming arms would
/// each need their own field allowlist to catch that"* — and PB-DX29 makes that "nine"
/// a **ten**. Recorded here rather than fixed: a per-arm field allowlist is a params.rs
/// design change, not a PB-DX29 deliverable, and the browser half is already covered by
/// `tools/play-server`'s `api.rs` candidate cross-check.
///
/// **This test reddens the day per-arm allowlists land.** That is deliberate — the
/// implementer must come here and restore the `Err(UnsupportedParam("attackers"))`
/// assertion.
#[test]
fn test_dx29_s3b_an_unread_param_on_a_loyalty_action_is_now_silently_ignored() {
    let state = two_candidate_board();
    let sarkhan = id_of(&state, "Sarkhan Vol");
    let alpha = id_of(&state, "Opp Alpha");

    let result = action_to_command_with_params(
        &state,
        P1,
        &loyalty_action(sarkhan, SARKHAN_MINUS_2),
        &ActionParams {
            attackers: vec![(alpha, mtg_engine::AttackTarget::Player(P2))],
            ..Default::default()
        },
    );
    assert_eq!(
        result,
        Ok(Command::ActivateLoyaltyAbility {
            player: P1,
            source: sarkhan,
            ability_index: SARKHAN_MINUS_2,
            targets: vec![],
            x_value: None,
        }),
        "PB-DX29 RESIDUAL PIN: `attackers` on a loyalty activation is dropped in \
         silence (it was `Err(UnsupportedParam)` before this batch). If this is now an \
         Err, per-arm field allowlists have landed -- restore the refusal assertion."
    );

    // The half that did NOT widen: a param on a NON-allowlisted arm is still refused,
    // so the allowlist really is a per-arm switch and not a global one.
    assert_eq!(
        action_to_command_with_params(
            &state,
            P1,
            &LegalAction::PassPriority,
            &ActionParams {
                attackers: vec![(alpha, mtg_engine::AttackTarget::Player(P2))],
                ..Default::default()
            },
        ),
        Err(ParamError::UnsupportedParam("attackers")),
        "non-vacuity: `first_announced_field` still refuses on an arm outside the \
         allowlist, so the Ok above is about the allowlist and not about the field \
         having become unreadable"
    );
}

// ── S4: the BOT half ──────────────────────────────────────────────────────────

/// CR 601.2c / CR 602.2b — S4. `targeting::plan_targets` returns
/// `TargetPlan::Announce(..)` for a targeted loyalty action. Before this batch
/// `target_query_source` had no loyalty arm, so it returned `None` and `plan_targets`
/// short-circuited to `TargetPlan::NotTargeted` — every bot's loyalty activation was
/// untargeted, which is SIM-5's zero-target-cast defect re-created on a new action.
///
/// Both halves of the fix are exercised: `target_query_source` (without which the
/// result is `NotTargeted`) and `action_target_requirements` (without which it is also
/// `NotTargeted`, via the empty-requirements short circuit). The `Unsatisfiable` arm
/// is exercised on the candidate-free board, which is the predicate S5's offer
/// suppression rests on.
///
/// **Revert to watch red**: delete either loyalty arm from `crates/simulator/src/
/// targeting.rs`.
#[test]
fn test_dx29_s4_plan_targets_announces_for_a_targeted_loyalty_action() {
    let state = two_candidate_board();
    let sarkhan = id_of(&state, "Sarkhan Vol");
    let candidates = minus_2_candidates(&state, sarkhan);
    assert!(
        candidates.len() >= 2,
        "non-vacuity: the board must offer at least two candidates, got {candidates:?}"
    );

    let plan = plan_targets(&state, P1, &loyalty_action(sarkhan, SARKHAN_MINUS_2));
    assert_eq!(
        plan,
        TargetPlan::Announce(vec![candidates[0].clone()]),
        "CR 601.2c: a bot announces the first legal candidate for the mandatory slot"
    );

    // The untargeted `+1` is still `NotTargeted` — the arm must not manufacture a
    // target where the ability declares none.
    assert_eq!(
        plan_targets(&state, P1, &loyalty_action(sarkhan, SARKHAN_PLUS_1)),
        TargetPlan::NotTargeted,
        "CR 606.3: Sarkhan Vol's +1 declares no targets"
    );

    // With the only creature in a graveyard, the mandatory slot is unsatisfiable.
    let barren = sarkhan_board(ZoneId::Graveyard(P2));
    let barren_sarkhan = id_of(&barren, "Sarkhan Vol");
    assert_eq!(
        plan_targets(
            &barren,
            P1,
            &loyalty_action(barren_sarkhan, SARKHAN_MINUS_2)
        ),
        TargetPlan::Unsatisfiable,
        "CR 601.2c: no legal candidate makes the announcement impossible"
    );
}

// ── S5: SR-38 offer suppression, as a before/after pair on one board ──────────

/// CR 601.2c / SR-38 — S5. `StubProvider::legal_actions` does NOT offer a targeted
/// loyalty ability while its mandatory slot has no legal candidate, and DOES offer it
/// once one exists. Same Sarkhan Vol, same two players, same step, same object set —
/// the ONLY difference between the two readings is which zone the single creature sits
/// in, so the delta is attributable to the candidate and to nothing else.
///
/// The engine's own refusal of the suppressed offer is asserted too, so the
/// suppression is proven to be hiding something real rather than being a taste call.
///
/// **Revert to watch red**: delete the `if loyalty_ability_is_offerable(..)` guard in
/// `legal_actions.rs` and push the action unconditionally.
#[test]
fn test_dx29_s5_targeted_loyalty_offer_is_suppressed_without_a_candidate() {
    let before = sarkhan_board(ZoneId::Graveyard(P2));
    let after = sarkhan_board(ZoneId::Battlefield);

    let before_actions = StubProvider.legal_actions(&before, P1);
    let after_actions = StubProvider.legal_actions(&after, P1);

    let sarkhan_before = id_of(&before, "Sarkhan Vol");
    let sarkhan_after = id_of(&after, "Sarkhan Vol");
    let minus_2_before = loyalty_action(sarkhan_before, SARKHAN_MINUS_2);

    // The whole delta, stated as one equality per board rather than as four
    // independent membership checks: the untargeted `+1` is offered on BOTH boards
    // (the non-vacuity floor — `before` is not simply an empty list), the `-6` is
    // affordable on neither (4 < 6, CR 606.6), and the targeted `-2` appears only
    // once a candidate exists.
    assert_eq!(
        loyalty_offers(&before_actions, sarkhan_before),
        vec![SARKHAN_PLUS_1],
        "SR-38: with no legal candidate only the untargeted +1 may be offered; \
         got {before_actions:?}"
    );
    assert_eq!(
        loyalty_offers(&after_actions, sarkhan_after),
        vec![SARKHAN_PLUS_1, SARKHAN_MINUS_2],
        "SR-38: with a legal candidate the -2 joins the offer list, in declaration \
         order; got {after_actions:?}"
    );

    // The suppression hides something the engine really refuses (CR 601.2c) — the
    // offer was not merely unfashionable. The params are exactly what a bot would
    // have submitted for it: `plan_targets`' own announcement, which on this board is
    // `Unsatisfiable` and therefore empty.
    let params = ActionParams {
        targets: plan_targets(&before, P1, &minus_2_before).announced(),
        ..Default::default()
    };
    assert!(
        params.targets.is_empty(),
        "precondition: there is nothing legal to announce on this board"
    );
    let command = action_to_command_with_params(&before, P1, &minus_2_before, &params)
        .expect("building the command is not what fails");
    match mtg_engine::process_command(before, command) {
        Err(mtg_engine::GameStateError::InvalidTarget(msg)) => assert!(
            msg.contains("expected 1..=1 target(s) but got 0"),
            "CR 601.2c refusal should name the count range, got {msg:?}"
        ),
        other => panic!(
            "the suppressed offer must be one the engine refuses; got {:?}",
            other.map(|(_, ev)| ev.len())
        ),
    }
}

// ── S6: the suppression must not over-fire ────────────────────────────────────

/// CR 606.3 / SR-38 — S6. An UNtargeted loyalty ability is still offered on a board
/// with no candidates at all — indeed with no other objects at all. `loyalty_ability_
/// is_offerable` returns `true` on an empty requirement list before it ever asks
/// `legal_targets_per_slot` anything, and this pins that early return.
///
/// **Revert to watch red**: change `if requirements.is_empty() { return true; }` in
/// `legal_actions.rs::loyalty_ability_is_offerable` to `return false;`, or delete the
/// early return so an empty requirement list falls through the (vacuously true) loop —
/// the latter is a CONTROL and does NOT redden, which is the point of stating both.
#[test]
fn test_dx29_s6_untargeted_loyalty_ability_is_offered_with_no_candidates_at_all() {
    let defs = card_defs_by_name();
    let mut state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(build_registry())
        .active_player(P1)
        .at_step(Step::PreCombatMain)
        .object(sarkhan_spec(&defs))
        .build()
        .expect("PB-DX29 S6 fixture must build");
    state.turn_mut().priority_holder = Some(P1);

    // Non-vacuity: the board really has nothing but the planeswalker.
    assert_eq!(
        state.objects().len(),
        1,
        "precondition: Sarkhan Vol is the only object in the game"
    );

    let sarkhan = id_of(&state, "Sarkhan Vol");
    let actions = StubProvider.legal_actions(&state, P1);
    assert_eq!(
        loyalty_offers(&actions, sarkhan),
        vec![SARKHAN_PLUS_1],
        "SR-38: an untargeted loyalty ability must still be offered on a board with \
         no candidates at all, and the targeted -2 must stay suppressed; \
         got {actions:?}"
    );
}

// ── S7: end to end through LocalGame, with a NON-DEFAULT target ───────────────

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 5,
        max_commands: 1000,
        max_consecutive_passes: 500,
        record_journal: true,
    }
}

/// CR 606.3 / CR 601.2c / CR 400.7 — S7. A human seat drives a real `LocalGame`, is
/// offered Sarkhan Vol's `-2`, and answers it with the **second** candidate
/// `legal_targets_per_slot` enumerates rather than the first.
///
/// "Second" is provably non-default: `targeting::plan_targets` — the only automatic
/// chooser in the tree — takes `candidates.first()`, and the test asserts the chosen
/// candidate differs from it before submitting. Verification is BY NAME (CR 400.7).
///
/// `LocalGame::start` runs `start_game`, which resets the turn to `Step::Untap`, so the
/// main-phase window CR 606.3 requires is reached by passing priority rather than by
/// building into it.
///
/// **Revert to watch red**: restore `targets: Vec::new()` on `params.rs`'
/// `ActivateLoyaltyAbility` arm — `submit` then produces an untargeted command, the
/// engine refuses it with `InvalidTarget`, and `submit` returns `Rejected`.
#[test]
fn test_dx29_s7_human_seat_activates_a_loyalty_ability_with_a_non_default_target() {
    let state = two_candidate_board();
    let human_seats: BTreeSet<PlayerId> = [P1].into_iter().collect();
    let bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    let (mut game, _events) =
        LocalGame::start(state, 1, StubProvider, bots, human_seats, limits(), true)
            .expect("game should start");

    // Walk to a decision that offers the -2. `start_game` resets to Untap.
    let mut decision = None;
    for _ in 0..200 {
        let d = match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => d,
            other => panic!("expected AwaitingHuman, got {other:?}"),
        };
        if d.actions.iter().any(|a| {
            matches!(a, LegalAction::ActivateLoyaltyAbility { ability_index, .. }
                if *ability_index == SARKHAN_MINUS_2)
        }) {
            decision = Some(d);
            break;
        }
        let pass = d
            .actions
            .iter()
            .position(|a| matches!(a, LegalAction::PassPriority))
            .expect("PassPriority is always offered at a priority window");
        game.submit(
            d.seq,
            HumanChoice {
                action_index: pass,
                params: ActionParams::default(),
            },
        )
        .expect("passing priority is always legal");
    }
    let decision = decision
        .expect("SR-38: the human must be offered Sarkhan Vol's -2 within 200 priority windows");

    let action_index = decision
        .actions
        .iter()
        .position(|a| {
            matches!(a, LegalAction::ActivateLoyaltyAbility { ability_index, .. }
            if *ability_index == SARKHAN_MINUS_2)
        })
        .unwrap();

    let sarkhan = id_of(game.state(), "Sarkhan Vol");
    let candidates = minus_2_candidates(game.state(), sarkhan);
    assert!(
        candidates.len() >= 2,
        "non-vacuity: two candidates are needed for 'the second' to be non-default"
    );
    let chosen = candidates[1].clone();
    assert_ne!(
        chosen, candidates[0],
        "the chosen candidate must differ from the one a bot would take"
    );
    let chosen_name = match chosen {
        Target::Object(id) => name_of(game.state(), id),
        ref other => panic!("expected an object candidate, got {other:?}"),
    };
    let other_name = match candidates[0] {
        Target::Object(id) => name_of(game.state(), id),
        ref other => panic!("expected an object candidate, got {other:?}"),
    };

    game.submit(
        decision.seq,
        HumanChoice {
            action_index,
            params: ActionParams {
                targets: vec![chosen.clone()],
                ..Default::default()
            },
        },
    )
    .expect("CR 601.2c: a legal announced target must be accepted through LocalGame");

    // Resolve the ability: pass priority until the stack is empty.
    for _ in 0..20 {
        if game.state().stack_objects().is_empty() {
            break;
        }
        let d = match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => d,
            other => panic!("expected AwaitingHuman, got {other:?}"),
        };
        let pass = d
            .actions
            .iter()
            .position(|a| matches!(a, LegalAction::PassPriority))
            .expect("PassPriority is always offered");
        game.submit(
            d.seq,
            HumanChoice {
                action_index: pass,
                params: ActionParams::default(),
            },
        )
        .expect("passing priority is always legal");
    }
    assert!(
        game.state().stack_objects().is_empty(),
        "the loyalty ability should have resolved"
    );

    // CR 400.7: verify BY NAME.
    let chosen_after = object_by_name(game.state(), &chosen_name);
    let other_after = object_by_name(game.state(), &other_name);
    assert_eq!(
        chosen_after.controller, P1,
        "CR 613.1b: '{chosen_name}' -- the target the HUMAN announced -- must have \
         changed controller"
    );
    assert!(
        !chosen_after.status.tapped,
        "'{chosen_name}' must have been untapped"
    );
    assert_eq!(
        other_after.controller, P2,
        "'{other_name}' was not targeted and must not have changed controller"
    );
    assert!(
        other_after.status.tapped,
        "'{other_name}' was not targeted and must remain tapped"
    );
    assert_eq!(
        object_by_name(game.state(), "Sarkhan Vol")
            .counters
            .get(&CounterType::Loyalty)
            .copied()
            .unwrap_or(0),
        2,
        "CR 606.4: the -2 was paid"
    );
}
