//! HeuristicBot — weighted scoring for more realistic gameplay.
//!
//! Scoring priorities:
//! - Play a land: +100 (always first)
//! - Cast a spell: +50 base, +10 per mana value, +20 if removal-like
//! - Activate ability: +40
//! - Attack with creature: +30 if opponent tapped out, +10 otherwise
//! - Pass priority: +1 (last resort)
//! - Tap for mana: 0 — **below passing** (SIM-2; see the `TapForMana` arm of
//!   `score_action` for why "prep" was the wrong model)

use std::collections::HashMap;

use mtg_engine::{AttackTarget, Command, GameState, ObjectId, PlayerId};
use rand::prelude::*;

use crate::bot::Bot;
use crate::legal_actions::LegalAction;
use crate::random_bot::action_to_command;

/// Identifies a *repeatable* action for the per-turn preference damper. See
/// [`HeuristicBot::repeats_this_turn`].
///
/// Deliberately coarse: it names the class of choice, not the parameters, because
/// re-declaring the same combat with a *different* blocker assignment is exactly as
/// much of a non-move as re-declaring the identical one.
///
/// **No `DeclareAttackers` variant.** CR 508.1's once-per-combat legality is now
/// enforced by the engine itself (`GameStateError::AlreadyDeclaredAttackers`,
/// PB-DX21 / `OOS-M11-9`) and `legal_actions.rs` no longer offers the action once
/// `CombatState::attackers_declared` is set, so a *preference* damper on top of it
/// would be redundant scoring logic with nothing left to guard against. See
/// [`HeuristicBot::repeats_this_turn`]'s "CLOSED by PB-DX21" note for the history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum RepeatKey {
    /// CR 602: one entry per `(source, ability_index)`.
    Activate(ObjectId, usize),
    /// CR 509.1.
    DeclareBlockers,
}

impl RepeatKey {
    /// How many times this bot will *prefer* this action in one turn before scoring
    /// it below `PassPriority`.
    fn cap(self) -> u32 {
        match self {
            // Repeat activation is legitimate play (pump twice, crew then equip), so
            // allow a couple before treating it as a stall.
            RepeatKey::Activate(..) => 2,
            // CR 509.1 is a turn-based action performed once **per combat phase**,
            // and the tally for it is reset on each combat entry rather than on each
            // turn — see `HeuristicBot::repeats_this_turn`. Anything past the first
            // *within one combat* is never a real play.
            RepeatKey::DeclareBlockers => 1,
        }
    }

    fn of(action: &LegalAction) -> Option<Self> {
        match action {
            LegalAction::ActivateAbility {
                source,
                ability_index,
                ..
            } => Some(RepeatKey::Activate(*source, *ability_index)),
            LegalAction::DeclareBlockers { .. } => Some(RepeatKey::DeclareBlockers),
            _ => None,
        }
    }
}

