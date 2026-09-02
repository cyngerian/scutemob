//! PB-DX48 (`OOS-ENG2-1` ≡ `OOS-ENG2-2`) — CR 702.21a Ward, through the REAL channels.
//!
//! The engine-side probes live in
//! `crates/engine/tests/primitives/pb_dx48_ward_dispatch.rs` and prove the engine
//! itself dispatches Ward at every `push_target_announcement` site. This file exists
//! because **existence is never sufficiency** (the `kaito_shizuki` lesson, PB-DX43):
//! a dispatch the engine performs but no client can ever REACH is not a repaired
//! decision. Every probe here drives a real `LocalGame`, `StubProvider`'s offer
//! layer, and `HumanChoice`/`ActionParams` — the same three surfaces the browser and
//! the bots go through.
//!
//! The Ward permanent is `Tyrranax Rex` (Ward {4}), one of the plan's three
//! deck-legal `Complete` Ward defs, built through `enrich_spec_from_def` +
//! `card_name_to_id` exactly as `mtg-plan-DX48.md` §2 verified — never a stand-in.
//! It is not legendary, so the legend rule is not a variable here.
//!
//! # `MayPayOrElse` is non-interactive at HEAD — do not try to pay it
//!
//! `ward.rs`'s own module doc records that `Effect::MayPayOrElse` always applies its
//! `or_else` branch at HEAD: the "pay {N} to stop the counter" half of CR 702.21a is
//! structurally unreachable from every channel (filed `OOS-DX48-2`, a fact handed
//! down for this batch rather than re-derived). So every probe below counters the
//! targeting ability unconditionally — that is the only reachable outcome, and it is
//! also the resolution effect the CR requires by default (the payment is optional in
//! the other direction: no payment offered, no payment made).
//!
//! # Assert by RESOLUTION EFFECT, never by the offer
//!
//! Following `pb_dx45_optional_cost_channel.rs`'s standard: the verdict in every
//! probe below is "the ward creature's `damage_marked` stayed at 0" (the targeting
//! ability's `DealDamage` effect never ran because Ward countered it before it could
//! resolve), corroborated — never substituted — by a `GameEvent::SpellCountered` in
//! the recorded event/journal stream.
//!
//! **This verdict is only load-bearing if the drive stops at the RIGHT moment.**
//! `damage_marked` is reset to 0 by CR 514.2's cleanup-step damage removal, so a
//! drive that runs all the way to the end of turn 1 makes `damage_marked == 0` true
//! for two completely different reasons — Ward countered the ability (the real
//! claim), or the ability resolved and dealt its damage and Cleanup then erased the
//! evidence (a false pass with the identical assertion). A coordinator-run revert
//! against an earlier draft of this file caught exactly this: under the revert, the
//! journal showed `DamageDealt` firing early in Upkeep and `damage_marked` was STILL
//! 0 at the point the old drive checked it, because that drive ran clear through
//! Cleanup first. Every probe below now stops the drive the moment the stack and
//! `pending_triggers` both settle back to empty — well before Draw, Main, Combat or
//! Cleanup ever run — and asserts that settlement explicitly (`stack_objects()` is
//! empty) as a precondition, so a drive that stopped merely because it gave up early
//! cannot masquerade as one that stopped because resolution finished.
//!
//! # Fixture shape: single candidate vs. two candidates
//!
//! CR 601.2c makes an announcement with exactly one legal answer DETERMINED — the
//! engine places it without ever asking (`forced_trigger_target_answer`, the same
//! shape `pb_dx48_ward_dispatch.rs`'s `t1`/`t6` use). c1 and c2 use a
//! single-candidate fixture (the Ward creature is the *only* creature an opponent of
//! the trigger's controller controls) precisely so that BOTH channels — a human who
//! only ever has to pass priority, and a bot who is never even asked a CR 603.3d
//! question — reach the identical Ward dispatch with no decision in between. c3
//! adds a second, non-Ward creature so the slot genuinely has two legal candidates,
//! which is what makes "the human chose it" and "the engine defaulted" two different
//! outcomes to discriminate between.

