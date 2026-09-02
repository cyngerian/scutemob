//! PB-DX47 — is the `WhenDealsCombatDamageToPlayer` double-push real?
//! (`OOS-DX24-4`, v4 queue rank 5.)
//!
//! **This file is the experiment, and it was committed before any fix.** The
//! seed is filed MEDIUM confidence and the v4 memo explicitly blesses the small
//! outcome: if a dedup already exists, the batch collapses to a comment fix.
//! So the probe runs first and decides.
//!
//! # The claim under test
//!
//! `rules/abilities.rs`'s `GameEvent::CombatDamageDealt` arm dispatches a
//! `WhenDealsCombatDamageToPlayer` trigger by **two independent, non-exclusive
//! paths**:
//!
//! * **(A) the runtime lowering.** `build_face_ability_vectors` (called by
//!   `enrich_spec_from_def`) converts the CardDef `AbilityDefinition::Triggered`
//!   into a runtime `TriggeredAbilityDef { trigger_on:
//!   TriggerEvent::SelfDealsCombatDamageToPlayer, .. }` on
//!   `characteristics.triggered_abilities`, which `collect_triggers_for_event`
//!   then finds and pushes as `PendingTriggerKind::Normal`.
//! * **(B) the card-registry scan.** The same arm then walks
//!   `def.effective_abilities(..)` out of `state.card_registry` and pushes a
//!   second `PendingTrigger`, this one `PendingTriggerKind::CardDefETB`.
//!
//! Neither suppresses the other. The in-source comment that justifies (B) says
//! the (A) lowering "only happens in `enrich_spec_from_def` for tests" — and
//! `enrich_spec_from_def` is the **production pregame path**
//! (`setup.rs:419/433/440`, `fuzz_setup.rs:119/130`). That is why this probe
//! builds through `setup::build_initial_state` and NOT through
//! `GameStateBuilder`: a hand-built fixture is exactly the shape the false
//! comment claims is special, so proving anything on one would prove nothing.
//!
//! # The subject
//!
//! `drana_liberator_of_malakir` — `Complete` by derive, deck-legal, and
//! **legendary**, so it starts in the command zone by construction (CR 903.6)
//! rather than depending on a shuffle putting it in an opening hand. Its trigger
//! is *"put a +1/+1 counter on each attacking creature you control"*, so a
//! double dispatch is visible as **two counters on a lone attacker**, not merely
//! as two stack entries. Flying + first strike is incidental but keeps the
//! attack clean.
//!
//! Both seats are human here (`human_seats = {p1, p2}`), so no bot RNG enters:
//! every decision in the game is made by `drive()` below.

use std::collections::{BTreeMap, BTreeSet};

use rand::rngs::StdRng;
use rand::SeedableRng;

use mtg_engine::rules::abilities::check_triggers;
use mtg_engine::{
    all_cards, card_name_to_id, AbilityDefinition, AttackTarget, CardId, CombatDamageAssignment,
    CombatDamageTarget, CounterType, GameEvent, GameState, ObjectId, PlayerId, TriggerCondition,
    TriggerEvent, ZoneId,
};
use mtg_simulator::deck::{random_deck, DeckConfig};
use mtg_simulator::legal_actions::{LegalAction, StubProvider};
use mtg_simulator::local_game::{AdvanceOutcome, LocalGame, LocalGameLimits};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::setup::{self, BotKind, DeckSource, LocalGameConfig};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SUBJECT: &str = "Drana, Liberator of Malakir";
const SEED: u64 = 47_47_47;

/// p1's deck: the subject as commander plus 99 Swamps.
///
/// Basic lands are exempt from CR 903.5b's singleton rule, so 99 copies is a
/// legal Commander deck and `validate_deck` (Architecture Invariant 9, run for
/// real inside `build_initial_state`) admits it. Mono-black keeps CR 903.4
/// colour identity satisfied.
fn subject_deck() -> DeckConfig {
    DeckConfig {
        commander: card_name_to_id(SUBJECT),
        main_deck: (0..99).map(|_| card_name_to_id("Swamp")).collect(),
    }
}

fn config(decks: DeckSource) -> LocalGameConfig {
    LocalGameConfig {
        player_count: 2,
        human_seats: [p(1), p(2)].into_iter().collect::<BTreeSet<_>>(),
        bot_kind: BotKind::Heuristic,
        seed: SEED,
        decks,
        limits: LocalGameLimits {
            max_turns: 12,
            max_commands: 4_000,
            max_consecutive_passes: 500,
            record_journal: true,
        },
    }
}

/// The production pregame path. `build_initial_state` is what `tools/play-server`
/// and `tools/tui` both build through.
fn pregame() -> GameState {
    let mut rng = StdRng::seed_from_u64(SEED);
    let filler = random_deck(&mut rng, &all_cards()).expect("a random deck for the passive seat");
    let decks = DeckSource::Fixed(vec![(p(1), subject_deck()), (p(2), filler)]);
    setup::build_initial_state(&config(decks))
        .expect("PB-DX47 pregame must build through the real validate_deck gate")
        .0
}

