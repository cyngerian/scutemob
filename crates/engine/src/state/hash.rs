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

use super::combat::{AttackTarget, CombatState};
use super::continuous_effect::{
    ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
use super::game_object::{
    AbilityInstance, ActivatedAbility, ActivationCost, Characteristics, GameObject, InterveningIf,
    ManaAbility, ManaCost, ObjectId, ObjectStatus, TriggerEvent, TriggeredAbilityDef,
};
use super::player::{CardId, ManaPool, PlayerId, PlayerState};
use super::replacement_effect::{
    DamageTargetFilter, ObjectFilter, PendingZoneChange, PlayerFilter, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger,
};
use super::stack::{StackObject, StackObjectKind};
use super::stubs::{DelayedTrigger, PendingTrigger};
use super::targeting::{SpellTarget, Target};
use super::turn::{Phase, Step, TurnState};
use super::types::{
    CardType, Color, CounterType, KeywordAbility, ManaColor, ProtectionQuality, SubType, SuperType,
};
use super::zone::{Zone, ZoneId, ZoneType};
use super::GameState;
use crate::cards::card_definition::{
    AbilityDefinition, Condition, ContinuousEffectDef, Cost, Effect, EffectAmount, EffectTarget,
    ForEachTarget, LibraryPosition, ModeSelection, PlayerTarget, TargetController, TargetFilter,
    TargetRequirement, TimingRestriction, TokenSpec, TriggerCondition, TypeLine, ZoneTarget,
};
use crate::rules::events::{CombatDamageAssignment, CombatDamageTarget, GameEvent, LossReason};

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

impl<T: HashInto> HashInto for Box<T> {
    fn hash_into(&self, hasher: &mut Hasher) {
        (**self).hash_into(hasher);
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

impl HashInto for ProtectionQuality {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ProtectionQuality::FromColor(c) => {
                0u8.hash_into(hasher);
                c.hash_into(hasher);
            }
            ProtectionQuality::FromCardType(ct) => {
                1u8.hash_into(hasher);
                ct.hash_into(hasher);
            }
            ProtectionQuality::FromSubType(st) => {
                2u8.hash_into(hasher);
                st.hash_into(hasher);
            }
            ProtectionQuality::FromAll => 3u8.hash_into(hasher),
        }
    }
}

impl HashInto for KeywordAbility {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            KeywordAbility::Deathtouch => 0u8.hash_into(hasher),
            KeywordAbility::Defender => 1u8.hash_into(hasher),
            KeywordAbility::DoubleStrike => 2u8.hash_into(hasher),
            KeywordAbility::Enchant => 3u8.hash_into(hasher),
            KeywordAbility::Equip => 4u8.hash_into(hasher),
            KeywordAbility::FirstStrike => 5u8.hash_into(hasher),
            KeywordAbility::Flash => 6u8.hash_into(hasher),
            KeywordAbility::Flying => 7u8.hash_into(hasher),
            KeywordAbility::Haste => 8u8.hash_into(hasher),
            KeywordAbility::Hexproof => 9u8.hash_into(hasher),
            KeywordAbility::Indestructible => 10u8.hash_into(hasher),
            KeywordAbility::Intimidate => 11u8.hash_into(hasher),
            KeywordAbility::Landwalk => 12u8.hash_into(hasher),
            KeywordAbility::Lifelink => 13u8.hash_into(hasher),
            KeywordAbility::Menace => 14u8.hash_into(hasher),
            KeywordAbility::ProtectionFrom(q) => {
                15u8.hash_into(hasher);
                q.hash_into(hasher);
            }
            KeywordAbility::Prowess => 16u8.hash_into(hasher),
            KeywordAbility::Reach => 17u8.hash_into(hasher),
            KeywordAbility::Shroud => 18u8.hash_into(hasher),
            KeywordAbility::Trample => 19u8.hash_into(hasher),
            KeywordAbility::Vigilance => 20u8.hash_into(hasher),
            KeywordAbility::Ward => 21u8.hash_into(hasher),
            KeywordAbility::Partner => 22u8.hash_into(hasher),
            KeywordAbility::NoMaxHandSize => 23u8.hash_into(hasher),
            KeywordAbility::CantBeBlocked => 24u8.hash_into(hasher),
        }
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
        self.has_summoning_sickness.hash_into(hasher);
        self.enchants_creatures.hash_into(hasher);
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
        self.companion.hash_into(hasher);
        self.companion_used.hash_into(hasher);
        self.mulligan_count.hash_into(hasher);
        self.no_max_hand_size.hash_into(hasher);
        self.cards_drawn_this_turn.hash_into(hasher);
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
        self.cleanup_sba_rounds.hash_into(hasher);
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

// --- ContinuousEffect type implementations (M5) ---

impl HashInto for EffectId {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.0.hash_into(hasher);
    }
}

