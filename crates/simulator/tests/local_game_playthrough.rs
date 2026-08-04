//! Scripted-human playthrough (M11-local **Session 8**, plan item 1).
//!
//! A deterministic policy — prefer a land drop, then the cheapest castable spell,
//! then attack with everything eligible, otherwise pass — drives seat 1 through a
//! full four-player Commander game **through `LocalGame` alone**. No HTTP, no
//! `oneshot`, no server: this is the acceptance test for "the core can actually be
//! played to a conclusion", separate from the play server that wraps it.
//!
//! What it asserts, per the plan:
//!
//! * no `LocalGameError::Engine` (and no `Rejected` from a policy that only ever
//!   submits an action the game just offered it);
//! * **zero** `InvariantViolation`s (`invariants::check_all`, running on every
//!   tracked command because `check_invariants` is on) — with exactly one class
//!   separated out and reported rather than asserted: see `OOS-M11-7` on
//!   [`Playthrough::transient_token_violations`], where what is asserted instead is
//!   the strictly stronger end-state property that no token leaked at all;
//! * the game reaches `GameOver` or the configured turn cap — never
//!   `Halted(NoLegalActions)`, never `Halted(EngineError)`.
//!
//! Run for five fixed seeds.
//!
//! # Two deliberate departures from the surrounding test file
//!
//! 1. **The real pregame path.** `crates/simulator/tests/local_game.rs` builds a
//!    99-Plains fixed deck to keep resolution shallow. This file uses
//!    `setup::build_initial_state` with `DeckSource::RandomPerSeat` — the actual
//!    1,804-def pool, admitted through the real `validate_deck` — because a
//!    playthrough over a deck that cannot do anything proves nothing about
//!    playability. That is also why it runs on a hand-built 64 MiB thread: deep
//!    resolution chains in the full pool have been observed to exhaust the default
//!    2 MiB test stack, a pre-existing engine characteristic (OOS-DP3-9 /
//!    OOS-M11-3), unrelated to anything this session changed.
//! 2. **A turn cap well below a natural game length.** Reaching an actual CR 104.2a
//!    conclusion from a random 100-card pile takes hundreds of turns and minutes of
//!    wall clock per seed. The plan's acceptance is "`GameOver` **or** the turn
//!    cap", so the cap is a legitimate terminal state and the test says which one it
//!    got rather than requiring a win.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::PlayerId;
use mtg_simulator::{
    setup, ActionParams, AdvanceOutcome, Bot, BotKind, DeckSource, HeuristicBot, HumanChoice,
    LegalAction, LocalGame, LocalGameConfig, LocalGameError, LocalGameLimits, PendingDecision,
    RandomBot, StubProvider,
};

/// The five fixed seeds the plan asks for.
const SEEDS: [u64; 5] = [1, 7, 42, 1234, 9001];

/// Turn cap — see the module doc. Low enough that five seeds finish in a normal
/// test run, high enough to get well past the opening (lands drop, creatures
/// resolve, combat happens; the observed games reach a populated battlefield).
const MAX_TURNS: u32 = 25;

/// 64 MiB. See the module doc's departure 1.
const STACK_SIZE: usize = 64 * 1024 * 1024;

const HUMAN: PlayerId = PlayerId(1);

/// What one seed's playthrough produced. Returned rather than asserted inside the
/// worker thread so a failure surfaces as a readable assertion on the test thread
/// instead of a panic inside a `JoinHandle`.
#[derive(Debug)]
struct Playthrough {
    seed: u64,
    /// `Some` iff the policy ever hit an error path. Any value here fails the test.
    error: Option<String>,
    /// Every violation except the known-transient CR 704.3 token class below. Any
    /// entry here fails the test.
    violations: Vec<String>,
    /// `no_orphaned_tokens` violations, kept separate because this engine checks
    /// SBAs on **step entry and at resolution**, not on every priority grant as CR
    /// 704.3 requires — so a Treasure sacrificed to pay a mana cost sits in the
    /// graveyard, legally under this engine's model, until the next of those.
    /// Pre-existing and out of M11-local's scope (no engine change this milestone);
    /// filed as **OOS-M11-7**. Reported, not asserted on — what *is* asserted is
    /// [`Self::leaked_tokens`], which proves every one was transient.
    transient_token_violations: Vec<String>,
    /// Tokens outside the battlefield in the **final** state. Must be empty: a token
    /// still there once the game has stopped survived every SBA check the engine
    /// ever ran, which is the real leak the `no_orphaned_tokens` check is for.
    leaked_tokens: Vec<String>,
    /// How the game ended: `"GameOver"` or `"MaxTurns"`. `"Halted:<reason>"` fails.
    outcome: String,
    turns: u32,
    commands: u32,
    /// How many decisions seat 1 was handed.
    decisions: u32,
    /// Distinct `LegalAction` kinds the human policy actually submitted, so the test
    /// can show the game was *played* and not merely passed through.
    submitted_kinds: BTreeSet<&'static str>,
}

