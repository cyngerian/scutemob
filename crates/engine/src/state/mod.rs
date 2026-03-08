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
    AbilityInstance, ActivatedAbility, ActivationCost, Characteristics, ETBTriggerFilter,
    GameObject, InterveningIf, ManaAbility, ManaCost, ObjectId, ObjectStatus, TriggerEvent,
    TriggeredAbilityDef,
};
pub use player::{CardId, ManaPool, PlayerId, PlayerState};
pub use replacement_effect::{
    DamageTargetFilter, ObjectFilter, PendingZoneChange, PlayerFilter, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger,
};
pub use stack::{StackObject, StackObjectKind};
pub use stubs::{DelayedTrigger, PendingTrigger, TriggerDoubler, TriggerDoublerFilter};
pub use targeting::{SpellTarget, Target};
pub use turn::{Phase, Step, TurnState};
pub use types::{
    AffinityTarget, CardType, ChampionFilter, Color, CounterType, CumulativeUpkeepCost,
    EnchantTarget, KeywordAbility, LandwalkType, ManaColor, ProtectionQuality, SubType, SuperType,
};
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
    /// Active trigger-doubling effects (Panharmonicon-style, CR 603.2d).
    ///
    /// When a trigger that matches any doubler's filter is about to be queued,
    /// it is queued `additional_triggers` additional times. Entries are added
    /// when a permanent with a trigger-doubling ability enters the battlefield
    /// and removed when that permanent leaves.
    pub trigger_doublers: Vector<TriggerDoubler>,
    /// Stack objects (spells and abilities on the stack).
    pub stack_objects: Vector<StackObject>,
    /// Current combat state, if in a combat phase.
    pub combat: Option<CombatState>,
    /// Monotonic counter for generating ObjectIds and timestamps.
    pub timestamp_counter: u64,
    /// Tracks state hash occurrences for mandatory infinite loop detection (CR 104.4b).
    ///
    /// Maps a truncated game-state hash (u64) to the number of times that hash has
    /// been seen during the current mandatory-action sequence (SBA + trigger cycles).
    /// Reset whenever a player makes a meaningful game choice.
    ///
    /// Excluded from `public_state_hash` — this is metadata, not game state.
    /// See `rules/loop_detection.rs` for the detection algorithm.
    pub loop_detection_hashes: im::OrdMap<u64, u32>,
    /// Append-only event log for triggers that look back at history.
    pub history: Vector<GameEvent>,
    /// CR 702.69a: Global count of permanents put into a graveyard from the battlefield
    /// this turn, across all players and including tokens.
    ///
    /// Used by Gravestorm to determine how many copies to create. Incremented in
    /// `move_object_to_zone` / `move_object_to_bottom_of_zone` when `from ==
    /// ZoneId::Battlefield` and `to == ZoneId::Graveyard(_)`. Reset by
    /// `reset_turn_state` at the start of each turn.
    ///
    /// Tokens count: they briefly exist in the graveyard (CR 704.5d) before ceasing to
    /// exist as an SBA. The increment happens before any subsequent SBA check.
    #[serde(default)]
    pub permanents_put_into_graveyard_this_turn: u32,
    /// CR 702.30a: Pending echo payment choices.
    ///
    /// When an EchoTrigger resolves, the controller must choose to pay or sacrifice.
    /// The game pauses until a `Command::PayEcho` is received for each entry.
    /// Each entry is `(player, permanent_id, echo_cost)`.
    ///
    /// Only one echo payment can be pending at a time (triggers resolve one at a time
    /// from the stack), but using `Vector` is consistent with other pending-choice patterns.
    #[serde(default)]
    pub pending_echo_payments: im::Vector<(PlayerId, ObjectId, ManaCost)>,
    /// CR 702.24a: Pending cumulative upkeep payment choices.
    ///
    /// When a CumulativeUpkeepTrigger resolves (after adding the age counter), the
    /// controller must choose to pay or sacrifice. The game pauses until a
    /// `Command::PayCumulativeUpkeep` is received for each entry.
    /// Each entry is `(player, permanent_id, per_counter_cost)`.
    #[serde(default)]
    pub pending_cumulative_upkeep_payments: im::Vector<(
        PlayerId,
        ObjectId,
        crate::state::types::CumulativeUpkeepCost,
    )>,
    /// CR 702.59a: Pending recover payment choices.
    ///
    /// When a RecoverTrigger resolves, the controller must choose to pay the
    /// recover cost or exile the card. The game pauses until a
    /// `Command::PayRecover` is received for each entry.
    /// Each entry is `(player, recover_card_id, recover_cost)`.
    #[serde(default)]
    pub pending_recover_payments: im::Vector<(PlayerId, ObjectId, ManaCost)>,
    /// CR 702.57b: Cards that have activated their forecast ability this turn.
    ///
    /// Keyed by CardId (not ObjectId) since the card stays in hand and retains
    /// its identity. Reset at the start of each turn in `reset_turn_state`.
    /// Each forecast ability can be activated at most once per turn (CR 702.57b).
    #[serde(default)]
    pub forecast_used_this_turn: im::OrdSet<crate::state::player::CardId>,
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
        let mut new_object = GameObject {
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
            // CR 400.7: goad state is not preserved across zone changes.
            goaded_by: im::Vector::new(),
            // CR 400.7: kicked status is not preserved across zone changes
            // (a permanent re-entering is not kicked).
            kicker_times_paid: 0,
            // CR 400.7: alt-cost status (evoke/escape/dash) is not preserved across zone changes.
            cast_alt_cost: None,
            // CR 400.7: bestow status is not preserved across zone changes.
            is_bestowed: false,
            // CR 400.7: foretold status is not preserved across zone changes.
            is_foretold: false,
            foretold_turn: 0,
            // CR 400.7: unearth status is not preserved across zone changes.
            was_unearthed: false,
            // CR 400.7: myriad token exile flag is not preserved across zone changes.
            myriad_exile_at_eoc: false,
            // CR 400.7: decayed sacrifice flag is not preserved across zone changes.
            decayed_sacrifice_at_eoc: false,
            // CR 400.7: suspend status is not preserved across zone changes.
            is_suspended: false,
            // CR 400.7: hideaway exile link is cleared on zone change.
            exiled_by_hideaway: None,
            // CR 400.7: renowned designation is not preserved across zone changes (CR 702.112b).
            is_renowned: false,
            // CR 701.60a / CR 400.7: suspected designation is cleared on zone change.
            is_suspected: false,
            // CR 400.7: encore sacrifice flag is not preserved across zone changes.
            encore_sacrifice_at_end_step: false,
            // CR 400.7: encore mandatory attack target is not preserved across zone changes.
            encore_must_attack: None,
            // CR 400.7: encore original activator is not preserved across zone changes.
            encore_activated_by: None,
            // CR 400.7: plot status is not preserved across zone changes.
            is_plotted: false,
            plotted_turn: 0,
            is_prototyped: false,
            // CR 400.7: bargained status is not preserved across zone changes.
            was_bargained: false,
            // CR 400.7: collect evidence status is not preserved across zone changes.
            evidence_collected: false,
            // CR 400.7: echo pending flag is not preserved across zone changes.
            echo_pending: false,
            // CR 400.7: phasing flags are not preserved across zone changes.
            phased_out_indirectly: false,
            phased_out_controller: None,
            // CR 400.7: devour count is not preserved across zone changes.
            creatures_devoured: 0,
            // CR 702.72a / CR 603.10a: champion_exiled_card is preserved across zone changes
            // so the LTB trigger can read it from the post-move object (last-known information).
            champion_exiled_card: old_object.champion_exiled_card,
            // CR 702.95e / CR 400.7: soulbond pairing is broken on zone change (new object identity).
            paired_with: None,
            // CR 400.7: tribute_was_paid is not preserved across zone changes.
            tribute_was_paid: false,
            // CR 107.3m / CR 400.7: x_value is not preserved across zone changes.
            x_value: 0,
            // CR 702.157a / CR 400.7: squad_count is not preserved across zone changes.
            squad_count: 0,
            // CR 702.175a / CR 400.7: offspring_paid is not preserved across zone changes.
            offspring_paid: false,
            // CR 702.174a / CR 400.7: gift status is not preserved across zone changes.
            gift_was_given: false,
            gift_opponent: None,
            // CR 702.171b / CR 400.7: saddled designation is cleared on zone change.
            is_saddled: false,
            // CR 702.99b / CR 400.7: encoded cipher cards are cleared on zone change.
            // The exiled cards remain in exile but are no longer encoded on anything.
            // CR 702.99c: encoding is broken when the creature leaves the battlefield.
            encoded_cards: im::Vector::new(),
        };

        // CR 702.95e: If the departing object was paired, clear the partner's paired_with.
        // We already have old_object.paired_with from the clone taken before removal.
        if let Some(partner_id) = old_object.paired_with {
            if let Some(partner) = self.objects.get_mut(&partner_id) {
                partner.paired_with = None;
            }
        }

        // CR 718.4: When a prototyped permanent leaves the battlefield to any zone
        // that is not the stack or battlefield, revert characteristics to the card's
        // printed values. The prototype-modified P/T, mana_cost, and colors were written
        // into the base characteristics at cast time and must be undone here.
        if old_object.is_prototyped && to != ZoneId::Battlefield && to != ZoneId::Stack {
            if let Some(ref cid) = new_object.card_id {
                if let Some(def) = self.card_registry.get(cid.clone()) {
                    new_object.characteristics.power = def.power;
                    new_object.characteristics.toughness = def.toughness;
                    new_object.characteristics.mana_cost = def.mana_cost.clone();
                    // CR 105.2: colors are derived from the printed mana cost.
                    new_object.characteristics.colors = if let Some(ref mc) = def.mana_cost {
                        crate::rules::casting::colors_from_mana_cost(mc)
                    } else {
                        im::OrdSet::new()
                    };
                }
            }
        }

        // Add to new zone — MR-M1-02/MR-M1-04: single access, no redundant guard.
        let to_zone = self
            .zones
            .get_mut(&to)
            .ok_or(GameStateError::ZoneNotFound(to))?;
        to_zone.insert(new_id);

        // Insert new object
        self.objects.insert(new_id, new_object);

        // CR 702.69a: Track permanents entering a graveyard from the battlefield.
        // Tokens count (CR 704.5d — they briefly exist in the graveyard before SBA removes them).
        // Non-permanent cards (instants, sorceries) go from the stack, not the battlefield, so
        // they are naturally excluded by the ZoneId::Battlefield source check.
        if old_object.zone == ZoneId::Battlefield {
            if let ZoneId::Graveyard(_) = to {
                self.permanents_put_into_graveyard_this_turn += 1;
            }
        }

        Ok((new_id, old_object))
    }

    /// Move an object to the bottom of an ordered zone (CR 702.85a).
    ///
    /// Identical to `move_object_to_zone` but inserts the new object at
    /// position 0 (the bottom) instead of appending to the back (the top).
    /// For unordered zones, behaves the same as `move_object_to_zone`.
    pub fn move_object_to_bottom_of_zone(
        &mut self,
        object_id: ObjectId,
        to: ZoneId,
    ) -> Result<(ObjectId, GameObject), GameStateError> {
        // Validate destination zone before mutating.
        if !self.zones.contains_key(&to) {
            return Err(GameStateError::ZoneNotFound(to));
        }

        let old_object = self
            .objects
            .get(&object_id)
            .ok_or(GameStateError::ObjectNotFound(object_id))?
            .clone();
        let from = old_object.zone;

        // Remove from old zone.
        let from_zone = self
            .zones
            .get_mut(&from)
            .ok_or(GameStateError::ZoneNotFound(from))?;
        if !from_zone.remove(&object_id) {
            return Err(GameStateError::ObjectNotInZone(object_id, from));
        }

        // Remove old object from objects map.
        self.objects.remove(&object_id);

        // Create new object with fresh ID (CR 400.7).
        let new_id = self.next_object_id();
        let mut new_object = GameObject {
            id: new_id,
            card_id: old_object.card_id.clone(),
            characteristics: old_object.characteristics.clone(),
            controller: old_object.owner,
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
            has_summoning_sickness: to == ZoneId::Battlefield,
            goaded_by: im::Vector::new(),
            // CR 400.7: kicked status is not preserved across zone changes.
            kicker_times_paid: 0,
            // CR 400.7: alt-cost status (evoke/escape/dash) is not preserved across zone changes.
            cast_alt_cost: None,
            // CR 400.7: bestow status is not preserved across zone changes.
            is_bestowed: false,
            // CR 400.7: foretold status is not preserved across zone changes.
            is_foretold: false,
            foretold_turn: 0,
            // CR 400.7: unearth status is not preserved across zone changes.
            was_unearthed: false,
            // CR 400.7: myriad token exile flag is not preserved across zone changes.
            myriad_exile_at_eoc: false,
            // CR 400.7: decayed sacrifice flag is not preserved across zone changes.
            decayed_sacrifice_at_eoc: false,
            // CR 400.7: suspend status is not preserved across zone changes.
            is_suspended: false,
            // CR 400.7: hideaway exile link is cleared on zone change.
            exiled_by_hideaway: None,
            // CR 400.7: renowned designation is not preserved across zone changes (CR 702.112b).
            is_renowned: false,
            // CR 701.60a / CR 400.7: suspected designation is cleared on zone change.
            is_suspected: false,
            // CR 400.7: encore sacrifice flag is not preserved across zone changes.
            encore_sacrifice_at_end_step: false,
            // CR 400.7: encore mandatory attack target is not preserved across zone changes.
            encore_must_attack: None,
            // CR 400.7: encore original activator is not preserved across zone changes.
            encore_activated_by: None,
            // CR 400.7: plot status is not preserved across zone changes.
            is_plotted: false,
            plotted_turn: 0,
            is_prototyped: false,
            // CR 400.7: bargained status is not preserved across zone changes.
            was_bargained: false,
            // CR 400.7: collect evidence status is not preserved across zone changes.
            evidence_collected: false,
            // CR 400.7: echo pending flag is not preserved across zone changes.
            echo_pending: false,
            // CR 400.7: phasing flags are not preserved across zone changes.
            phased_out_indirectly: false,
            phased_out_controller: None,
            // CR 400.7: devour count is not preserved across zone changes.
            creatures_devoured: 0,
            // CR 702.72a / CR 603.10a: champion_exiled_card is preserved across zone changes
            // so the LTB trigger can read it from the post-move object (last-known information).
            champion_exiled_card: old_object.champion_exiled_card,
            // CR 702.95e / CR 400.7: soulbond pairing is broken on zone change (new object identity).
            paired_with: None,
            // CR 400.7: tribute_was_paid is not preserved across zone changes.
            tribute_was_paid: false,
            // CR 107.3m / CR 400.7: x_value is not preserved across zone changes.
            x_value: 0,
            // CR 702.157a / CR 400.7: squad_count is not preserved across zone changes.
            squad_count: 0,
            // CR 702.175a / CR 400.7: offspring_paid is not preserved across zone changes.
            offspring_paid: false,
            // CR 702.174a / CR 400.7: gift status is not preserved across zone changes.
            gift_was_given: false,
            gift_opponent: None,
            // CR 702.171b / CR 400.7: saddled designation is cleared on zone change.
            is_saddled: false,
            // CR 702.99b / CR 400.7: encoded cipher cards are cleared on zone change.
            // The exiled cards remain in exile but are no longer encoded on anything.
            // CR 702.99c: encoding is broken when the creature leaves the battlefield.
            encoded_cards: im::Vector::new(),
        };

        // CR 702.95e: If the departing object was paired, clear the partner's paired_with.
        if let Some(partner_id) = old_object.paired_with {
            if let Some(partner) = self.objects.get_mut(&partner_id) {
                partner.paired_with = None;
            }
        }

        // CR 718.4: When a prototyped permanent leaves the battlefield to any zone
        // that is not the stack or battlefield, revert characteristics to the card's
        // printed values.
        if old_object.is_prototyped && to != ZoneId::Battlefield && to != ZoneId::Stack {
            if let Some(ref cid) = new_object.card_id {
                if let Some(def) = self.card_registry.get(cid.clone()) {
                    new_object.characteristics.power = def.power;
                    new_object.characteristics.toughness = def.toughness;
                    new_object.characteristics.mana_cost = def.mana_cost.clone();
                    // CR 105.2: colors are derived from the printed mana cost.
                    new_object.characteristics.colors = if let Some(ref mc) = def.mana_cost {
                        crate::rules::casting::colors_from_mana_cost(mc)
                    } else {
                        im::OrdSet::new()
                    };
                }
            }
        }

        // Insert at the front (= bottom) of the destination zone.
        let to_zone = self
            .zones
            .get_mut(&to)
            .ok_or(GameStateError::ZoneNotFound(to))?;
        to_zone.push_front(new_id);

        // Insert new object.
        self.objects.insert(new_id, new_object);

        // CR 702.69a: Track permanents entering a graveyard from the battlefield.
        if old_object.zone == ZoneId::Battlefield {
            if let ZoneId::Graveyard(_) = to {
                self.permanents_put_into_graveyard_this_turn += 1;
            }
        }

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
