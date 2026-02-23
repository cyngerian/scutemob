//! Game state model: zones, objects, players, and the core GameState struct.
//!
//! All state uses `im` persistent data structures for structural sharing,
//! enabling cheap snapshots and deterministic replay.

pub mod builder;
pub mod combat;
pub mod continuous_effect;
pub mod error;
pub mod game_object;
pub mod hash;
pub mod player;
pub mod replacement_effect;
pub mod stack;
pub mod stubs;
pub mod targeting;
pub mod turn;
pub mod types;
pub mod zone;

// Re-export primary types for convenient access via `use mtg_engine::state::*`
pub use builder::{
    register_commander_zone_replacements, GameStateBuilder, ObjectSpec, PlayerBuilder,
};
pub use combat::{AttackTarget, CombatState};
pub use continuous_effect::{
    ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
pub use error::GameStateError;
pub use game_object::{
    AbilityInstance, ActivatedAbility, ActivationCost, Characteristics, GameObject, InterveningIf,
    ManaAbility, ManaCost, ObjectId, ObjectStatus, TriggerEvent, TriggeredAbilityDef,
};
pub use player::{CardId, ManaPool, PlayerId, PlayerState};
pub use replacement_effect::{
    DamageTargetFilter, ObjectFilter, PendingZoneChange, PlayerFilter, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger,
};
pub use stack::{StackObject, StackObjectKind};
pub use stubs::{DelayedTrigger, PendingTrigger};
pub use targeting::{SpellTarget, Target};
pub use turn::{Phase, Step, TurnState};
pub use types::{CardType, Color, CounterType, KeywordAbility, ManaColor, SubType, SuperType};
pub use zone::{Zone, ZoneId, ZoneType};

use std::sync::Arc;

use im::{OrdMap, Vector};
use serde::{Deserialize, Serialize};

use crate::cards::CardRegistry;
use crate::rules::events::GameEvent;

/// The complete state of an MTG game at a point in time.
///
/// Uses `im` persistent data structures for O(1) cloning via structural sharing.
/// All state transitions produce new `GameState` values; old values are retained
/// for undo, replay, and "what if" analysis.
///
/// See architecture doc Section 2.1 for the full field listing and rationale.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    /// Current turn/phase/step/priority state.
    pub turn: TurnState,
    /// All players indexed by PlayerId.
    pub players: OrdMap<PlayerId, PlayerState>,
    /// All zones indexed by ZoneId.
    pub zones: OrdMap<ZoneId, Zone>,
    /// All game objects indexed by ObjectId.
    pub objects: OrdMap<ObjectId, GameObject>,
    /// Active continuous effects (CR 611). Applied via the layer system.
    pub continuous_effects: Vector<ContinuousEffect>,
    /// Delayed triggers waiting for conditions (CR 603.7).
    pub delayed_triggers: Vector<DelayedTrigger>,
    /// Active replacement effects (CR 614).
    pub replacement_effects: Vector<ReplacementEffect>,
    /// Monotonic counter for generating ReplacementIds.
    pub next_replacement_id: u64,
    /// Zone changes waiting for player choice among replacement effects (CR 616.1).
    /// SBA loop skips objects with pending entries; resolved by `OrderReplacements`.
    pub pending_zone_changes: Vector<PendingZoneChange>,
    /// Commanders awaiting the owner's zone-return choice (CR 903.9a).
    ///
    /// Each entry is `(owner, object_id)`. The SBA skips commanders already in
    /// this list so the choice event is not re-emitted every SBA pass.
    /// Cleared when the owner sends `ReturnCommanderToCommandZone` or
    /// `LeaveCommanderInZone`.
    pub pending_commander_zone_choices: Vector<(PlayerId, ObjectId)>,
    /// Prevention shield counters: remaining capacity for `PreventDamage(n)` effects (CR 615.7).
    /// Keyed by ReplacementId. When a counter reaches 0 the corresponding ReplacementEffect
    /// is removed from `replacement_effects`. `PreventAllDamage` effects need no counter.
    pub prevention_counters: im::OrdMap<ReplacementId, u32>,
    /// Triggered abilities waiting to be put on the stack.
    pub pending_triggers: Vector<PendingTrigger>,
    /// Stack objects (spells and abilities on the stack).
    pub stack_objects: Vector<StackObject>,
    /// Current combat state, if in a combat phase.
    pub combat: Option<CombatState>,
    /// Monotonic counter for generating ObjectIds and timestamps.
    pub timestamp_counter: u64,
    /// Append-only event log for triggers that look back at history.
    pub history: Vector<GameEvent>,
    /// Card definitions registry: maps CardId → CardDefinition.
    ///
    /// Static data, never changes during a game. Held as `Arc` so state clones
    /// share the registry without copying it. Excluded from state hashing and
    /// serialization (reconstructed from the card database on load).
    #[serde(skip)]
    pub card_registry: Arc<CardRegistry>,
}

impl GameState {
    /// Generates the next unique ObjectId, incrementing the timestamp counter.
    pub fn next_object_id(&mut self) -> ObjectId {
        self.timestamp_counter += 1;
        ObjectId(self.timestamp_counter)
    }

    /// Returns the current timestamp value (for continuous effect ordering).
    pub fn current_timestamp(&self) -> u64 {
        self.timestamp_counter
    }

    /// Generates the next unique ReplacementId, incrementing the counter.
    pub fn next_replacement_id(&mut self) -> ReplacementId {
        let id = ReplacementId(self.next_replacement_id);
        self.next_replacement_id += 1;
        id
    }