/// Stable tag for a `LegalAction`, for the coverage set above.
fn kind_of(action: &LegalAction) -> &'static str {
    match action {
        LegalAction::PassPriority => "PassPriority",
        LegalAction::Concede => "Concede",
        LegalAction::PlayLand { .. } => "PlayLand",
        LegalAction::CastSpell { .. } => "CastSpell",
        LegalAction::TapForMana { .. } => "TapForMana",
        LegalAction::ActivateAbility { .. } => "ActivateAbility",
        LegalAction::DeclareAttackers { .. } => "DeclareAttackers",
        LegalAction::DeclareBlockers { .. } => "DeclareBlockers",
        LegalAction::OrderBlockers { .. } => "OrderBlockers",
        LegalAction::TakeMulligan => "TakeMulligan",
        LegalAction::KeepHand => "KeepHand",
        LegalAction::ReturnCommanderToCommandZone { .. } => "ReturnCommanderToCommandZone",
        LegalAction::LeaveCommanderInZone { .. } => "LeaveCommanderInZone",
        LegalAction::ActivateBloodrush { .. } => "ActivateBloodrush",
        LegalAction::SaddleMount { .. } => "SaddleMount",
        LegalAction::CastWithMutate { .. } => "CastWithMutate",
        LegalAction::TurnFaceUp { .. } => "TurnFaceUp",
        LegalAction::ActivateLoyaltyAbility { .. } => "ActivateLoyaltyAbility",
        LegalAction::CastMorphFaceDown { .. } => "CastMorphFaceDown",
        LegalAction::PayEcho { .. } => "PayEcho",
        LegalAction::PayCumulativeUpkeep { .. } => "PayCumulativeUpkeep",
        LegalAction::PayRecover { .. } => "PayRecover",
        LegalAction::DiscardToHandSize { .. } => "DiscardToHandSize",
        LegalAction::ChooseTriggerTargets { .. } => "ChooseTriggerTargets",
        LegalAction::AnswerEffectChoice { .. } => "AnswerEffectChoice",
    }
}

/// Per-combat memory for [`choose`] — the scripted policy's half of the `OOS-M11-9`
/// mitigation (SIM-1).
///
/// # Why a stateless policy is not enough any more
///
/// `HeuristicBot` has carried a per-combat cap of **1** on `RepeatKey::DeclareAttackers`
/// since M11-local S8, and that cap's own doc records both the defect and the decision
/// about where to mitigate it: neither `StubProvider` nor `combat.rs::handle_declare_attackers`
/// gates *"attackers have already been declared this combat"*, so with a **vigilant**
/// attacker — which stays untapped and therefore stays `eligible` — `DeclareAttackers` is
/// offered and accepted without limit (CR 508.1 makes it a turn-based action performed
/// **once**; the engine accepting a second is `OOS-M11-9`, and fixing it is an engine
/// change). S8 put the mitigation in the *client* explicitly "rather than in `StubProvider`
/// … keeps the provider's action list, and therefore every recorded `mtg-fuzzer` seed,
/// untouched."
///
/// This policy is the **second client** to need it, and it needed it the moment SIM-1
/// landed. Before SIM-1 the human's commander could never be cast, so a commander never
/// reached the battlefield here; seed 1's human commander is `Samut, Voice of Dissent`,
/// which has **Vigilance**. Observed on this branch before the cap: seed 1 halted
/// `InfiniteLoop` at turn 17 having applied exactly 20,000 commands, of which **19,351
/// were `DeclareAttackers` submitted in that single turn** (measured, not reasoned to).
/// That is the same seed, the same turn range and the same 20,000 commands the S8 bot-side
/// instance produced — recorded in `docs/audits/decision-point-audit.md` §8.1's
/// `OOS-M11-9` row.
///
/// **This is a policy cap, not a relaxed assertion.** Every assertion in the test below is
/// unchanged: `error == None`, no violations, no leaked tokens, outcome ∈ {GameOver,
/// MaxTurns}, and the `PlayLand`/`DeclareAttackers` coverage set. The cap only stops the
/// policy from *preferring* an action CR 508.1 says is not a real second play.
#[derive(Default)]
struct PolicyState {
    /// Whether a `CombatState` existed the last time the policy acted. Reset of the
    /// per-combat tally keys on the `false → true` edge, exactly as
    /// `HeuristicBot::refresh_repeat_scope` does — `turn_actions.rs` clears
    /// `state.combat` at end of combat and installs a fresh one at `BeginningOfCombat`,
    /// so the edge is a reliable "a new combat phase has begun" signal. Keying on the
    /// **turn number** instead would silently disable attacks in every CR 506.5 extra
    /// combat, which is precisely the regression `MR-M11-09` found in the bot.
    in_combat: bool,
    /// How many times this policy has already declared attackers in the current combat.
    declared_attackers_this_combat: u32,
}

