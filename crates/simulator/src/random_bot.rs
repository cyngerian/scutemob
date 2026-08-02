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
        if let Some(action @ LegalAction::DeclareAttackers { eligible, targets }) = attack_action {
            if !eligible.is_empty() && self.rng.random_bool(0.8) {
                let attackers = self.choose_attackers(state, eligible, targets);
                // PB-DX6 §9.2: route through `action_to_command_with_params`
                // (params.rs) rather than hand-building the `Command` here, so
                // the CR 508.1h attack-tax payment plan — which can only be
                // built once the attacker SET is known — is computed in
                // exactly one place. See that arm's doc for why the plan
                // cannot live on `LegalAction::DeclareAttackers` itself.
                let params = ActionParams {
                    attackers,
                    ..ActionParams::default()
                };
                if let Ok(cmd) = action_to_command_with_params(state, player, action, &params) {
                    return cmd;
                }
                // Unreachable in practice for `DeclareAttackers` (the arm
                // never returns `Err`), kept as a non-panicking fallback.
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
///
/// **SIM-5 (CR 601.2c) adds one more filled field: `targets`.** Until then this
/// function's "every other field stays at its default" was literally true and was the
/// structural reason bots could not cast a single targeted spell — G5 of
/// `memory/playtest-triage-2026-08-02b.md`. `targeting::plan_targets` is RNG-free
/// (see its doc), so the sentence above still holds for the RNG: this remains an
/// RNG-only wrapper as far as randomness is concerned, and a recorded seed's draw
/// sequence is unchanged by the addition.
///
/// `HeuristicBot` shares this function (`heuristic_bot.rs:19`, called at `:346`), so
/// both bots gain targeting from the single edit below.
pub(crate) fn action_to_command(
    rng: &mut StdRng,
    state: &GameState,
    player: PlayerId,
    action: &LegalAction,
) -> Command {
    let mut params = ActionParams::default();

    // CR 601.2c / CR 602.2b (SIM-5): announce targets. `NotTargeted` and
    // `Unsatisfiable` both leave this empty -- the first because there is nothing to
    // announce, the second because no announcement can be legal, in which case the
    // engine refuses the command and `LocalGame::advance()` records the refusal
    // (`RejectedCommand`) and passes, spending nothing (SIM-5 fix (1)).
    params.targets = crate::targeting::plan_targets(state, player, action).announced();

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

#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::state::ActiveRestriction;
    use mtg_engine::{
        process_command, GameRestriction, GameStateBuilder, HybridMana, ManaColor, ManaCost,
        ObjectSpec, Step, ZoneId,
    };

    fn id_of(state: &GameState, name: &str) -> ObjectId {
        state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == name)
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("no object named {name:?}"))
    }

    /// PB-DX6 §9.2: `RandomBot`'s 80%-bias `DeclareAttackers` path (the direct-build
    /// branch in `choose_action`, which used to hand-construct `Command` with
    /// hard-coded empty payment vectors) now routes through
    /// `action_to_command_with_params` exactly like every other arm, so a pipped CR
    /// 508.1h attack tax gets a real plan there too -- not just via the
    /// `action_to_command` fallback path. Swept across several seeds so both the
    /// 80% branch and the 20%-fallback branch fire at least once, and BOTH must
    /// produce a `Command` the engine actually accepts and that spends the pip.
    #[test]
    fn choose_action_pays_a_hybrid_attack_tax_on_every_seed() {
        // PB-DX6 fix cycle, Finding 12: this loop's own comment concedes the bot may
        // legally decline to attack on any given seed (0-count random subset on the
        // fallback path), and every non-attacking seed `continue`s past all the real
        // assertions below -- so a non-vacuity floor is needed, matching this suite's
        // own standard for pinned-empty/skip-guarded assertions elsewhere.
        let mut attacked_count = 0u32;
        for seed in 0..20u64 {
            let p1 = PlayerId(1);
            let p2 = PlayerId(2);
            let mut state = GameStateBuilder::new()
                .add_player(p1)
                .add_player(p2)
                .active_player(p1)
                .at_step(Step::DeclareAttackers)
                .object(ObjectSpec::creature(p2, "Tax Source", 0, 4).in_zone(ZoneId::Battlefield))
                .object(
                    ObjectSpec::creature(p1, "Attacking Bear", 2, 2).in_zone(ZoneId::Battlefield),
                )
                .build()
                .unwrap();
            let tax_source = id_of(&state, "Tax Source");
            state.restrictions_mut().push_back(ActiveRestriction {
                source: tax_source,
                controller: p2,
                restriction: GameRestriction::CantAttackYouUnlessPay {
                    cost_per_creature: ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                        ..Default::default()
                    },
                },
            });
            state.turn_mut().priority_holder = Some(p1);
            let bear = id_of(&state, "Attacking Bear");
            state.players_mut().get_mut(&p1).unwrap().mana_pool.green = 1;

            let legal = vec![LegalAction::DeclareAttackers {
                eligible: vec![bear],
                targets: vec![AttackTarget::Player(p2)],
            }];
            let mut bot = RandomBot::new(seed, "seeded".to_string());
            let cmd = bot.choose_action(&state, p1, &legal);

            // The bot is allowed to decline to attack at all (0-count random subset
            // on the fallback path) -- that is legal and outside this test's scope.
            // But whenever it DOES declare the attacker, the pip must be paid.
            let attacked = matches!(
                &cmd,
                Command::DeclareAttackers { attackers, .. } if !attackers.is_empty()
            );
            if !attacked {
                continue;
            }
            attacked_count += 1;
            let (state, _events) = process_command(state, cmd)
                .unwrap_or_else(|e| panic!("seed {seed}: bot built an unpayable plan: {e:?}"));
            assert!(
                state
                    .combat()
                    .as_ref()
                    .map(|c| c.attackers.contains_key(&bear))
                    .unwrap_or(false),
                "seed {seed}: attacker must be genuinely declared"
            );
            let pool = &state.player(p1).unwrap().mana_pool;
            assert_eq!(
                pool.total(),
                0,
                "seed {seed}: the Green pip must have been spent: {pool:?}"
            );
        }
        assert!(
            attacked_count >= 1,
            "non-vacuity floor: at least one of the 20 seeds must have declared the \
             attack for the assertions above to have run at all"
        );
    }
}