pub struct HeuristicBot {
    rng: StdRng,
    name: String,
    /// The turn `repeats_this_turn` was last reset for.
    repeat_turn: u32,
    /// Whether a `CombatState` existed the last time this bot acted.
    ///
    /// The combat-scoped [`RepeatKey`] (`DeclareBlockers`) is reset when this goes
    /// `false` → `true`, which is what makes its cap **per combat phase** rather than
    /// per turn — see [`HeuristicBot::repeats_this_turn`]'s "the extra-combat
    /// regression" note. `turn_actions.rs` sets `state.combat = None` at end of
    /// combat and installs a fresh `CombatState` at `BeginningOfCombat`, so the
    /// transition is a reliable "a new combat phase has begun" signal with no
    /// counter to maintain.
    in_combat: bool,
    /// How many times this bot has already chosen each [`RepeatKey`] this turn.
    ///
    /// # The stuck game this prevents (M11-local S8)
    ///
    /// `score_action` rates every real play above `PassPriority`'s 1, and a bot that
    /// always prefers a real play will take one *forever* if the game keeps offering
    /// the same one. Found by the S8 scripted playthrough halting on `max_commands`,
    /// not by reading code:
    ///
    /// **A free, repeatable activated ability** (seed 9001, turn 2, 5,000 commands).
    /// `lightning_greaves`: Equip `{0}`. Its runtime `ActivatedAbility` declares
    /// `targets: []` while its `AttachEquipment` effect names
    /// `DeclaredTarget { index: 0 }`, so it is not merely cheap — it resolves as a
    /// **no-op**, changing nothing that could ever make the bot stop wanting it.
    ///
    /// # CLOSED by PB-DX21 (`OOS-M11-9`) — historical record, not a live instance
    ///
    /// This map used to carry a second stuck-game instance: **re-declaring the same
    /// combat** (seed 1, turn 19, 20,000 commands). Neither `StubProvider` nor
    /// `combat.rs::handle_declare_attackers` gated "attackers have already been
    /// declared this combat", so with a vigilant attacker (which stays untapped and
    /// therefore stays `eligible`) `DeclareAttackers` was offered and accepted
    /// without limit, even though CR 508.1 makes declaring attackers a turn-based
    /// action performed *once*. **PB-DX21 closes this at the engine**:
    /// `handle_declare_attackers` now rejects a second declaration with
    /// `GameStateError::AlreadyDeclaredAttackers`, and `legal_actions.rs` no longer
    /// offers the action once `CombatState::attackers_declared` is set — so there is
    /// no `RepeatKey::DeclareAttackers` any more; a preference damper on top of an
    /// action the provider never offers would guard nothing.
    ///
    /// CR 104.4b loop detection does not catch the surviving instance above: that
    /// rule is for a loop of **mandatory** actions, and a free Equip is optional.
    /// The bot is the right place for the mitigation — this is plan §8 R5 ("bot play
    /// quality") — and doing it here rather than in `StubProvider` keeps the
    /// provider's action list, and therefore every recorded `mtg-fuzzer` seed,
    /// untouched. (The fuzzer's default bot is `RandomBot`, which has no such
    /// problem: it picks uniformly, so it passes often enough to advance.)
    ///
    /// The cap is a **preference** cap, not a legality cap. A capped action is scored
    /// 0 rather than removed, so it is still chosen when nothing else is available —
    /// this can never make the bot fail to act.
    ///
    /// # The extra-combat regression this map's *scope* fixes (review MR-M11-09)
    ///
    /// The first version keyed the whole map on `turn_number` alone, which is right
    /// for `Activate` and **wrong** for the combat-scoped `DeclareBlockers` key.
    /// CR 506.5: a turn can contain more than one combat phase, and
    /// `aurelia_the_warleader` is `Complete`, deck-legal, and — since PB-DX1
    /// (`scutemob-160`) made her `once_per_turn` trigger actually fire — grants a
    /// real one. With a per-*turn* cap of 1, a bot that had already declared
    /// blockers in the first combat scored `DeclareBlockers` at 0 in the second,
    /// below `PassPriority`'s 1, and so **silently declined to block in every extra
    /// combat** — a quiet play-quality regression introduced by the fix for a loud
    /// stall.
    ///
    /// So `DeclareBlockers` is reset on combat-phase entry
    /// ([`HeuristicBot::in_combat`]) and `Activate` stays on the turn. Both resets
    /// are applied in `refresh_repeat_scope`.
    repeats_this_turn: HashMap<RepeatKey, u32>,
}

