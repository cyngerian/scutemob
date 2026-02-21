//! Deterministic state hashing for distributed verification (Tier 1).
//!
//! Provides `public_state_hash()` and `private_state_hash()` on `GameState`
//! for detecting non-determinism across independent engine instances.
//!
//! Uses blake3 for fast, cryptographically secure hashing. A custom `HashInto`
//! trait feeds fields into the hasher in deterministic order, giving explicit
//! control over which fields contribute to public vs private hashes.
//!
//! See `docs/mtg-engine-network-security.md` for the three-tier security model.

use blake3::Hasher;
use im::{OrdMap, OrdSet, Vector};

use super::game_object::{
    AbilityInstance, ActivatedAbility, ActivationCost, Characteristics, GameObject, InterveningIf,
    ManaAbility, ManaCost, ObjectId, ObjectStatus, TriggeredAbilityDef, TriggerEvent,
};
use super::player::{CardId, ManaPool, PlayerId, PlayerState};
use super::stack::{StackObject, StackObjectKind};
use super::stubs::{CombatState, ContinuousEffect, DelayedTrigger, PendingTrigger, ReplacementEffect};
use super::targeting::{SpellTarget, Target};
use super::turn::{Phase, Step, TurnState};
use super::types::{CardType, Color, CounterType, KeywordAbility, ManaColor, SubType, SuperType};
use super::zone::{Zone, ZoneId};
use super::GameState;
use crate::rules::events::{GameEvent, LossReason};

/// Feeds data into a `blake3::Hasher` in a deterministic, canonical order.
///
/// Unlike `std::hash::Hash`, this trait:
/// - Guarantees byte-level determinism across platforms and compiler versions
/// - Uses length-prefixed strings to prevent concatenation collisions
/// - Is explicitly implemented for each type (no derive) for full control
pub trait HashInto {
    fn hash_into(&self, hasher: &mut Hasher);
}

// --- Primitive implementations ---

impl HashInto for u8 {
    fn hash_into(&self, hasher: &mut Hasher) {
        hasher.update(&[*self]);
    }
}

impl HashInto for u32 {
    fn hash_into(&self, hasher: &mut Hasher) {
        hasher.update(&self.to_le_bytes());
    }
}

impl HashInto for u64 {
    fn hash_into(&self, hasher: &mut Hasher) {
        hasher.update(&self.to_le_bytes());
    }
}

impl HashInto for i32 {
    fn hash_into(&self, hasher: &mut Hasher) {
        hasher.update(&self.to_le_bytes());
    }
}

impl HashInto for usize {
    fn hash_into(&self, hasher: &mut Hasher) {
        // Always hash as u64 for cross-platform determinism
        (*self as u64).hash_into(hasher);
    }
}

impl HashInto for bool {
    fn hash_into(&self, hasher: &mut Hasher) {
        hasher.update(&[*self as u8]);
    }
}

impl HashInto for String {
    fn hash_into(&self, hasher: &mut Hasher) {
        // Length-prefix prevents concatenation collisions:
        // "ab" + "cd" hashes differently from "abc" + "d"
        (self.len() as u64).hash_into(hasher);
        hasher.update(self.as_bytes());
    }
}

impl HashInto for str {
    fn hash_into(&self, hasher: &mut Hasher) {
        (self.len() as u64).hash_into(hasher);
        hasher.update(self.as_bytes());
    }
}

// --- Generic container implementations ---

impl<T: HashInto> HashInto for Option<T> {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            None => 0u8.hash_into(hasher),
            Some(v) => {
                1u8.hash_into(hasher);
                v.hash_into(hasher);
            }
        }
    }
}

impl<T: HashInto> HashInto for Vec<T> {
    fn hash_into(&self, hasher: &mut Hasher) {
        (self.len() as u64).hash_into(hasher);
        for item in self {
            item.hash_into(hasher);
        }
    }
}

impl<T: HashInto + Clone> HashInto for Vector<T> {
    fn hash_into(&self, hasher: &mut Hasher) {
        (self.len() as u64).hash_into(hasher);
        for item in self {
            item.hash_into(hasher);
        }
    }
}