use std::collections::HashMap;

use mtg_engine::state::stubs::{PendingTrigger, PendingTriggerKind};
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, CardEffectTarget, Effect,
    EffectAmount, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Target,
    TargetController, TargetFilter, TargetRequirement, TriggerEvent, TriggeredAbilityDef, ZoneId,
};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, HeuristicBot, LegalAction, LegalActionProvider, LocalGame,
    LocalGameLimits, PendingDecision, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 48_48_48;
const WARD_NAME: &str = "Tyrranax Rex";
const PLAIN_NAME: &str = "DX48 Plain Bear";
const SOURCE_NAME: &str = "DX48 Channel Pinger";

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// A generous SAFETY BOUND, not the stopping point: every probe below stops its own
/// drive the moment the trigger chain settles (see `stop_at_settlement` below), well
/// inside turn 1, so `max_turns: 2` is slack for a probe that goes wrong, not a
/// budget any probe is expected to use.
fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 2,
        max_commands: 400,
        max_consecutive_passes: 300,
        record_journal: true,
    }
}

/// CR 702.21a's source: a `p1`-controlled, non-creature permanent (so it cannot
/// target itself) whose one triggered ability deals 1 damage to target creature an
/// opponent controls. `trigger_on` is irrelevant here -- every fixture below pushes
/// the trigger directly via `PendingTrigger::blank` (the same real dispatch/flush
/// channel `local_game.rs`'s own `state_with_pending_targeted_trigger` uses), which
/// bypasses event matching by construction.
fn source_spec() -> ObjectSpec {
    ObjectSpec::enchantment(p(1), SOURCE_NAME).with_triggered_ability(TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: "deals 1 damage to target creature an opponent controls".to_string(),
        effect: Some(Effect::DealDamage {
            source: None,
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            amount: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
            controller: TargetController::Opponent,
            ..Default::default()
        })],
    })
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

/// `extra_plain`: whether p2 also controls a second, ordinary creature. `false`
/// gives the CR 603.3d slot exactly one legal candidate (forced, no suspend) for
/// c1/c2; `true` gives it two (a real choice) for c3.
fn fixture(extra_plain: bool) -> GameState {
    let defs = card_defs_by_name();
    let ward = enrich_spec_from_def(
        ObjectSpec::card(p(2), WARD_NAME)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id(WARD_NAME)),
        &defs,
    );

    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(source_spec());
    if extra_plain {
        builder = builder.object(ObjectSpec::creature(p(2), PLAIN_NAME, 2, 2));
    }
    builder = builder.object(ward);

    let mut state = builder.build().expect("PB-DX48 channel fixture must build");

    let source_id = find_object(&state, SOURCE_NAME);
    state
        .pending_triggers_mut()
        .push_back(PendingTrigger::blank(
            source_id,
            p(1),
            PendingTriggerKind::Normal,
        ));
    state
}

fn ward_untouched(state: &GameState) -> bool {
    state
        .objects()
        .get(&find_object(state, WARD_NAME))
        .expect("ward creature must still be on the battlefield")
        .damage_marked
        == 0
}

/// Reads the FULL journal, not just the events a direct `submit()` call returned.
///
/// `LocalGame::submit`'s return value is only the events its own command produced;
/// `advance()` drives every BOT seat's own commands internally before handing
/// control back to a human decision, and those bot-driven events land in
/// `game.journal()` without ever passing through a `submit()` return value the human
/// side of the drive can see. c1/c2's pending trigger is answered by the ward
/// creature's own controller (p2, a bot in every probe here), so ward's counter is
/// exactly one of those internally-driven events -- checking only `submit()`
/// returns would have made this corroboration vacuous on every human-seat probe.
fn any_spell_countered_in_journal<P: LegalActionProvider>(game: &LocalGame<P>) -> bool {
    game.journal()
        .iter()
        .flat_map(|r| r.events.iter())
        .any(|e| matches!(e, GameEvent::SpellCountered { .. }))
}