impl HashInto for EffectLayer {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for EffectDuration {
    fn hash_into(&self, hasher: &mut Hasher) {
        (*self as u8).hash_into(hasher);
    }
}

impl HashInto for EffectFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            EffectFilter::SingleObject(id) => {
                0u8.hash_into(hasher);
                id.hash_into(hasher);
            }
            EffectFilter::AllCreatures => 1u8.hash_into(hasher),
            EffectFilter::AllLands => 2u8.hash_into(hasher),
            EffectFilter::AllNonbasicLands => 3u8.hash_into(hasher),
            EffectFilter::AllEnchantments => 4u8.hash_into(hasher),
            EffectFilter::AllNonAuraEnchantments => 5u8.hash_into(hasher),
            EffectFilter::AllPermanents => 6u8.hash_into(hasher),
            EffectFilter::AllCardsInGraveyards => 7u8.hash_into(hasher),
            EffectFilter::ControlledBy(player) => {
                8u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            EffectFilter::CreaturesControlledBy(player) => {
                9u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            EffectFilter::AttachedCreature => 10u8.hash_into(hasher),
            EffectFilter::DeclaredTarget { index } => {
                11u8.hash_into(hasher);
                index.hash_into(hasher);
            }
        }
    }
}

impl HashInto for LayerModification {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            LayerModification::CopyOf(id) => {
                0u8.hash_into(hasher);
                id.hash_into(hasher);
            }
            LayerModification::SetController(player) => {
                1u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            LayerModification::SetTypeLine {
                supertypes,
                card_types,
                subtypes,
            } => {
                2u8.hash_into(hasher);
                supertypes.hash_into(hasher);
                card_types.hash_into(hasher);
                subtypes.hash_into(hasher);
            }
            LayerModification::AddCardTypes(types) => {
                3u8.hash_into(hasher);
                types.hash_into(hasher);
            }
            LayerModification::AddSubtypes(subtypes) => {
                4u8.hash_into(hasher);
                subtypes.hash_into(hasher);
            }
            LayerModification::LoseAllSubtypes => 5u8.hash_into(hasher),
            LayerModification::SetColors(colors) => {
                6u8.hash_into(hasher);
                colors.hash_into(hasher);
            }
            LayerModification::AddColors(colors) => {
                7u8.hash_into(hasher);
                colors.hash_into(hasher);
            }
            LayerModification::BecomeColorless => 8u8.hash_into(hasher),
            LayerModification::AddKeyword(kw) => {
                9u8.hash_into(hasher);
                kw.hash_into(hasher);
            }
            LayerModification::AddKeywords(kws) => {
                10u8.hash_into(hasher);
                kws.hash_into(hasher);
            }
            LayerModification::RemoveAllAbilities => 11u8.hash_into(hasher),
            LayerModification::RemoveKeyword(kw) => {
                12u8.hash_into(hasher);
                kw.hash_into(hasher);
            }
            LayerModification::SetPtViaCda { power, toughness } => {
                13u8.hash_into(hasher);
                power.hash_into(hasher);
                toughness.hash_into(hasher);
            }
            LayerModification::SetPtToManaValue => 14u8.hash_into(hasher),
            LayerModification::SetPowerToughness { power, toughness } => {
                15u8.hash_into(hasher);
                power.hash_into(hasher);
                toughness.hash_into(hasher);
            }
            LayerModification::ModifyPower(d) => {
                16u8.hash_into(hasher);
                d.hash_into(hasher);
            }
            LayerModification::ModifyToughness(d) => {
                17u8.hash_into(hasher);
                d.hash_into(hasher);
            }
            LayerModification::ModifyBoth(d) => {
                18u8.hash_into(hasher);
                d.hash_into(hasher);
            }
            LayerModification::SwitchPowerToughness => 19u8.hash_into(hasher),
        }
    }
}

impl HashInto for ContinuousEffect {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.id.hash_into(hasher);
        self.source.hash_into(hasher);
        self.timestamp.hash_into(hasher);
        self.layer.hash_into(hasher);
        self.duration.hash_into(hasher);
        self.filter.hash_into(hasher);
        self.modification.hash_into(hasher);
        self.is_cda.hash_into(hasher);
    }
}

// --- Stub type implementations ---

impl HashInto for DelayedTrigger {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
    }
}

// --- Replacement effect type implementations (M8) ---

impl HashInto for ReplacementId {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.0.hash_into(hasher);
    }
}

impl HashInto for ObjectFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ObjectFilter::Any => 0u8.hash_into(hasher),
            ObjectFilter::SpecificObject(id) => {
                1u8.hash_into(hasher);
                id.hash_into(hasher);
            }
            ObjectFilter::ControlledBy(player) => {
                2u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            ObjectFilter::AnyCreature => 3u8.hash_into(hasher),
            ObjectFilter::HasCardType(ct) => {
                4u8.hash_into(hasher);
                ct.hash_into(hasher);
            }
            ObjectFilter::Commander => 5u8.hash_into(hasher),
            ObjectFilter::HasCardId(card_id) => {
                6u8.hash_into(hasher);
                card_id.hash_into(hasher);
            }
            ObjectFilter::OwnedByOpponentsOf(player) => {
                7u8.hash_into(hasher);
                player.hash_into(hasher);
            }
        }
    }
}