impl<T: HashInto + Ord + Clone> HashInto for OrdSet<T> {
    fn hash_into(&self, hasher: &mut Hasher) {
        (self.len() as u64).hash_into(hasher);
        for item in self {
            item.hash_into(hasher);
        }
    }
}

impl<K: HashInto + Ord + Clone, V: HashInto + Clone> HashInto for OrdMap<K, V> {
    fn hash_into(&self, hasher: &mut Hasher) {
        (self.len() as u64).hash_into(hasher);
        for (k, v) in self {
            k.hash_into(hasher);
            v.hash_into(hasher);
        }
    }
}

// --- Leaf type implementations (discriminant byte + payload) ---

impl HashInto for PlayerId {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.0.hash_into(hasher);
    }
}

impl HashInto for ObjectId {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.0.hash_into(hasher);
    }
}

impl HashInto for CardId {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.0.hash_into(hasher);
    }
}

impl HashInto for Color {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for ManaColor {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for SuperType {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for CardType {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for SubType {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.0.hash_into(hasher);
    }
}

impl HashInto for CounterType {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            CounterType::PlusOnePlusOne => 0u8.hash_into(hasher),
            CounterType::MinusOneMinusOne => 1u8.hash_into(hasher),
            CounterType::Loyalty => 2u8.hash_into(hasher),
            CounterType::Charge => 3u8.hash_into(hasher),
            CounterType::Energy => 4u8.hash_into(hasher),
            CounterType::Experience => 5u8.hash_into(hasher),
            CounterType::Level => 6u8.hash_into(hasher),
            CounterType::Lore => 7u8.hash_into(hasher),
            CounterType::Oil => 8u8.hash_into(hasher),
            CounterType::Poison => 9u8.hash_into(hasher),
            CounterType::Shield => 10u8.hash_into(hasher),
            CounterType::Stun => 11u8.hash_into(hasher),
            CounterType::Time => 12u8.hash_into(hasher),
            CounterType::Custom(s) => {
                13u8.hash_into(hasher);
                s.hash_into(hasher);
            }
        }
    }
}

impl HashInto for KeywordAbility {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for Phase {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for Step {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for ZoneId {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ZoneId::Library(p) => {
                0u8.hash_into(hasher);
                p.hash_into(hasher);
            }
            ZoneId::Hand(p) => {
                1u8.hash_into(hasher);
                p.hash_into(hasher);
            }
            ZoneId::Battlefield => 2u8.hash_into(hasher),
            ZoneId::Graveyard(p) => {
                3u8.hash_into(hasher);
                p.hash_into(hasher);
            }
            ZoneId::Stack => 4u8.hash_into(hasher),
            ZoneId::Exile => 5u8.hash_into(hasher),
            ZoneId::Command(p) => {
                6u8.hash_into(hasher);
                p.hash_into(hasher);
            }
        }
    }
}

impl HashInto for LossReason {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            LossReason::LifeTotal => 0u8.hash_into(hasher),
            LossReason::LibraryEmpty => 1u8.hash_into(hasher),
            LossReason::PoisonCounters => 2u8.hash_into(hasher),
            LossReason::CommanderDamage => 3u8.hash_into(hasher),
            LossReason::Conceded => 4u8.hash_into(hasher),
        }
    }
}

// --- Composite type implementations ---

impl HashInto for ManaCost {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.white.hash_into(hasher);
        self.blue.hash_into(hasher);
        self.black.hash_into(hasher);
        self.red.hash_into(hasher);
        self.green.hash_into(hasher);
        self.colorless.hash_into(hasher);
        self.generic.hash_into(hasher);
    }
}

impl HashInto for ManaPool {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.white.hash_into(hasher);
        self.blue.hash_into(hasher);
        self.black.hash_into(hasher);
        self.red.hash_into(hasher);
        self.green.hash_into(hasher);
        self.colorless.hash_into(hasher);
    }
}

impl HashInto for ObjectStatus {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.tapped.hash_into(hasher);
        self.flipped.hash_into(hasher);
        self.face_down.hash_into(hasher);
        self.phased_out.hash_into(hasher);
    }
}

impl HashInto for AbilityInstance {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.id.hash_into(hasher);
        self.description.hash_into(hasher);
    }
}

