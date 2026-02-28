//! HeuristicBot — weighted scoring for more realistic gameplay.
//!
//! Scoring priorities:
//! - Play a land: +100 (always first)
//! - Cast a spell: +50 base, +10 per mana value, +20 if removal-like
//! - Activate ability: +40
//! - Attack with creature: +30 if opponent tapped out, +10 otherwise
//! - Tap for mana: +5 (only useful as prep)
//! - Pass priority: +1 (last resort)

use mtg_engine::{AttackTarget, Command, GameState, ObjectId, PlayerId};
use rand::prelude::*;

use crate::bot::Bot;
use crate::legal_actions::LegalAction;
use crate::random_bot::action_to_command;

pub struct HeuristicBot {
    rng: StdRng,
    name: String,
}

impl HeuristicBot {
    pub fn new(seed: u64, name: String) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            name,
        }
    }

    fn score_action(&self, state: &GameState, _player: PlayerId, action: &LegalAction) -> i32 {
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

        // Score all actions and pick the highest (with random tiebreaking)
        let mut scored: Vec<(i32, usize)> = legal
            .iter()
            .enumerate()
            .map(|(idx, action)| (self.score_action(state, player, action), idx))
            .collect();

        // Sort by score descending, then random tiebreak
        scored.sort_by(|a, b| b.0.cmp(&a.0));

        // Among top-scored actions (same score), pick randomly
        let top_score = scored[0].0;
        let top_actions: Vec<usize> = scored
            .iter()
            .take_while(|(s, _)| *s == top_score)
            .map(|(_, idx)| *idx)
            .collect();

        let chosen_idx = top_actions[self.rng.gen_range(0..top_actions.len())];
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