impl HashInto for PlayerFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            PlayerFilter::Any => 0u8.hash_into(hasher),
            PlayerFilter::Specific(id) => {
                1u8.hash_into(hasher);
                id.hash_into(hasher);
            }
            PlayerFilter::OpponentsOf(id) => {
                2u8.hash_into(hasher);
                id.hash_into(hasher);
            }
        }
    }
}

impl HashInto for DamageTargetFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            DamageTargetFilter::Any => 0u8.hash_into(hasher),
            DamageTargetFilter::Player(id) => {
                1u8.hash_into(hasher);
                id.hash_into(hasher);
            }
            DamageTargetFilter::Permanent(id) => {
                2u8.hash_into(hasher);
                id.hash_into(hasher);
            }
            DamageTargetFilter::FromSource(id) => {
                3u8.hash_into(hasher);
                id.hash_into(hasher);
            }
        }
    }
}

impl HashInto for ZoneType {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ZoneType::Library => 0u8.hash_into(hasher),
            ZoneType::Hand => 1u8.hash_into(hasher),
            ZoneType::Battlefield => 2u8.hash_into(hasher),
            ZoneType::Graveyard => 3u8.hash_into(hasher),
            ZoneType::Stack => 4u8.hash_into(hasher),
            ZoneType::Exile => 5u8.hash_into(hasher),
            ZoneType::Command => 6u8.hash_into(hasher),
        }
    }
}

impl HashInto for ReplacementTrigger {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ReplacementTrigger::WouldChangeZone { from, to, filter } => {
                0u8.hash_into(hasher);
                from.hash_into(hasher);
                to.hash_into(hasher);
                filter.hash_into(hasher);
            }
            ReplacementTrigger::WouldDraw { player_filter } => {
                1u8.hash_into(hasher);
                player_filter.hash_into(hasher);
            }
            ReplacementTrigger::WouldEnterBattlefield { filter } => {
                2u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            ReplacementTrigger::WouldGainLife { player_filter } => {
                3u8.hash_into(hasher);
                player_filter.hash_into(hasher);
            }
            ReplacementTrigger::DamageWouldBeDealt { target_filter } => {
                4u8.hash_into(hasher);
                target_filter.hash_into(hasher);
            }
        }
    }
}

impl HashInto for ReplacementModification {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ReplacementModification::RedirectToZone(zone) => {
                0u8.hash_into(hasher);
                zone.hash_into(hasher);
            }
            ReplacementModification::EntersTapped => 1u8.hash_into(hasher),
            ReplacementModification::EntersWithCounters { counter, count } => {
                2u8.hash_into(hasher);
                counter.hash_into(hasher);
                count.hash_into(hasher);
            }
            ReplacementModification::SkipDraw => 3u8.hash_into(hasher),
            ReplacementModification::PreventDamage(n) => {
                4u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            ReplacementModification::PreventAllDamage => 5u8.hash_into(hasher),
            ReplacementModification::ShuffleIntoOwnerLibrary => 6u8.hash_into(hasher),
        }
    }
}

impl HashInto for ReplacementEffect {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.id.hash_into(hasher);
        self.source.hash_into(hasher);
        self.controller.hash_into(hasher);
        self.duration.hash_into(hasher);
        self.is_self_replacement.hash_into(hasher);
        self.trigger.hash_into(hasher);
        self.modification.hash_into(hasher);
    }
}

impl HashInto for PendingZoneChange {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.object_id.hash_into(hasher);
        self.original_from.hash_into(hasher);
        self.original_destination.hash_into(hasher);
        self.affected_player.hash_into(hasher);
        (self.already_applied.len() as u64).hash_into(hasher);
        for id in &self.already_applied {
            id.hash_into(hasher);
        }
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
        self.effect.hash_into(hasher);
    }
}

impl HashInto for TriggerEvent {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            TriggerEvent::SelfEntersBattlefield => 0u8.hash_into(hasher),
            TriggerEvent::AnyPermanentEntersBattlefield => 1u8.hash_into(hasher),
            TriggerEvent::AnySpellCast => 2u8.hash_into(hasher),
            TriggerEvent::SelfBecomesTapped => 3u8.hash_into(hasher),
            TriggerEvent::SelfAttacks => 4u8.hash_into(hasher),
            TriggerEvent::SelfBlocks => 5u8.hash_into(hasher),
        }
    }
}

