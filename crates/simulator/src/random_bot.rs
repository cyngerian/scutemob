//! RandomBot — uniform random selection from legal actions.
//!
//! Seeded RNG for reproducibility. Biased toward attacking (80/20)
//! to ensure games progress toward a conclusion.

use mtg_engine::{AltCostKind, AttackTarget, Command, GameState, ObjectId, PlayerId};
use rand::prelude::*;

use crate::bot::Bot;
use crate::legal_actions::LegalAction;

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
            if !eligible.is_empty() && self.rng.gen_bool(0.8) {
                let attackers = self.choose_attackers(state, eligible, targets);
                return Command::DeclareAttackers {
                    player,
                    attackers,
                    enlist_choices: Vec::new(),
                };
            }
        }

        let idx = self.rng.gen_range(0..legal.len());
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
        let count = self.rng.gen_range(1..=eligible.len());
        let mut shuffled = eligible.to_vec();
        shuffled.shuffle(&mut self.rng);
        shuffled
            .into_iter()
            .take(count)
            .map(|id| {
                let target = targets[self.rng.gen_range(0..targets.len())].clone();
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
            if self.rng.gen_bool(0.5) {
                let attacker = attackers[self.rng.gen_range(0..attackers.len())];
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
pub(crate) fn action_to_command(
    rng: &mut StdRng,
    _state: &GameState,
    player: PlayerId,
    action: &LegalAction,
) -> Command {
    match action {
        LegalAction::PassPriority => Command::PassPriority { player },
        LegalAction::Concede => Command::Concede { player },
        LegalAction::PlayLand { card } => Command::PlayLand {
            player,
            card: *card,
        },
        LegalAction::CastSpell { card, .. } => Command::CastSpell {
            player,
            card: *card,
            targets: Vec::new(),
            convoke_creatures: Vec::new(),
            improvise_artifacts: Vec::new(),
            delve_cards: Vec::new(),
            kicker_times: 0,
            escape_exile_cards: Vec::new(),
            retrace_discard_land: None,
            jump_start_discard: None,
            alt_cost: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: Vec::new(),
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: Vec::new(),
            modes_chosen: Vec::new(),
            fuse: false,
            x_value: 0,
            collect_evidence_cards: Vec::new(),
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
        LegalAction::TapForMana {
            source,
            ability_index,
        } => Command::TapForMana {
            player,
            source: *source,
            ability_index: *ability_index,
        },
        LegalAction::ActivateAbility {
            source,
            ability_index,
        } => Command::ActivateAbility {
            player,
            source: *source,
            ability_index: *ability_index,
            targets: Vec::new(),
            discard_card: None,
        },
        LegalAction::DeclareAttackers { eligible, targets } => {
            // Pick random subset
            if eligible.is_empty() || targets.is_empty() {
                return Command::DeclareAttackers {
                    player,
                    attackers: Vec::new(),
                    enlist_choices: Vec::new(),
                };
            }
            let count = rng.gen_range(0..=eligible.len());
            let mut shuffled = eligible.clone();
            shuffled.shuffle(rng);
            let attackers: Vec<(ObjectId, AttackTarget)> = shuffled
                .into_iter()
                .take(count)
                .map(|id| {
                    let target = targets[rng.gen_range(0..targets.len())].clone();
                    (id, target)
                })
                .collect();
            Command::DeclareAttackers {
                player,
                attackers,
                enlist_choices: Vec::new(),
            }
        }
        LegalAction::DeclareBlockers {
            eligible,
            attackers,
        } => {
            // Block with random subset
            let mut blocks = Vec::new();
            for &blocker in eligible {
                if rng.gen_bool(0.4) && !attackers.is_empty() {
                    let attacker = attackers[rng.gen_range(0..attackers.len())];
                    blocks.push((blocker, attacker));
                }
            }
            Command::DeclareBlockers {
                player,
                blockers: blocks,
            }
        }
        LegalAction::TakeMulligan => Command::TakeMulligan { player },
        LegalAction::KeepHand => {
            // For simplicity, bottom nothing (London mulligan count = 0 for first keep)
            Command::KeepHand {
                player,
                cards_to_bottom: Vec::new(),
            }
        }
        LegalAction::ReturnCommanderToCommandZone { object_id } => {
            Command::ReturnCommanderToCommandZone {
                player,
                object_id: *object_id,
            }
        }
        LegalAction::LeaveCommanderInZone { object_id } => Command::LeaveCommanderInZone {
            player,
            object_id: *object_id,
        },
        // ── Bloodrush (CR 207.2c / B12) ─────────────────────────────────────────
        // Activate the bloodrush ability: discard the card, target an attacking creature.
        LegalAction::ActivateBloodrush { card, target } => Command::ActivateBloodrush {
            player,
            card: *card,
            target: *target,
        },
        // ── Saddle (CR 702.171 / B13) ────────────────────────────────────────────
        // Saddle a Mount by tapping the pre-selected creatures.
        LegalAction::SaddleMount {
            mount,
            saddle_creatures,
        } => Command::SaddleMount {
            player,
            mount: *mount,
            saddle_creatures: saddle_creatures.clone(),
        },
        // ── Mutate (CR 702.140) ──────────────────────────────────────────────────
        // Cast the card using its mutate alternative cost. Default: place on top (true)
        // so the mutating spell's characteristics become the merged permanent's.
        LegalAction::CastWithMutate {
            card,
            mutate_target,
        } => Command::CastSpell {
            player,
            card: *card,
            targets: Vec::new(),
            convoke_creatures: Vec::new(),
            improvise_artifacts: Vec::new(),
            delve_cards: Vec::new(),
            kicker_times: 0,
            escape_exile_cards: Vec::new(),
            retrace_discard_land: None,
            jump_start_discard: None,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: Vec::new(),
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: Vec::new(),
            modes_chosen: Vec::new(),
            fuse: false,
            x_value: 0,
            collect_evidence_cards: Vec::new(),
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
            mutate_target: Some(*mutate_target),
            mutate_on_top: true,
            face_down_kind: None,
        },
        LegalAction::TurnFaceUp { permanent, method } => Command::TurnFaceUp {
            player,
            permanent: *permanent,
            method: method.clone(),
        },
        LegalAction::CastMorphFaceDown { card, .. } => Command::CastSpell {
            player,
            card: *card,
            targets: Vec::new(),
            convoke_creatures: Vec::new(),
            improvise_artifacts: Vec::new(),
            delve_cards: Vec::new(),
            kicker_times: 0,
            escape_exile_cards: Vec::new(),
            retrace_discard_land: None,
            jump_start_discard: None,
            alt_cost: Some(AltCostKind::Morph),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: Vec::new(),
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: Vec::new(),
            modes_chosen: Vec::new(),
            fuse: false,
            x_value: 0,
            collect_evidence_cards: Vec::new(),
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    }
}