/// Drive the human seat, passing priority, until `want` finds an action in the
/// offered list. Returns the decision, the index of that action, and every event
/// emitted along the way. **Panics rather than returning `None`** — a probe that
/// silently ends early is a probe that asserts nothing.
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    label: &str,
    want: impl Fn(&LegalAction) -> bool,
) -> (PendingDecision, usize, Vec<GameEvent>) {
    let mut collected = Vec::new();
    for _ in 0..80 {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                if let Some(i) = d.actions.iter().position(&want) {
                    return (d, i, collected);
                }
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!(
                            "no {label} offer and no PassPriority either: {:?}",
                            d.actions
                        )
                    });
                let ev = game
                    .submit(
                        d.seq,
                        HumanChoice {
                            action_index: pass,
                            params: ActionParams::default(),
                        },
                    )
                    .expect("passing priority should be accepted");
                collected.extend(ev);
            }
            other => panic!("expected AwaitingHuman while hunting {label}, got {other:?}"),
        }
    }
    panic!("no {label} offer within 80 human decisions");
}

/// `true` once the placed trigger chain has fully resolved: both the stack and
/// `pending_triggers` are empty. Checked BEFORE the chain is ever placed too (both
/// start empty in a fresh state that hasn't reached its flush point yet), which is
/// why every caller below tracks "has the stack ever gone non-empty" separately --
/// this predicate alone cannot distinguish "resolved" from "not started".
fn chain_is_settled(state: &GameState) -> bool {
    state.stack_objects().is_empty() && state.pending_triggers().is_empty()
}

/// Drive the human seat, passing priority, until the placed trigger chain settles
/// back to empty -- and STOP THERE, well before Draw/Main/Combat/Cleanup. Returns
/// every event emitted along the way.
///
/// **Why this stops here rather than draining to `Halted(MaxTurns)`:** CR 514.2's
/// cleanup-step damage removal resets `damage_marked` to 0 every turn, so a drive
/// that runs all the way to the end of turn 1 makes the eventual `damage_marked ==
/// 0` check pass whether Ward countered the ability OR the ability resolved and
/// Cleanup erased the evidence -- exactly the vacuous-verdict shape a coordinator
/// revert caught in an earlier draft (see the module doc). Stopping the instant the
/// chain settles means every caller's `damage_marked` check happens in a state
/// Cleanup has not touched yet, which is what makes it discriminate.
fn resolve_the_placed_trigger_chain(game: &mut LocalGame<StubProvider>) -> Vec<GameEvent> {
    let mut collected = Vec::new();
    let mut saw_stack_nonempty = false;
    for _ in 0..80 {
        if !game.state().stack_objects().is_empty() {
            saw_stack_nonempty = true;
        }
        if saw_stack_nonempty && chain_is_settled(game.state()) {
            return collected;
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                let idx = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!(
                            "no PassPriority while resolving the trigger chain: {:?}",
                            d.actions
                        )
                    });
                let ev = game
                    .submit(
                        d.seq,
                        HumanChoice {
                            action_index: idx,
                            params: ActionParams::default(),
                        },
                    )
                    .expect("passing priority should be accepted");
                collected.extend(ev);
            }
            other => panic!("unexpected outcome while resolving the trigger chain: {other:?}"),
        }
    }
    panic!("the placed trigger chain never settled within 80 iterations");
}

