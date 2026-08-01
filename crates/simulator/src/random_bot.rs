//! RandomBot — uniform random selection from legal actions.
//!
//! Seeded RNG for reproducibility. Biased toward attacking (80/20)
//! to ensure games progress toward a conclusion.

use mtg_engine::{AttackTarget, Command, GameState, ObjectId, PlayerId};
use rand::prelude::*;

use crate::bot::Bot;
use crate::legal_actions::LegalAction;
use crate::params::{action_to_command_with_params, ActionParams};

pub struct RandomBot {
    rng: StdRng,
    name: String,
}

impl RandomBot {
    pub fn new(seed: u64, name: String) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            name,
        }
    }
}

impl Bot for RandomBot {
    fn choose_action(
        &mut self,
        state: &GameState,
        player: PlayerId,
        legal: &[LegalAction],
    ) -> Command {
        if legal.is_empty() {
            return Command::PassPriority { player };
        }

        // Bias: 80% chance to attack if DeclareAttackers is available
        let attack_action = legal
            .iter()
            .find(|a| matches!(a, LegalAction::DeclareAttackers { .. }));
        if let Some(LegalAction::DeclareAttackers { eligible, targets }) = attack_action {
            if !eligible.is_empty() && self.rng.random_bool(0.8) {
                let attackers = self.choose_attackers(state, eligible, targets);
                return Command::DeclareAttackers {
                    player,
                    attackers,
                    enlist_choices: Vec::new(),
                    exert_choices: Vec::new(),
                };
            }
        }

        let idx = self.rng.random_range(0..legal.len());
        action_to_command(&mut self.rng, state, player, &legal[idx])
    }

    fn choose_targets(
        &mut self,
        _state: &GameState,
        valid: &[ObjectId],
        count: usize,
    ) -> Vec<ObjectId> {
        let mut targets: Vec<ObjectId> = valid.to_vec();
        targets.shuffle(&mut self.rng);
        targets.truncate(count);
        targets
    }

    fn choose_attackers(
        &mut self,
        _state: &GameState,
        eligible: &[ObjectId],
        targets: &[AttackTarget],
    ) -> Vec<(ObjectId, AttackTarget)> {
        if eligible.is_empty() || targets.is_empty() {
            return Vec::new();
        }
        // Attack with a random subset of eligible creatures
        let count = self.rng.random_range(1..=eligible.len());
        let mut shuffled = eligible.to_vec();
        shuffled.shuffle(&mut self.rng);
        shuffled
            .into_iter()
            .take(count)
            .map(|id| {
                let target = targets[self.rng.random_range(0..targets.len())].clone();
                (id, target)
            })
            .collect()
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
        // Block with ~50% of eligible creatures
        let mut blocks = Vec::new();
        for &blocker in eligible {
            if self.rng.random_bool(0.5) {
                let attacker = attackers[self.rng.random_range(0..attackers.len())];
                blocks.push((blocker, attacker));
            }
        }
        blocks
    }

    fn choose_mulligan_bottom(&mut self, hand: &[ObjectId], count: usize) -> Vec<ObjectId> {
        let mut cards = hand.to_vec();
        cards.shuffle(&mut self.rng);
        cards.truncate(count);
        cards
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Convert a LegalAction into a Command the engine can process.
///
/// M11-local Session 3 (item 6): this is now a thin RNG-only wrapper over
/// `action_to_command_with_params` (`params.rs`), the single `LegalAction` ->
/// `Command` mapping table in the codebase. The RNG is used ONLY to fill
/// `ActionParams::attackers`/`blockers` for `DeclareAttackers`/`DeclareBlockers` —
/// every other field stays at its `ActionParams::default()`, and
/// `action_to_command_with_params` itself is RNG-free and deterministic.
pub(crate) fn action_to_command(
    rng: &mut StdRng,
    state: &GameState,
    player: PlayerId,
    action: &LegalAction,
) -> Command {
    let mut params = ActionParams::default();

    match action {
        // Random subset of attackers (moved verbatim from the pre-Session-3 body).
        LegalAction::DeclareAttackers { eligible, targets }
            if !eligible.is_empty() && !targets.is_empty() =>
        {
            let count = rng.random_range(0..=eligible.len());
            let mut shuffled = eligible.clone();
            shuffled.shuffle(rng);
            let attackers: Vec<(ObjectId, AttackTarget)> = shuffled
                .into_iter()
                .take(count)
                .map(|id| {
                    let target = targets[rng.random_range(0..targets.len())].clone();
                    (id, target)
                })
                .collect();
            params.attackers = attackers;
        }
        // Random subset of blockers (moved verbatim from the pre-Session-3 body).
        LegalAction::DeclareBlockers {
            eligible,
            attackers,
        } => {
            let mut blocks = Vec::new();
            for &blocker in eligible {
                if rng.random_bool(0.4) && !attackers.is_empty() {
                    let attacker = attackers[rng.random_range(0..attackers.len())];
                    blocks.push((blocker, attacker));
                }
            }
            params.blockers = blocks;
        }
        _ => {}
    }

    // Unreachable in practice: the provider guarantees a `chosen_color` for
    // every `any_color` `TapForMana` it offers
    // (`legal_actions::resolve_hybrid_phyrexian_plan` / the `TapForMana`
    // enumeration in `legal_actions.rs`), and no other `LegalAction` this bot
    // sees can produce a `ParamError` from an all-default `ActionParams`. A
    // pass is the safe, non-panicking answer for a fuzz run if that ever
    // stops holding.
    action_to_command_with_params(state, player, action, &params)
        .unwrap_or(Command::PassPriority { player })
}