impl PolicyState {
    /// Mirrors `HeuristicBot::refresh_repeat_scope`'s combat-entry edge detection.
    fn refresh_scope(&mut self, state: &mtg_engine::GameState) {
        let in_combat = state.combat().is_some();
        if in_combat && !self.in_combat {
            self.declared_attackers_this_combat = 0;
        }
        self.in_combat = in_combat;
    }
}

/// The scripted human policy: **prefer a land → prefer the cheapest castable spell
/// → attack when able → otherwise pass**, exactly as plan item 1 words it.
///
/// Returns the chosen `(action_index, ActionParams)`.
///
/// # What it deliberately never picks
///
/// * **`Concede`** (CR 104.3a). It is offered to every human decision as of S8, is
///   always accepted by the engine, and would end the game on turn 1 — the single
///   fastest way to make this test pass while proving nothing.
/// * **`OrderBlockers`** (CR 509.2). Excluded to keep the policy a pure priority
///   policy; the action is covered directly by
///   `test_s8_order_blockers_is_offered_to_a_human_attacker` rather than incidentally
///   here. (It would be safe to take — `local_game::order_blocker_actions` stops
///   offering an attacker once its order is set — but "safe" is not "tested".)
///
/// # The blocking-decision fallthrough
///
/// While a `BlockingDecision` is outstanding the provider offers exactly one
/// answer (`DiscardToHandSize` / `ChooseTriggerTargets` / `AnswerEffectChoice`),
/// plus S8's human-only `Concede`. None of those is a land, a spell or an attack,
/// and none is `PassPriority`, so the final fallback below — *the first action that
/// is not `Concede`* — is what answers them. That ordering is the reason the
/// fallback is "first non-Concede" and not "PassPriority or bust": passing is not
/// legal there and the engine's admission gate would refuse it, turning a
/// recoverable state into `Halted(EngineError)` (OOS-DP7-12).
fn choose(
    state: &mtg_engine::GameState,
    decision: &PendingDecision,
    policy: &mut PolicyState,
) -> (usize, ActionParams) {
    let actions = &decision.actions;
    policy.refresh_scope(state);

    // 1. A land drop is always the best available play in this policy.
    if let Some(i) = actions
        .iter()
        .position(|a| matches!(a, LegalAction::PlayLand { .. }))
    {
        return (i, ActionParams::default());
    }

    // 2. The cheapest castable spell that needs no announcement. A spell with a
    //    target requirement is skipped: `ActionParams::default()` announces none,
    //    and `casting.rs` correctly refuses that under CR 601.2c — submitting it
    //    would be the policy generating its own `Rejected`, which is precisely what
    //    this test exists to show does not happen. (Targeted casting by a human is
    //    already covered end to end by
    //    `local_game.rs::test_human_casts_targeted_spell_through_local_game`.)
    let cheapest =
        actions
            .iter()
            .enumerate()
            .filter_map(|(i, a)| {
                let LegalAction::CastSpell { card, .. } = a else {
                    return None;
                };
                let obj = state.object(*card).ok()?;
                let (min, _) = mtg_engine::target_count_range(
                    &mtg_engine::spell_target_requirements(state, *card, &[], None),
                );
                if min > 0 {
                    return None;
                }
                let mv = obj
                    .characteristics
                    .mana_cost
                    .as_ref()
                    .map(|c| c.mana_value())
                    .unwrap_or(0);
                Some((mv, i))
            })
            .min();
    if let Some((_, i)) = cheapest {
        // `auto_tap: true`, which is what a real client sends
        // (`ActionParamsDto`'s `default_auto_tap`) and NOT what
        // `ActionParams::default()` gives (`false`). `StubProvider` offers a
        // `CastSpell` whose cost it can see a way to pay from *untapped sources*,
        // not from the pool, so submitting one with `auto_tap: false` is an
        // immediate `Rejected("player does not have enough mana to pay the cost")`.
        // Found by running this test, not by reading the code.
        return (
            i,
            ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        );
    }

    // 3. Attack with everything eligible, each at the first offered attack target
    //    (CR 508.1a). An empty `eligible` list still yields an empty declaration,
    //    which is legal and is how a combat with no creatures proceeds.
    //
    //    CR 508.1 / `OOS-M11-9` (SIM-1): **once per combat phase**. See
    //    [`PolicyState`] — past the first declaration the policy falls through to
    //    `PassPriority` below rather than re-declaring, exactly as `HeuristicBot`'s
    //    `RepeatKey::DeclareAttackers` cap of 1 makes the bot seats do.
    if policy.declared_attackers_this_combat == 0 {
        if let Some((i, LegalAction::DeclareAttackers { eligible, targets })) = actions
            .iter()
            .enumerate()
            .find(|(_, a)| matches!(a, LegalAction::DeclareAttackers { .. }))
        {
            let params = ActionParams {
                attackers: match targets.first() {
                    Some(t) => eligible.iter().map(|e| (*e, t.clone())).collect(),
                    None => Vec::new(),
                },
                ..ActionParams::default()
            };
            policy.declared_attackers_this_combat += 1;
            return (i, params);
        }
    }

    // 4. Pass.
    if let Some(i) = actions
        .iter()
        .position(|a| matches!(a, LegalAction::PassPriority))
    {
        return (i, ActionParams::default());
    }

    // 5. Whatever is left that is not a concession — see the doc comment.
    let i = actions
        .iter()
        .position(|a| !matches!(a, LegalAction::Concede))
        .unwrap_or(0);
    (i, ActionParams::default())
}

