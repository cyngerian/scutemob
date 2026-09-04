//! Combat state tracking: attackers, blockers, damage assignment (CR 506-511).
//!
//! `CombatState` is stored in `GameState::combat`. It is initialized when the
//! active player enters the `BeginningOfCombat` step and cleared at `EndOfCombat`.
use super::game_object::ObjectId;
use super::player::PlayerId;
use imbl::{OrdMap, OrdSet};
use serde::{Deserialize, Serialize};
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
    /// CR 508.1 (PB-DX21, OOS-M11-9): whether the active player has performed the
    /// once-per-combat "declare attackers" turn-based action this combat phase.
    ///
    /// `true` even for an **empty** declaration: CR 508.1a's "if any" makes the
    /// empty choice a completed declaration. Do **not** replace this field with
    /// `!attackers.is_empty()` -- see plan `memory/primitives/pb-plan-DX21.md`
    /// §1.3 for the three CR-grounded reasons that guard is unsound (an empty
    /// declaration is a live, shipped client action; a rejected re-declaration
    /// must not be indistinguishable from "no declaration yet"; and CR
    /// 508.4/506.3 populate `attackers` directly, bypassing declaration
    /// entirely).
    ///
    /// **This field is NOT read by CR 508.8's skip predicate, and does not close
    /// that residue** (PB-DX21 review, finding M1). `rules::turn_structure::
    /// advance_step` decides "skip declare-blockers and combat-damage" from a
    /// STEP-END read of `combat.attackers.is_empty()` (`turn_structure.rs:43-47`
    /// at the time of writing), not from this marker -- a pre-existing deviation
    /// from CR 508.8's own declaration-TIME predicate (declare one attacker, then
    /// remove it from combat before the step ends, and the skip still fires even
    /// though creatures WERE declared this combat). This field answers a
    /// different question ("was the CR 508.1 turn-based action PERFORMED") from
    /// "how many attackers survive to step end", and the two must not be
    /// conflated. See `docs/audits/decision-point-audit.md` for the tracking id.
    ///
    /// Creatures **put onto the battlefield attacking** (CR 508.4, e.g. Ninjutsu)
    /// populate `attackers` without ever setting this flag -- CR 508.4 says such
    /// creatures "never *attacked*". See `effects/mod.rs` (the `enters_attacking`
    /// handler), which inserts into `attackers` directly and never calls
    /// `handle_declare_attackers`.
    ///
    /// This is a `bool`, unlike its sibling `defenders_declared` below
    /// (`OrdSet<PlayerId>`), because CR 508.1 gives the declare-attackers action
    /// to exactly one player -- the active player, already named by
    /// `attacking_player` above -- while CR 509.1a lets each defending player
    /// declare blockers independently.
    ///
    /// Cleared naturally when `CombatState` is dropped at `EndOfCombat` and
    /// rebuilt fresh (`false`) at the next `BeginningOfCombat`, so this marker is
    /// per **combat phase** (CR 500.8 / 506.5), not per turn -- an extra combat
    /// phase (e.g. Aurelia, the Warleader) gets its own fresh declaration.
    ///
    /// `#[serde(default)]`: an older serialized `CombatState` deserializes as
    /// `false` ("the declaration has not been performed"). This is a deliberate,
    /// lossy default -- an old snapshot resumed mid-combat permits one extra
    /// declaration (`OOS-DX21-4`).
    #[serde(default)]
    pub attackers_declared: bool,
    /// CR 508.8 (PB-DX51, `OOS-DX21-4`): whether any creature has been **declared as
    /// an attacker** (CR 508.1) **or put onto the battlefield attacking** (CR 508.4)
    /// during this combat phase.
    ///
    /// CR 508.8 reads verbatim: *"If no creatures are declared as attackers or put
    /// onto the battlefield attacking, skip the declare blockers and combat damage
    /// steps."* That predicate is a **historical** fact about two events, not a
    /// question about what is in combat now -- which is exactly why
    /// `rules::turn_structure::advance_step` must not decide the skip from
    /// `attackers.is_empty()` alone. CR 506.4 removes a creature from combat when it
    /// leaves the battlefield, phases out, changes controller or stops being a
    /// creature, so an instant-speed answer to a lone attacker empties `attackers`
    /// while the declare-attackers step is still open; before PB-DX51 the engine then
    /// skipped declare-blockers and combat-damage in a combat where creatures **were**
    /// declared, taking every other creature's block, every later CR 508.4 entrant and
    /// the whole of CR 510 with it.
    ///
    /// **Monotone: set `true`, never cleared.** That is the fix. `remove_from_combat`
    /// (CR 506.4) deliberately does not unset it.
    ///
    /// **This is NOT `attackers_declared`, and the two must not be merged.** CR 508.1a's
    /// *"if any"* makes an **empty** declaration a completed declaration, so
    /// `attackers_declared` is `true` for it while CR 508.8 still demands the skip;
    /// conversely a CR 508.4 entrant sets this marker while never setting
    /// `attackers_declared`, because CR 508.4 says such creatures *"never attacked"*.
    /// Neither field is derivable from the other.
    ///
    /// Maintained by exactly one mutator, [`CombatState::add_attacker`] -- the only
    /// place in production that writes `attackers` -- so a sixth entry site cannot
    /// forget it. `crates/engine/tests/core/pb_dx51_attacker_entry_roster.rs` (`r1`)
    /// is the gate.
    ///
    /// Per **combat phase**, not per turn: `CombatState` is dropped at `EndOfCombat`
    /// and rebuilt `false` at the next `BeginningOfCombat` (CR 500.8 / 506.5), so an
    /// extra combat phase gets its own answer -- the same scoping as
    /// `attackers_declared` above.
    ///
    /// `#[serde(default)]`: an older serialized `CombatState` deserialises as `false`,
    /// i.e. *"nothing was declared"*, which is lossy in the **skip-happy** direction
    /// for a snapshot resumed mid-combat. Same class as `OOS-DX21-3`; filed as
    /// `OOS-DX51-1`.
    #[serde(default)]
    pub had_attackers: bool,
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
    /// CR 701.43d / CR 508.1g: Attackers exerted as an optional attack cost this combat.
    ///
    /// Populated during `handle_declare_attackers()`. Used by `abilities::check_triggers`
    /// to queue each attacker's card-def `TriggerCondition::WhenExertedAsAttacks` linked
    /// trigger (CR 607.2h) -- ONLY for attackers in this set, not on every attack.
    /// Cleared naturally when CombatState is dropped at end of combat.
    pub exerted_attackers: OrdSet<ObjectId>,
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
            attackers_declared: false,
            had_attackers: false,
            defenders_declared: OrdSet::new(),
            forced_blocks: OrdMap::new(),
            enlist_pairings: Vec::new(),
            blocked_attackers: OrdSet::new(),
            exerted_attackers: OrdSet::new(),
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
    /// CR 508.1 / CR 508.4 (PB-DX51, `OOS-DX21-4`): the **only** production path that
    /// puts a creature into `attackers`.
    ///
    /// Both routes into combat go through here -- the CR 508.1 declaration loop in
    /// `rules::combat::handle_declare_attackers`, and each of the four CR 508.4
    /// "put onto the battlefield attacking" sites (`effects::mod`'s two token paths,
    /// `resolution`'s Myriad CR 702.116a and Ninjutsu CR 702.49a paths). Writing them
    /// as one mutator is what makes CR 508.8's `had_attackers` marker impossible to
    /// forget at a fifth site; `pb_dx51_attacker_entry_roster::r1` fails if any
    /// production file spells `.attackers.insert(` outside this method.
    ///
    /// **An EMPTY declaration needs no special case, and that is why one mutator
    /// serves both CR rules**: a declaration of zero attackers never enters its loop,
    /// so `had_attackers` stays `false` and CR 508.8's skip still fires (CR 508.1a's
    /// *"if any"*).
    ///
    /// Does **not** set `attackers_declared` -- that marks the CR 508.1 turn-based
    /// action, which a CR 508.4 entrant never performs (CR 508.4: such creatures
    /// *"never attacked"*). Its caller sets it.
    pub fn add_attacker(&mut self, id: ObjectId, target: AttackTarget) {
        self.attackers.insert(id, target);
        // CR 508.8: monotone. Never cleared by CR 506.4 removal -- see the field doc.
        self.had_attackers = true;
    }

    /// PB-XA2: Returns `true` if `id` is currently declared as a blocker
    /// (CR 509.1c — `id` keys into `CombatState.blockers`).
    ///
    /// Distinct from `is_blocked(attacker_id)` — this checks whether `id`
    /// IS a blocker, not whether `id` IS BLOCKED. Used by
    /// `TargetFilter.is_blocking` enforcement at validate sites and the
    /// trigger auto-target picker.
    pub fn is_blocking(&self, id: ObjectId) -> bool {
        self.blockers.contains_key(&id)
    }
    /// PB-AC3: Returns true if `id` is currently declared as an attacker
    /// (CR 508.1 — keys into `CombatState.attackers`).
    pub fn is_attacking(&self, id: ObjectId) -> bool {
        self.attackers.contains_key(&id)
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