/// The bot-path equivalent of `resolve_the_placed_trigger_chain`, for a game with NO
/// human seat at all.
///
/// `LocalGame::advance()` is opaque for a fully bot-driven game: with no human seat
/// to stop on, one call drives everything internally until `Halted`/`GameOver`, with
/// no way to inspect state in between -- which is exactly the control this probe
/// needs (see the module doc: checking state after `Halted(MaxTurns)` would read a
/// post-Cleanup state and be vacuous). So this drives the SAME two real components
/// `LocalGame`'s own internal bot loop uses -- `StubProvider::legal_actions` (the
/// real offer layer) and `Bot::choose_action` (the real bot decision) -- through
/// `mtg_engine::process_command` (the real engine entry point) directly, stopping at
/// the same settlement condition the human-path helper above stops at. This is not a
/// synthetic shortcut: it is the identical mechanism, just driven by the test instead
/// of by `LocalGame`, which is the only way to get a stopping point out of it.
fn drive_bots_until_chain_resolves(
    mut state: GameState,
    mut bots: HashMap<PlayerId, Box<dyn Bot>>,
) -> (GameState, Vec<GameEvent>) {
    let mut collected = Vec::new();
    let mut saw_stack_nonempty = false;
    for _ in 0..80 {
        if !state.stack_objects().is_empty() {
            saw_stack_nonempty = true;
        }
        if saw_stack_nonempty && chain_is_settled(&state) {
            return (state, collected);
        }
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or(state.turn().active_player);
        let legal = StubProvider.legal_actions(&state, holder);
        let bot = bots
            .get_mut(&holder)
            .unwrap_or_else(|| panic!("no bot registered for seat {holder:?}"));
        let command = bot.choose_action(&state, holder, &legal);
        let (next_state, ev) = mtg_engine::process_command(state, command)
            .expect("a bot's own offered command must be accepted (SR-38)");
        state = next_state;
        collected.extend(ev);
    }
    panic!("the bot-driven trigger chain never settled within 80 iterations");
}

// ── c1: the human channel ─────────────────────────────────────────────────────────

/// **c1** — CR 702.21a / CR 601.2c, end to end through a real `LocalGame` human
/// seat. The single-candidate fixture makes the CR 603.3d slot DETERMINED
/// (`forced_trigger_target_answer`), so the human's only job here is to keep
/// passing priority through the whole turn -- the same shape a browser client with
/// nothing more to click would produce -- and the assertion is entirely about what
/// the engine did on its own initiative in response.
///
/// CR 702.21a: "Whenever this permanent becomes the target of a spell or ability an
/// opponent controls, counter that spell or ability unless that player pays {4}."
#[test]
fn c1_human_channel_ward_counters_the_targeting_ability_and_takes_no_damage() {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(2), Box::new(HeuristicBot::new(SEED, "p2".to_string())));
    let human: std::collections::BTreeSet<PlayerId> = [p(1)].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        fixture(false),
        SEED,
        StubProvider,
        bots,
        human,
        limits(),
        true,
    )
    .expect("PB-DX48 c1 game must start");

    resolve_the_placed_trigger_chain(&mut game);

    assert!(
        game.state().stack_objects().is_empty(),
        "precondition: the chain must have genuinely settled (not just given up \
         early) before the damage check below means anything"
    );
    assert!(
        ward_untouched(game.state()),
        "CR 702.21a: the ward creature must take NO damage -- its targeting \
         ability was countered before its DealDamage effect could resolve. \
         Checked immediately after the chain settles, BEFORE Cleanup would erase \
         the evidence either way -- see the module doc"
    );
    assert!(
        any_spell_countered_in_journal(&game),
        "CR 702.21a: ward's counter should be visible as a SpellCountered event \
         somewhere in the drive (corroboration, not the verdict)"
    );
}

// ── c2: the bot path ────────────────────────────────────────────────────────────

/// **c2** — the bot path: both seats bot-driven, no human anywhere. Because the
/// single-candidate fixture makes the CR 603.3d slot forced, `StubProvider` is never
/// even asked a trigger-target question on this path -- which is worth asserting
/// rather than assuming: a `LegalActionProvider` gap on a variant it was never
/// exercised against is exactly the class PB-DX45's `OOS-SIM6-3`-style findings
/// warn about, and `drive_bots_until_chain_resolves`'s own
/// `.unwrap_or_else(|| panic!("no bot registered for seat ..."))` / `.expect("a
/// bot's own offered command must be accepted (SR-38)")` are exactly what would
/// catch a provider that silently produced no legal actions, or one the engine then
/// refused, on this path.
#[test]
fn c2_bot_path_reaches_the_identical_ward_dispatch_with_no_human_seat() {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(1), Box::new(HeuristicBot::new(SEED, "p1".to_string())));
    bots.insert(p(2), Box::new(HeuristicBot::new(SEED, "p2".to_string())));

    let (state, events) = drive_bots_until_chain_resolves(fixture(false), bots);

    assert!(
        state.stack_objects().is_empty(),
        "precondition: the chain must have genuinely settled (not just given up \
         early) before the damage check below means anything"
    );
    let ward_id = find_object(&state, WARD_NAME);
    let ward_damage = state
        .objects()
        .get(&ward_id)
        .expect("ward creature must still be on the battlefield")
        .damage_marked;
    assert_eq!(
        ward_damage, 0,
        "CR 702.21a: the bot path must reach the identical outcome -- the ward \
         creature takes no damage, with StubProvider needing no change at all. \
         Checked immediately after the chain settles, BEFORE Cleanup would erase \
         the evidence either way -- see the module doc"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "CR 702.21a: ward's counter should be visible in the events the bot path \
         produced (corroboration, not the verdict)"
    );
}