impl HeuristicBot {
    pub fn new(seed: u64, name: String) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            name,
            repeat_turn: 0,
            in_combat: false,
            repeats_this_turn: HashMap::new(),
        }
    }

    /// Drop each tally at the end of its own scope: the whole map on a new turn, and
    /// the combat-scoped `DeclareBlockers` key on entry to each combat phase
    /// (CR 506.5 — a turn may have several). See [`HeuristicBot::repeats_this_turn`].
    fn refresh_repeat_scope(&mut self, state: &GameState) {
        let turn = state.turn().turn_number;
        if turn != self.repeat_turn {
            self.repeat_turn = turn;
            self.repeats_this_turn.clear();
        }

        let in_combat = state.combat().is_some();
        if in_combat && !self.in_combat {
            self.repeats_this_turn.remove(&RepeatKey::DeclareBlockers);
        }
        self.in_combat = in_combat;
    }

    /// `true` once this action's [`RepeatKey`] has been chosen its `cap()` times this
    /// turn.
    fn is_capped_repeat(&self, action: &LegalAction) -> bool {
        let Some(key) = RepeatKey::of(action) else {
            return false;
        };
        self.repeats_this_turn
            .get(&key)
            .is_some_and(|n| *n >= key.cap())
    }

    fn score_action(&self, state: &GameState, player: PlayerId, action: &LegalAction) -> i32 {
        // Below `PassPriority`'s 1, so the bot passes instead of looping — but still
        // above nothing, so it remains choosable when it is all there is.
        if self.is_capped_repeat(action) {
            return 0;
        }
        match action {
            LegalAction::PlayLand { .. } => 100,
            LegalAction::CastSpell { card, .. } => {
                let base = 50;
                let mv_bonus = if let Ok(obj) = state.object(*card) {
                    obj.characteristics
                        .mana_cost
                        .as_ref()
                        .map(|c| c.mana_value() as i32 * 10)
                        .unwrap_or(0)
                } else {
                    0
                };
                base + mv_bonus
            }
            // SIM-6 (CR 602.2): an activation whose cost makes the bot NAME an
            // object it owns — sacrifice a permanent, discard a card — is scored
            // BELOW `PassPriority`, so this bot declines it by default.
            //
            // Not a legality gate and not a hole in the channel. `params.rs` fills
            // the plan's own default, so if this action is ever the only thing on
            // offer the resulting command is one the engine ACCEPTS (the 0 idiom
            // above: below `PassPriority`, above nothing).
            //
            // The reason is that this bot has no valuation for the resource it
            // would spend. It scores an activation at 40 because activations are
            // usually free upside; "sacrifice a creature" is not, and the cap above
            // would let it eat two of its own creatures per turn, every turn, for
            // whatever the ability does. The dispatch brief is explicit that
            // teaching bots sacrifice STRATEGY is out of scope and that declining
            // is an acceptable answer — so this declines rather than guesses.
            //
            // `RandomBot` still picks these uniformly, so the fuzzer keeps
            // exercising the channel end to end; only the *heuristic* seat abstains.
            LegalAction::ActivateAbility {
                activation_costs, ..
            } => {
                if activation_costs.sacrifice.is_some() || activation_costs.discard.is_some() {
                    0
                } else {
                    40
                }
            }
            LegalAction::DeclareAttackers {
                eligible,
                targets: _,
            } => {
                if eligible.is_empty() {
                    return 0;
                }
                // Higher score if we have lots of attackers
                let base = 30;
                let count_bonus = (eligible.len() as i32) * 5;
                base + count_bonus
            }
            LegalAction::DeclareBlockers {
                eligible,
                attackers,
                ..
            } => {
                // Block if we have enough creatures
                if eligible.is_empty() || attackers.is_empty() {
                    return 0;
                }
                20
            }
            // SIM-2 (playtest triage F5): **below** `PassPriority`'s 1, not the +5 this
            // scored until 2026-08-02.
            //
            // The header's "only useful as prep" was never conditioned on anything, so
            // with nothing to cast the bot deterministically tapped every source it
            // controlled, CR 500.4 emptied the pool at the step boundary, and it arrived
            // at its main phase tapped out. A human watching a browser game saw it every
            // upkeep. Systematic, not noise: `score_action` is deterministic and 5 > 1.
            //
            // Scoring it *below* passing rather than gating it on a spend target is the
            // whole fix because the two are observationally identical here: every action
            // that could consume mana already outscores 5 (`CastSpell` 50+,
            // `ActivateAbility` 40, `PlayLand` 100), so a tap was only ever CHOSEN when
            // it was the sole alternative to passing -- which is exactly the empty-upkeep
            // tap-out and nothing else. `LocalGame` auto-taps for a bot's casts
            // (`advance()` -> `auto_tap_commands_for`), so nothing the bot can actually
            // pay for depends on it pre-floating mana.
            //
            // Scored 0, not removed: like a capped repeat, the action stays choosable
            // when it is all there is, so this can never make the bot fail to act.
            //
            // **Known consequence, deliberately not fixed here** (`OOS-SIM2-3`): a bot
            // still cannot pay an activated ability's mana cost, because `advance()`
            // auto-taps for `CastSpell` only. That was already true -- `ActivateAbility`
            // scored 40 and was therefore always chosen *before* any tap, then rejected
            // for lack of mana and absorbed by the `PassPriority` fallback -- so this
            // change does not cause it and does not deepen it.
            LegalAction::TapForMana { .. } => 0,
            LegalAction::TakeMulligan => 10,
            LegalAction::KeepHand => 50,
            LegalAction::ReturnCommanderToCommandZone { .. } => 80,
            LegalAction::LeaveCommanderInZone { .. } => 20,
            LegalAction::PassPriority => 1,
            LegalAction::Concede => 0,
            // CR 509.2 (M11-local S8): human-only — `StubProvider` never emits it, so
            // this arm is unreachable for a bot seat and exists only because the match
            // is exhaustive. Scored 0 alongside `Concede` (the other human-only action)
            // so that if some future provider *did* emit it, a bot would never prefer it
            // to a real play. `local_game::human_only_actions` documents why it is not
            // in the provider.
            LegalAction::OrderBlockers { .. } => 0,
            // Bloodrush (B12): activated from hand targeting an attacking creature.
            // Treat like an activated ability — decent priority.
            LegalAction::ActivateBloodrush { .. } => 40,
            // Saddle (B13): saddling a Mount is a useful setup action.
            LegalAction::SaddleMount { .. } => 35,
            // Mutate: treat like casting a spell — good priority.
            LegalAction::CastWithMutate { .. } => 50,
            // TurnFaceUp: reveal a face-down permanent (morph/disguise/manifest/cloak).
            // Good priority — turning face up usually improves board state.
            LegalAction::TurnFaceUp { .. } => 45,
            // CastMorphFaceDown: cast a card face-down for {3}. Moderate priority —
            // useful for bluffing but less impactful than casting normally.
            LegalAction::CastMorphFaceDown { .. } => 30,
            // Loyalty ability: important — planeswalker abilities are high value.
            LegalAction::ActivateLoyaltyAbility { .. } => 60,
            // PB-DP4 / DP-11: an outstanding echo / cumulative upkeep / recover payment.
            // Paying keeps the permanent (or returns the card to hand) and is offered only
            // when affordable, so treat it like a worthwhile activated ability. Declining
            // is a legal but usually-undesirable last resort — score it just above passing
            // so the bot doesn't reflexively give away a permanent/card it could afford to
            // keep, but still exercises the decline path when it can't afford `pay: true`.
            LegalAction::PayEcho { pay, .. }
            | LegalAction::PayCumulativeUpkeep { pay, .. }
            | LegalAction::PayRecover { pay, .. } => {
                if *pay {
                    45
                } else {
                    2
                }
            }
            // PB-DP7 / DP-3 (CR 514.1): it is the ONLY legal action while a
            // cleanup discard is outstanding (the provider offers nothing
            // else), so any score works -- scored high to document that it is
            // not optional.
            LegalAction::DiscardToHandSize { .. } => 100,
            // CR 603.3d (PB-DP8 / DP-6): the one action offered while the CR
            // 603.3b batch is suspended -- same precedent and rationale as
            // DiscardToHandSize above.
            LegalAction::ChooseTriggerTargets { .. } => 100,
            // CR 608.2d (PB-DP9 / DP-7/8/9): the one action offered while a
            // resolution-time choice is outstanding -- same precedent and
            // rationale as DiscardToHandSize above.
            LegalAction::AnswerEffectChoice { .. } => 100,
            // PB-DX23 (CR 702.52a, 104.3c; plan §3 Q4). `None` (decline) mirrors
            // `PayEcho { pay: false }` verbatim -- just above `PassPriority`'s 1
            // so the bot always discharges an outstanding offer rather than
            // sitting on it, and below every real play so answering never
            // displaces one.
            LegalAction::ChooseDredge { card: None, .. } => 2,
            // `Some(_)` scores 3 (above the decline, above `PassPriority`) only
            // with 2x library headroom over the mill count -- CR 702.52b's
            // `library_count >= n` is a LEGALITY floor the engine and the offer
            // already enforce; `2 * n` is a SURVIVAL rule against CR 104.3c
            // (milling out) that is this bot's only defence, since it has no
            // other way to value what it would give up. Below the margin it
            // scores 0 -- the "below PassPriority, above nothing" idiom used
            // throughout this function -- so the action stays choosable when it
            // is all there is and the resulting command is one the engine
            // ACCEPTS (SR-38). (Inherited idiom -- shared with `TapForMana`;
            // for `ChooseDredge` specifically this 0 arm is effectively
            // "never the top score" in practice, since `PassPriority` (1) is
            // pushed unconditionally before this block runs, so a `0` can
            // never outscore it -- review finding S3.)
            LegalAction::ChooseDredge {
                card: Some(_),
                mill,
            } => {
                let library_count = state
                    .zones()
                    .get(&mtg_engine::ZoneId::Library(player))
                    .map(|z| z.object_ids().len())
                    .unwrap_or(0);
                if library_count >= 2 * (*mill as usize) {
                    3
                } else {
                    0
                }
            }
        }
    }
}