/// Bots for every seat but `HUMAN`, seeded the same way `session.rs::bots_for` and
/// `mtg-fuzzer` seed theirs.
fn bots_for(cfg: &LocalGameConfig) -> HashMap<PlayerId, Box<dyn Bot>> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for i in 1..=u64::from(cfg.player_count) {
        let pid = PlayerId(i);
        if cfg.human_seats.contains(&pid) {
            continue;
        }
        let bot_seed = cfg.seed.wrapping_add(100 + i);
        let name = format!("Bot-{i}");
        let bot: Box<dyn Bot> = match cfg.bot_kind {
            BotKind::Heuristic => Box::new(HeuristicBot::new(bot_seed, name)),
            BotKind::Random => Box::new(RandomBot::new(bot_seed, name)),
        };
        bots.insert(pid, bot);
    }
    bots
}

fn config(seed: u64) -> LocalGameConfig {
    LocalGameConfig {
        player_count: 4,
        human_seats: [HUMAN].into_iter().collect(),
        bot_kind: BotKind::Heuristic,
        seed,
        decks: DeckSource::RandomPerSeat,
        limits: LocalGameLimits {
            max_turns: MAX_TURNS,
            // Deliberately far above `GameDriver`'s `max_turns * 200` ratio. That
            // ratio is the fuzzer's, and the fuzzer's games start with empty hands;
            // a real four-player table dealt from the full pool runs ~260 commands
            // per turn once boards develop, so at 200 the *command* valve fires
            // first and the turn cap — the terminal state the plan actually names —
            // becomes unreachable. Measured, not guessed: at `* 200` seed 1 halted
            // on `InfiniteLoop` at turn 19 having applied exactly 5,000 commands.
            // The valve is still live at 4× headroom; it is just no longer the
            // binding constraint.
            max_commands: MAX_TURNS * 800,
            max_consecutive_passes: 500,
            // Off: nothing here reads the journal, and a 5,000-command game would
            // retain a cloned `Vec<GameEvent>` per command for no reason.
            // Off: nothing here reads the journal, and a 1,000-command game would
            // retain a cloned `Vec<GameEvent>` per command for no reason.
            record_journal: false,
        },
    }
}