impl HashInto for AttackTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            AttackTarget::Player(p) => {
                0u8.hash_into(hasher);
                p.hash_into(hasher);
            }
            AttackTarget::Planeswalker(id) => {
                1u8.hash_into(hasher);
                id.hash_into(hasher);
            }
        }
    }
}

impl HashInto for CombatDamageTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            CombatDamageTarget::Creature(id) => {
                0u8.hash_into(hasher);
                id.hash_into(hasher);
            }
            CombatDamageTarget::Player(p) => {
                1u8.hash_into(hasher);
                p.hash_into(hasher);
            }
            CombatDamageTarget::Planeswalker(id) => {
                2u8.hash_into(hasher);
                id.hash_into(hasher);
            }
        }
    }
}

impl HashInto for CombatDamageAssignment {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
        self.target.hash_into(hasher);
        self.amount.hash_into(hasher);
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
        self.effect.hash_into(hasher);
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
        self.cant_be_countered.hash_into(hasher);
    }
}

impl HashInto for CombatState {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.attacking_player.hash_into(hasher);
        self.attackers.hash_into(hasher);
        self.blockers.hash_into(hasher);
        self.damage_assignment_order.hash_into(hasher);
        self.first_strike_damage_resolved.hash_into(hasher);
        self.defenders_declared.hash_into(hasher);
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
                put_to_graveyard.len().hash_into(hasher);
                for (old_id, new_id) in put_to_graveyard {
                    old_id.hash_into(hasher);
                    new_id.hash_into(hasher);
                }
            }

            // M6 Combat events
            GameEvent::AttackersDeclared {
                attacking_player,
                attackers,
            } => {
                34u8.hash_into(hasher);
                attacking_player.hash_into(hasher);
                (attackers.len() as u64).hash_into(hasher);
                for (obj_id, target) in attackers {
                    obj_id.hash_into(hasher);
                    target.hash_into(hasher);
                }
            }
            GameEvent::BlockersDeclared {
                defending_player,
                blockers,
            } => {
                35u8.hash_into(hasher);
                defending_player.hash_into(hasher);
                (blockers.len() as u64).hash_into(hasher);
                for (blocker_id, attacker_id) in blockers {
                    blocker_id.hash_into(hasher);
                    attacker_id.hash_into(hasher);
                }
            }
            GameEvent::CombatDamageDealt { assignments } => {
                36u8.hash_into(hasher);
                assignments.hash_into(hasher);
            }
            GameEvent::CombatEnded => {
                37u8.hash_into(hasher);
            }
            GameEvent::LifeGained { player, amount } => {
                38u8.hash_into(hasher);
                player.hash_into(hasher);
                amount.hash_into(hasher);
            }
            GameEvent::LifeLost { player, amount } => {
                39u8.hash_into(hasher);
                player.hash_into(hasher);
                amount.hash_into(hasher);
            }
            // ── M7 events (discriminants 40–49) ──
            GameEvent::DamageDealt {
                source,
                target,
                amount,
            } => {
                40u8.hash_into(hasher);
                source.hash_into(hasher);
                target.hash_into(hasher);
                amount.hash_into(hasher);
            }
            GameEvent::ObjectExiled {
                player,
                object_id,
                new_exile_id,
            } => {
                41u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_exile_id.hash_into(hasher);
            }
            GameEvent::PermanentDestroyed {
                object_id,
                new_grave_id,
            } => {
                42u8.hash_into(hasher);
                object_id.hash_into(hasher);
                new_grave_id.hash_into(hasher);
            }
            GameEvent::PermanentUntapped { player, object_id } => {
                43u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
            }
            GameEvent::CardDiscarded {
                player,
                object_id,
                new_id,
            } => {
                44u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_id.hash_into(hasher);
            }
            GameEvent::CardMilled { player, new_id } => {
                45u8.hash_into(hasher);
                player.hash_into(hasher);
                new_id.hash_into(hasher);
            }
            GameEvent::TokenCreated { player, object_id } => {
                46u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
            }
            GameEvent::LibraryShuffled { player } => {
                47u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            GameEvent::CounterAdded {
                object_id,
                counter,
                count,
            } => {
                48u8.hash_into(hasher);
                object_id.hash_into(hasher);
                counter.hash_into(hasher);
                count.hash_into(hasher);
            }
            GameEvent::CounterRemoved {
                object_id,
                counter,
                count,
            } => {
                49u8.hash_into(hasher);
                object_id.hash_into(hasher);
                counter.hash_into(hasher);
                count.hash_into(hasher);
            }
            // MR-M7-01: new zone-move event variants (discriminants 50-52)
            GameEvent::ObjectReturnedToHand {
                player,
                object_id,
                new_hand_id,
            } => {
                50u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_hand_id.hash_into(hasher);
            }
            GameEvent::ObjectPutInGraveyard {
                player,
                object_id,
                new_grave_id,
            } => {
                51u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_grave_id.hash_into(hasher);
            }
            GameEvent::ObjectPutOnLibrary {
                player,
                object_id,
                new_lib_id,
            } => {
                52u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_lib_id.hash_into(hasher);
            }
            // ── M8 replacement/prevention events (discriminants 53-55) ──
            GameEvent::ReplacementEffectApplied {
                effect_id,
                description,
            } => {
                53u8.hash_into(hasher);
                effect_id.hash_into(hasher);
                description.hash_into(hasher);
            }
            GameEvent::ReplacementChoiceRequired {
                player,
                event_description,
                choices,
            } => {
                54u8.hash_into(hasher);
                player.hash_into(hasher);
                event_description.hash_into(hasher);
                (choices.len() as u64).hash_into(hasher);
                for id in choices {
                    id.hash_into(hasher);
                }
            }
            GameEvent::DamagePrevented {
                source,
                target,
                prevented,
                remaining,
            } => {
                55u8.hash_into(hasher);
                source.hash_into(hasher);
                target.hash_into(hasher);
                prevented.hash_into(hasher);
                remaining.hash_into(hasher);
            }
            // MR-M8-05: CommanderZoneRedirect variant (discriminant 56)
            GameEvent::CommanderZoneRedirect {
                object_id,
                new_command_id,
                owner,
            } => {
                56u8.hash_into(hasher);
                object_id.hash_into(hasher);
                new_command_id.hash_into(hasher);
                owner.hash_into(hasher);
            }
            // M9: CommanderCastFromCommandZone variant (discriminant 57)
            GameEvent::CommanderCastFromCommandZone {
                player,
                card_id,
                tax_paid,
            } => {
                57u8.hash_into(hasher);
                player.hash_into(hasher);
                card_id.hash_into(hasher);
                tax_paid.hash_into(hasher);
            }
            // M9: CommanderReturnedToCommandZone variant (discriminant 58)
            GameEvent::CommanderReturnedToCommandZone {
                card_id,
                owner,
                from_zone,
            } => {
                58u8.hash_into(hasher);
                card_id.hash_into(hasher);
                owner.hash_into(hasher);
                from_zone.hash_into(hasher);
            }
            // M9: MulliganTaken (discriminant 59)
            GameEvent::MulliganTaken {
                player,
                mulligan_number,
                is_free,
            } => {
                59u8.hash_into(hasher);
                player.hash_into(hasher);
                mulligan_number.hash_into(hasher);
                is_free.hash_into(hasher);
            }
            // M9: MulliganKept (discriminant 60)
            GameEvent::MulliganKept {
                player,
                cards_to_bottom,
            } => {
                60u8.hash_into(hasher);
                player.hash_into(hasher);
                cards_to_bottom.hash_into(hasher);
            }
            // M9: CompanionBroughtToHand (discriminant 61)
            GameEvent::CompanionBroughtToHand { player, card_id } => {
                61u8.hash_into(hasher);
                player.hash_into(hasher);
                card_id.hash_into(hasher);
            }
            // M9 fix MR-M9-01: CommanderZoneReturnChoiceRequired (discriminant 62)
            GameEvent::CommanderZoneReturnChoiceRequired {
                owner,
                card_id,
                object_id,
                from_zone,
            } => {
                62u8.hash_into(hasher);
                owner.hash_into(hasher);
                card_id.hash_into(hasher);
                object_id.hash_into(hasher);
                from_zone.hash_into(hasher);
            }
            // M9.4: Scried (discriminant 63)
            GameEvent::Scried { player, count } => {
                63u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            // M9.4: Goaded (discriminant 64)
            GameEvent::Goaded {
                object_id,
                goading_player,
            } => {
                64u8.hash_into(hasher);
                object_id.hash_into(hasher);
                goading_player.hash_into(hasher);
            }
        }
    }
}

