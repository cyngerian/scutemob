//! Combat state tracking: attackers, blockers, damage assignment (CR 506-511).
//!
//! `CombatState` is stored in `GameState::combat`. It is initialized when the
//! active player enters the `BeginningOfCombat` step and cleared at `EndOfCombat`.

use im::{OrdMap, OrdSet};
use serde::{Deserialize, Serialize};

use super::game_object::ObjectId;
use super::player::PlayerId;

/// An attack target: a player or a planeswalker permanent (CR 508.1).
///
/// In Commander, the active player may attack any opponent or an opponent's
/// controlled planeswalker (CR 903.6).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AttackTarget {
    /// Attacking a player directly.
    Player(PlayerId),
    /// Attacking a planeswalker on the battlefield.
    Planeswalker(ObjectId),
}

/// Complete state of the current combat phase (CR 506–511).
///
/// Populated on entry to the `BeginningOfCombat` step; cleared at `EndOfCombat`.
/// `GameState::combat` is `None` outside of the combat phase.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatState {
    /// The player whose turn it is (who declares attackers).
    pub attacking_player: PlayerId,

    /// Attacking creatures and their targets.
    /// Key: attacker `ObjectId`. Value: `AttackTarget`.
    pub attackers: OrdMap<ObjectId, AttackTarget>,

    /// Blocking creatures and the attacker each is blocking.
    /// Key: blocker `ObjectId`. Value: attacker `ObjectId` being blocked.
    pub blockers: OrdMap<ObjectId, ObjectId>,

    /// Active player's chosen damage assignment order for attackers with
    /// multiple blockers (CR 509.2).
    /// Key: attacker `ObjectId`. Value: ordered list of blocker `ObjectId`s
    /// (front = first to receive damage; must receive lethal before next).
    pub damage_assignment_order: OrdMap<ObjectId, Vec<ObjectId>>,

    /// Snapshot of creatures that had FirstStrike or DoubleStrike at the start
    /// of the first-strike damage step (CR 702.7b).
    ///
    /// Populated when `Step::FirstStrikeDamage` begins (before damage is applied).
    /// Used by `deals_damage_in_step` to determine regular-step eligibility based
    /// on keywords at snapshot time, not current keywords (CR 702.7c, 702.4c/d).
    ///
    /// Empty set = first-strike step has not yet occurred this combat.
    pub first_strike_participants: OrdSet<ObjectId>,

    /// Defending players who have already declared blockers this step.
    /// In multiplayer, each defending player declares independently (CR 509.1).
    pub defenders_declared: OrdSet<PlayerId>,

    /// CR 702.39a / CR 509.1c: Blocking requirements created by Provoke triggers.
    ///
    /// Each entry maps a provoked creature (ObjectId) to the attacker it must block
    /// (ObjectId of the provoking creature) "if able". Populated when a
    /// `StackObjectKind::KeywordTrigger` (Provoke) resolves. Checked in `handle_declare_blockers`
    /// to enforce CR 509.1c (blocking requirements cannot override restrictions).
    ///
    /// Cleared naturally when `CombatState` is dropped at end of combat.
    pub forced_blocks: OrdMap<ObjectId, ObjectId>,
    /// CR 702.154a: Enlist pairings made during declare-attackers.
    ///
    /// Each entry is (enlisting_attacker_id, enlisted_creature_id).
    /// Used by abilities.rs to fire EnlistTrigger for each pairing.
    /// Cleared naturally when CombatState is dropped at end of combat.
    pub enlist_pairings: Vec<(ObjectId, ObjectId)>,

    /// CR 509.1h: Attackers that had at least one blocker declared against them.
    ///
    /// Populated during `handle_declare_blockers()` and never modified afterward
    /// (entries are not removed when blockers leave the battlefield). This is
    /// distinct from `blockers`, which shrinks as blockers die/leave.
    ///
    /// `is_blocked()` checks this set so that a creature remains "blocked" even
    /// after all its blockers are removed from combat (CR 509.1h).
    pub blocked_attackers: OrdSet<ObjectId>,
}

impl CombatState {
    /// Create a fresh `CombatState` for the given attacking player.
    pub fn new(attacking_player: PlayerId) -> Self {
        Self {
            attacking_player,
            attackers: OrdMap::new(),
            blockers: OrdMap::new(),
            damage_assignment_order: OrdMap::new(),
            first_strike_participants: OrdSet::new(),
            defenders_declared: OrdSet::new(),
            forced_blocks: OrdMap::new(),
            enlist_pairings: Vec::new(),
            blocked_attackers: OrdSet::new(),
        }
    }

    /// Returns the blockers assigned to `attacker` in damage assignment order.
    ///
    /// Uses `damage_assignment_order` if set (via `OrderBlockers`); otherwise
    /// returns blockers in `OrdMap` (ObjectId) order.
    pub fn blockers_for(&self, attacker: ObjectId) -> Vec<ObjectId> {
        if let Some(order) = self.damage_assignment_order.get(&attacker) {
            order.clone()
        } else {
            self.blockers
                .iter()
                .filter(|(_, &a)| a == attacker)
                .map(|(&b, _)| b)
                .collect()
        }
    }

    /// Returns `true` if the attacker had at least one blocker declared against it.
    ///
    /// A creature remains "blocked" even if all its blockers are later removed
    /// from combat or destroyed (CR 509.1h). This checks `blocked_attackers`,
    /// which is set at declare-blockers time and never cleared (even when
    /// blockers die), not the live `blockers` map.
    pub fn is_blocked(&self, attacker: ObjectId) -> bool {
        self.blocked_attackers.contains(&attacker)
    }

    /// Returns the set of players being attacked directly (not via a planeswalker).
    pub fn players_being_attacked(&self) -> OrdSet<PlayerId> {
        let mut players = OrdSet::new();
        for target in self.attackers.values() {
            if let AttackTarget::Player(p) = target {
                players.insert(*p);
            }
        }
        players
    }
}