fn subject_object(state: &GameState) -> ObjectId {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == SUBJECT)
        .map(|o| o.id)
        .expect("the subject must exist in the pregame state")
}

// ─────────────────────────────────────────────────────────────────────────────
// P1 — both dispatch preconditions hold on a PRODUCTION-built object
// ─────────────────────────────────────────────────────────────────────────────

/// The two paths' preconditions, measured on the object `setup.rs` actually
/// built — not on one this test assembled.
///
/// This is the half that refutes the justifying comment directly: if the runtime
/// lowering really "only happens in `enrich_spec_from_def` for tests", the first
/// assertion here is 0.
#[test]
fn p1_production_pregame_satisfies_both_dispatch_preconditions() {
    let state = pregame();
    let id = subject_object(&state);
    let obj = state.objects().get(&id).expect("subject object");

    assert_eq!(
        obj.zone,
        ZoneId::Command(p(1)),
        "CR 903.6: the commander starts in the command zone, which is what makes \
         this fixture deterministic (no shuffle dependency)"
    );

    // Path (A): the runtime lowering, on a production-built object.
    let lowered = obj
        .characteristics
        .triggered_abilities
        .iter()
        .filter(|t| t.trigger_on == TriggerEvent::SelfDealsCombatDamageToPlayer)
        .count();

    // Path (B): the card-registry scan the CombatDamageDealt arm performs.
    let card_id: CardId = obj.card_id.clone().expect("a real card, not a token");
    let def = state
        .card_registry()
        .get(card_id)
        .expect("the registry must know the commander it validated");
    let in_registry = def
        .effective_abilities(false)
        .iter()
        .filter(|a| {
            matches!(
                a,
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                    ..
                }
            )
        })
        .count();

    println!(
        "PB-DX47 P1: subject={SUBJECT} lowered(A)={lowered} registry(B)={in_registry}"
    );

    assert_eq!(
        in_registry, 1,
        "path (B) precondition: the registry def carries exactly one \
         WhenDealsCombatDamageToPlayer ability"
    );
    assert_eq!(
        lowered, 1,
        "path (A) precondition on a PRODUCTION-built object. If this is 0, the \
         abilities.rs comment is right and the seed is refuted here."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// P2 — the behavioural probe: count the pushed PendingTriggers by kind
// ─────────────────────────────────────────────────────────────────────────────

/// The raw `PendingTrigger` census the engine's own dispatcher produces for one
/// `GameEvent::CombatDamageDealt`, grouped by `PendingTriggerKind`.
///
/// # Why this is measured through `check_triggers` and not at a command boundary
///
/// The first draft of this probe scanned `state.pending_triggers()` after every
/// `advance()` / `submit()` and measured **0 at every boundary** — not because
/// nothing was pushed, but because `check_and_flush_triggers` drains the queue
/// onto the stack inside the SAME `process_command` call, so the queue is never
/// non-empty at any point a test can look. That reading is recorded rather than
/// quietly dropped: a census that returns 0 because it never gets to look is
/// indistinguishable from one that returns 0 because nothing happened, and the
/// end-to-end counter assertion below is what caught it.
///
/// So the census calls the engine's own `check_triggers` — the exact function
/// `process_command` calls — on the REAL driven state, with the same event shape
/// `rules/combat.rs` emits.
#[derive(Debug, Default, Clone)]
struct Census {
    by_kind: BTreeMap<String, usize>,
}

impl Census {
    fn total(&self) -> usize {
        self.by_kind.values().sum()
    }
}

/// Ask the engine's dispatcher what it would push for `subject` connecting for
/// `amount` combat damage to `p2`.
fn census_for_combat_damage(state: &GameState, subject: ObjectId, amount: u32) -> Census {
    let event = GameEvent::CombatDamageDealt {
        assignments: vec![CombatDamageAssignment {
            source: subject,
            target: CombatDamageTarget::Player(p(2)),
            amount,
        }],
    };
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for t in check_triggers(state, std::slice::from_ref(&event)) {
        if t.triggering_event == Some(TriggerEvent::SelfDealsCombatDamageToPlayer)
            && t.source == subject
        {
            *by_kind.entry(format!("{:?}", t.kind)).or_default() += 1;
        }
    }
    Census { by_kind }
}

/// Pick the action this probe wants, or `PassPriority`.
///
/// Deliberately narrow: play a land, cast the commander, attack the opponent
/// with it, decline every block. Everything else passes.
fn choose(state: &GameState, actions: &[LegalAction], me: PlayerId) -> usize {
    let subject_on_battlefield = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == SUBJECT && o.zone == ZoneId::Battlefield)
        .map(|o| o.id);

    // 1. Cast the commander out of the command zone as soon as it is offered.
    if me == p(1) && subject_on_battlefield.is_none() {
        if let Some(i) = actions.iter().position(|a| {
            matches!(a, LegalAction::CastSpell { card, .. }
                if state.objects().get(card).map(|o| o.characteristics.name.as_str())
                    == Some(SUBJECT))
        }) {
            return i;
        }
    }
    // 2. Attack the opponent with it once it is out and able.
    if me == p(1) {
        if let Some(subject) = subject_on_battlefield {
            if let Some(i) = actions.iter().position(|a| {
                matches!(a, LegalAction::DeclareAttackers { eligible, .. }
                    if eligible.contains(&subject))
            }) {
                return i;
            }
        }
    }
    // 3. Land drops fund step 1.
    if me == p(1) {
        if let Some(i) = actions
            .iter()
            .position(|a| matches!(a, LegalAction::PlayLand { .. }))
        {
            return i;
        }
    }
    // 4. Keep every opening hand — this fixture must not re-deal.
    if let Some(i) = actions
        .iter()
        .position(|a| matches!(a, LegalAction::KeepHand))
    {
        return i;
    }
    actions
        .iter()
        .position(|a| matches!(a, LegalAction::PassPriority))
        .unwrap_or(0)
}

