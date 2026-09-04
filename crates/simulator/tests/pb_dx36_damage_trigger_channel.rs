//! PB-DX36 (`OOS-CARDS2-6`) — the human channel: `Sigil of Sleep`'s NONcombat
//! damage trigger, offered and answered through `LocalGame`/`HumanChoice`, with
//! a genuine (non-default) target choice.
//!
//! Design record: `memory/primitives/pb-DX36-execution-notes.md` §0 (binding)
//! and `memory/primitives/pb-plan-DX36.md` step 8. This is deliberately the
//! ONLY file of the three that drives the full `LocalGame` decision loop — the
//! engine-level dispatch arithmetic is `primitives::pb_dx36_damage_trigger_dispatch`'s
//! job and the corpus census is `core::pb_dx36_deals_damage_roster`'s. This file
//! answers a narrower, load-bearing question: does the primitive reach a real
//! player through the SAME two decision points a browser or TUI client uses --
//! `DecisionKind::Priority` (to activate the ping, announcing its target via
//! `ActionParams::targets`) and `DecisionKind::TriggerTargets` (CR 603.3d, to
//! answer Sigil's own target, via `ActionParams::trigger_targets`)?
//!
//! `p1`, `p2` are BOTH human seats (`human_seats = {p1, p2}`), so no bot RNG
//! enters and every decision in the game is made by `drive()` below -- the same
//! shape PB-DX47's `pb_dx47_double_push_probe.rs` uses for its own "both seats
//! human, no bot RNG" guarantee.

use std::collections::BTreeSet;

use mtg_engine::{
    ActivatedAbility, ActivationCost, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Effect, EffectAmount, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step,
    Target, TargetRequirement, ZoneId,
};
use mtg_simulator::legal_actions::{LegalAction, LegalActionProvider, StubProvider};
use mtg_simulator::local_game::{AdvanceOutcome, LocalGame, LocalGameLimits};
use mtg_simulator::params::{ActionParams, HumanChoice};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Hand(player))
        .count()
}

fn corpus_def(name: &str) -> CardDefinition {
    mtg_engine::all_cards()
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("corpus def '{name}' not found by all_cards()"))
}

fn defs_of(def: &CardDefinition) -> std::collections::HashMap<String, CardDefinition> {
    let mut m = std::collections::HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

fn on_battlefield(player: PlayerId, name: &str, card_id: &str, def: &CardDefinition) -> ObjectSpec {
    mtg_engine::enrich_spec_from_def(ObjectSpec::card(player, name), &defs_of(def))
        .with_card_id(CardId(card_id.to_string()))
        .in_zone(ZoneId::Battlefield)
}

fn attach(state: &mut GameState, aura_name: &str, creature_name: &str) {
    let aura_id = find_object(state, aura_name);
    let creature_id = find_object(state, creature_name);
    if let Some(obj) = state.objects_mut().get_mut(&aura_id) {
        obj.attached_to = Some(creature_id);
    }
    if let Some(obj) = state.objects_mut().get_mut(&creature_id) {
        obj.attachments.push_back(aura_id);
    }
}

/// `{T}: this creature deals `amount` damage to target player.` -- same
/// engine-level synthetic ability `primitives::pb_dx36_damage_trigger_dispatch`
/// uses (duplicated here per this tree's own convention -- these two crates
/// cannot share a test helper module).
fn ping_ability(amount: i32) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            ..Default::default()
        },
        description: format!("{{T}}: This creature deals {amount} damage to target player."),
        effect: Some(Effect::DealDamage {
            source: None,
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            amount: EffectAmount::Fixed(amount),
        }),
        sorcery_speed: false,
        targets: vec![TargetRequirement::TargetPlayer],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        ..Default::default()
    }
}

/// `p1` controls `Pinger` (with the ping ability) enchanted by the REAL,
/// `Complete`, deck-legal `Sigil of Sleep`. `p2` controls TWO legal Sigil
/// targets (`Bear`, `Bear2`) so the CR 603.3d choice this test answers is
/// genuine, not forced (`trigger_target_slot_forced_answer` only skips
/// suspension -- and thus the human channel entirely -- for a single-candidate
/// slot).
fn build_state() -> GameState {
    let p1 = p(1);
    let p2 = p(2);
    let sigil = corpus_def("Sigil of Sleep");

    let pinger = ObjectSpec::creature(p1, "Pinger", 3, 3).with_activated_ability(ping_ability(2));
    let sigil_spec = on_battlefield(p1, "Sigil of Sleep", "sigil-of-sleep-channel", &sigil);
    let bear = ObjectSpec::creature(p2, "Bear", 2, 2).in_zone(ZoneId::Battlefield);
    let bear2 = ObjectSpec::creature(p2, "Bear2", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![sigil.clone()]))
        .object(pinger)
        .object(sigil_spec)
        .object(bear)
        .object(bear2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("PB-DX36 channel fixture must build");
    attach(&mut state, "Sigil of Sleep", "Pinger");
    state.turn_mut().priority_holder = Some(p1);
    state
}