/// Play one seed to a conclusion. Never panics on a game outcome — every failure
/// mode is reported through [`Playthrough`] so the assertions live on the test
/// thread.
fn play(seed: u64) -> Playthrough {
    let cfg = config(seed);
    let mut result = Playthrough {
        seed,
        error: None,
        violations: Vec::new(),
        transient_token_violations: Vec::new(),
        leaked_tokens: Vec::new(),
        outcome: String::new(),
        turns: 0,
        commands: 0,
        decisions: 0,
        submitted_kinds: BTreeSet::new(),
    };

    let (state, _names) = match setup::build_initial_state(&cfg) {
        Ok(v) => v,
        Err(e) => {
            result.error = Some(format!("setup failed: {e}"));
            return result;
        }
    };
    let (mut game, _start_events) = match LocalGame::start(
        state,
        cfg.seed,
        StubProvider,
        bots_for(&cfg),
        cfg.human_seats.clone(),
        cfg.limits,
        // The whole point: `invariants::check_all` on every tracked command.
        true,
    ) {
        Ok(v) => v,
        Err(e) => {
            result.error = Some(format!("start failed: {e:?}"));
            return result;
        }
    };

    // CR 508.1 / `OOS-M11-9` — see [`PolicyState`]. One per playthrough, so the
    // per-combat tally is scoped to this seed's game and nothing leaks between seeds.
    let mut policy = PolicyState::default();

    loop {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(decision) => {
                result.decisions += 1;
                let (index, params) = choose(game.state(), &decision, &mut policy);
                result
                    .submitted_kinds
                    .insert(kind_of(&decision.actions[index]));
                if let Err(e) = game.submit(
                    decision.seq,
                    HumanChoice {
                        action_index: index,
                        params,
                    },
                ) {
                    // Any error at all is a failure — `Rejected` included. The
                    // policy only ever submits an action the game offered it one
                    // instant earlier, so a rejection means the offer was wrong.
                    result.error = Some(match e {
                        LocalGameError::Rejected(inner) => format!(
                            "engine rejected a just-offered action ({}): {inner}",
                            kind_of(&decision.actions[index])
                        ),
                        other => format!("{other:?}"),
                    });
                    break;
                }
            }
            AdvanceOutcome::GameOver(game_result) => {
                result.outcome = "GameOver".to_string();
                result.turns = game_result.turn_count;
                result.commands = game_result.total_commands as u32;
                break;
            }
            AdvanceOutcome::Halted(reason) => {
                result.outcome = match &reason {
                    mtg_simulator::HaltReason::MaxTurns { .. } => "MaxTurns".to_string(),
                    other => format!("Halted:{other:?}"),
                };
                result.turns = game.state().turn().turn_number;
                result.commands = game.command_count();
                break;
            }
        }
    }

    // Split the violations by check name — see `Playthrough`'s field docs and the
    // OOS-M11-7 note on the test below.
    //
    // PB-DX32 fix cycle (review finding M1): the `no_orphaned_tokens` split now
    // happens upstream, inside `LocalGame::record_violations` (PB-DX32 Stage 4) —
    // `game.violations()` can no longer contain that check at all, so the old
    // `if v.check == "no_orphaned_tokens"` branch here was permanently dead and
    // `transient_token_violations` printed `0` on every seed forever. Read each
    // half from the game's own two accessors instead of re-deriving the split.
    for v in game.violations() {
        result.violations.push(format!("{v:?}"));
    }
    for v in game.transient_violations() {
        result.transient_token_violations.push(format!("{v:?}"));
    }
    // The proof that every one of those was transient: at the end of the game no
    // token is anywhere but the battlefield.
    result.leaked_tokens = game
        .state()
        .objects()
        .iter()
        .filter(|(_, o)| o.is_token && o.zone != mtg_engine::ZoneId::Battlefield)
        .map(|(id, o)| format!("{:?} {:?} in {:?}", id, o.characteristics.name, o.zone))
        .collect();
    if result.turns == 0 {
        result.turns = game.state().turn().turn_number;
    }
    if result.commands == 0 {
        result.commands = game.command_count();
    }
    result
}