impl HashInto for ManaAbility {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.produces.hash_into(hasher);
        self.requires_tap.hash_into(hasher);
    }
}

impl HashInto for Characteristics {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.name.hash_into(hasher);
        self.mana_cost.hash_into(hasher);
        self.colors.hash_into(hasher);
        self.color_indicator.hash_into(hasher);
        self.supertypes.hash_into(hasher);
        self.card_types.hash_into(hasher);
        self.subtypes.hash_into(hasher);
        self.rules_text.hash_into(hasher);
        self.abilities.hash_into(hasher);
        self.keywords.hash_into(hasher);
        self.mana_abilities.hash_into(hasher);
        self.activated_abilities.hash_into(hasher);
        self.triggered_abilities.hash_into(hasher);
        self.power.hash_into(hasher);
        self.toughness.hash_into(hasher);
        self.loyalty.hash_into(hasher);
        self.defense.hash_into(hasher);
    }
}

impl HashInto for GameObject {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.id.hash_into(hasher);
        self.card_id.hash_into(hasher);
        self.characteristics.hash_into(hasher);
        self.controller.hash_into(hasher);
        self.owner.hash_into(hasher);
        self.zone.hash_into(hasher);
        self.status.hash_into(hasher);
        self.counters.hash_into(hasher);
        self.attachments.hash_into(hasher);
        self.attached_to.hash_into(hasher);
        self.damage_marked.hash_into(hasher);
        self.deathtouch_damage.hash_into(hasher);
        self.is_token.hash_into(hasher);
        self.timestamp.hash_into(hasher);
    }
}

impl HashInto for PlayerState {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.id.hash_into(hasher);
        self.life_total.hash_into(hasher);
        self.mana_pool.hash_into(hasher);
        self.commander_tax.hash_into(hasher);
        self.commander_damage_received.hash_into(hasher);
        self.poison_counters.hash_into(hasher);
        self.land_plays_remaining.hash_into(hasher);
        self.has_drawn_for_turn.hash_into(hasher);
        self.has_lost.hash_into(hasher);
        self.has_conceded.hash_into(hasher);
        self.commander_ids.hash_into(hasher);
        self.max_hand_size.hash_into(hasher);
    }
}

impl HashInto for TurnState {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.phase.hash_into(hasher);
        self.step.hash_into(hasher);
        self.active_player.hash_into(hasher);
        self.priority_holder.hash_into(hasher);
        self.players_passed.hash_into(hasher);
        self.turn_number.hash_into(hasher);
        self.turn_order.hash_into(hasher);
        self.extra_turns.hash_into(hasher);
        self.extra_combats.hash_into(hasher);
        self.in_extra_combat.hash_into(hasher);
        self.is_first_turn_of_game.hash_into(hasher);
        self.last_regular_active.hash_into(hasher);
    }
}

impl HashInto for Zone {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            Zone::Ordered(v) => {
                0u8.hash_into(hasher);
                v.hash_into(hasher);
            }
            Zone::Unordered(s) => {
                1u8.hash_into(hasher);
                s.hash_into(hasher);
            }
        }
    }
}

// --- Stub type implementations ---

impl HashInto for ContinuousEffect {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
        self.timestamp.hash_into(hasher);
    }
}

impl HashInto for DelayedTrigger {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
    }
}

impl HashInto for ReplacementEffect {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
    }
}

impl HashInto for PendingTrigger {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
        self.ability_index.hash_into(hasher);
        self.controller.hash_into(hasher);
    }
}

impl HashInto for ActivationCost {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.requires_tap.hash_into(hasher);
        self.mana_cost.hash_into(hasher);
    }
}

impl HashInto for ActivatedAbility {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.cost.hash_into(hasher);
        self.description.hash_into(hasher);
    }
}

impl HashInto for TriggerEvent {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            TriggerEvent::SelfEntersBattlefield => 0u8.hash_into(hasher),
            TriggerEvent::AnyPermanentEntersBattlefield => 1u8.hash_into(hasher),
            TriggerEvent::AnySpellCast => 2u8.hash_into(hasher),
            TriggerEvent::SelfBecomesTapped => 3u8.hash_into(hasher),
        }
    }
}