// ── c3: a genuinely non-default answer ─────────────────────────────────────────

/// **c3** — a NON-DEFAULT answer, so the probe discriminates "the human chose it"
/// from "the engine defaulted". The two-candidate fixture puts `DX48 Plain Bear`
/// (built first, so it holds the lower `ObjectId` and is the engine's own
/// deterministic default per `TriggerTargetOption::default`) and `Tyrranax Rex`
/// (built second) both in the CR 603.3d slot. The human explicitly overrides the
/// offered default and names the ward creature -- proven a real override, not an
/// echo, by asserting the offered default is `Plain Bear` BEFORE the human's answer
/// is submitted.
#[test]
fn c3_human_overrides_the_default_target_and_ward_still_counters() {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(2), Box::new(HeuristicBot::new(SEED, "p2".to_string())));
    let human: std::collections::BTreeSet<PlayerId> = [p(1)].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        fixture(true),
        SEED,
        StubProvider,
        bots,
        human,
        limits(),
        true,
    )
    .expect("PB-DX48 c3 game must start");

    let plain_id = find_object(game.state(), PLAIN_NAME);
    let ward_id = find_object(game.state(), WARD_NAME);

    let (decision, idx, _) = drive_until(&mut game, "ChooseTriggerTargets", |a| {
        matches!(a, LegalAction::ChooseTriggerTargets { .. })
    });

    // The offer's own default must NOT already be the ward creature -- otherwise
    // this probe cannot distinguish a genuine override from an echo of the default.
    match &decision.actions[idx] {
        LegalAction::ChooseTriggerTargets { slots, targets, .. } => {
            assert_eq!(slots.len(), 1, "one TargetCreature slot");
            assert_eq!(
                targets,
                &vec![vec![Target::Object(plain_id)]],
                "precondition: the engine's own default must target Plain Bear, \
                 not the ward creature -- or choosing the ward below would not be \
                 a real override"
            );
        }
        other => panic!("wrong action: {other:?}"),
    }

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                trigger_targets: vec![vec![Target::Object(ward_id)]],
                ..ActionParams::default()
            },
        },
    )
    .expect("choosing the ward creature explicitly should be accepted");

    resolve_the_placed_trigger_chain(&mut game);

    assert!(
        game.state().stack_objects().is_empty(),
        "precondition: the chain must have genuinely settled (not just given up \
         early) before the damage checks below mean anything"
    );
    assert!(
        ward_untouched(game.state()),
        "CR 702.21a: the EXPLICITLY chosen ward creature must take no damage -- \
         Ward fired off the human's own non-default announcement. Checked \
         immediately after the chain settles, BEFORE Cleanup would erase the \
         evidence either way -- see the module doc"
    );
    let plain_damage = game
        .state()
        .objects()
        .get(&plain_id)
        .expect("plain bear must still be on the battlefield")
        .damage_marked;
    assert_eq!(
        plain_damage, 0,
        "the default target was never chosen, so it must show no sign of having \
         resolved the DealDamage effect either"
    );
    assert!(
        any_spell_countered_in_journal(&game),
        "CR 702.21a: ward's counter should be visible in the post-choice drive"
    );
}
