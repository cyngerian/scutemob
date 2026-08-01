//! HeuristicBot — weighted scoring for more realistic gameplay.
//!
//! Scoring priorities:
//! - Play a land: +100 (always first)
//! - Cast a spell: +50 base, +10 per mana value, +20 if removal-like
//! - Activate ability: +40
//! - Attack with creature: +30 if opponent tapped out, +10 otherwise
//! - Tap for mana: +5 (only useful as prep)
//! - Pass priority: +1 (last resort)

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
/// re-declaring the same combat with a *different* attacker set is exactly as much
/// of a non-move as re-declaring the identical one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum RepeatKey {
    /// CR 602: one entry per `(source, ability_index)`.
    Activate(ObjectId, usize),
    /// CR 508.1.
    DeclareAttackers,
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
            // CR 508.1 / 509.1 are turn-based actions performed once per combat.
            // Anything past the first is never a real play.
            RepeatKey::DeclareAttackers | RepeatKey::DeclareBlockers => 1,
        }
    }

    fn of(action: &LegalAction) -> Option<Self> {
        match action {
            LegalAction::ActivateAbility {
                source,
                ability_index,
                ..
            } => Some(RepeatKey::Activate(*source, *ability_index)),
            LegalAction::DeclareAttackers { .. } => Some(RepeatKey::DeclareAttackers),
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
    /// How many times this bot has already chosen each [`RepeatKey`] this turn.
    ///
    /// # The two stuck games this prevents (M11-local S8)
    ///
    /// `score_action` rates every real play above `PassPriority`'s 1, and a bot that
    /// always prefers a real play will take one *forever* if the game keeps offering
    /// the same one. Both instances were found by the S8 scripted playthrough halting
    /// on `max_commands`, not by reading code:
    ///
    /// 1. **A free, repeatable activated ability** (seed 9001, turn 2, 5,000 commands).
    ///    `lightning_greaves`: Equip `{0}`. Its runtime `ActivatedAbility` declares
    ///    `targets: []` while its `AttachEquipment` effect names
    ///    `DeclaredTarget { index: 0 }`, so it is not merely cheap — it resolves as a
    ///    **no-op**, changing nothing that could ever make the bot stop wanting it.
    /// 2. **Re-declaring the same combat** (seed 1, turn 19, 20,000 commands). Neither
    ///    `StubProvider` nor `combat.rs::handle_declare_attackers` gates "attackers
    ///    have already been declared this combat", so with a vigilant attacker (which
    ///    stays untapped and therefore stays `eligible`) `DeclareAttackers` is offered
    ///    and accepted without limit. CR 508.1 makes declaring attackers a turn-based
    ///    action performed *once*, so the engine accepting a second one is a real gap
    ///    — filed as **OOS-M11-9**; fixing it is an engine change and M11-local makes
    ///    none.
    ///
    /// CR 104.4b loop detection catches neither: that rule is for a loop of
    /// **mandatory** actions, and both of these are optional. The bot is the right
    /// place for the mitigation — this is plan §8 R5 ("bot play quality") — and doing
    /// it here rather than in `StubProvider` keeps the provider's action list, and
    /// therefore every recorded `mtg-fuzzer` seed, untouched. (The fuzzer's default
    /// bot is `RandomBot`, which has neither problem: it picks uniformly, so it passes
    /// often enough to advance.)
    ///
    /// The cap is a **preference** cap, not a legality cap. A capped action is scored
    /// 0 rather than removed, so it is still chosen when nothing else is available —
    /// this can never make the bot fail to act.
    repeats_this_turn: HashMap<RepeatKey, u32>,
}

impl HeuristicBot {
    pub fn new(seed: u64, name: String) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            name,
            repeat_turn: 0,
            repeats_this_turn: HashMap::new(),
        }
    }

    /// Drop the per-turn tally when the turn number moves.
    fn reset_repeats_if_new_turn(&mut self, state: &GameState) {
        let turn = state.turn().turn_number;
        if turn != self.repeat_turn {
            self.repeat_turn = turn;
            self.repeats_this_turn.clear();
        }
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

    fn score_action(&self, state: &GameState, _player: PlayerId, action: &LegalAction) -> i32 {
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
            LegalAction::ActivateAbility { .. } => 40,
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
            } => {
                // Block if we have enough creatures
                if eligible.is_empty() || attackers.is_empty() {
                    return 0;
                }
                20
            }
            LegalAction::TapForMana { .. } => 5,
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

        self.reset_repeats_if_new_turn(state);

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