impl HashInto for InterveningIf {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            InterveningIf::ControllerLifeAtLeast(n) => {
                0u8.hash_into(hasher);
                n.hash_into(hasher);
            }
        }
    }
}

impl HashInto for TriggeredAbilityDef {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.trigger_on.hash_into(hasher);
        self.intervening_if.hash_into(hasher);
        self.description.hash_into(hasher);
    }
}

impl HashInto for StackObjectKind {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            StackObjectKind::Spell { source_object } => {
                0u8.hash_into(hasher);
                source_object.hash_into(hasher);
            }
            StackObjectKind::ActivatedAbility {
                source_object,
                ability_index,
            } => {
                1u8.hash_into(hasher);
                source_object.hash_into(hasher);
                ability_index.hash_into(hasher);
            }
            StackObjectKind::TriggeredAbility {
                source_object,
                ability_index,
            } => {
                2u8.hash_into(hasher);
                source_object.hash_into(hasher);
                ability_index.hash_into(hasher);
            }
        }
    }
}

impl HashInto for Target {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            Target::Player(id) => {
                0u8.hash_into(hasher);
                id.hash_into(hasher);
            }
            Target::Object(id) => {
                1u8.hash_into(hasher);
                id.hash_into(hasher);
            }
        }
    }
}

impl HashInto for SpellTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.target.hash_into(hasher);
        self.zone_at_cast.hash_into(hasher);
    }
}

impl HashInto for StackObject {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.id.hash_into(hasher);
        self.controller.hash_into(hasher);
        self.kind.hash_into(hasher);
        self.targets.hash_into(hasher);
    }
}

impl HashInto for CombatState {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.attacking_player.hash_into(hasher);
    }
}

// --- GameEvent implementation ---

