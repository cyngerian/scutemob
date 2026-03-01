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

    /// Whether the first-strike damage step has already resolved this combat.
    /// Set to `true` after `Step::FirstStrikeDamage` executes. Used so that
    /// double-strike creatures deal damage in both steps.
    pub first_strike_damage_resolved: bool,

    /// Defending players who have already declared blockers this step.
    /// In multiplayer, each defending player declares independently (CR 509.1).
    pub defenders_declared: OrdSet<PlayerId>,

    /// CR 702.39a / CR 509.1c: Blocking requirements created by Provoke triggers.
    ///
    /// Each entry maps a provoked creature (ObjectId) to the attacker it must block
    /// (ObjectId of the provoking creature) "if able". Populated when a
    /// `StackObjectKind::ProvokeTrigger` resolves. Checked in `handle_declare_blockers`
    /// to enforce CR 509.1c (blocking requirements cannot override restrictions).
    ///
    /// Cleared naturally when `CombatState` is dropped at end of combat.
    pub forced_blocks: OrdMap<ObjectId, ObjectId>,
}

impl CombatState {
    /// Create a fresh `CombatState` for the given attacking player.
    pub fn new(attacking_player: PlayerId) -> Self {
        Self {
            attacking_player,
            attackers: OrdMap::new(),
            blockers: OrdMap::new(),
            damage_assignment_order: OrdMap::new(),
            first_strike_damage_resolved: false,
            defenders_declared: OrdSet::new(),
            forced_blocks: OrdMap::new(),
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

    /// Returns `true` if the attacker has at least one declared blocker.
    ///
    /// A creature remains "blocked" even if all its blockers are later removed
    /// from combat or destroyed (CR 509.1h). This tracks declaration, not
    /// current battlefield presence.
    pub fn is_blocked(&self, attacker: ObjectId) -> bool {
        self.blockers.values().any(|&a| a == attacker)
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