/// Plan item 1 / acceptance criterion 5974.
#[test]
fn test_s8_scripted_human_playthrough_is_clean_on_five_seeds() {
    let handle = std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(|| SEEDS.iter().map(|&seed| play(seed)).collect::<Vec<_>>())
        .expect("worker thread should spawn");
    let runs = handle.join().expect("playthrough thread must not panic");

    // UI-2 (2026-08-02): the LAST entry ever carried here, playtest finding **F9**
    // (`spell_additional_costs` invisible to the provider, CR 118.8), stopped
    // firing the moment `legal_actions.rs` learned to build a real
    // `AdditionalCostPlan` and suppress the offer when no sacrifice is eligible —
    // exactly the shape the two entries deleted before it (OOS-M11-9,
    // OOS-CARDS2-9) took. An excusal list is a debt register with a maturity
    // date, not a permanent fixture: the register is now EMPTY, and the whole
    // excusal mechanism is deleted along with it rather than kept around for a
    // future entry that has not been filed yet.
    for run in &runs {
        assert!(
            run.violations.is_empty(),
            "seed {}: {} simulator invariant violation(s): {:?}",
            run.seed,
            run.violations.len(),
            run.violations
        );
        assert!(
            run.leaked_tokens.is_empty(),
            "seed {}: {} token(s) outside the battlefield in the FINAL state — the \
             CR 704.5d cleanup genuinely failed rather than merely running late \
             (contrast OOS-M11-7): {:?}",
            run.seed,
            run.leaked_tokens.len(),
            run.leaked_tokens
        );
        assert!(
            run.error.is_none(),
            "seed {}: the playthrough hit an error path; full run: {run:?}",
            run.seed
        );
        assert!(
            run.outcome == "GameOver" || run.outcome == "MaxTurns",
            "seed {}: expected GameOver or the turn cap, got {:?}; full run: {run:?}",
            run.seed,
            run.outcome
        );
        // A run that made no decisions would satisfy every assertion above
        // vacuously. The human must actually have been asked.
        assert!(
            run.decisions > 0,
            "seed {}: seat 1 was never handed a decision; full run: {run:?}",
            run.seed
        );
    }

    // Non-vacuity, at the suite level rather than per seed: across the five games
    // the policy must have done more than pass. `PlayLand` is the discriminating
    // one — it is only offered when a land is in hand at sorcery speed with the
    // land drop unspent, so its presence proves real turns were played, not that
    // priority was ping-ponged.
    let all_kinds: BTreeSet<&'static str> = runs
        .iter()
        .flat_map(|r| r.submitted_kinds.iter().copied())
        .collect();
    assert!(
        all_kinds.contains("PlayLand"),
        "no seed ever played a land — the policy is not exercising real turns; kinds seen: {all_kinds:?}"
    );
    assert!(
        all_kinds.contains("DeclareAttackers"),
        "no seed ever reached a declare-attackers decision; kinds seen: {all_kinds:?}"
    );

    // Reported rather than asserted: the shape of what ran, so a future regression
    // that quietly shortens every game is visible in the log rather than silent.
    for run in &runs {
        println!(
            "seed {:>5}: {} after {} turns / {} commands, {} human decisions, \
             {} transient-token reports (OOS-M11-7), kinds {:?}",
            run.seed,
            run.outcome,
            run.turns,
            run.commands,
            run.decisions,
            run.transient_token_violations.len(),
            run.submitted_kinds
        );
    }
}

/// Plan item 1, the determinism half — the same seed must produce the same
/// playthrough. Not in the plan's wording, but the whole artefact of item 5
/// (`GET /api/game/report`) is a `{seed, config, ...}` repro bundle, and a bundle
/// that does not reproduce is worthless.
#[test]
fn test_s8_playthrough_is_reproducible_from_the_seed_alone() {
    let handle = std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(|| (play(SEEDS[0]), play(SEEDS[0])))
        .expect("worker thread should spawn");
    let (first, second) = handle.join().expect("playthrough thread must not panic");

    assert_eq!(first.outcome, second.outcome);
    assert_eq!(first.turns, second.turns);
    assert_eq!(first.commands, second.commands);
    assert_eq!(first.decisions, second.decisions);
    assert_eq!(first.submitted_kinds, second.submitted_kinds);
}