// --- Card definition type implementations (MR-M3-05/06) ---

impl HashInto for TypeLine {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.supertypes.hash_into(hasher);
        self.card_types.hash_into(hasher);
        self.subtypes.hash_into(hasher);
    }
}

impl HashInto for TargetController {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            TargetController::Any => 0u8.hash_into(hasher),
            TargetController::You => 1u8.hash_into(hasher),
            TargetController::Opponent => 2u8.hash_into(hasher),
        }
    }
}

impl HashInto for TargetFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.max_power.hash_into(hasher);
        self.min_power.hash_into(hasher);
        self.has_card_type.hash_into(hasher);
        self.has_keywords.hash_into(hasher);
        self.colors.hash_into(hasher);
        self.exclude_colors.hash_into(hasher);
        self.non_creature.hash_into(hasher);
        self.non_land.hash_into(hasher);
        self.basic.hash_into(hasher);
        self.controller.hash_into(hasher);
        self.has_subtype.hash_into(hasher);
    }
}

impl HashInto for TargetRequirement {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            TargetRequirement::TargetCreature => 0u8.hash_into(hasher),
            TargetRequirement::TargetPlayer => 1u8.hash_into(hasher),
            TargetRequirement::TargetPermanent => 2u8.hash_into(hasher),
            TargetRequirement::TargetCreatureOrPlayer => 3u8.hash_into(hasher),
            TargetRequirement::TargetAny => 4u8.hash_into(hasher),
            TargetRequirement::TargetSpell => 5u8.hash_into(hasher),
            TargetRequirement::TargetArtifact => 6u8.hash_into(hasher),
            TargetRequirement::TargetEnchantment => 7u8.hash_into(hasher),
            TargetRequirement::TargetLand => 8u8.hash_into(hasher),
            TargetRequirement::TargetPlaneswalker => 9u8.hash_into(hasher),
            TargetRequirement::TargetCreatureWithFilter(filter) => {
                10u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            TargetRequirement::TargetPermanentWithFilter(filter) => {
                11u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            TargetRequirement::TargetPlayerOrPlaneswalker => 12u8.hash_into(hasher),
            TargetRequirement::TargetSpellWithFilter(filter) => {
                13u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
        }
    }
}

impl HashInto for TokenSpec {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.name.hash_into(hasher);
        self.power.hash_into(hasher);
        self.toughness.hash_into(hasher);
        self.colors.hash_into(hasher);
        self.card_types.hash_into(hasher);
        self.subtypes.hash_into(hasher);
        self.keywords.hash_into(hasher);
        self.count.hash_into(hasher);
        self.tapped.hash_into(hasher);
        self.mana_color.hash_into(hasher);
    }
}