impl HashInto for GameEvent {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            GameEvent::TurnStarted {
                player,
                turn_number,
            } => {
                0u8.hash_into(hasher);
                player.hash_into(hasher);
                turn_number.hash_into(hasher);
            }
            GameEvent::StepChanged { step, phase } => {
                1u8.hash_into(hasher);
                step.hash_into(hasher);
                phase.hash_into(hasher);
            }
            GameEvent::PriorityGiven { player } => {
                2u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            GameEvent::PriorityPassed { player } => {
                3u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            GameEvent::AllPlayersPassed => {
                4u8.hash_into(hasher);
            }
            GameEvent::PermanentsUntapped { player, objects } => {
                5u8.hash_into(hasher);
                player.hash_into(hasher);
                objects.hash_into(hasher);
            }
            GameEvent::CardDrawn {
                player,
                new_object_id,
            } => {
                6u8.hash_into(hasher);
                player.hash_into(hasher);
                new_object_id.hash_into(hasher);
            }
            GameEvent::ManaPoolsEmptied => {
                7u8.hash_into(hasher);
            }
            GameEvent::CleanupPerformed => {
                8u8.hash_into(hasher);
            }
            GameEvent::DiscardedToHandSize {
                player,
                object_id,
                zone_from,
                zone_to,
            } => {
                9u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                zone_from.hash_into(hasher);
                zone_to.hash_into(hasher);
            }
            GameEvent::DamageCleared => {
                10u8.hash_into(hasher);
            }
            GameEvent::PlayerLost { player, reason } => {
                11u8.hash_into(hasher);
                player.hash_into(hasher);
                reason.hash_into(hasher);
            }
            GameEvent::PlayerConceded { player } => {
                12u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            GameEvent::GameOver { winner } => {
                13u8.hash_into(hasher);
                winner.hash_into(hasher);
            }
            GameEvent::ExtraTurnAdded { player } => {
                14u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            GameEvent::LandPlayed {
                player,
                new_land_id,
            } => {
                15u8.hash_into(hasher);
                player.hash_into(hasher);
                new_land_id.hash_into(hasher);
            }
            GameEvent::ManaAdded {
                player,
                color,
                amount,
            } => {
                16u8.hash_into(hasher);
                player.hash_into(hasher);
                color.hash_into(hasher);
                amount.hash_into(hasher);
            }
            GameEvent::PermanentTapped { player, object_id } => {
                17u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
            }
            GameEvent::SpellCast {
                player,
                stack_object_id,
                source_object_id,
            } => {
                18u8.hash_into(hasher);
                player.hash_into(hasher);
                stack_object_id.hash_into(hasher);
                source_object_id.hash_into(hasher);
            }
            GameEvent::SpellResolved {
                player,
                stack_object_id,
                source_object_id,
            } => {
                19u8.hash_into(hasher);
                player.hash_into(hasher);
                stack_object_id.hash_into(hasher);
                source_object_id.hash_into(hasher);
            }
            GameEvent::PermanentEnteredBattlefield { player, object_id } => {
                20u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
            }
            GameEvent::SpellCountered {
                player,
                stack_object_id,
                source_object_id,
            } => {
                21u8.hash_into(hasher);
                player.hash_into(hasher);
                stack_object_id.hash_into(hasher);
                source_object_id.hash_into(hasher);
            }
            GameEvent::SpellFizzled {
                player,
                stack_object_id,
                source_object_id,
            } => {
                22u8.hash_into(hasher);
                player.hash_into(hasher);
                stack_object_id.hash_into(hasher);
                source_object_id.hash_into(hasher);
            }
            GameEvent::ManaCostPaid { player, cost } => {
                23u8.hash_into(hasher);
                player.hash_into(hasher);
                cost.hash_into(hasher);
            }
            GameEvent::AbilityActivated {
                player,
                source_object_id,
                stack_object_id,
            } => {
                24u8.hash_into(hasher);
                player.hash_into(hasher);
                source_object_id.hash_into(hasher);
                stack_object_id.hash_into(hasher);
            }
            GameEvent::AbilityTriggered {
                controller,
                source_object_id,
                stack_object_id,
            } => {
                25u8.hash_into(hasher);
                controller.hash_into(hasher);
                source_object_id.hash_into(hasher);
                stack_object_id.hash_into(hasher);
            }
            GameEvent::AbilityResolved {
                controller,
                stack_object_id,
            } => {
                26u8.hash_into(hasher);
                controller.hash_into(hasher);
                stack_object_id.hash_into(hasher);
            }

            // M4 SBA events
            GameEvent::CreatureDied {
                object_id,
                new_grave_id,
            } => {
                27u8.hash_into(hasher);
                object_id.hash_into(hasher);
                new_grave_id.hash_into(hasher);
            }
            GameEvent::PlaneswalkerDied {
                object_id,
                new_grave_id,
            } => {
                28u8.hash_into(hasher);
                object_id.hash_into(hasher);
                new_grave_id.hash_into(hasher);
            }
            GameEvent::AuraFellOff {
                object_id,
                new_grave_id,
            } => {
                29u8.hash_into(hasher);
                object_id.hash_into(hasher);
                new_grave_id.hash_into(hasher);
            }
            GameEvent::EquipmentUnattached { object_id } => {
                30u8.hash_into(hasher);
                object_id.hash_into(hasher);
            }
            GameEvent::TokenCeasedToExist { object_id } => {
                31u8.hash_into(hasher);
                object_id.hash_into(hasher);
            }
            GameEvent::CountersAnnihilated { object_id, amount } => {
                32u8.hash_into(hasher);
                object_id.hash_into(hasher);
                amount.hash_into(hasher);
            }
            GameEvent::LegendaryRuleApplied {
                kept_id,
                put_to_graveyard,
            } => {
                33u8.hash_into(hasher);
                kept_id.hash_into(hasher);
                for (old_id, new_id) in put_to_graveyard {
                    old_id.hash_into(hasher);
                    new_id.hash_into(hasher);
                }
            }
        }
    }
}

// --- GameState hashing ---

impl GameState {
    /// Computes a deterministic hash of all publicly visible game state.
    ///
    /// Two independent engine instances processing the same command sequence
    /// MUST produce identical public state hashes. A mismatch indicates
    /// non-determinism or tampering.
    ///
    /// Includes: turn state, timestamp counter, player public fields (hand/library
    /// sizes only, not contents), public zones (battlefield, graveyard, stack, exile,
    /// command), all game objects in those zones, continuous effects, delayed triggers,
    /// replacement effects, pending triggers, stack objects, combat state.
    ///
    /// Excludes: event history (O(n) in game length), hand contents, library contents.
    pub fn public_state_hash(&self) -> [u8; 32] {
        let mut hasher = Hasher::new();

        // 1. Turn state
        self.turn.hash_into(&mut hasher);

        // 2. Timestamp counter
        self.timestamp_counter.hash_into(&mut hasher);

        // 3. Player public state (via OrdMap iteration — deterministic order)
        (self.players.len() as u64).hash_into(&mut hasher);
        for (player_id, player) in &self.players {
            player_id.hash_into(&mut hasher);
            player.hash_into(&mut hasher);

            // Hand SIZE (not contents) — publicly observable
            let hand_size = self
                .zones
                .get(&ZoneId::Hand(*player_id))
                .map(|z| z.len())
                .unwrap_or(0);
            (hand_size as u64).hash_into(&mut hasher);

            // Library SIZE (not contents) — publicly observable
            let library_size = self
                .zones
                .get(&ZoneId::Library(*player_id))
                .map(|z| z.len())
                .unwrap_or(0);
            (library_size as u64).hash_into(&mut hasher);
        }

        // 4. Public zones — skip Hand(*) and Library(*), hash everything else
        //    Iterate zones in OrdMap order (deterministic)
        for (zone_id, zone) in &self.zones {
            if matches!(zone_id, ZoneId::Hand(_) | ZoneId::Library(_)) {
                continue;
            }
            zone_id.hash_into(&mut hasher);
            zone.hash_into(&mut hasher);
            // Hash full GameObjects for objects in this zone
            for obj_id in zone.object_ids() {
                if let Some(obj) = self.objects.get(&obj_id) {
                    obj.hash_into(&mut hasher);
                }
            }
        }

        // 5. Vectors of game-wide state
        self.continuous_effects.hash_into(&mut hasher);
        self.delayed_triggers.hash_into(&mut hasher);
        self.replacement_effects.hash_into(&mut hasher);
        self.pending_triggers.hash_into(&mut hasher);
        self.stack_objects.hash_into(&mut hasher);

        // 6. Combat state
        self.combat.hash_into(&mut hasher);

        *hasher.finalize().as_bytes()
    }

    /// Computes a deterministic hash of a player's private (hidden) state.
    ///
    /// Covers the contents of their hand and library (card identities and order),
    /// plus any face-down cards they control. Used for detecting non-determinism
    /// in hidden information handling.
    ///
    /// In the distributed verification model, each player can verify their own
    /// private hash but cannot see other players' private hashes.
    pub fn private_state_hash(&self, player: PlayerId) -> [u8; 32] {
        let mut hasher = Hasher::new();

        // 1. Player identity
        player.hash_into(&mut hasher);

        // 2. Hand zone contents (unordered — OrdSet gives deterministic iteration)
        if let Some(hand_zone) = self.zones.get(&ZoneId::Hand(player)) {
            let obj_ids = hand_zone.object_ids();
            (obj_ids.len() as u64).hash_into(&mut hasher);
            for obj_id in &obj_ids {
                obj_id.hash_into(&mut hasher);
                if let Some(obj) = self.objects.get(obj_id) {
                    obj.hash_into(&mut hasher);
                }
            }
        } else {
            0u64.hash_into(&mut hasher);
        }

        // 3. Library zone contents (ordered — position matters)
        if let Some(library_zone) = self.zones.get(&ZoneId::Library(player)) {
            let obj_ids = library_zone.object_ids();
            (obj_ids.len() as u64).hash_into(&mut hasher);
            for obj_id in &obj_ids {
                obj_id.hash_into(&mut hasher);
                if let Some(obj) = self.objects.get(obj_id) {
                    obj.hash_into(&mut hasher);
                }
            }
        } else {
            0u64.hash_into(&mut hasher);
        }

        // 4. Face-down cards controlled by this player (future-proofing)
        //    Currently empty — morphs/manifests not yet implemented
        let face_down: Vec<&GameObject> = self
            .objects
            .values()
            .filter(|obj| obj.controller == player && obj.status.face_down)
            .collect();
        (face_down.len() as u64).hash_into(&mut hasher);
        for obj in face_down {
            obj.hash_into(&mut hasher);
        }

        *hasher.finalize().as_bytes()
    }
}