    /// Look up a player by ID.
    pub fn player(&self, id: PlayerId) -> Result<&PlayerState, GameStateError> {
        self.players
            .get(&id)
            .ok_or(GameStateError::PlayerNotFound(id))
    }

    /// Look up a mutable player by ID.
    pub fn player_mut(&mut self, id: PlayerId) -> Result<&mut PlayerState, GameStateError> {
        self.players
            .get_mut(&id)
            .ok_or(GameStateError::PlayerNotFound(id))
    }

    /// Look up a game object by ID.
    pub fn object(&self, id: ObjectId) -> Result<&GameObject, GameStateError> {
        self.objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))
    }

    /// Look up a mutable game object by ID.
    pub fn object_mut(&mut self, id: ObjectId) -> Result<&mut GameObject, GameStateError> {
        self.objects
            .get_mut(&id)
            .ok_or(GameStateError::ObjectNotFound(id))
    }

    /// Look up a zone by ID.
    pub fn zone(&self, id: &ZoneId) -> Result<&Zone, GameStateError> {
        self.zones.get(id).ok_or(GameStateError::ZoneNotFound(*id))
    }

    /// Look up a mutable zone by ID.
    pub fn zone_mut(&mut self, id: &ZoneId) -> Result<&mut Zone, GameStateError> {
        self.zones
            .get_mut(id)
            .ok_or(GameStateError::ZoneNotFound(*id))
    }

    /// Add a new game object to a zone, assigning it a fresh ObjectId and timestamp.
    /// Returns the assigned ObjectId.
    pub fn add_object(
        &mut self,
        mut object: GameObject,
        zone_id: ZoneId,
    ) -> Result<ObjectId, GameStateError> {
        let id = self.next_object_id();
        object.id = id;
        object.zone = zone_id;
        object.timestamp = self.timestamp_counter;

        // Add to zone — MR-M1-01/MR-M1-04: single access, no redundant guard.
        let zone = self
            .zones
            .get_mut(&zone_id)
            .ok_or(GameStateError::ZoneNotFound(zone_id))?;
        zone.insert(id);

        // Add to objects map
        self.objects.insert(id, object);
        Ok(id)
    }

    /// Move a game object from its current zone to a new zone.
    ///
    /// Implements CR 400.7: "An object that moves from one zone to another becomes
    /// a new object with no memory of, or relation to, its previous existence."
    ///
    /// The old ObjectId is retired. A new ObjectId is assigned. Status, counters,
    /// attachments, and controller are reset. Returns the new ObjectId and a
    /// snapshot of the old object (for trigger processing).
    pub fn move_object_to_zone(
        &mut self,
        object_id: ObjectId,
        to: ZoneId,
    ) -> Result<(ObjectId, GameObject), GameStateError> {
        // MR-M0-13: Validate destination zone exists BEFORE any mutation.
        // Previously the source was removed before checking the destination,
        // leaving the state corrupt if the destination zone was missing.
        if !self.zones.contains_key(&to) {
            return Err(GameStateError::ZoneNotFound(to));
        }

        // Look up the current object
        let old_object = self
            .objects
            .get(&object_id)
            .ok_or(GameStateError::ObjectNotFound(object_id))?
            .clone();
        let from = old_object.zone;

        // Remove from old zone
        let from_zone = self
            .zones
            .get_mut(&from)
            .ok_or(GameStateError::ZoneNotFound(from))?;
        if !from_zone.remove(&object_id) {
            return Err(GameStateError::ObjectNotInZone(object_id, from));
        }

        // Remove old object from objects map
        self.objects.remove(&object_id);

        // Create new object with fresh ID (CR 400.7)
        let new_id = self.next_object_id();
        let new_object = GameObject {
            id: new_id,
            card_id: old_object.card_id.clone(),
            characteristics: old_object.characteristics.clone(),
            controller: old_object.owner, // Controller resets to owner
            owner: old_object.owner,
            zone: to,
            status: ObjectStatus::default(),
            counters: OrdMap::new(),
            attachments: Vector::new(),
            attached_to: None,
            damage_marked: 0,
            deathtouch_damage: false,
            is_token: old_object.is_token,
            timestamp: self.timestamp_counter,
            // CR 302.6: a permanent entering the battlefield has summoning sickness
            // until the beginning of its controller's next untap step.
            has_summoning_sickness: to == ZoneId::Battlefield,
            enchants_creatures: old_object.enchants_creatures,
        };

        // Add to new zone — MR-M1-02/MR-M1-04: single access, no redundant guard.
        let to_zone = self
            .zones
            .get_mut(&to)
            .ok_or(GameStateError::ZoneNotFound(to))?;
        to_zone.insert(new_id);

        // Insert new object
        self.objects.insert(new_id, new_object);

        Ok((new_id, old_object))
    }

    /// Returns all active (non-lost, non-conceded) player IDs in turn order.
    pub fn active_players(&self) -> Vec<PlayerId> {
        self.turn
            .turn_order
            .iter()
            .filter(|id| {
                self.players
                    .get(id)
                    .map(|p| !p.has_lost && !p.has_conceded)
                    .unwrap_or(false)
            })
            .copied()
            .collect()
    }

    /// Returns the total number of game objects across all zones.
    pub fn total_objects(&self) -> usize {
        self.objects.len()
    }

    /// Find all objects in a given zone.
    pub fn objects_in_zone(&self, zone_id: &ZoneId) -> Vec<&GameObject> {
        match self.zones.get(zone_id) {
            Some(zone) => zone
                .object_ids()
                .iter()
                .filter_map(|id| self.objects.get(id))
                .collect(),
            None => Vec::new(),
        }
    }
}