impl HashInto for EffectTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            EffectTarget::DeclaredTarget { index } => {
                0u8.hash_into(hasher);
                index.hash_into(hasher);
            }
            EffectTarget::Controller => 1u8.hash_into(hasher),
            EffectTarget::EachPlayer => 2u8.hash_into(hasher),
            EffectTarget::EachOpponent => 3u8.hash_into(hasher),
            EffectTarget::AllCreatures => 4u8.hash_into(hasher),
            EffectTarget::AllPermanents => 5u8.hash_into(hasher),
            EffectTarget::AllPermanentsMatching(filter) => {
                6u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            EffectTarget::Source => 7u8.hash_into(hasher),
        }
    }
}

impl HashInto for PlayerTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            PlayerTarget::Controller => 0u8.hash_into(hasher),
            PlayerTarget::EachPlayer => 1u8.hash_into(hasher),
            PlayerTarget::EachOpponent => 2u8.hash_into(hasher),
            PlayerTarget::DeclaredTarget { index } => {
                3u8.hash_into(hasher);
                index.hash_into(hasher);
            }
            PlayerTarget::ControllerOf(target) => {
                4u8.hash_into(hasher);
                target.hash_into(hasher);
            }
        }
    }
}

impl HashInto for LibraryPosition {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            LibraryPosition::Top => 0u8.hash_into(hasher),
            LibraryPosition::Bottom => 1u8.hash_into(hasher),
            LibraryPosition::ShuffledIn => 2u8.hash_into(hasher),
        }
    }
}

impl HashInto for ZoneTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ZoneTarget::Battlefield { tapped } => {
                0u8.hash_into(hasher);
                tapped.hash_into(hasher);
            }
            ZoneTarget::Graveyard { owner } => {
                1u8.hash_into(hasher);
                owner.hash_into(hasher);
            }
            ZoneTarget::Hand { owner } => {
                2u8.hash_into(hasher);
                owner.hash_into(hasher);
            }
            ZoneTarget::Library { owner, position } => {
                3u8.hash_into(hasher);
                owner.hash_into(hasher);
                position.hash_into(hasher);
            }
            ZoneTarget::Exile => 4u8.hash_into(hasher),
            ZoneTarget::CommandZone => 5u8.hash_into(hasher),
        }
    }
}

impl HashInto for EffectAmount {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            EffectAmount::Fixed(n) => {
                0u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            EffectAmount::XValue => 1u8.hash_into(hasher),
            EffectAmount::PowerOf(target) => {
                2u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            EffectAmount::ToughnessOf(target) => {
                3u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            EffectAmount::ManaValueOf(target) => {
                4u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            EffectAmount::CardCount {
                zone,
                player,
                filter,
            } => {
                5u8.hash_into(hasher);
                zone.hash_into(hasher);
                player.hash_into(hasher);
                filter.hash_into(hasher);
            }
        }
    }
}

impl HashInto for ForEachTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ForEachTarget::EachOpponent => 0u8.hash_into(hasher),
            ForEachTarget::EachPlayer => 1u8.hash_into(hasher),
            ForEachTarget::EachCreature => 2u8.hash_into(hasher),
            ForEachTarget::EachCreatureYouControl => 3u8.hash_into(hasher),
            ForEachTarget::EachOpponentsCreature => 4u8.hash_into(hasher),
            ForEachTarget::EachPermanentMatching(filter) => {
                5u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            ForEachTarget::EachCardInAllGraveyards => 6u8.hash_into(hasher),
        }
    }
}