/// The params for the chosen action. Only the attack needs anything.
fn params_for(state: &GameState, action: &LegalAction) -> ActionParams {
    let mut params = ActionParams {
        auto_tap: true,
        ..Default::default()
    };
    if let LegalAction::DeclareAttackers { eligible, .. } = action {
        let subject = eligible
            .iter()
            .copied()
            .find(|id| {
                state.objects().get(id).map(|o| o.characteristics.name.as_str()) == Some(SUBJECT)
            })
            .expect("choose() only picks this action when the subject is eligible");
        params.attackers = vec![(subject, AttackTarget::Player(p(2)))];
    }
    params
}

/// Drive the game until the subject has connected for combat damage.
///
/// Returns (the by-kind census taken at the moment the subject was an attacker
/// on the battlefield, the `+1/+1` counters it ended up with, the game).
fn drive() -> (Census, u32, LocalGame<StubProvider>) {
    let state = pregame();
    let (mut game, _) = LocalGame::start(
        state,
        SEED,
        StubProvider,
        Default::default(),
        [p(1), p(2)].into_iter().collect::<BTreeSet<_>>(),
        config(DeckSource::RandomPerSeat).limits,
        false,
    )
    .expect("LocalGame::start");

    let mut census = Census::default();
    let mut counters_on_subject = 0u32;
    for _ in 0..4_000 {
        // Take the census the first time the subject is a live attacker: this is
        // the state `rules/combat.rs` is about to emit `CombatDamageDealt` from.
        if census.total() == 0 {
            if let Some(id) = battlefield_subject(game.state()) {
                if is_attacking(game.state(), id) {
                    census = census_for_combat_damage(game.state(), id, 2);
                }
            }
        }
        counters_on_subject = counters_on_subject.max(subject_counters(game.state()));
        if counters_on_subject > 0 && game.state().stack_objects().is_empty() {
            break;
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(pending) => {
                let idx = choose(game.state(), &pending.actions, pending.player);
                let action = pending.actions[idx].clone();
                let params = params_for(game.state(), &action);
                let seq = pending.seq;
                game.submit(
                    seq,
                    HumanChoice {
                        action_index: idx,
                        params,
                    },
                )
                .expect("PB-DX47 driver must never submit an illegal choice");
            }
            AdvanceOutcome::GameOver(_) | AdvanceOutcome::Halted(_) => break,
        }
    }
    (census, counters_on_subject, game)
}

fn battlefield_subject(state: &GameState) -> Option<ObjectId> {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == SUBJECT && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
}

fn is_attacking(state: &GameState, id: ObjectId) -> bool {
    state
        .combat()
        .as_ref()
        .map(|c| c.attackers.contains_key(&id))
        .unwrap_or(false)
}

fn subject_counters(state: &GameState) -> u32 {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == SUBJECT && o.zone == ZoneId::Battlefield)
        .map(|o| {
            o.counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0) as u32
        })
        .unwrap_or(0)
}

/// **The decisive probe.** It publishes the number either way; the assertions
/// below are what the FIRST commit measured, written so that a dedup appearing
/// later reddens this test rather than silently passing it.
#[test]
fn p2_combat_damage_pushes_two_triggers_one_per_dispatch_path() {
    let (census, counters, game) = drive();
    println!(
        "PB-DX47 P2: PendingTrigger census by kind = {:?} (total {}); \
         +1/+1 counters on the lone attacker = {counters}; commands = {commands}",
        census.by_kind,
        census.total(),
        commands = game.command_count()
    );
    assert!(
        counters > 0,
        "the probe must actually connect — the subject never dealt combat damage \
         to a player, so nothing was measured"
    );
    assert_eq!(
        census.by_kind,
        BTreeMap::from([("Normal".to_string(), 1), ("CardDefETB".to_string(), 1)]),
        "MEASURED, first commit, before any fix: ONE PendingTrigger per dispatch \
         path — `Normal` from the runtime lowering `collect_triggers_for_event` \
         reads, `CardDefETB` from the card-registry scan in the same \
         `GameEvent::CombatDamageDealt` arm. Neither suppresses the other."
    );
    assert_eq!(
        counters, 2,
        "MEASURED, first commit, before any fix: the double dispatch is not \
         academic — a card printing ONE +1/+1 counter put TWO on its lone \
         attacker, in a game built through the production pregame path."
    );
}
