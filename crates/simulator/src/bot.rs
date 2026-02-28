//! Bot trait — the decision-making interface for automated players.

use mtg_engine::{AttackTarget, Command, GameState, ObjectId, PlayerId};

use crate::legal_actions::LegalAction;

/// A bot that can play a game of MTG Commander.
pub trait Bot: Send {
    /// Choose an action from the list of legal actions.
    fn choose_action(
        &mut self,
        state: &GameState,
        player: PlayerId,
        legal: &[LegalAction],
    ) -> Command;

    /// Choose targets for a spell or ability.
    fn choose_targets(
        &mut self,
        state: &GameState,
        valid: &[ObjectId],
        count: usize,
    ) -> Vec<ObjectId>;

    /// Choose which creatures to attack with, and which targets.
    fn choose_attackers(
        &mut self,
        state: &GameState,
        eligible: &[ObjectId],
        targets: &[AttackTarget],
    ) -> Vec<(ObjectId, AttackTarget)>;

    /// Choose blockers: pairs of (blocker, attacker_being_blocked).
    fn choose_blockers(
        &mut self,
        state: &GameState,
        eligible: &[ObjectId],
        attackers: &[ObjectId],
    ) -> Vec<(ObjectId, ObjectId)>;

    /// Choose cards from hand to put on the bottom during mulligan.
    fn choose_mulligan_bottom(&mut self, hand: &[ObjectId], count: usize) -> Vec<ObjectId>;

    /// Bot display name.
    fn name(&self) -> &str;
}