impl Bot for HeuristicBot {
    fn choose_action(
        &mut self,
        state: &GameState,
        player: PlayerId,
        legal: &[LegalAction],
    ) -> Command {
        if legal.is_empty() {
            return Command::PassPriority { player };
        }

        self.refresh_repeat_scope(state);

        // Score all actions and pick the highest (with random tiebreaking)
        let mut scored: Vec<(i32, usize)> = legal
            .iter()
            .enumerate()
            .map(|(idx, action)| (self.score_action(state, player, action), idx))
            .collect();

        // Sort by score descending, then random tiebreak
        #[allow(clippy::unnecessary_sort_by)]
        scored.sort_by(|a, b| b.0.cmp(&a.0));

        // Among top-scored actions (same score), pick randomly
        let top_score = scored[0].0;
        let top_actions: Vec<usize> = scored
            .iter()
            .take_while(|(s, _)| *s == top_score)
            .map(|(_, idx)| *idx)
            .collect();

        let chosen_idx = top_actions[self.rng.random_range(0..top_actions.len())];
        // Tally BEFORE building the command: the count is of what this bot chose,
        // which is knowable here and nowhere else — `LocalGame` never reports back
        // whether the engine accepted it. A choice the engine then rejects still
        // counts, which is the conservative direction (a rejected action is exactly
        // one this bot should stop preferring).
        if let Some(key) = RepeatKey::of(&legal[chosen_idx]) {
            *self.repeats_this_turn.entry(key).or_insert(0) += 1;
        }
        action_to_command(&mut self.rng, state, player, &legal[chosen_idx])
    }