impl HashInto for TimingRestriction {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            TimingRestriction::SorcerySpeed => 0u8.hash_into(hasher),
            TimingRestriction::AnyTime => 1u8.hash_into(hasher),
        }
    }
}

impl HashInto for TriggerCondition {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            TriggerCondition::WhenEntersBattlefield => 0u8.hash_into(hasher),
            TriggerCondition::WhenDies => 1u8.hash_into(hasher),
            TriggerCondition::WhenAttacks => 2u8.hash_into(hasher),
            TriggerCondition::WhenBlocks => 3u8.hash_into(hasher),
            TriggerCondition::WhenDealsCombatDamageToPlayer => 4u8.hash_into(hasher),
            TriggerCondition::WheneverOpponentCastsSpell => 5u8.hash_into(hasher),
            TriggerCondition::WheneverPlayerDrawsCard => 6u8.hash_into(hasher),
            TriggerCondition::WheneverCreatureDies => 7u8.hash_into(hasher),
            TriggerCondition::WheneverCreatureEntersBattlefield { filter } => {
                8u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            TriggerCondition::WheneverPermanentEntersBattlefield { filter } => {
                9u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            TriggerCondition::AtBeginningOfYourUpkeep => 10u8.hash_into(hasher),
            TriggerCondition::AtBeginningOfEachUpkeep => 11u8.hash_into(hasher),
            TriggerCondition::AtBeginningOfYourEndStep => 12u8.hash_into(hasher),
            TriggerCondition::AtBeginningOfCombat => 13u8.hash_into(hasher),
            TriggerCondition::WheneverYouCastSpell {
                during_opponent_turn,
            } => {
                14u8.hash_into(hasher);
                during_opponent_turn.hash_into(hasher);
            }
            TriggerCondition::WheneverYouGainLife => 15u8.hash_into(hasher),
            TriggerCondition::WheneverYouDrawACard => 16u8.hash_into(hasher),
        }
    }
}

impl HashInto for Condition {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            Condition::ControllerLifeAtLeast(n) => {
                0u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            Condition::SourceOnBattlefield => 1u8.hash_into(hasher),
            Condition::YouControlPermanent(filter) => {
                2u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            Condition::OpponentControlsPermanent(filter) => {
                3u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            Condition::TargetIsLegal { index } => {
                4u8.hash_into(hasher);
                index.hash_into(hasher);
            }
            Condition::SourceHasCounters { counter, min } => {
                5u8.hash_into(hasher);
                counter.hash_into(hasher);
                min.hash_into(hasher);
            }
            Condition::Always => 6u8.hash_into(hasher),
        }
    }
}

impl HashInto for ContinuousEffectDef {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.layer.hash_into(hasher);
        self.modification.hash_into(hasher);
        self.filter.hash_into(hasher);
        self.duration.hash_into(hasher);
    }
}

impl HashInto for Cost {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            Cost::Mana(mc) => {
                0u8.hash_into(hasher);
                mc.hash_into(hasher);
            }
            Cost::Tap => 1u8.hash_into(hasher),
            Cost::Sacrifice(filter) => {
                2u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            Cost::PayLife(n) => {
                3u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            Cost::DiscardCard => 4u8.hash_into(hasher),
            Cost::Sequence(costs) => {
                5u8.hash_into(hasher);
                costs.hash_into(hasher);
            }
        }
    }
}

impl HashInto for ModeSelection {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.min_modes.hash_into(hasher);
        self.max_modes.hash_into(hasher);
        self.modes.hash_into(hasher);
    }
}