/// Drives the game through `LocalGame`'s human decision loop until the stack is
/// empty and the ping has fired (or `limit` steps pass). Both seats are human;
/// `StubProvider` (the bot move generator) is present only to satisfy
/// `LocalGame<P: LegalActionProvider>`'s type parameter and is never consulted
/// (`bots` is empty).
///
/// Behaviour, at each `AdvanceOutcome::AwaitingHuman`:
/// * A `LegalAction::ActivateAbility` sourced from `Pinger`, offered exactly
///   once, is submitted with `ActionParams::targets = [Target::Player(p2)]` --
///   the human ANNOUNCING the ping's target, the same channel a browser client
///   uses (`ActionBar` -> `TargetPicker` -> `ActionParams.targets`).
/// * A `LegalAction::ChooseTriggerTargets` is submitted with a NON-DEFAULT
///   answer (`slots[0].candidates[1]`, the SECOND Bear) via
///   `ActionParams::trigger_targets` -- proving the channel carries real
///   information rather than merely accepting whatever the engine would have
///   auto-picked (PB-DX35's `t3b`/`OOS-DX45-...` precedent: a probe that only
///   ever submits the default proves nothing about the channel itself).
/// * Anything else (both seats' ordinary `PassPriority` windows) passes.
fn drive() -> LocalGame<StubProvider> {
    let state = build_state();
    let (mut game, _) = LocalGame::start(
        state,
        36_36_36,
        StubProvider,
        Default::default(),
        [p(1), p(2)].into_iter().collect::<BTreeSet<_>>(),
        LocalGameLimits {
            max_turns: 3,
            max_commands: 500,
            max_consecutive_passes: 50,
            record_journal: true,
        },
        true,
    )
    .expect("LocalGame::start must succeed on a Complete-only fixture");

    // No early "stack is empty, we're done" exit: the stack goes briefly empty
    // the instant the ping itself resolves, BEFORE Sigil's CR 603.3d target
    // choice has even been asked (the trigger sits suspended in
    // `pending_trigger_targets`, not on the stack, until it is answered) --
    // breaking on that would skip the very decision this test exists to drive.
    // Bounded by iteration count and `GameOver`/`Halted` only.
    let mut ping_activated = false;
    for _ in 0..40 {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(pending) => {
                let seq = pending.seq;
                let player = pending.player;

                if !ping_activated {
                    if let Some(idx) = pending.actions.iter().position(|a| {
                        matches!(a, LegalAction::ActivateAbility { source, .. }
                            if *source == find_object(game.state(), "Pinger"))
                    }) {
                        let params = ActionParams {
                            targets: vec![Target::Player(p(2))],
                            ..Default::default()
                        };
                        game.submit(
                            seq,
                            HumanChoice {
                                action_index: idx,
                                params,
                            },
                        )
                        .expect("PB-DX36 channel: ActivateAbility (ping) must be accepted");
                        ping_activated = true;
                        continue;
                    }
                }

                if let Some(idx) = pending
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::ChooseTriggerTargets { .. }))
                {
                    let LegalAction::ChooseTriggerTargets { slots, .. } = &pending.actions[idx]
                    else {
                        unreachable!("position() just matched this arm");
                    };
                    assert_eq!(slots.len(), 1, "Sigil of Sleep has exactly one target slot");
                    assert_eq!(
                        slots[0].candidates.len(),
                        2,
                        "PB-DX36 channel: a genuine two-candidate choice, not a forced single one"
                    );
                    // Deliberately NOT the default (candidates[0]) -- the second
                    // Bear, to prove this channel carries real information.
                    let non_default_target = slots[0].candidates[1].target.clone();
                    let params = ActionParams {
                        trigger_targets: vec![vec![non_default_target]],
                        ..Default::default()
                    };
                    game.submit(
                        seq,
                        HumanChoice {
                            action_index: idx,
                            params,
                        },
                    )
                    .expect("PB-DX36 channel: ChooseTriggerTargets must be accepted");
                    continue;
                }

                let idx = pending
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!(
                            "PB-DX36 channel: player {player:?} has no PassPriority and no \
                             action this driver understands: {:?}",
                            pending.actions
                        )
                    });
                game.submit(
                    seq,
                    HumanChoice {
                        action_index: idx,
                        params: ActionParams::default(),
                    },
                )
                .expect("PB-DX36 channel: PassPriority must be accepted");
            }
            AdvanceOutcome::GameOver(_) | AdvanceOutcome::Halted(_) => break,
        }
    }
    assert!(
        ping_activated,
        "the driver never found the ping's ActivateAbility offer"
    );
    game
}

/// CR 510.3a / CR 603.2c / CR 603.3d: the noncombat ping's `GameEvent::DamageDealt`
/// dispatches Sigil of Sleep's trigger, the human channel offers
/// `DecisionKind::TriggerTargets` with the real two-candidate slot, and
/// answering it with the NON-default choice moves the SECOND Bear (not the
/// first) to p2's hand -- proof the channel's answer, not the engine's own
/// default, decided the outcome.
#[test]
fn t1_sigil_of_sleep_noncombat_trigger_offered_and_answered_through_the_human_channel() {
    let p2 = p(2);
    let game = drive();
    let state = game.state();

    assert!(
        state.stack_objects().is_empty(),
        "everything must have resolved"
    );
    assert_eq!(
        hand_count(state, p2),
        1,
        "Sigil of Sleep's effect must have returned a creature to p2's hand"
    );
    // The FIRST Bear (candidates[0], the engine's own default) must still be on
    // the battlefield -- only the SECOND (the human's actual, non-default
    // choice) left.
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Bear" && o.zone == ZoneId::Battlefield),
        "the FIRST Bear (the un-chosen candidate) must remain on the battlefield"
    );
    assert!(
        !state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Bear2" && o.zone == ZoneId::Battlefield),
        "the SECOND Bear (the human's non-default choice) must have left the battlefield"
    );
}

/// Non-vacuity / mechanism check: `StubProvider` (a `LegalActionProvider`) is
/// present only to satisfy `LocalGame`'s type parameter with an empty `bots`
/// map -- confirm it implements the trait so a future refactor cannot silently
/// change what "no bots" means here.
#[test]
fn t2_stub_provider_is_a_legal_action_provider() {
    fn assert_provider<P: LegalActionProvider>() {}
    assert_provider::<StubProvider>();
}