    fn choose_targets(
        &mut self,
        _state: &GameState,
        valid: &[ObjectId],
        count: usize,
    ) -> Vec<ObjectId> {
        // Pick first N targets (could be smarter later)
        valid.iter().take(count).copied().collect()
    }

    fn choose_attackers(
        &mut self,
        state: &GameState,
        eligible: &[ObjectId],
        targets: &[AttackTarget],
    ) -> Vec<(ObjectId, AttackTarget)> {
        if eligible.is_empty() || targets.is_empty() {
            return Vec::new();
        }

        // Attack with all eligible creatures, targeting the opponent with lowest life
        let target = targets
            .iter()
            .min_by_key(|t| match t {
                AttackTarget::Player(pid) => {
                    state.player(*pid).map(|p| p.life_total).unwrap_or(999)
                }
                AttackTarget::Planeswalker(oid) => state
                    .object(*oid)
                    .map(|o| o.characteristics.loyalty.unwrap_or(999))
                    .unwrap_or(999),
            })
            .cloned()
            .unwrap_or(targets[0].clone());

        eligible.iter().map(|&id| (id, target.clone())).collect()
    }

    fn choose_blockers(
        &mut self,
        _state: &GameState,
        eligible: &[ObjectId],
        attackers: &[ObjectId],
    ) -> Vec<(ObjectId, ObjectId)> {
        if eligible.is_empty() || attackers.is_empty() {
            return Vec::new();
        }

        // Block each attacker with one blocker if available
        let mut blocks = Vec::new();
        let mut available_blockers: Vec<ObjectId> = eligible.to_vec();

        for &attacker in attackers {
            if available_blockers.is_empty() {
                break;
            }
            // Assign the first available blocker
            let blocker = available_blockers.remove(0);
            blocks.push((blocker, attacker));
        }

        blocks
    }

    fn choose_mulligan_bottom(&mut self, hand: &[ObjectId], count: usize) -> Vec<ObjectId> {
        // Bottom the last N cards (arbitrary heuristic)
        hand.iter().rev().take(count).copied().collect()
    }

    fn name(&self) -> &str {
        &self.name
    }
}