impl HashInto for Effect {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            Effect::DealDamage { target, amount } => {
                0u8.hash_into(hasher);
                target.hash_into(hasher);
                amount.hash_into(hasher);
            }
            Effect::GainLife { player, amount } => {
                1u8.hash_into(hasher);
                player.hash_into(hasher);
                amount.hash_into(hasher);
            }
            Effect::LoseLife { player, amount } => {
                2u8.hash_into(hasher);
                player.hash_into(hasher);
                amount.hash_into(hasher);
            }
            Effect::DrawCards { player, count } => {
                3u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            Effect::DiscardCards { player, count } => {
                4u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            Effect::MillCards { player, count } => {
                5u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            Effect::CreateToken { spec } => {
                6u8.hash_into(hasher);
                spec.hash_into(hasher);
            }
            Effect::DestroyPermanent { target } => {
                7u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            Effect::ExileObject { target } => {
                8u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            Effect::CounterSpell { target } => {
                9u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            Effect::TapPermanent { target } => {
                10u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            Effect::UntapPermanent { target } => {
                11u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            Effect::AddMana { player, mana } => {
                12u8.hash_into(hasher);
                player.hash_into(hasher);
                mana.hash_into(hasher);
            }
            Effect::AddManaAnyColor { player } => {
                13u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            Effect::AddManaChoice { player, count } => {
                14u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            Effect::AddCounter {
                target,
                counter,
                count,
            } => {
                15u8.hash_into(hasher);
                target.hash_into(hasher);
                counter.hash_into(hasher);
                count.hash_into(hasher);
            }
            Effect::RemoveCounter {
                target,
                counter,
                count,
            } => {
                16u8.hash_into(hasher);
                target.hash_into(hasher);
                counter.hash_into(hasher);
                count.hash_into(hasher);
            }
            Effect::MoveZone { target, to } => {
                17u8.hash_into(hasher);
                target.hash_into(hasher);
                to.hash_into(hasher);
            }
            Effect::SearchLibrary {
                player,
                filter,
                reveal,
                destination,
            } => {
                18u8.hash_into(hasher);
                player.hash_into(hasher);
                filter.hash_into(hasher);
                reveal.hash_into(hasher);
                destination.hash_into(hasher);
            }
            Effect::Shuffle { player } => {
                19u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            Effect::ApplyContinuousEffect { effect_def } => {
                20u8.hash_into(hasher);
                effect_def.hash_into(hasher);
            }
            Effect::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                21u8.hash_into(hasher);
                condition.hash_into(hasher);
                if_true.hash_into(hasher);
                if_false.hash_into(hasher);
            }
            Effect::ForEach { over, effect } => {
                22u8.hash_into(hasher);
                over.hash_into(hasher);
                effect.hash_into(hasher);
            }
            Effect::Choose { prompt, choices } => {
                23u8.hash_into(hasher);
                prompt.hash_into(hasher);
                choices.hash_into(hasher);
            }
            Effect::Sequence(effects) => {
                24u8.hash_into(hasher);
                effects.hash_into(hasher);
            }
            Effect::MayPayOrElse {
                cost,
                payer,
                or_else,
            } => {
                25u8.hash_into(hasher);
                cost.hash_into(hasher);
                payer.hash_into(hasher);
                or_else.hash_into(hasher);
            }
            Effect::Nothing => 26u8.hash_into(hasher),
            Effect::PutOnLibrary {
                player,
                count,
                from,
            } => {
                27u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
                from.hash_into(hasher);
            }
            // M9.4: Scry (discriminant 28) — CR 701.18
            Effect::Scry { player, count } => {
                28u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            // M9.4: Goad (discriminant 29) — CR 701.38
            Effect::Goad { target } => {
                29u8.hash_into(hasher);
                target.hash_into(hasher);
            }
        }
    }
}

impl HashInto for AbilityDefinition {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            AbilityDefinition::Activated {
                cost,
                effect,
                timing_restriction,
            } => {
                0u8.hash_into(hasher);
                cost.hash_into(hasher);
                effect.hash_into(hasher);
                timing_restriction.hash_into(hasher);
            }
            AbilityDefinition::Triggered {
                trigger_condition,
                effect,
                intervening_if,
            } => {
                1u8.hash_into(hasher);
                trigger_condition.hash_into(hasher);
                effect.hash_into(hasher);
                intervening_if.hash_into(hasher);
            }
            AbilityDefinition::Static { continuous_effect } => {
                2u8.hash_into(hasher);
                continuous_effect.hash_into(hasher);
            }
            AbilityDefinition::Keyword(kw) => {
                3u8.hash_into(hasher);
                kw.hash_into(hasher);
            }
            AbilityDefinition::Spell {
                effect,
                targets,
                modes,
                cant_be_countered,
            } => {
                4u8.hash_into(hasher);
                effect.hash_into(hasher);
                targets.hash_into(hasher);
                modes.hash_into(hasher);
                cant_be_countered.hash_into(hasher);
            }
            AbilityDefinition::Replacement {
                trigger,
                modification,
                is_self,
            } => {
                5u8.hash_into(hasher);
                trigger.hash_into(hasher);
                modification.hash_into(hasher);
                is_self.hash_into(hasher);
            }
            AbilityDefinition::OpeningHand => 6u8.hash_into(hasher),
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

        // 2. Timestamp counter and replacement ID counter
        self.timestamp_counter.hash_into(&mut hasher);
        self.next_replacement_id.hash_into(&mut hasher);

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
        self.pending_zone_changes.hash_into(&mut hasher);
        for (owner, oid) in self.pending_commander_zone_choices.iter() {
            owner.hash_into(&mut hasher);
            oid.hash_into(&mut hasher);
        }
        for (id, n) in self.prevention_counters.iter() {
            id.hash_into(&mut hasher);
            n.hash_into(&mut hasher);
        }
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
