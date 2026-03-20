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
use super::dungeon::{DungeonId, DungeonState};
use super::game_object::{
    AbilityInstance, ActivatedAbility, ActivationCost, Characteristics, ETBTriggerFilter,
    GameObject, HybridMana, InterveningIf, ManaAbility, ManaCost, ObjectId, ObjectStatus,
    PhyrexianMana, SacrificeFilter, TriggerEvent, TriggeredAbilityDef,
};
use super::player::{CardId, ManaPool, PlayerId, PlayerState, RestrictedMana};
use super::replacement_effect::{
    DamageTargetFilter, ObjectFilter, PendingZoneChange, PlayerFilter, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger,
};
use super::stack::{StackObject, StackObjectKind, TriggerData, UpkeepCostKind};
use super::stubs::{
    ActiveRestriction, DelayedTrigger, ETBSuppressFilter, ETBSuppressor, GameRestriction,
    PendingTrigger, TriggerDoubler, TriggerDoublerFilter,
};
use super::targeting::{SpellTarget, Target};
use super::turn::{Phase, Step, TurnState};
use super::types::{
    AdditionalCost, AffinityTarget, CardType, ChampionFilter, Color, CounterType,
    CumulativeUpkeepCost, EnchantTarget, KeywordAbility, LandwalkType, ManaColor,
    ProtectionQuality, SubType, SuperType,
};
use super::zone::{Zone, ZoneId, ZoneType};
use super::GameState;
use crate::cards::card_definition::ManaRestriction;
use crate::cards::card_definition::{
    AbilityDefinition, Condition, ContinuousEffectDef, Cost, Effect, EffectAmount, EffectTarget,
    ForEachTarget, LibraryPosition, LoyaltyCost, ModeSelection, PlayerTarget, SoulbondGrant,
    TargetController, TargetFilter, TargetRequirement, TimingRestriction, TokenSpec,
    TriggerCondition, TypeLine, ZoneTarget,
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
            CounterType::Fade => 14u8.hash_into(hasher),
            // Age (discriminant 15) -- CR 702.24a
            CounterType::Age => 15u8.hash_into(hasher),
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

impl HashInto for LandwalkType {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            LandwalkType::BasicType(st) => {
                0u8.hash_into(hasher);
                st.hash_into(hasher);
            }
            LandwalkType::Nonbasic => 1u8.hash_into(hasher),
        }
    }
}

impl HashInto for EnchantTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            EnchantTarget::Creature => 0u8.hash_into(hasher),
            EnchantTarget::Permanent => 1u8.hash_into(hasher),
            EnchantTarget::Artifact => 2u8.hash_into(hasher),
            EnchantTarget::Enchantment => 3u8.hash_into(hasher),
            EnchantTarget::Land => 4u8.hash_into(hasher),
            EnchantTarget::Planeswalker => 5u8.hash_into(hasher),
            EnchantTarget::Player => 6u8.hash_into(hasher),
            EnchantTarget::CreatureOrPlaneswalker => 7u8.hash_into(hasher),
        }
    }
}

impl HashInto for AffinityTarget {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            AffinityTarget::Artifacts => 0u8.hash_into(hasher),
            AffinityTarget::BasicLandType(st) => {
                1u8.hash_into(hasher);
                st.hash_into(hasher);
            }
        }
    }
}

impl HashInto for CumulativeUpkeepCost {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            CumulativeUpkeepCost::Mana(cost) => {
                0u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            CumulativeUpkeepCost::Life(amount) => {
                1u8.hash_into(hasher);
                amount.hash_into(hasher);
            }
        }
    }
}

impl HashInto for KeywordAbility {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            KeywordAbility::Deathtouch => 0u8.hash_into(hasher),
            KeywordAbility::Defender => 1u8.hash_into(hasher),
            KeywordAbility::DoubleStrike => 2u8.hash_into(hasher),
            KeywordAbility::Enchant(target) => {
                3u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            KeywordAbility::Equip => 4u8.hash_into(hasher),
            KeywordAbility::FirstStrike => 5u8.hash_into(hasher),
            KeywordAbility::Flash => 6u8.hash_into(hasher),
            KeywordAbility::Flying => 7u8.hash_into(hasher),
            KeywordAbility::Haste => 8u8.hash_into(hasher),
            KeywordAbility::Hexproof => 9u8.hash_into(hasher),
            KeywordAbility::Indestructible => 10u8.hash_into(hasher),
            KeywordAbility::Intimidate => 11u8.hash_into(hasher),
            KeywordAbility::Landwalk(lw_type) => {
                12u8.hash_into(hasher);
                lw_type.hash_into(hasher);
            }
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
            KeywordAbility::Ward(cost) => {
                21u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            KeywordAbility::Partner => 22u8.hash_into(hasher),
            KeywordAbility::NoMaxHandSize => 23u8.hash_into(hasher),
            KeywordAbility::CantBeBlocked => 24u8.hash_into(hasher),
            // M9.4: Storm (discriminant 25) — CR 702.40
            KeywordAbility::Storm => 25u8.hash_into(hasher),
            // M9.4: Cascade (discriminant 26) — CR 702.85
            KeywordAbility::Cascade => 26u8.hash_into(hasher),
            // Flashback (discriminant 27) — CR 702.34
            KeywordAbility::Flashback => 27u8.hash_into(hasher),
            // Cycling (discriminant 28) — CR 702.29
            KeywordAbility::Cycling => 28u8.hash_into(hasher),
            // Dredge (discriminant 29) — CR 702.52
            KeywordAbility::Dredge(n) => {
                29u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Convoke (discriminant 30) — CR 702.51
            KeywordAbility::Convoke => 30u8.hash_into(hasher),
            // Delve (discriminant 31) — CR 702.66
            KeywordAbility::Delve => 31u8.hash_into(hasher),
            // Kicker (discriminant 32) — CR 702.33
            KeywordAbility::Kicker => 32u8.hash_into(hasher),
            // SplitSecond (discriminant 33) — CR 702.61
            KeywordAbility::SplitSecond => 33u8.hash_into(hasher),
            // Exalted (discriminant 34) — CR 702.83
            KeywordAbility::Exalted => 34u8.hash_into(hasher),
            // Annihilator (discriminant 35) — CR 702.86
            KeywordAbility::Annihilator(n) => {
                35u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Persist (discriminant 36) — CR 702.79
            KeywordAbility::Persist => 36u8.hash_into(hasher),
            // Undying (discriminant 37) -- CR 702.93
            KeywordAbility::Undying => 37u8.hash_into(hasher),
            // Changeling (discriminant 38) -- CR 702.73
            KeywordAbility::Changeling => 38u8.hash_into(hasher),
            // Evoke (discriminant 39) -- CR 702.74
            KeywordAbility::Evoke => 39u8.hash_into(hasher),
            // Crew (discriminant 40) -- CR 702.122
            KeywordAbility::Crew(n) => {
                40u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // BattleCry (discriminant 41) -- CR 702.91
            KeywordAbility::BattleCry => 41u8.hash_into(hasher),
            // Afterlife (discriminant 42) -- CR 702.135
            KeywordAbility::Afterlife(n) => {
                42u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Extort (discriminant 43) -- CR 702.101
            KeywordAbility::Extort => 43u8.hash_into(hasher),
            // Improvise (discriminant 44) -- CR 702.126
            KeywordAbility::Improvise => 44u8.hash_into(hasher),
            // Bestow (discriminant 45) -- CR 702.103
            KeywordAbility::Bestow => 45u8.hash_into(hasher),
            // Fear (discriminant 46) -- CR 702.36
            KeywordAbility::Fear => 46u8.hash_into(hasher),
            // LivingWeapon (discriminant 47) -- CR 702.92
            KeywordAbility::LivingWeapon => 47u8.hash_into(hasher),
            // Madness (discriminant 48) -- CR 702.35
            KeywordAbility::Madness => 48u8.hash_into(hasher),
            // Miracle (discriminant 49) -- CR 702.94
            KeywordAbility::Miracle => 49u8.hash_into(hasher),
            // Escape (discriminant 50) -- CR 702.138
            KeywordAbility::Escape => 50u8.hash_into(hasher),
            // Foretell (discriminant 51) -- CR 702.143
            KeywordAbility::Foretell => 51u8.hash_into(hasher),
            // Unearth (discriminant 52) -- CR 702.84
            KeywordAbility::Unearth => 52u8.hash_into(hasher),
            // Affinity (discriminant 53) -- CR 702.41
            KeywordAbility::Affinity(target) => {
                53u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            // Undaunted (discriminant 54) -- CR 702.125
            KeywordAbility::Undaunted => 54u8.hash_into(hasher),
            // Dethrone (discriminant 55) -- CR 702.105
            KeywordAbility::Dethrone => 55u8.hash_into(hasher),
            // Riot (discriminant 56) -- CR 702.136
            KeywordAbility::Riot => 56u8.hash_into(hasher),
            // Exploit (discriminant 57) -- CR 702.110
            KeywordAbility::Exploit => 57u8.hash_into(hasher),
            // Wither (discriminant 58) -- CR 702.80
            KeywordAbility::Wither => 58u8.hash_into(hasher),
            // Modular (discriminant 59) -- CR 702.43
            KeywordAbility::Modular(n) => {
                59u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Evolve (discriminant 60) -- CR 702.100
            KeywordAbility::Evolve => 60u8.hash_into(hasher),
            // Buyback (discriminant 61) -- CR 702.27
            KeywordAbility::Buyback => 61u8.hash_into(hasher),
            // Ascend (discriminant 62) -- CR 702.131
            KeywordAbility::Ascend => 62u8.hash_into(hasher),
            // Infect (discriminant 63) -- CR 702.90
            KeywordAbility::Infect => 63u8.hash_into(hasher),
            // Myriad (discriminant 64) -- CR 702.116
            KeywordAbility::Myriad => 64u8.hash_into(hasher),
            // Suspend (discriminant 65) -- CR 702.62
            KeywordAbility::Suspend => 65u8.hash_into(hasher),
            // Hideaway (discriminant 66) -- CR 702.75
            KeywordAbility::Hideaway(n) => {
                66u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Adapt (discriminant 67) -- CR 701.46
            KeywordAbility::Adapt(n) => {
                67u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Shadow (discriminant 68) -- CR 702.28
            KeywordAbility::Shadow => 68u8.hash_into(hasher),
            // PartnerWith (discriminant 69) -- CR 702.124j
            KeywordAbility::PartnerWith(name) => {
                69u8.hash_into(hasher);
                name.hash_into(hasher);
            }
            // Overload (discriminant 70) -- CR 702.96
            KeywordAbility::Overload => 70u8.hash_into(hasher),
            // Horsemanship (discriminant 71) -- CR 702.31
            KeywordAbility::Horsemanship => 71u8.hash_into(hasher),
            // Skulk (discriminant 72) -- CR 702.118
            KeywordAbility::Skulk => 72u8.hash_into(hasher),
            // Devoid (discriminant 73) -- CR 702.114
            KeywordAbility::Devoid => 73u8.hash_into(hasher),
            // Decayed (discriminant 74) -- CR 702.147
            KeywordAbility::Decayed => 74u8.hash_into(hasher),
            // Ingest (discriminant 75) -- CR 702.115
            KeywordAbility::Ingest => 75u8.hash_into(hasher),
            // Flanking (discriminant 76) -- CR 702.25
            KeywordAbility::Flanking => 76u8.hash_into(hasher),
            // Bushido (discriminant 77) -- CR 702.45
            KeywordAbility::Bushido(n) => {
                77u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Rampage (discriminant 78) -- CR 702.23
            KeywordAbility::Rampage(n) => {
                78u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Provoke (discriminant 79) -- CR 702.39
            KeywordAbility::Provoke => 79u8.hash_into(hasher),
            // Afflict (discriminant 80) -- CR 702.130
            KeywordAbility::Afflict(n) => {
                80u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Renown (discriminant 81) -- CR 702.112
            KeywordAbility::Renown(n) => {
                81u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Training (discriminant 82) -- CR 702.149
            KeywordAbility::Training => 82u8.hash_into(hasher),
            // Melee (discriminant 83) -- CR 702.121
            KeywordAbility::Melee => 83u8.hash_into(hasher),
            // Poisonous (discriminant 84) -- CR 702.70
            KeywordAbility::Poisonous(n) => {
                84u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Toxic (discriminant 85) -- CR 702.164
            KeywordAbility::Toxic(n) => {
                85u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Enlist (discriminant 86) -- CR 702.154
            KeywordAbility::Enlist => 86u8.hash_into(hasher),
            // Ninjutsu (discriminant 87) -- CR 702.49
            KeywordAbility::Ninjutsu => 87u8.hash_into(hasher),
            // Commander Ninjutsu (discriminant 88) -- CR 702.49d
            KeywordAbility::CommanderNinjutsu => 88u8.hash_into(hasher),
            // Retrace (discriminant 89) -- CR 702.81
            KeywordAbility::Retrace => 89u8.hash_into(hasher),
            // Jump-Start (discriminant 90) -- CR 702.133
            KeywordAbility::JumpStart => 90u8.hash_into(hasher),
            // Aftermath (discriminant 91) -- CR 702.127
            KeywordAbility::Aftermath => 91u8.hash_into(hasher),
            // Embalm (discriminant 92) -- CR 702.128
            KeywordAbility::Embalm => 92u8.hash_into(hasher),
            // Eternalize (discriminant 93) -- CR 702.129
            KeywordAbility::Eternalize => 93u8.hash_into(hasher),
            // Encore (discriminant 94) -- CR 702.141
            KeywordAbility::Encore => 94u8.hash_into(hasher),
            // Dash (discriminant 95) -- CR 702.109
            KeywordAbility::Dash => 95u8.hash_into(hasher),
            // Blitz (discriminant 96) -- CR 702.152
            KeywordAbility::Blitz => 96u8.hash_into(hasher),
            // Plot (discriminant 97) -- CR 702.170
            KeywordAbility::Plot => 97u8.hash_into(hasher),
            // Prototype (discriminant 98) -- CR 702.160
            KeywordAbility::Prototype => 98u8.hash_into(hasher),
            // Impending (discriminant 99) -- CR 702.176
            KeywordAbility::Impending => 99u8.hash_into(hasher),
            // Bargain (discriminant 100) -- CR 702.166
            KeywordAbility::Bargain => 100u8.hash_into(hasher),
            // Emerge (discriminant 101) -- CR 702.119
            KeywordAbility::Emerge => 101u8.hash_into(hasher),
            // Spectacle (discriminant 102) -- CR 702.137
            KeywordAbility::Spectacle => 102u8.hash_into(hasher),
            // Surge (discriminant 103) -- CR 702.117
            KeywordAbility::Surge => 103u8.hash_into(hasher),
            // Casualty (discriminant 104) -- CR 702.153
            KeywordAbility::Casualty(n) => {
                104u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Assist (discriminant 105) -- CR 702.132
            KeywordAbility::Assist => 105u8.hash_into(hasher),
            // Replicate (discriminant 106) -- CR 702.56
            KeywordAbility::Replicate => 106u8.hash_into(hasher),
            // Gravestorm (discriminant 107) -- CR 702.69
            KeywordAbility::Gravestorm => 107u8.hash_into(hasher),
            // Cleave (discriminant 108) -- CR 702.148
            KeywordAbility::Cleave => 108u8.hash_into(hasher),
            // Splice (discriminant 109) -- CR 702.47
            KeywordAbility::Splice => 109u8.hash_into(hasher),
            // Entwine (discriminant 110) -- CR 702.42
            KeywordAbility::Entwine => 110u8.hash_into(hasher),
            // Escalate (discriminant 111) -- CR 702.120
            KeywordAbility::Escalate => 111u8.hash_into(hasher),
            // Vanishing (discriminant 112) -- CR 702.63
            KeywordAbility::Vanishing(n) => {
                112u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Fading (discriminant 113) -- CR 702.32
            KeywordAbility::Fading(n) => {
                113u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Echo (discriminant 114) -- CR 702.30
            KeywordAbility::Echo(cost) => {
                114u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // CumulativeUpkeep (discriminant 115) -- CR 702.24
            KeywordAbility::CumulativeUpkeep(cost) => {
                115u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Recover (discriminant 116) -- CR 702.59
            KeywordAbility::Recover => 116u8.hash_into(hasher),
            // Forecast (discriminant 117) -- CR 702.57
            KeywordAbility::Forecast => 117u8.hash_into(hasher),
            KeywordAbility::Phasing => 118u8.hash_into(hasher),
            // Graft (discriminant 119) -- CR 702.58
            KeywordAbility::Graft(n) => {
                119u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Scavenge (discriminant 120) -- CR 702.97
            KeywordAbility::Scavenge => 120u8.hash_into(hasher),
            // Outlast (discriminant 121) -- CR 702.107
            KeywordAbility::Outlast => 121u8.hash_into(hasher),
            // Amplify (discriminant 122) -- CR 702.38
            KeywordAbility::Amplify(n) => {
                122u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Bloodthirst (discriminant 123) -- CR 702.54
            KeywordAbility::Bloodthirst(n) => {
                123u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Devour (discriminant 124) -- CR 702.82
            KeywordAbility::Devour(n) => {
                124u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Backup (discriminant 125) -- CR 702.165
            KeywordAbility::Backup(n) => {
                125u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Champion (discriminant 126) -- CR 702.72
            KeywordAbility::Champion => 126u8.hash_into(hasher),
            // UmbraArmor (discriminant 127) -- CR 702.89
            KeywordAbility::UmbraArmor => 127u8.hash_into(hasher),
            // LivingMetal (discriminant 128) -- CR 702.161
            KeywordAbility::LivingMetal => 128u8.hash_into(hasher),
            // Soulbond (discriminant 129) -- CR 702.95
            KeywordAbility::Soulbond => 129u8.hash_into(hasher),
            // Fortify (discriminant 130) -- CR 702.67
            KeywordAbility::Fortify => 130u8.hash_into(hasher),
            // Tribute (discriminant 131) -- CR 702.104
            KeywordAbility::Tribute(n) => {
                131u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Fabricate (discriminant 132) -- CR 702.123
            KeywordAbility::Fabricate(n) => {
                132u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Fuse (discriminant 133) -- CR 702.102
            KeywordAbility::Fuse => 133u8.hash_into(hasher),
            // Spree (discriminant 134) -- CR 702.172
            KeywordAbility::Spree => 134u8.hash_into(hasher),
            // Ravenous (discriminant 135) -- CR 702.156
            KeywordAbility::Ravenous => 135u8.hash_into(hasher),
            // Discover (discriminant 136) -- CR 701.57
            KeywordAbility::Discover => 136u8.hash_into(hasher),
            // Squad (discriminant 137) -- CR 702.157
            KeywordAbility::Squad => 137u8.hash_into(hasher),
            // Offspring (discriminant 138) -- CR 702.175a
            KeywordAbility::Offspring => 138u8.hash_into(hasher),
            // Gift (discriminant 139) -- CR 702.174
            KeywordAbility::Gift => 139u8.hash_into(hasher),
            // Saddle (discriminant 140) -- CR 702.171
            KeywordAbility::Saddle(n) => {
                140u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Cipher (discriminant 141) -- CR 702.99
            KeywordAbility::Cipher => 141u8.hash_into(hasher),
            // Haunt (discriminant 142) -- CR 702.55
            KeywordAbility::Haunt => 142u8.hash_into(hasher),
            // Reconfigure (discriminant 143) -- CR 702.151
            KeywordAbility::Reconfigure => 143u8.hash_into(hasher),
            // FriendsForever (discriminant 144) -- CR 702.124i
            KeywordAbility::FriendsForever => 144u8.hash_into(hasher),
            // ChooseABackground (discriminant 145) -- CR 702.124k
            KeywordAbility::ChooseABackground => 145u8.hash_into(hasher),
            // DoctorsCompanion (discriminant 146) -- CR 702.124m
            KeywordAbility::DoctorsCompanion => 146u8.hash_into(hasher),
            // Mutate (discriminant 147) -- CR 702.140
            KeywordAbility::Mutate => 147u8.hash_into(hasher),
            // Transform (discriminant 148) -- CR 701.27
            KeywordAbility::Transform => 148u8.hash_into(hasher),
            // Daybound (discriminant 149) -- CR 702.145b
            KeywordAbility::Daybound => 149u8.hash_into(hasher),
            // Nightbound (discriminant 150) -- CR 702.145e
            KeywordAbility::Nightbound => 150u8.hash_into(hasher),
            // Disturb (discriminant 151) -- CR 702.146
            KeywordAbility::Disturb => 151u8.hash_into(hasher),
            // Craft (discriminant 152) -- CR 702.167
            KeywordAbility::Craft => 152u8.hash_into(hasher),
            // Morph (discriminant 153) -- CR 702.37
            KeywordAbility::Morph => 153u8.hash_into(hasher),
            // Megamorph (discriminant 154) -- CR 702.37b
            KeywordAbility::Megamorph => 154u8.hash_into(hasher),
            // Disguise (discriminant 155) -- CR 702.168
            KeywordAbility::Disguise => 155u8.hash_into(hasher),
            // Manifest (discriminant 156) -- CR 701.40
            KeywordAbility::Manifest => 156u8.hash_into(hasher),
            // Cloak (discriminant 157) -- CR 701.58
            KeywordAbility::Cloak => 157u8.hash_into(hasher),
            // MustAttackEachCombat (discriminant 158) -- CR 508.1d
            KeywordAbility::MustAttackEachCombat => 158u8.hash_into(hasher),
            // HexproofPlayer (discriminant 159) -- CR 702.11d
            KeywordAbility::HexproofPlayer => 159u8.hash_into(hasher),
        }
    }
}

impl HashInto for ChampionFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ChampionFilter::AnyCreature => 0u8.hash_into(hasher),
            ChampionFilter::Subtype(st) => {
                1u8.hash_into(hasher);
                st.hash_into(hasher);
            }
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

impl HashInto for HybridMana {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            HybridMana::ColorColor(a, b) => {
                0u8.hash_into(hasher);
                a.hash_into(hasher);
                b.hash_into(hasher);
            }
            HybridMana::GenericColor(c) => {
                1u8.hash_into(hasher);
                c.hash_into(hasher);
            }
        }
    }
}

impl HashInto for PhyrexianMana {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            PhyrexianMana::Single(c) => {
                0u8.hash_into(hasher);
                c.hash_into(hasher);
            }
            PhyrexianMana::Hybrid(a, b) => {
                1u8.hash_into(hasher);
                a.hash_into(hasher);
                b.hash_into(hasher);
            }
        }
    }
}

impl HashInto for ManaCost {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.white.hash_into(hasher);
        self.blue.hash_into(hasher);
        self.black.hash_into(hasher);
        self.red.hash_into(hasher);
        self.green.hash_into(hasher);
        self.colorless.hash_into(hasher);
        self.generic.hash_into(hasher);
        (self.hybrid.len() as u32).hash_into(hasher);
        for h in &self.hybrid {
            h.hash_into(hasher);
        }
        (self.phyrexian.len() as u32).hash_into(hasher);
        for p in &self.phyrexian {
            p.hash_into(hasher);
        }
        self.x_count.hash_into(hasher);
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
        (self.restricted.len() as u32).hash_into(hasher);
        for r in &self.restricted {
            r.hash_into(hasher);
        }
    }
}

impl HashInto for RestrictedMana {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.color.hash_into(hasher);
        self.amount.hash_into(hasher);
        self.restriction.hash_into(hasher);
    }
}

impl HashInto for ManaRestriction {
    fn hash_into(&self, hasher: &mut Hasher) {
        // Discriminant tag + payload
        match self {
            ManaRestriction::CreatureSpellsOnly => 0u8.hash_into(hasher),
            ManaRestriction::SubtypeOnly(st) => {
                1u8.hash_into(hasher);
                st.0.hash_into(hasher);
            }
            ManaRestriction::SubtypeOrSubtype(a, b) => {
                2u8.hash_into(hasher);
                a.0.hash_into(hasher);
                b.0.hash_into(hasher);
            }
            ManaRestriction::CreatureWithSubtype(st) => {
                3u8.hash_into(hasher);
                st.0.hash_into(hasher);
            }
            ManaRestriction::ChosenTypeCreaturesOnly => 4u8.hash_into(hasher),
            ManaRestriction::ChosenTypeSpellsOnly => 5u8.hash_into(hasher),
        }
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
        self.sacrifice_self.hash_into(hasher);
        self.any_color.hash_into(hasher);
        self.damage_to_controller.hash_into(hasher);
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
        self.goaded_by.hash_into(hasher);
        // Kicker (CR 702.33d) — times kicker was paid when this permanent was cast
        self.kicker_times_paid.hash_into(hasher);
        // Alt cost (CR 702.74a / CR 702.138b / CR 702.109a) — which alternative cost was paid
        self.cast_alt_cost.map(|k| k as u8).hash_into(hasher);
        // Designations bitfield (Renowned, Suspected, Saddled, Echo, Bestow, Foretold, Suspended, Reconfigured)
        (self.designations.bits() as u32).hash_into(hasher);
        // Foretell turn number
        self.foretold_turn.hash_into(hasher);
        // Unearth (CR 702.84a) — permanent was returned to battlefield via unearth
        self.was_unearthed.hash_into(hasher);
        // Myriad (CR 702.116a) — token copy must be exiled at end of combat
        self.myriad_exile_at_eoc.hash_into(hasher);
        // Decayed (CR 702.147a) — creature must be sacrificed at end of combat
        self.decayed_sacrifice_at_eoc.hash_into(hasher);
        // Ring level 3 (CR 701.54c) — blocker of ring-bearer must be sacrificed at end of combat
        self.ring_block_sacrifice_at_eoc.hash_into(hasher);
        // Hideaway (CR 702.75a / CR 607.2a) — card was exiled face-down by a Hideaway trigger
        self.exiled_by_hideaway.hash_into(hasher);
        // Encore (CR 702.141a) — token must be sacrificed at beginning of next end step
        self.encore_sacrifice_at_end_step.hash_into(hasher);
        // Encore (CR 702.141a) — mandatory attack target for this turn
        self.encore_must_attack.hash_into(hasher);
        // Encore (CR 702.141a / Ruling 2020-11-10) — original activator identity
        self.encore_activated_by.hash_into(hasher);
        // Plot (CR 702.170a) — card was plotted (exiled face-up via plot special action)
        self.is_plotted.hash_into(hasher);
        self.plotted_turn.hash_into(hasher);
        // Prototype (CR 718.3b) — whether this permanent was cast prototyped
        self.is_prototyped.hash_into(hasher);
        // Bargain (CR 702.166b) — permanent was cast with bargain cost paid
        self.was_bargained.hash_into(hasher);
        // Collect Evidence (CR 701.59c) — permanent was cast with collect evidence cost paid
        self.evidence_collected.hash_into(hasher);
        // Note: Surge's "was_surged" is tracked via cast_alt_cost == Some(AltCostKind::Surge),
        // which is already hashed as part of cast_alt_cost above.
        // Phasing (CR 702.26g) — permanent phased out indirectly via a host
        self.phased_out_indirectly.hash_into(hasher);
        // Phasing (CR 702.26a) — player who controlled this permanent when it phased out
        self.phased_out_controller.hash_into(hasher);
        // Devour (CR 702.82b) — number of creatures devoured on ETB
        self.creatures_devoured.hash_into(hasher);
        // Champion (CR 702.72a) — ObjectId of exiled card tracked by linked LTB trigger
        self.champion_exiled_card.hash_into(hasher);
        // Soulbond (CR 702.95b) — ObjectId of the creature this is paired with
        self.paired_with.hash_into(hasher);
        // Tribute (CR 702.104b) — whether tribute was paid for this permanent
        self.tribute_was_paid.hash_into(hasher);
        // X value (CR 107.3m) — the value of X chosen at cast time for ETB abilities
        self.x_value.hash_into(hasher);
        // Squad (CR 702.157a) — number of times squad cost was paid at cast time
        self.squad_count.hash_into(hasher);
        // Offspring (CR 702.175a) — whether offspring cost was paid at cast time
        self.offspring_paid.hash_into(hasher);
        // Gift (CR 702.174a) — whether gift cost was paid and who was chosen
        self.gift_was_given.hash_into(hasher);
        self.gift_opponent.hash_into(hasher);
        // Cipher (CR 702.99b) — exiled cipher cards encoded on this permanent
        for (obj_id, card_id) in self.encoded_cards.iter() {
            obj_id.hash_into(hasher);
            card_id.hash_into(hasher);
        }
        // Haunt (CR 702.55b) — creature this exiled card is haunting
        self.haunting_target.hash_into(hasher);
        // Mutate (CR 729.2) — merged components (empty for unmerged permanents)
        (self.merged_components.len() as u64).hash_into(hasher);
        for component in self.merged_components.iter() {
            component.hash_into(hasher);
        }
        // Transform (CR 712.8d/e) — permanent has back face up
        self.is_transformed.hash_into(hasher);
        // Transform (CR 701.27f) — timestamp of last transform (for once-guard)
        self.last_transform_timestamp.hash_into(hasher);
        // Disturb (CR 702.146 ruling) — permanent was cast via disturb (exile if would die)
        self.was_cast_disturbed.hash_into(hasher);
        // Craft (CR 702.167c) — ObjectIds of cards exiled as craft materials
        (self.craft_exiled_cards.len() as u64).hash_into(hasher);
        for id in self.craft_exiled_cards.iter() {
            id.hash_into(hasher);
        }
        // CR 106.12: chosen creature type for mana restriction
        match &self.chosen_creature_type {
            None => 0u8.hash_into(hasher),
            Some(st) => {
                1u8.hash_into(hasher);
                st.0.hash_into(hasher);
            }
        }
        // CR 606.3: loyalty ability activated this turn
        self.loyalty_ability_activated_this_turn.hash_into(hasher);
        self.class_level.hash_into(hasher);
        // Morph/Manifest/Cloak (CR 702.37/701.40/701.58) — face-down kind
        match &self.face_down_as {
            None => 0u8.hash_into(hasher),
            Some(crate::state::types::FaceDownKind::Morph) => 1u8.hash_into(hasher),
            Some(crate::state::types::FaceDownKind::Megamorph) => 2u8.hash_into(hasher),
            Some(crate::state::types::FaceDownKind::Disguise) => 3u8.hash_into(hasher),
            Some(crate::state::types::FaceDownKind::Manifest) => 4u8.hash_into(hasher),
            Some(crate::state::types::FaceDownKind::Cloak) => 5u8.hash_into(hasher),
        }
        // CR 712.4a: Meld component tracking
        self.meld_component.hash_into(hasher);
    }
}

impl HashInto for crate::state::game_object::MergedComponent {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.card_id.hash_into(hasher);
        self.characteristics.hash_into(hasher);
        self.is_token.hash_into(hasher);
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
        // M9.4: spells_cast_this_turn (CR 702.40a)
        self.spells_cast_this_turn.hash_into(hasher);
        self.has_citys_blessing.hash_into(hasher);
        // CR 702.137a: per-turn life-loss counter for Spectacle eligibility.
        self.life_lost_this_turn.hash_into(hasher);
        // CR 702.54a: per-turn damage-received counter for Bloodthirst eligibility.
        self.damage_received_this_turn.hash_into(hasher);
        // CR 702.16b/e: player protection qualities.
        for q in &self.protection_qualities {
            q.hash_into(hasher);
        }
        // CR 309.7: dungeons completed by this player.
        self.dungeons_completed.hash_into(hasher);
        // CR 309.7: specific dungeons completed (for CompletedSpecificDungeon condition).
        for dungeon_id in &self.dungeons_completed_set {
            dungeon_id.hash_into(hasher);
        }
        // CR 701.54c: ring level (0-4) for this player.
        self.ring_level.hash_into(hasher);
        // CR 701.54a: ring-bearer ObjectId for this player.
        self.ring_bearer_id.hash_into(hasher);
    }
}

/// CR 309.4: Hash dungeon identifier as a stable discriminant byte.
impl HashInto for DungeonId {
    fn hash_into(&self, hasher: &mut Hasher) {
        let disc: u8 = match self {
            DungeonId::LostMineOfPhandelver => 0,
            DungeonId::DungeonOfTheMadMage => 1,
            DungeonId::TombOfAnnihilation => 2,
            DungeonId::TheUndercity => 3,
        };
        disc.hash_into(hasher);
    }
}

/// CR 309.4: Hash per-player dungeon state (which dungeon + current room).
impl HashInto for DungeonState {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.dungeon.hash_into(hasher);
        (self.current_room as u64).hash_into(hasher);
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
        self.additional_phases.hash_into(hasher);
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
        match self {
            EffectDuration::WhileSourceOnBattlefield => 0u8.hash_into(hasher),
            EffectDuration::UntilEndOfTurn => 1u8.hash_into(hasher),
            EffectDuration::Indefinite => 2u8.hash_into(hasher),
            // CR 702.95a: WhilePaired includes both ObjectIds for uniqueness.
            EffectDuration::WhilePaired(a, b) => {
                3u8.hash_into(hasher);
                a.hash_into(hasher);
                b.hash_into(hasher);
            }
        }
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
            EffectFilter::Source => 12u8.hash_into(hasher),
            EffectFilter::AttachedLand => 13u8.hash_into(hasher),
            EffectFilter::CreaturesYouControl => 14u8.hash_into(hasher),
            EffectFilter::OtherCreaturesYouControl => 15u8.hash_into(hasher),
            EffectFilter::OtherCreaturesYouControlWithSubtype(subtype) => {
                16u8.hash_into(hasher);
                subtype.hash_into(hasher);
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
            // AddAllCreatureTypes (discriminant 20) -- CR 702.73a, 205.3m
            LayerModification::AddAllCreatureTypes => 20u8.hash_into(hasher),
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

impl HashInto for TriggerDoublerFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            TriggerDoublerFilter::ArtifactOrCreatureETB => 0u8.hash_into(hasher),
            TriggerDoublerFilter::CreatureDeath => 1u8.hash_into(hasher),
        }
    }
}

impl HashInto for TriggerDoubler {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
        self.controller.hash_into(hasher);
        self.filter.hash_into(hasher);
        self.additional_triggers.hash_into(hasher);
    }
}

impl HashInto for DelayedTrigger {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
    }
}

impl HashInto for ETBSuppressFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            ETBSuppressFilter::CreaturesOnly => 0u8.hash_into(hasher),
            ETBSuppressFilter::AllPermanents => 1u8.hash_into(hasher),
        }
    }
}

impl HashInto for ETBSuppressor {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
        self.filter.hash_into(hasher);
    }
}

// --- Active restriction type implementations (PB-18) ---

impl HashInto for GameRestriction {
    fn hash_into(&self, hasher: &mut Hasher) {
        use GameRestriction::*;
        match self {
            MaxSpellsPerTurn { max } => {
                0u8.hash_into(hasher);
                max.hash_into(hasher);
            }
            OpponentsCantCastDuringYourTurn => 1u8.hash_into(hasher),
            OpponentsCantCastOrActivateDuringYourTurn => 2u8.hash_into(hasher),
            OpponentsCantCastFromNonHand => 3u8.hash_into(hasher),
            CantAttackYouUnlessPay { cost_per_creature } => {
                4u8.hash_into(hasher);
                cost_per_creature.hash_into(hasher);
            }
            ArtifactAbilitiesCantBeActivated => 5u8.hash_into(hasher),
        }
    }
}

impl HashInto for ActiveRestriction {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
        self.controller.hash_into(hasher);
        self.restriction.hash_into(hasher);
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
            DamageTargetFilter::FromControllerSources(pid) => {
                4u8.hash_into(hasher);
                pid.hash_into(hasher);
            }
            DamageTargetFilter::ToOpponentOrTheirPermanent(pid) => {
                5u8.hash_into(hasher);
                pid.hash_into(hasher);
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
            // CR 701.8/614.8: WouldBeDestroyed (discriminant 5)
            ReplacementTrigger::WouldBeDestroyed { filter } => {
                5u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            // CR 122.6/614.1: WouldPlaceCounters (discriminant 6)
            ReplacementTrigger::WouldPlaceCounters {
                placer_filter,
                receiver_filter,
            } => {
                6u8.hash_into(hasher);
                placer_filter.hash_into(hasher);
                receiver_filter.hash_into(hasher);
            }
            // CR 111.1/614.1: WouldCreateTokens (discriminant 7)
            ReplacementTrigger::WouldCreateTokens { controller_filter } => {
                7u8.hash_into(hasher);
                controller_filter.hash_into(hasher);
            }
            // CR 701.23/614.1: WouldSearchLibrary (discriminant 8)
            ReplacementTrigger::WouldSearchLibrary { searcher_filter } => {
                8u8.hash_into(hasher);
                searcher_filter.hash_into(hasher);
            }
            // CR 614.1: WouldLoseLife (discriminant 9)
            ReplacementTrigger::WouldLoseLife { player_filter } => {
                9u8.hash_into(hasher);
                player_filter.hash_into(hasher);
            }
            // CR 701.34: WouldProliferate (discriminant 10)
            ReplacementTrigger::WouldProliferate { player_filter } => {
                10u8.hash_into(hasher);
                player_filter.hash_into(hasher);
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
            // CR 701.19a/614.8: Regenerate (discriminant 7)
            ReplacementModification::Regenerate => 7u8.hash_into(hasher),
            // CR 614.1c: shockland pay-life-or-tapped (discriminant 8)
            ReplacementModification::EntersTappedUnlessPayLife(life) => {
                8u8.hash_into(hasher);
                life.hash_into(hasher);
            }
            ReplacementModification::ChooseCreatureType(st) => {
                9u8.hash_into(hasher);
                st.0.hash_into(hasher);
            }
            // CR 122.6: DoubleCounters (discriminant 10)
            ReplacementModification::DoubleCounters => 10u8.hash_into(hasher),
            // CR 122.6: HalveCounters (discriminant 11)
            ReplacementModification::HalveCounters => 11u8.hash_into(hasher),
            // CR 122.6: AddExtraCounter (discriminant 12)
            ReplacementModification::AddExtraCounter => 12u8.hash_into(hasher),
            // CR 111.1: DoubleTokens (discriminant 13)
            ReplacementModification::DoubleTokens => 13u8.hash_into(hasher),
            // CR 701.23: RestrictSearchTopN (discriminant 14)
            ReplacementModification::RestrictSearchTopN(n) => {
                14u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // CR 614.1: DoubleDamage (discriminant 15)
            ReplacementModification::DoubleDamage => 15u8.hash_into(hasher),
            // CR 614.1: DoubleLifeLoss (discriminant 16)
            ReplacementModification::DoubleLifeLoss => 16u8.hash_into(hasher),
            // CR 701.34: DoubleProliferate (discriminant 17)
            ReplacementModification::DoubleProliferate => 17u8.hash_into(hasher),
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

impl HashInto for crate::state::stubs::PendingTriggerKind {
    fn hash_into(&self, hasher: &mut Hasher) {
        use crate::state::stubs::PendingTriggerKind;
        match self {
            PendingTriggerKind::Normal => 0u8.hash_into(hasher),
            PendingTriggerKind::CardDefETB => 46u8.hash_into(hasher),
            PendingTriggerKind::Evoke => 1u8.hash_into(hasher),
            PendingTriggerKind::Madness => 2u8.hash_into(hasher),
            PendingTriggerKind::Miracle => 3u8.hash_into(hasher),
            PendingTriggerKind::Unearth => 4u8.hash_into(hasher),
            PendingTriggerKind::Exploit => 5u8.hash_into(hasher),
            PendingTriggerKind::Modular => 6u8.hash_into(hasher),
            PendingTriggerKind::Evolve => 7u8.hash_into(hasher),
            PendingTriggerKind::Myriad => 8u8.hash_into(hasher),
            PendingTriggerKind::SuspendCounter => 9u8.hash_into(hasher),
            PendingTriggerKind::SuspendCast => 10u8.hash_into(hasher),
            PendingTriggerKind::Hideaway => 11u8.hash_into(hasher),
            PendingTriggerKind::PartnerWith => 12u8.hash_into(hasher),
            PendingTriggerKind::Ingest => 13u8.hash_into(hasher),
            PendingTriggerKind::Flanking => 14u8.hash_into(hasher),
            PendingTriggerKind::Rampage => 15u8.hash_into(hasher),
            PendingTriggerKind::Provoke => 16u8.hash_into(hasher),
            PendingTriggerKind::Renown => 17u8.hash_into(hasher),
            PendingTriggerKind::Melee => 18u8.hash_into(hasher),
            PendingTriggerKind::Poisonous => 19u8.hash_into(hasher),
            PendingTriggerKind::Enlist => 20u8.hash_into(hasher),
            PendingTriggerKind::EncoreSacrifice => 21u8.hash_into(hasher),
            PendingTriggerKind::DashReturn => 22u8.hash_into(hasher),
            PendingTriggerKind::BlitzSacrifice => 23u8.hash_into(hasher),
            // ImpendingCounter (24): migrated to KeywordTrigger
            // VanishingCounter (25) and VanishingSacrifice (26): migrated to KeywordTrigger
            // FadingUpkeep (27): migrated to KeywordTrigger
            // EchoUpkeep (28): migrated to KeywordTrigger
            // CumulativeUpkeep (29): migrated to KeywordTrigger
            PendingTriggerKind::Recover => 30u8.hash_into(hasher),
            PendingTriggerKind::Graft => 31u8.hash_into(hasher),
            PendingTriggerKind::Backup => 32u8.hash_into(hasher),
            PendingTriggerKind::ChampionETB => 33u8.hash_into(hasher),
            PendingTriggerKind::ChampionLTB => 34u8.hash_into(hasher),
            PendingTriggerKind::SoulbondSelfETB => 35u8.hash_into(hasher),
            PendingTriggerKind::SoulbondOtherETB => 36u8.hash_into(hasher),
            PendingTriggerKind::RavenousDraw => 37u8.hash_into(hasher),
            PendingTriggerKind::SquadETB => 38u8.hash_into(hasher),
            PendingTriggerKind::OffspringETB => 39u8.hash_into(hasher),
            PendingTriggerKind::GiftETB => 40u8.hash_into(hasher),
            PendingTriggerKind::CipherCombatDamage => 41u8.hash_into(hasher),
            PendingTriggerKind::HauntExile => 42u8.hash_into(hasher),
            PendingTriggerKind::HauntedCreatureDies => 43u8.hash_into(hasher),
            PendingTriggerKind::TurnFaceUp => 44u8.hash_into(hasher),
            PendingTriggerKind::KeywordTrigger { keyword, data } => {
                45u8.hash_into(hasher);
                keyword.hash_into(hasher);
                data.hash_into(hasher);
            }
            PendingTriggerKind::RingLoot => 47u8.hash_into(hasher),
            PendingTriggerKind::RingBlockSacrifice => 48u8.hash_into(hasher),
            PendingTriggerKind::RingCombatDamage => 49u8.hash_into(hasher),
        }
    }
}

impl HashInto for PendingTrigger {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.source.hash_into(hasher);
        self.ability_index.hash_into(hasher);
        self.controller.hash_into(hasher);
        // kind discriminant replaces all is_X_trigger boolean fields
        self.kind.hash_into(hasher);
        // M9.4: triggering_event (CR 603.2d) — used for Panharmonicon doubling
        self.triggering_event.hash_into(hasher);
        // M9.4 fix session 3: entering_object_id — used by ArtifactOrCreatureETB filter
        self.entering_object_id.hash_into(hasher);
        // CR 702.21a: targeting_stack_id — the stack object Ward will counter
        self.targeting_stack_id.hash_into(hasher);
        // CR 603.2 / CR 102.2: triggering_player — the opponent who cast the spell
        self.triggering_player.hash_into(hasher);
        // CR 702.83a: exalted_attacker_id — the lone attacker for Exalted triggers
        self.exalted_attacker_id.hash_into(hasher);
        // CR 508.5 / CR 702.86a: defending_player_id — the defending player for SelfAttacks triggers
        self.defending_player_id.hash_into(hasher);
        // CR 702.35a: madness-specific fields
        self.madness_exiled_card.hash_into(hasher);
        self.madness_cost.hash_into(hasher);
        // CR 702.94a: miracle-specific fields
        self.miracle_revealed_card.hash_into(hasher);
        self.miracle_cost.hash_into(hasher);
        // CR 702.141a: encore-specific fields
        self.encore_activator.hash_into(hasher);
        // CR 702.43a: modular-specific field
        self.modular_counter_count.hash_into(hasher);
        // CR 702.100a: evolve-specific field
        self.evolve_entering_creature.hash_into(hasher);
        // CR 702.62a: suspend-specific field
        self.suspend_card_id.hash_into(hasher);
        // CR 702.75a: hideaway-specific field
        self.hideaway_count.hash_into(hasher);
        // CR 702.124j: partner-with-specific field
        self.partner_with_name.hash_into(hasher);
        // CR 702.115a: ingest-specific field
        self.ingest_target_player.hash_into(hasher);
        // CR 702.25a: flanking-specific field
        self.flanking_blocker_id.hash_into(hasher);
        // CR 702.23a: rampage-specific field
        self.rampage_n.hash_into(hasher);
        // CR 702.39a: provoke-specific field
        self.provoke_target_creature.hash_into(hasher);
        // CR 702.112a: renown-specific field
        self.renown_n.hash_into(hasher);
        // CR 702.70a: poisonous-specific fields
        self.poisonous_n.hash_into(hasher);
        self.poisonous_target_player.hash_into(hasher);
        // CR 702.154a: enlist-specific field
        self.enlist_enlisted_creature.hash_into(hasher);
        // CR 702.58a: graft-specific field
        self.graft_entering_creature.hash_into(hasher);
        // CR 702.30a: echo-specific field
        self.echo_cost.hash_into(hasher);
        // CR 702.24a: cumulative upkeep-specific field
        self.cumulative_upkeep_cost.hash_into(hasher);
        // CR 702.59a: recover-specific fields
        self.recover_cost.hash_into(hasher);
        self.recover_card.hash_into(hasher);
        // CR 702.165a: backup-specific fields
        self.backup_abilities.hash_into(hasher);
        self.backup_n.hash_into(hasher);
        // CR 702.174a: gift-specific field
        self.gift_opponent.hash_into(hasher);
        // CR 702.99a: cipher-specific fields
        self.cipher_encoded_card_id.hash_into(hasher);
        self.cipher_encoded_object_id.hash_into(hasher);
    }
}

impl HashInto for SacrificeFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            SacrificeFilter::Creature => 0u8.hash_into(hasher),
            SacrificeFilter::Land => 1u8.hash_into(hasher),
            SacrificeFilter::Artifact => 2u8.hash_into(hasher),
            SacrificeFilter::ArtifactOrCreature => 3u8.hash_into(hasher),
            SacrificeFilter::Subtype(sub) => {
                4u8.hash_into(hasher);
                sub.hash_into(hasher);
            }
        }
    }
}

impl HashInto for ActivationCost {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.requires_tap.hash_into(hasher);
        self.mana_cost.hash_into(hasher);
        self.sacrifice_self.hash_into(hasher);
        self.discard_card.hash_into(hasher);
        self.discard_self.hash_into(hasher);
        self.forage.hash_into(hasher);
        self.sacrifice_filter.hash_into(hasher);
    }
}

impl HashInto for ActivatedAbility {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.cost.hash_into(hasher);
        self.description.hash_into(hasher);
        self.effect.hash_into(hasher);
        // CR 602.5d: sorcery-speed restriction field
        self.sorcery_speed.hash_into(hasher);
        self.targets.hash_into(hasher);
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
            // CR 702.21a: Ward trigger — discriminant 6
            TriggerEvent::SelfBecomesTargetByOpponent => 6u8.hash_into(hasher),
            // CR 702.108a: Prowess trigger — discriminant 7
            TriggerEvent::ControllerCastsNoncreatureSpell => 7u8.hash_into(hasher),
            // CR 603.6c / CR 700.4: Dies trigger — discriminant 8
            TriggerEvent::SelfDies => 8u8.hash_into(hasher),
            // CR 510.3a / CR 603.2: Combat damage to player trigger — discriminant 9
            TriggerEvent::SelfDealsCombatDamageToPlayer => 9u8.hash_into(hasher),
            // CR 603.2 / CR 102.2: Opponent-casts trigger — discriminant 10
            TriggerEvent::OpponentCastsSpell => 10u8.hash_into(hasher),
            // CR 702.83a: Exalted "attacks alone" trigger — discriminant 11
            TriggerEvent::ControllerCreatureAttacksAlone => 11u8.hash_into(hasher),
            // CR 701.25d: Surveil trigger — discriminant 12
            TriggerEvent::ControllerSurveils => 12u8.hash_into(hasher),
            // CR 702.101a: Controller-casts-any-spell trigger — discriminant 13
            TriggerEvent::ControllerCastsSpell => 13u8.hash_into(hasher),
            // CR 702.105a: Dethrone "attacks player with most life" trigger — discriminant 14
            TriggerEvent::SelfAttacksPlayerWithMostLife => 14u8.hash_into(hasher),
            // CR 701.50b: SourceConnives trigger — discriminant 15
            TriggerEvent::SourceConnives => 15u8.hash_into(hasher),
            // CR 701.16a: ControllerInvestigates trigger — discriminant 16
            TriggerEvent::ControllerInvestigates => 16u8.hash_into(hasher),
            // CR 701.34: ControllerProliferates trigger — discriminant 17
            TriggerEvent::ControllerProliferates => 17u8.hash_into(hasher),
            // CR 509.1h / CR 702.45a: SelfBecomesBlocked trigger — discriminant 18
            TriggerEvent::SelfBecomesBlocked => 18u8.hash_into(hasher),
            // CR 702.149a: Training "attacks with greater power ally" trigger — discriminant 19
            TriggerEvent::SelfAttacksWithGreaterPowerAlly => 19u8.hash_into(hasher),
            // CR 207.2c / CR 120.3: Enrage "whenever this creature is dealt damage" — discriminant 20
            TriggerEvent::SelfIsDealtDamage => 20u8.hash_into(hasher),
            // CR 702.55c: Haunt "when the creature it haunts dies" — discriminant 21
            TriggerEvent::HauntedCreatureDies => 21u8.hash_into(hasher),
            // CR 702.140d: Mutate "whenever this creature mutates" — discriminant 22
            TriggerEvent::SelfMutates => 22u8.hash_into(hasher),
            // CR 708.8: "When this permanent is turned face up" — discriminant 23
            TriggerEvent::SelfTurnedFaceUp => 23u8.hash_into(hasher),
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
            InterveningIf::SourceHadNoCounterOfType(ct) => {
                1u8.hash_into(hasher);
                ct.hash_into(hasher);
            }
        }
    }
}

impl HashInto for ETBTriggerFilter {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.creature_only.hash_into(hasher);
        self.controller_you.hash_into(hasher);
        self.exclude_self.hash_into(hasher);
    }
}

impl HashInto for TriggeredAbilityDef {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.trigger_on.hash_into(hasher);
        self.intervening_if.hash_into(hasher);
        self.description.hash_into(hasher);
        self.effect.hash_into(hasher);
        self.etb_filter.hash_into(hasher);
        self.targets.hash_into(hasher);
    }
}

impl HashInto for TriggerData {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            TriggerData::Simple => 0u8.hash_into(hasher),
            TriggerData::CounterRemoval { permanent } => {
                1u8.hash_into(hasher);
                permanent.hash_into(hasher);
            }
            TriggerData::CounterSacrifice { permanent } => {
                2u8.hash_into(hasher);
                permanent.hash_into(hasher);
            }
            TriggerData::UpkeepCost { permanent, cost } => {
                3u8.hash_into(hasher);
                permanent.hash_into(hasher);
                cost.hash_into(hasher);
            }
            TriggerData::CombatFlanking { blocker } => {
                4u8.hash_into(hasher);
                blocker.hash_into(hasher);
            }
            TriggerData::CombatRampage { n } => {
                5u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            TriggerData::CombatProvoke { target } => {
                6u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            TriggerData::CombatPoisonous { target_player, n } => {
                7u8.hash_into(hasher);
                target_player.hash_into(hasher);
                n.hash_into(hasher);
            }
            TriggerData::CombatEnlist { enlisted } => {
                8u8.hash_into(hasher);
                enlisted.hash_into(hasher);
            }
            TriggerData::ETBBackup {
                target,
                count,
                abilities,
            } => {
                9u8.hash_into(hasher);
                target.hash_into(hasher);
                count.hash_into(hasher);
                abilities.hash_into(hasher);
            }
            TriggerData::ETBGraft { entering_creature } => {
                10u8.hash_into(hasher);
                entering_creature.hash_into(hasher);
            }
            TriggerData::ETBChampion { filter } => {
                11u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            TriggerData::ETBSoulbond { pair_target } => {
                12u8.hash_into(hasher);
                pair_target.hash_into(hasher);
            }
            TriggerData::ETBRavenousDraw { permanent, x_value } => {
                13u8.hash_into(hasher);
                permanent.hash_into(hasher);
                x_value.hash_into(hasher);
            }
            TriggerData::ETBSquad { count } => {
                14u8.hash_into(hasher);
                count.hash_into(hasher);
            }
            TriggerData::ETBOffspring { source_card_id } => {
                15u8.hash_into(hasher);
                source_card_id.hash_into(hasher);
            }
            TriggerData::ETBGift {
                source_card_id,
                gift_opponent,
            } => {
                16u8.hash_into(hasher);
                source_card_id.hash_into(hasher);
                gift_opponent.hash_into(hasher);
            }
            TriggerData::ETBHideaway { count } => {
                17u8.hash_into(hasher);
                count.hash_into(hasher);
            }
            TriggerData::ETBPartnerWith {
                partner_name,
                target_player,
            } => {
                18u8.hash_into(hasher);
                partner_name.hash_into(hasher);
                target_player.hash_into(hasher);
            }
            TriggerData::SpellCopy {
                original_stack_id,
                copy_count,
            } => {
                19u8.hash_into(hasher);
                original_stack_id.hash_into(hasher);
                copy_count.hash_into(hasher);
            }
            TriggerData::CascadeExile { spell_mana_value } => {
                20u8.hash_into(hasher);
                spell_mana_value.hash_into(hasher);
            }
            TriggerData::CasualtyCopy { original_stack_id } => {
                21u8.hash_into(hasher);
                original_stack_id.hash_into(hasher);
            }
            TriggerData::DelayedZoneChange => 22u8.hash_into(hasher),
            TriggerData::EncoreSacrifice { activator } => {
                23u8.hash_into(hasher);
                activator.hash_into(hasher);
            }
            TriggerData::DeathModular { counter_count } => {
                24u8.hash_into(hasher);
                counter_count.hash_into(hasher);
            }
            TriggerData::DeathHauntExile {
                haunt_card,
                haunt_card_id,
            } => {
                25u8.hash_into(hasher);
                haunt_card.hash_into(hasher);
                haunt_card_id.hash_into(hasher);
            }
            TriggerData::DeathHauntedCreatureDies {
                haunt_source,
                haunt_card_id,
            } => {
                26u8.hash_into(hasher);
                haunt_source.hash_into(hasher);
                haunt_card_id.hash_into(hasher);
            }
            TriggerData::LTBChampion { exiled_card } => {
                27u8.hash_into(hasher);
                exiled_card.hash_into(hasher);
            }
            TriggerData::DeathRecover {
                recover_card,
                recover_cost,
            } => {
                28u8.hash_into(hasher);
                recover_card.hash_into(hasher);
                recover_cost.hash_into(hasher);
            }
            TriggerData::CipherDamage {
                source_creature,
                encoded_card_id,
                encoded_object_id,
            } => {
                29u8.hash_into(hasher);
                source_creature.hash_into(hasher);
                encoded_card_id.hash_into(hasher);
                encoded_object_id.hash_into(hasher);
            }
            TriggerData::IngestExile { target_player } => {
                30u8.hash_into(hasher);
                target_player.hash_into(hasher);
            }
            TriggerData::RenownDamage { n } => {
                31u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            TriggerData::EvolveTrigger { entering_creature } => {
                32u8.hash_into(hasher);
                entering_creature.hash_into(hasher);
            }
            TriggerData::MyriadAttack { defending_player } => {
                33u8.hash_into(hasher);
                defending_player.hash_into(hasher);
            }
        }
    }
}

impl HashInto for UpkeepCostKind {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            UpkeepCostKind::Echo(mana_cost) => {
                0u8.hash_into(hasher);
                mana_cost.hash_into(hasher);
            }
            UpkeepCostKind::CumulativeUpkeep(cu_cost) => {
                1u8.hash_into(hasher);
                cu_cost.hash_into(hasher);
            }
        }
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
                embedded_effect,
            } => {
                1u8.hash_into(hasher);
                source_object.hash_into(hasher);
                ability_index.hash_into(hasher);
                embedded_effect.is_some().hash_into(hasher);
            }
            StackObjectKind::TriggeredAbility {
                source_object,
                ability_index,
                is_carddef_etb,
            } => {
                2u8.hash_into(hasher);
                source_object.hash_into(hasher);
                ability_index.hash_into(hasher);
                is_carddef_etb.hash_into(hasher);
            }
            // MadnessTrigger (discriminant 6) — CR 702.35a
            StackObjectKind::MadnessTrigger {
                source_object,
                exiled_card,
                madness_cost,
                owner,
            } => {
                6u8.hash_into(hasher);
                source_object.hash_into(hasher);
                exiled_card.hash_into(hasher);
                madness_cost.hash_into(hasher);
                owner.hash_into(hasher);
            }
            // MiracleTrigger (discriminant 7) — CR 702.94a
            StackObjectKind::MiracleTrigger {
                source_object,
                revealed_card,
                miracle_cost,
                owner,
            } => {
                7u8.hash_into(hasher);
                source_object.hash_into(hasher);
                revealed_card.hash_into(hasher);
                miracle_cost.hash_into(hasher);
                owner.hash_into(hasher);
            }
            // UnearthAbility (discriminant 8) — CR 702.84a
            StackObjectKind::UnearthAbility { source_object } => {
                8u8.hash_into(hasher);
                source_object.hash_into(hasher);
            }
            // SuspendCounterTrigger (discriminant 14) — CR 702.62a
            StackObjectKind::SuspendCounterTrigger {
                source_object,
                suspended_card,
            } => {
                14u8.hash_into(hasher);
                source_object.hash_into(hasher);
                suspended_card.hash_into(hasher);
            }
            // SuspendCastTrigger (discriminant 15) — CR 702.62a
            StackObjectKind::SuspendCastTrigger {
                source_object,
                suspended_card,
                owner,
            } => {
                15u8.hash_into(hasher);
                source_object.hash_into(hasher);
                suspended_card.hash_into(hasher);
                owner.hash_into(hasher);
            }
            // FlankingTrigger, RampageTrigger, ProvokeTrigger, RenownTrigger,
            // MeleeTrigger, PoisonousTrigger, EnlistTrigger: migrated to KeywordTrigger
            // NinjutsuAbility (discriminant 26) -- CR 702.49a
            StackObjectKind::NinjutsuAbility {
                source_object,
                ninja_card,
                attack_target,
                from_command_zone,
            } => {
                26u8.hash_into(hasher);
                source_object.hash_into(hasher);
                ninja_card.hash_into(hasher);
                attack_target.hash_into(hasher);
                from_command_zone.hash_into(hasher);
            }
            // EmbalmAbility (discriminant 27) -- CR 702.128a
            StackObjectKind::EmbalmAbility { source_card_id } => {
                27u8.hash_into(hasher);
                source_card_id.hash_into(hasher);
            }
            // EternalizeAbility (discriminant 28) -- CR 702.129a
            StackObjectKind::EternalizeAbility {
                source_card_id,
                source_name,
            } => {
                28u8.hash_into(hasher);
                source_card_id.hash_into(hasher);
                source_name.hash_into(hasher);
            }
            // EncoreAbility (discriminant 29) -- CR 702.141a
            StackObjectKind::EncoreAbility {
                source_card_id,
                activator,
            } => {
                29u8.hash_into(hasher);
                source_card_id.hash_into(hasher);
                activator.hash_into(hasher);
            }
            // ForecastAbility (discriminant 43) -- CR 702.57a
            StackObjectKind::ForecastAbility {
                source_object,
                embedded_effect,
            } => {
                43u8.hash_into(hasher);
                source_object.hash_into(hasher);
                embedded_effect.hash_into(hasher);
            }
            // ScavengeAbility (discriminant 45) -- CR 702.97a
            StackObjectKind::ScavengeAbility {
                source_card_id,
                power_snapshot,
            } => {
                45u8.hash_into(hasher);
                source_card_id.hash_into(hasher);
                power_snapshot.hash_into(hasher);
            }
            // BloodrushAbility (discriminant 51) -- CR 207.2c
            StackObjectKind::BloodrushAbility {
                source_object,
                target_creature,
                power_boost,
                toughness_boost,
                grants_keyword,
            } => {
                51u8.hash_into(hasher);
                source_object.hash_into(hasher);
                target_creature.hash_into(hasher);
                power_boost.hash_into(hasher);
                toughness_boost.hash_into(hasher);
                if let Some(kw) = grants_keyword {
                    1u8.hash_into(hasher);
                    kw.hash_into(hasher);
                } else {
                    0u8.hash_into(hasher);
                }
            }
            // SaddleAbility (discriminant 55) -- CR 702.171a
            StackObjectKind::SaddleAbility { source_object } => {
                55u8.hash_into(hasher);
                source_object.hash_into(hasher);
            }
            // MutatingCreatureSpell (discriminant 59) -- CR 702.140a / CR 729.2
            StackObjectKind::MutatingCreatureSpell {
                source_object,
                target,
            } => {
                59u8.hash_into(hasher);
                source_object.hash_into(hasher);
                target.hash_into(hasher);
            }
            // TransformTrigger (discriminant 60) -- CR 701.27
            StackObjectKind::TransformTrigger {
                permanent,
                ability_timestamp,
            } => {
                60u8.hash_into(hasher);
                permanent.hash_into(hasher);
                ability_timestamp.hash_into(hasher);
            }
            // CraftAbility (discriminant 61) -- CR 702.167a
            StackObjectKind::CraftAbility {
                source_card_id,
                exiled_source,
                material_ids,
                activator,
            } => {
                61u8.hash_into(hasher);
                source_card_id.hash_into(hasher);
                exiled_source.hash_into(hasher);
                for id in material_ids {
                    id.hash_into(hasher);
                }
                activator.hash_into(hasher);
            }
            // DayboundTransformTrigger (discriminant 62) -- CR 702.145b/f
            StackObjectKind::DayboundTransformTrigger { permanent } => {
                62u8.hash_into(hasher);
                permanent.hash_into(hasher);
            }
            // TurnFaceUpTrigger (discriminant 63) -- CR 708.8
            StackObjectKind::TurnFaceUpTrigger {
                permanent,
                source_card_id,
                ability_index,
            } => {
                63u8.hash_into(hasher);
                permanent.hash_into(hasher);
                match source_card_id {
                    None => 0u8.hash_into(hasher),
                    Some(cid) => {
                        1u8.hash_into(hasher);
                        cid.0.hash_into(hasher);
                    }
                }
                (*ability_index as u32).hash_into(hasher);
            }
            // KeywordTrigger (discriminant 64) -- consolidated keyword triggers
            StackObjectKind::KeywordTrigger {
                source_object,
                keyword,
                data,
            } => {
                64u8.hash_into(hasher);
                source_object.hash_into(hasher);
                keyword.hash_into(hasher);
                data.hash_into(hasher);
            }
            // RoomAbility (discriminant 65) -- CR 309.4c room triggered ability
            StackObjectKind::RoomAbility {
                owner,
                dungeon,
                room,
            } => {
                65u8.hash_into(hasher);
                owner.hash_into(hasher);
                dungeon.hash_into(hasher);
                (*room as u32).hash_into(hasher);
            }
            // RingAbility (discriminant 66) -- CR 701.54c ring-bearer triggered ability
            StackObjectKind::RingAbility {
                source_object,
                effect,
                controller,
            } => {
                66u8.hash_into(hasher);
                source_object.hash_into(hasher);
                effect.hash_into(hasher);
                controller.hash_into(hasher);
            }
            // LoyaltyAbility (discriminant 67) -- CR 606 planeswalker loyalty ability
            StackObjectKind::LoyaltyAbility {
                source_object,
                ability_index,
                effect,
            } => {
                67u8.hash_into(hasher);
                source_object.hash_into(hasher);
                ability_index.hash_into(hasher);
                effect.hash_into(hasher);
            }
            // ClassLevelAbility (discriminant 68) -- CR 716.2a Class level-up
            StackObjectKind::ClassLevelAbility {
                source_object,
                target_level,
            } => {
                68u8.hash_into(hasher);
                source_object.hash_into(hasher);
                target_level.hash_into(hasher);
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
        // M9.4: is_copy (CR 707.10) — copies don't move cards on resolution
        self.is_copy.hash_into(hasher);
        // Flashback (CR 702.34a) — exiled instead of graveyard when cast_with_flashback
        self.cast_with_flashback.hash_into(hasher);
        // Kicker (CR 702.33d) — times kicker cost was paid at cast time
        self.kicker_times_paid.hash_into(hasher);
        // Evoke (CR 702.74a) — spell was cast by paying its evoke cost
        self.was_evoked.hash_into(hasher);
        // Bestow (CR 702.103b) — spell was cast by paying its bestow cost
        self.was_bestowed.hash_into(hasher);
        // Madness (CR 702.35a) — spell was cast via madness from exile
        self.cast_with_madness.hash_into(hasher);
        // Miracle (CR 702.94a) — spell was cast via miracle from hand
        self.cast_with_miracle.hash_into(hasher);
        // Escape (CR 702.138b) — spell was cast via escape from graveyard
        self.was_escaped.hash_into(hasher);
        // Foretell (CR 702.143a) — spell was cast via foretell from exile
        self.cast_with_foretell.hash_into(hasher);
        // Buyback — spell was cast with buyback cost paid
        self.was_buyback_paid.hash_into(hasher);
        // Suspend (CR 702.62a) — spell was cast via suspend cast trigger from exile
        self.was_suspended.hash_into(hasher);
        // Overload (CR 702.96a) — spell was cast with overload cost
        self.was_overloaded.hash_into(hasher);
        // Jump-Start (CR 702.133a) — exiled instead of graveyard when cast_with_jump_start
        self.cast_with_jump_start.hash_into(hasher);
        // Aftermath (CR 702.127a) — aftermath half cast from graveyard; uses aftermath effect
        self.cast_with_aftermath.hash_into(hasher);
        // Dash (CR 702.109a) — alternative cost paid; permanent gains haste + return trigger
        self.was_dashed.hash_into(hasher);
        // Blitz (CR 702.152a) — alternative cost paid; haste + draw-on-death + sacrifice trigger
        self.was_blitzed.hash_into(hasher);
        // Plot (CR 702.170d) — spell was cast from exile as a plotted card
        self.was_plotted.hash_into(hasher);
        // Prototype (CR 718.3b) — spell was cast as a prototyped spell
        self.was_prototyped.hash_into(hasher);
        // Impending (CR 702.176a) — alternative cost paid; enters with time counters
        self.was_impended.hash_into(hasher);
        // Bargain (CR 702.166b) — spell was cast with bargain cost paid
        self.was_bargained.hash_into(hasher);
        // Surge (CR 702.117a) — spell was cast by paying its surge cost
        self.was_surged.hash_into(hasher);
        // Casualty (CR 702.153a) — spell was cast with casualty cost paid
        self.was_casualty_paid.hash_into(hasher);
        // Cleave (CR 702.148a) — spell was cast by paying its cleave cost
        self.was_cleaved.hash_into(hasher);
        // was_entwined, escalate_modes_paid: REMOVED — now in additional_costs (hashed below)
        // Splice (CR 702.47a) — spliced effects attached to this spell
        for effect in &self.spliced_effects {
            effect.hash_into(hasher);
        }
        for id in &self.spliced_card_ids {
            id.hash_into(hasher);
        }
        // devour_sacrifices: REMOVED — now in additional_costs (hashed below)
        // Modal choice (CR 700.2a) — explicit mode indices chosen at cast time
        for idx in &self.modes_chosen {
            idx.hash_into(hasher);
        }
        // was_fused: REMOVED — now in additional_costs (hashed below)
        // X value (CR 107.3m) — the value of X chosen at cast time
        self.x_value.hash_into(hasher);
        // Collect Evidence (CR 701.59c) — additional cost paid; graveyard cards were exiled
        self.evidence_collected.hash_into(hasher);
        // squad_count, offspring_paid, gift_*, mutate_*: REMOVED — now in additional_costs (hashed below)
        // Disturb (CR 702.146a / CR 712.11a) — spell was cast transformed (back face up)
        self.is_cast_transformed.hash_into(hasher);
        // RC-1: Consolidated additional costs
        (self.additional_costs.len() as u64).hash_into(hasher);
        for cost in &self.additional_costs {
            cost.hash_into(hasher);
        }
        // Note: StackObject retains its own individual boolean fields for now (separate from
        // the GameObject.cast_alt_cost consolidation) to minimize blast radius of this refactor.
    }
}

impl HashInto for CombatState {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.attacking_player.hash_into(hasher);
        self.attackers.hash_into(hasher);
        self.blockers.hash_into(hasher);
        self.damage_assignment_order.hash_into(hasher);
        // CR 702.7b: first-strike participant snapshot -- populated at start of first-strike step
        self.first_strike_participants.hash_into(hasher);
        self.defenders_declared.hash_into(hasher);
        // CR 702.39a / CR 509.1c: forced_blocks -- provoke blocking requirements
        self.forced_blocks.hash_into(hasher);
        // CR 702.154a: enlist_pairings -- enlist cost choices during declare-attackers
        (self.enlist_pairings.len() as u64).hash_into(hasher);
        for (a, b) in &self.enlist_pairings {
            a.hash_into(hasher);
            b.hash_into(hasher);
        }
        // CR 509.1h: blocked_attackers -- set at declare-blockers, never cleared
        self.blocked_attackers.hash_into(hasher);
    }
}

impl HashInto for AdditionalCost {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            AdditionalCost::Sacrifice(ids) => {
                0u8.hash_into(hasher);
                (ids.len() as u64).hash_into(hasher);
                for id in ids {
                    id.hash_into(hasher);
                }
            }
            AdditionalCost::Discard(ids) => {
                1u8.hash_into(hasher);
                (ids.len() as u64).hash_into(hasher);
                for id in ids {
                    id.hash_into(hasher);
                }
            }
            AdditionalCost::EscapeExile { cards } => {
                2u8.hash_into(hasher);
                (cards.len() as u64).hash_into(hasher);
                for id in cards {
                    id.hash_into(hasher);
                }
            }
            AdditionalCost::CollectEvidenceExile { cards } => {
                13u8.hash_into(hasher);
                (cards.len() as u64).hash_into(hasher);
                for id in cards {
                    id.hash_into(hasher);
                }
            }
            AdditionalCost::Assist { player, amount } => {
                3u8.hash_into(hasher);
                player.hash_into(hasher);
                amount.hash_into(hasher);
            }
            AdditionalCost::Replicate { count } => {
                4u8.hash_into(hasher);
                count.hash_into(hasher);
            }
            AdditionalCost::Squad { count } => {
                12u8.hash_into(hasher);
                count.hash_into(hasher);
            }
            AdditionalCost::EscalateModes { count } => {
                5u8.hash_into(hasher);
                count.hash_into(hasher);
            }
            AdditionalCost::Splice { cards } => {
                6u8.hash_into(hasher);
                (cards.len() as u64).hash_into(hasher);
                for id in cards {
                    id.hash_into(hasher);
                }
            }
            AdditionalCost::Entwine => 7u8.hash_into(hasher),
            AdditionalCost::Fuse => 8u8.hash_into(hasher),
            AdditionalCost::Offspring => 9u8.hash_into(hasher),
            AdditionalCost::Gift { opponent } => {
                10u8.hash_into(hasher);
                opponent.hash_into(hasher);
            }
            AdditionalCost::Mutate { target, on_top } => {
                11u8.hash_into(hasher);
                target.hash_into(hasher);
                on_top.hash_into(hasher);
            }
        }
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
                controller,
                pre_death_counters,
            } => {
                27u8.hash_into(hasher);
                object_id.hash_into(hasher);
                new_grave_id.hash_into(hasher);
                controller.hash_into(hasher);
                // Hash counter map for determinism (CR 702.79a — persist counter check)
                for (ct, count) in pre_death_counters.iter() {
                    ct.hash_into(hasher);
                    count.hash_into(hasher);
                }
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
            // M9.4: CascadeExiled (discriminant 66)
            GameEvent::CascadeExiled {
                player,
                cards_exiled,
            } => {
                66u8.hash_into(hasher);
                player.hash_into(hasher);
                cards_exiled.hash_into(hasher);
            }
            // M9.4: CascadeCast (discriminant 67)
            GameEvent::CascadeCast { player, card_id } => {
                67u8.hash_into(hasher);
                player.hash_into(hasher);
                card_id.hash_into(hasher);
            }
            // M9.4: SpellCopied (discriminant 65)
            GameEvent::SpellCopied {
                original_stack_id,
                copy_stack_id,
                controller,
            } => {
                65u8.hash_into(hasher);
                original_stack_id.hash_into(hasher);
                copy_stack_id.hash_into(hasher);
                controller.hash_into(hasher);
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
            // M9.4: LoopDetected (discriminant 68) — CR 104.4b
            GameEvent::LoopDetected { description } => {
                68u8.hash_into(hasher);
                description.hash_into(hasher);
            }
            // CR 702.21a: PermanentTargeted (discriminant 69) — drives Ward triggers
            GameEvent::PermanentTargeted {
                target_id,
                targeting_stack_id,
                targeting_controller,
            } => {
                69u8.hash_into(hasher);
                target_id.hash_into(hasher);
                targeting_stack_id.hash_into(hasher);
                targeting_controller.hash_into(hasher);
            }
            // CR 702.6a: EquipmentAttached (discriminant 70)
            GameEvent::EquipmentAttached {
                equipment_id,
                target_id,
                controller,
            } => {
                70u8.hash_into(hasher);
                equipment_id.hash_into(hasher);
                target_id.hash_into(hasher);
                controller.hash_into(hasher);
            }
            // CR 702.29a: CardCycled (discriminant 71)
            GameEvent::CardCycled {
                player,
                object_id,
                new_id,
            } => {
                71u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_id.hash_into(hasher);
            }
            // CR 702.52: DredgeChoiceRequired (discriminant 72)
            GameEvent::DredgeChoiceRequired { player, options } => {
                72u8.hash_into(hasher);
                player.hash_into(hasher);
                (options.len() as u64).hash_into(hasher);
                for (card_id, n) in options {
                    card_id.hash_into(hasher);
                    n.hash_into(hasher);
                }
            }
            // CR 702.52: Dredged (discriminant 73)
            GameEvent::Dredged {
                player,
                card_new_id,
                milled,
            } => {
                73u8.hash_into(hasher);
                player.hash_into(hasher);
                card_new_id.hash_into(hasher);
                milled.hash_into(hasher);
            }
            // CR 303.4a/303.4b: AuraAttached (discriminant 74)
            GameEvent::AuraAttached {
                aura_id,
                target_id,
                controller,
            } => {
                74u8.hash_into(hasher);
                aura_id.hash_into(hasher);
                target_id.hash_into(hasher);
                controller.hash_into(hasher);
            }
            // CR 701.25: Surveilled (discriminant 75)
            GameEvent::Surveilled { player, count } => {
                75u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 702.103f: BestowReverted (discriminant 76)
            GameEvent::BestowReverted { object_id } => {
                76u8.hash_into(hasher);
                object_id.hash_into(hasher);
            }
            // CR 702.94a: MiracleRevealChoiceRequired (discriminant 77)
            GameEvent::MiracleRevealChoiceRequired {
                player,
                card_object_id,
                miracle_cost,
            } => {
                77u8.hash_into(hasher);
                player.hash_into(hasher);
                card_object_id.hash_into(hasher);
                miracle_cost.hash_into(hasher);
            }
            // CR 702.143a: CardForetold (discriminant 78)
            GameEvent::CardForetold {
                player,
                object_id,
                new_exile_id,
            } => {
                78u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_exile_id.hash_into(hasher);
            }
            // CR 701.50b: Connived (discriminant 79)
            GameEvent::Connived {
                object_id,
                player,
                counters_placed,
            } => {
                79u8.hash_into(hasher);
                object_id.hash_into(hasher);
                player.hash_into(hasher);
                counters_placed.hash_into(hasher);
            }
            // CR 702.131: CitysBlessingGained (discriminant 80)
            GameEvent::CitysBlessingGained { player } => {
                80u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            // CR 702.90b / CR 120.3b: PoisonCountersGiven (discriminant 81)
            GameEvent::PoisonCountersGiven {
                player,
                amount,
                source,
            } => {
                81u8.hash_into(hasher);
                player.hash_into(hasher);
                amount.hash_into(hasher);
                source.hash_into(hasher);
            }
            // CR 701.16a: Investigated (discriminant 82)
            GameEvent::Investigated { player, count } => {
                82u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 701.19a: RegenerationShieldCreated (discriminant 83)
            GameEvent::RegenerationShieldCreated {
                object_id,
                shield_id,
                controller,
            } => {
                83u8.hash_into(hasher);
                object_id.hash_into(hasher);
                shield_id.hash_into(hasher);
                controller.hash_into(hasher);
            }
            // CR 701.19a/614.8: Regenerated (discriminant 84)
            GameEvent::Regenerated {
                object_id,
                shield_id,
            } => {
                84u8.hash_into(hasher);
                object_id.hash_into(hasher);
                shield_id.hash_into(hasher);
            }
            // CR 701.34a: Proliferated (discriminant 85)
            GameEvent::Proliferated {
                controller,
                permanents_affected,
                players_affected,
            } => {
                85u8.hash_into(hasher);
                controller.hash_into(hasher);
                permanents_affected.hash_into(hasher);
                players_affected.hash_into(hasher);
            }
            // CR 702.62a / CR 116.2f: CardSuspended (discriminant 86)
            GameEvent::CardSuspended {
                player,
                object_id,
                new_exile_id,
                time_counters,
            } => {
                86u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_exile_id.hash_into(hasher);
                time_counters.hash_into(hasher);
            }
            // CR 702.75a: HideawayExiled (discriminant 87)
            GameEvent::HideawayExiled {
                player,
                source,
                exiled_card,
                remaining_count,
            } => {
                87u8.hash_into(hasher);
                player.hash_into(hasher);
                source.hash_into(hasher);
                exiled_card.hash_into(hasher);
                remaining_count.hash_into(hasher);
            }
            // CR 702.170a: CardPlotted (discriminant 88)
            GameEvent::CardPlotted {
                player,
                object_id,
                new_exile_id,
            } => {
                88u8.hash_into(hasher);
                player.hash_into(hasher);
                object_id.hash_into(hasher);
                new_exile_id.hash_into(hasher);
            }
            // CR 702.30a: EchoPaymentRequired (discriminant 89)
            GameEvent::EchoPaymentRequired {
                player,
                permanent,
                cost,
            } => {
                89u8.hash_into(hasher);
                player.hash_into(hasher);
                permanent.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // CR 702.30a: EchoPaid (discriminant 90)
            GameEvent::EchoPaid { player, permanent } => {
                90u8.hash_into(hasher);
                player.hash_into(hasher);
                permanent.hash_into(hasher);
            }
            // CR 702.24a: CumulativeUpkeepPaymentRequired (discriminant 91)
            GameEvent::CumulativeUpkeepPaymentRequired {
                player,
                permanent,
                per_counter_cost,
                age_counter_count,
            } => {
                91u8.hash_into(hasher);
                player.hash_into(hasher);
                permanent.hash_into(hasher);
                per_counter_cost.hash_into(hasher);
                age_counter_count.hash_into(hasher);
            }
            // CR 702.24a: CumulativeUpkeepPaid (discriminant 92)
            GameEvent::CumulativeUpkeepPaid {
                player,
                permanent,
                age_counter_count,
            } => {
                92u8.hash_into(hasher);
                player.hash_into(hasher);
                permanent.hash_into(hasher);
                age_counter_count.hash_into(hasher);
            }
            // CR 702.59a: RecoverPaymentRequired (discriminant 93)
            GameEvent::RecoverPaymentRequired {
                player,
                recover_card,
                cost,
            } => {
                93u8.hash_into(hasher);
                player.hash_into(hasher);
                recover_card.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // CR 702.59a: RecoverPaid (discriminant 94)
            GameEvent::RecoverPaid {
                player,
                recover_card,
                new_hand_id,
            } => {
                94u8.hash_into(hasher);
                player.hash_into(hasher);
                recover_card.hash_into(hasher);
                new_hand_id.hash_into(hasher);
            }
            // CR 702.59a: RecoverDeclined (discriminant 95)
            GameEvent::RecoverDeclined {
                player,
                recover_card,
                new_exile_id,
            } => {
                95u8.hash_into(hasher);
                player.hash_into(hasher);
                recover_card.hash_into(hasher);
                new_exile_id.hash_into(hasher);
            }
            // CR 702.26a: PermanentsPhasedOut (discriminant 96)
            GameEvent::PermanentsPhasedOut { player, objects } => {
                96u8.hash_into(hasher);
                player.hash_into(hasher);
                objects.hash_into(hasher);
            }
            // CR 702.26a: PermanentsPhasedIn (discriminant 97)
            GameEvent::PermanentsPhasedIn { player, objects } => {
                97u8.hash_into(hasher);
                player.hash_into(hasher);
                objects.hash_into(hasher);
            }
            // CR 701.47a: Amassed (discriminant 98)
            GameEvent::Amassed {
                player,
                army_id,
                count,
            } => {
                98u8.hash_into(hasher);
                player.hash_into(hasher);
                army_id.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 702.89a: UmbraArmorApplied (discriminant 99)
            GameEvent::UmbraArmorApplied {
                protected_id,
                aura_id,
            } => {
                99u8.hash_into(hasher);
                protected_id.hash_into(hasher);
                aura_id.hash_into(hasher);
            }
            // CR 702.67a: FortificationAttached (discriminant 100)
            GameEvent::FortificationAttached {
                fortification_id,
                target_id,
                controller,
            } => {
                100u8.hash_into(hasher);
                fortification_id.hash_into(hasher);
                target_id.hash_into(hasher);
                controller.hash_into(hasher);
            }
            // B13: DiscoverExiled (discriminant 101) -- CR 701.57a
            GameEvent::DiscoverExiled {
                player,
                cards_exiled,
            } => {
                101u8.hash_into(hasher);
                player.hash_into(hasher);
                cards_exiled.hash_into(hasher);
            }
            // B13: DiscoverCast (discriminant 102) -- CR 701.57a
            GameEvent::DiscoverCast { player, card_id } => {
                102u8.hash_into(hasher);
                player.hash_into(hasher);
                card_id.hash_into(hasher);
            }
            // B13: DiscoverToHand (discriminant 103) -- CR 701.57a
            GameEvent::DiscoverToHand { player, card_id } => {
                103u8.hash_into(hasher);
                player.hash_into(hasher);
                card_id.hash_into(hasher);
            }
            // B13: CreatureSuspected (discriminant 104) -- CR 701.60
            GameEvent::CreatureSuspected {
                object_id,
                controller,
            } => {
                104u8.hash_into(hasher);
                object_id.hash_into(hasher);
                controller.hash_into(hasher);
            }
            // B13: CreatureUnsuspected (discriminant 105) -- CR 701.60
            GameEvent::CreatureUnsuspected {
                object_id,
                controller,
            } => {
                105u8.hash_into(hasher);
                object_id.hash_into(hasher);
                controller.hash_into(hasher);
            }
            // Cipher (discriminant 106) -- CR 702.99a
            GameEvent::CipherEncoded {
                player,
                exiled_card,
                creature,
            } => {
                106u8.hash_into(hasher);
                player.hash_into(hasher);
                exiled_card.hash_into(hasher);
                creature.hash_into(hasher);
            }
            // HauntExiled (discriminant 107) -- CR 702.55a
            GameEvent::HauntExiled {
                controller,
                exiled_card,
                haunted_creature,
            } => {
                107u8.hash_into(hasher);
                controller.hash_into(hasher);
                exiled_card.hash_into(hasher);
                haunted_creature.hash_into(hasher);
            }
            // CR 702.140d: Mutate — merged with target (discriminant 108)
            GameEvent::CreatureMutated { object_id, player } => {
                108u8.hash_into(hasher);
                object_id.hash_into(hasher);
                player.hash_into(hasher);
            }
            // CR 701.27a / CR 712.18: permanent transformed (discriminant 109)
            GameEvent::PermanentTransformed {
                object_id,
                to_back_face,
            } => {
                109u8.hash_into(hasher);
                object_id.hash_into(hasher);
                to_back_face.hash_into(hasher);
            }
            // CR 730.1: day/night changed (discriminant 110)
            GameEvent::DayNightChanged { now } => {
                110u8.hash_into(hasher);
                (*now as u8).hash_into(hasher);
            }
            // CR 702.167a: craft activated (discriminant 111)
            GameEvent::CraftActivated {
                player,
                exiled_source,
                exiled_materials,
            } => {
                111u8.hash_into(hasher);
                player.hash_into(hasher);
                exiled_source.hash_into(hasher);
                for mat in exiled_materials {
                    mat.hash_into(hasher);
                }
            }
            // PermanentTurnedFaceUp -- CR 702.37e / 701.40b / 701.58b
            GameEvent::PermanentTurnedFaceUp { player, permanent } => {
                112u8.hash_into(hasher);
                player.hash_into(hasher);
                permanent.hash_into(hasher);
            }
            // FaceDownRevealed -- CR 708.9
            GameEvent::FaceDownRevealed {
                player,
                permanent,
                card_name,
            } => {
                113u8.hash_into(hasher);
                player.hash_into(hasher);
                permanent.hash_into(hasher);
                card_name.hash_into(hasher);
            }
            // VenturedIntoDungeon -- CR 309.5a / CR 701.49 (discriminant 114)
            GameEvent::VenturedIntoDungeon {
                player,
                dungeon,
                room,
            } => {
                114u8.hash_into(hasher);
                player.hash_into(hasher);
                dungeon.hash_into(hasher);
                (*room as u32).hash_into(hasher);
            }
            // DungeonCompleted -- CR 309.7 (discriminant 115)
            GameEvent::DungeonCompleted { player, dungeon } => {
                115u8.hash_into(hasher);
                player.hash_into(hasher);
                dungeon.hash_into(hasher);
            }
            // InitiativeTaken -- CR 725.2 (discriminant 116)
            GameEvent::InitiativeTaken { player } => {
                116u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            // RingTempted -- CR 701.54a (discriminant 117)
            GameEvent::RingTempted { player, new_level } => {
                117u8.hash_into(hasher);
                player.hash_into(hasher);
                new_level.hash_into(hasher);
            }
            // RingBearerChosen -- CR 701.54a (discriminant 118)
            GameEvent::RingBearerChosen { player, creature } => {
                118u8.hash_into(hasher);
                player.hash_into(hasher);
                creature.hash_into(hasher);
            }
            // PlayerBecameMonarch -- CR 724.1 (discriminant 119)
            GameEvent::PlayerBecameMonarch { player } => {
                119u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            // AdditionalCombatPhaseCreated -- CR 500.8 (discriminant 120)
            GameEvent::AdditionalCombatPhaseCreated {
                controller,
                followed_by_main,
            } => {
                120u8.hash_into(hasher);
                controller.hash_into(hasher);
                followed_by_main.hash_into(hasher);
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
        self.has_subtypes.hash_into(hasher);
        self.has_name.hash_into(hasher);
        self.max_cmc.hash_into(hasher);
        self.min_cmc.hash_into(hasher);
        self.has_card_types.hash_into(hasher);
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
            TargetRequirement::TargetCardInYourGraveyard(filter) => {
                14u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            TargetRequirement::TargetCardInGraveyard(filter) => {
                15u8.hash_into(hasher);
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
        self.supertypes.hash_into(hasher);
        self.card_types.hash_into(hasher);
        self.subtypes.hash_into(hasher);
        self.keywords.hash_into(hasher);
        self.count.hash_into(hasher);
        self.tapped.hash_into(hasher);
        self.mana_color.hash_into(hasher);
        self.mana_abilities.hash_into(hasher);
        self.activated_abilities.hash_into(hasher);
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
            PlayerTarget::OwnerOf(target) => {
                5u8.hash_into(hasher);
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
            EffectAmount::PermanentCount { filter, controller } => {
                6u8.hash_into(hasher);
                filter.hash_into(hasher);
                controller.hash_into(hasher);
            }
            EffectAmount::DevotionTo(color) => {
                7u8.hash_into(hasher);
                color.hash_into(hasher);
            }
            EffectAmount::CounterCount { target, counter } => {
                8u8.hash_into(hasher);
                target.hash_into(hasher);
                counter.hash_into(hasher);
            }
            // LastEffectCount (discriminant 9) — reads ctx.last_effect_count set by DestroyAll/ExileAll
            EffectAmount::LastEffectCount => 9u8.hash_into(hasher),
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
            // CR 702.91a: All attacking creatures except the source (battle cry source).
            ForEachTarget::EachOtherAttackingCreature => 7u8.hash_into(hasher),
            // CR 500.8: EachAttackingCreature (discriminant 8)
            ForEachTarget::EachAttackingCreature => 8u8.hash_into(hasher),
            // CR 702.61a: All creatures the controller controls except the source (discriminant 9)
            ForEachTarget::EachOtherCreatureYouControl => 9u8.hash_into(hasher),
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
            // CR 702.21a: Ward trigger condition — discriminant 17
            TriggerCondition::WhenBecomesTargetByOpponent => 17u8.hash_into(hasher),
            // CR 701.25d: "Whenever you surveil" — discriminant 18
            TriggerCondition::WheneverYouSurveil => 18u8.hash_into(hasher),
            // CR 701.50b: "Whenever this creature connives" — discriminant 19
            TriggerCondition::WhenConnives => 19u8.hash_into(hasher),
            // CR 701.16a: "Whenever you investigate" — discriminant 20
            TriggerCondition::WheneverYouInvestigate => 20u8.hash_into(hasher),
            // CR 702.104b: "When ~ enters, if tribute wasn't paid" — discriminant 21
            TriggerCondition::TributeNotPaid => 21u8.hash_into(hasher),
            // CR 207.2c / CR 120.3: "Whenever this creature is dealt damage" (Enrage) — discriminant 22
            TriggerCondition::WhenDealtDamage => 22u8.hash_into(hasher),
            // CR 702.55c: "When the creature it haunts dies" — discriminant 23
            TriggerCondition::HauntedCreatureDies => 23u8.hash_into(hasher),
            // CR 702.140d: "Whenever this creature mutates" — discriminant 24
            TriggerCondition::WhenMutates => 24u8.hash_into(hasher),
            // CR 708.8: "When this permanent is turned face up" — discriminant 25
            TriggerCondition::WhenTurnedFaceUp => 25u8.hash_into(hasher),
            // CR 701.54d: "Whenever the Ring tempts you" — discriminant 26
            TriggerCondition::WheneverRingTemptsYou => 26u8.hash_into(hasher),
            TriggerCondition::WhenSelfBecomesTapped => 27u8.hash_into(hasher),
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
            // Kicker condition (discriminant 7) — CR 702.33d
            Condition::WasKicked => 7u8.hash_into(hasher),
            // SourceHasNoCountersOfType (discriminant 8) — used by Adapt (CR 701.46)
            Condition::SourceHasNoCountersOfType { counter } => {
                8u8.hash_into(hasher);
                counter.hash_into(hasher);
            }
            // Overload condition (discriminant 9) — CR 702.96a
            Condition::WasOverloaded => 9u8.hash_into(hasher),
            // Bargain condition (discriminant 10) — CR 702.166b
            Condition::WasBargained => 10u8.hash_into(hasher),
            // Cleave condition (discriminant 11) — CR 702.148a
            Condition::WasCleaved => 11u8.hash_into(hasher),
            // Corrupted condition (discriminant 12) — CR 207.2c ability word
            Condition::OpponentHasPoisonCounters(n) => {
                12u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // EvidenceWasCollected condition (discriminant 13) — CR 701.59c
            Condition::EvidenceWasCollected => 13u8.hash_into(hasher),
            // GiftWasGiven condition (discriminant 14) — CR 702.174b
            Condition::GiftWasGiven => 14u8.hash_into(hasher),
            // CompletedADungeon condition (discriminant 15) — CR 309.7
            Condition::CompletedADungeon => 15u8.hash_into(hasher),
            // CompletedSpecificDungeon condition (discriminant 16) — CR 309.7
            Condition::CompletedSpecificDungeon(dungeon_id) => {
                16u8.hash_into(hasher);
                dungeon_id.hash_into(hasher);
            }
            // Not condition (discriminant 17) — logical negation of inner condition
            Condition::Not(inner) => {
                17u8.hash_into(hasher);
                inner.hash_into(hasher);
            }
            // RingHasTemptedYou condition (discriminant 18) — CR 701.54c
            Condition::RingHasTemptedYou(n) => {
                18u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            // Or condition (discriminant 19) — logical disjunction
            Condition::Or(a, b) => {
                19u8.hash_into(hasher);
                a.hash_into(hasher);
                b.hash_into(hasher);
            }
            // ── ETB condition variants (PB-2, discriminants 20-26) ───────────
            Condition::ControlLandWithSubtypes(subtypes) => {
                20u8.hash_into(hasher);
                (subtypes.len() as u32).hash_into(hasher);
                for st in subtypes {
                    st.hash_into(hasher);
                }
            }
            Condition::ControlAtMostNOtherLands(n) => {
                21u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            Condition::HaveTwoOrMoreOpponents => 22u8.hash_into(hasher),
            Condition::CanRevealFromHandWithSubtype(subtypes) => {
                23u8.hash_into(hasher);
                (subtypes.len() as u32).hash_into(hasher);
                for st in subtypes {
                    st.hash_into(hasher);
                }
            }
            Condition::ControlBasicLandsAtLeast(n) => {
                24u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            Condition::ControlAtLeastNOtherLands(n) => {
                25u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            Condition::ControlAtLeastNOtherLandsWithSubtype { count, subtype } => {
                26u8.hash_into(hasher);
                count.hash_into(hasher);
                subtype.hash_into(hasher);
            }
            Condition::ControlLegendaryCreature => {
                27u8.hash_into(hasher);
            }
            Condition::ControlCreatureWithSubtype(subtype) => {
                28u8.hash_into(hasher);
                subtype.hash_into(hasher);
            }
            Condition::HasCitysBlessing => 29u8.hash_into(hasher),
            // CR 500.8: IsFirstCombatPhase (discriminant 30)
            Condition::IsFirstCombatPhase => 30u8.hash_into(hasher),
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
            Cost::Forage => 6u8.hash_into(hasher),
            Cost::SacrificeSelf => 7u8.hash_into(hasher),
            Cost::DiscardSelf => 8u8.hash_into(hasher),
        }
    }
}

impl HashInto for LoyaltyCost {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            LoyaltyCost::Plus(n) => {
                0u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            LoyaltyCost::Minus(n) => {
                1u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            LoyaltyCost::Zero => 2u8.hash_into(hasher),
            LoyaltyCost::MinusX => 3u8.hash_into(hasher),
        }
    }
}

impl HashInto for ModeSelection {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.min_modes.hash_into(hasher);
        self.max_modes.hash_into(hasher);
        self.modes.hash_into(hasher);
        // CR 700.2d: allow_duplicate_modes flag affects legality of mode choices.
        self.allow_duplicate_modes.hash_into(hasher);
        // CR 700.2h / 702.172a: per-mode costs for spree spells.
        if let Some(costs) = &self.mode_costs {
            true.hash_into(hasher);
            for cost in costs {
                cost.hash_into(hasher);
            }
        } else {
            false.hash_into(hasher);
        }
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
            Effect::AddManaScaled {
                player,
                color,
                count,
            } => {
                56u8.hash_into(hasher);
                player.hash_into(hasher);
                color.hash_into(hasher);
                count.hash_into(hasher);
            }
            Effect::AddManaRestricted {
                player,
                mana,
                restriction,
            } => {
                57u8.hash_into(hasher);
                player.hash_into(hasher);
                mana.hash_into(hasher);
                restriction.hash_into(hasher);
            }
            Effect::AddManaAnyColorRestricted {
                player,
                restriction,
            } => {
                58u8.hash_into(hasher);
                player.hash_into(hasher);
                restriction.hash_into(hasher);
            }
            Effect::ChooseCreatureType { default } => {
                59u8.hash_into(hasher);
                default.0.hash_into(hasher);
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
            Effect::MoveZone {
                target,
                to,
                controller_override,
            } => {
                17u8.hash_into(hasher);
                target.hash_into(hasher);
                to.hash_into(hasher);
                controller_override.hash_into(hasher);
            }
            Effect::SearchLibrary {
                player,
                filter,
                reveal,
                destination,
                shuffle_before_placing,
            } => {
                18u8.hash_into(hasher);
                player.hash_into(hasher);
                filter.hash_into(hasher);
                reveal.hash_into(hasher);
                destination.hash_into(hasher);
                shuffle_before_placing.hash_into(hasher);
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
            // CR 701.17a: SacrificePermanents (discriminant 31) — used by Annihilator
            Effect::SacrificePermanents { player, count } => {
                31u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 702.6a / CR 701.3a: AttachEquipment (discriminant 30)
            Effect::AttachEquipment { equipment, target } => {
                30u8.hash_into(hasher);
                equipment.hash_into(hasher);
                target.hash_into(hasher);
            }
            // CR 701.25: Surveil (discriminant 32)
            Effect::Surveil { player, count } => {
                32u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 702.101a: DrainLife (discriminant 33)
            Effect::DrainLife { amount } => {
                33u8.hash_into(hasher);
                amount.hash_into(hasher);
            }
            // CR 702.92a: CreateTokenAndAttachSource (discriminant 34)
            Effect::CreateTokenAndAttachSource { spec } => {
                34u8.hash_into(hasher);
                spec.hash_into(hasher);
            }
            // Connive (discriminant 35)
            Effect::Connive { target, count } => {
                35u8.hash_into(hasher);
                target.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 701.16a: Investigate (discriminant 36)
            Effect::Investigate { count } => {
                36u8.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 701.19a: Regenerate (discriminant 37)
            Effect::Regenerate { target } => {
                37u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            // CR 701.34a: Proliferate (discriminant 38)
            Effect::Proliferate => {
                38u8.hash_into(hasher);
            }
            // CR 702.75a / CR 607.2a: PlayExiledCard (discriminant 39)
            Effect::PlayExiledCard => {
                39u8.hash_into(hasher);
            }
            // CR 701.39: Bolster (discriminant 40)
            Effect::Bolster { player, count } => {
                40u8.hash_into(hasher);
                player.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 701.47a: Amass (discriminant 41)
            Effect::Amass { subtype, count } => {
                41u8.hash_into(hasher);
                subtype.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 702.67a / CR 701.3a: AttachFortification (discriminant 42)
            Effect::AttachFortification {
                fortification,
                target,
            } => {
                42u8.hash_into(hasher);
                fortification.hash_into(hasher);
                target.hash_into(hasher);
            }
            // CR 701.57a: Discover (discriminant 43)
            Effect::Discover { player, n } => {
                43u8.hash_into(hasher);
                player.hash_into(hasher);
                n.hash_into(hasher);
            }
            // B13: Suspect (discriminant 44) -- CR 701.60a
            Effect::Suspect { target } => {
                44u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            // B13: Unsuspect (discriminant 45) -- CR 701.60a
            Effect::Unsuspect { target } => {
                45u8.hash_into(hasher);
                target.hash_into(hasher);
            }
            // CR 702.151a: DetachEquipment (discriminant 46)
            Effect::DetachEquipment { equipment } => {
                46u8.hash_into(hasher);
                equipment.hash_into(hasher);
            }
            // CR 701.40a: Manifest (discriminant 47)
            Effect::Manifest { player } => {
                47u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            // CR 701.58a: Cloak (discriminant 48)
            Effect::Cloak { player } => {
                48u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            // VentureIntoDungeon effect (discriminant 49) — CR 701.49
            Effect::VentureIntoDungeon => 49u8.hash_into(hasher),
            // TakeTheInitiative effect (discriminant 50) — CR 725.2
            Effect::TakeTheInitiative => 50u8.hash_into(hasher),
            // TheRingTemptsYou effect (discriminant 51) — CR 701.54
            Effect::TheRingTemptsYou => 51u8.hash_into(hasher),
            // BecomeMonarch effect (discriminant 52) — CR 724.1
            Effect::BecomeMonarch { player } => {
                52u8.hash_into(hasher);
                player.hash_into(hasher);
            }
            // Meld effect (discriminant 53) — CR 701.42
            Effect::Meld => 53u8.hash_into(hasher),
            // CR 701.8: DestroyAll (discriminant 54)
            Effect::DestroyAll {
                filter,
                cant_be_regenerated,
            } => {
                54u8.hash_into(hasher);
                filter.hash_into(hasher);
                cant_be_regenerated.hash_into(hasher);
            }
            // CR 406.2: ExileAll (discriminant 55)
            Effect::ExileAll { filter } => {
                55u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            // CR 122: AddCounterAmount (discriminant 56) — dynamic count via EffectAmount
            Effect::AddCounterAmount {
                target,
                counter,
                count,
            } => {
                56u8.hash_into(hasher);
                target.hash_into(hasher);
                counter.hash_into(hasher);
                count.hash_into(hasher);
            }
            // CR 500.8: AdditionalCombatPhase (discriminant 57)
            Effect::AdditionalCombatPhase { followed_by_main } => {
                57u8.hash_into(hasher);
                followed_by_main.hash_into(hasher);
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
                targets,
            } => {
                0u8.hash_into(hasher);
                cost.hash_into(hasher);
                effect.hash_into(hasher);
                timing_restriction.hash_into(hasher);
                targets.hash_into(hasher);
            }
            AbilityDefinition::Triggered {
                trigger_condition,
                effect,
                intervening_if,
                targets,
            } => {
                1u8.hash_into(hasher);
                trigger_condition.hash_into(hasher);
                effect.hash_into(hasher);
                intervening_if.hash_into(hasher);
                targets.hash_into(hasher);
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
                unless_condition,
            } => {
                5u8.hash_into(hasher);
                trigger.hash_into(hasher);
                modification.hash_into(hasher);
                is_self.hash_into(hasher);
                match unless_condition {
                    None => 0u8.hash_into(hasher),
                    Some(cond) => {
                        1u8.hash_into(hasher);
                        cond.hash_into(hasher);
                    }
                }
            }
            AbilityDefinition::OpeningHand => 6u8.hash_into(hasher),
            // M9.4: TriggerDoubling (discriminant 7) — CR 603.2d
            AbilityDefinition::TriggerDoubling {
                filter,
                additional_triggers,
            } => {
                7u8.hash_into(hasher);
                filter.hash_into(hasher);
                additional_triggers.hash_into(hasher);
            }
            // AltCastAbility (discriminant 8) — RC-3 consolidation
            AbilityDefinition::AltCastAbility {
                kind,
                cost,
                details,
            } => {
                8u8.hash_into(hasher);
                (*kind as u8).hash_into(hasher);
                cost.hash_into(hasher);
                match details {
                    Some(crate::cards::card_definition::AltCastDetails::Escape { exile_count }) => {
                        1u8.hash_into(hasher);
                        exile_count.hash_into(hasher);
                    }
                    Some(crate::cards::card_definition::AltCastDetails::Prototype {
                        power,
                        toughness,
                    }) => {
                        2u8.hash_into(hasher);
                        power.hash_into(hasher);
                        toughness.hash_into(hasher);
                    }
                    None => {
                        0u8.hash_into(hasher);
                    }
                }
            }
            // Cycling (discriminant 9) — CR 702.29
            AbilityDefinition::Cycling { cost } => {
                9u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Kicker (discriminant 10) — CR 702.33
            AbilityDefinition::Kicker {
                cost,
                is_multikicker,
            } => {
                10u8.hash_into(hasher);
                cost.hash_into(hasher);
                is_multikicker.hash_into(hasher);
            }
            // Evoke (discriminant 11) — CR 702.74
            AbilityDefinition::Evoke { cost } => {
                11u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Bestow (discriminant 12) — CR 702.103
            AbilityDefinition::Bestow { cost } => {
                12u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Madness (discriminant 13) -- CR 702.35
            AbilityDefinition::Madness { cost } => {
                13u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Miracle (discriminant 14) -- CR 702.94
            AbilityDefinition::Miracle { cost } => {
                14u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // EscapeWithCounter (discriminant 16) -- CR 702.138c
            AbilityDefinition::EscapeWithCounter {
                counter_type,
                count,
            } => {
                16u8.hash_into(hasher);
                counter_type.hash_into(hasher);
                count.hash_into(hasher);
            }
            // Foretell (discriminant 17) -- CR 702.143
            AbilityDefinition::Foretell { cost } => {
                17u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Buyback (discriminant 19) -- CR 702.27
            AbilityDefinition::Buyback { cost } => {
                19u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Suspend (discriminant 20) -- CR 702.62
            AbilityDefinition::Suspend {
                cost,
                time_counters,
            } => {
                20u8.hash_into(hasher);
                cost.hash_into(hasher);
                time_counters.hash_into(hasher);
            }
            // Overload (discriminant 21) -- CR 702.96
            AbilityDefinition::Overload { cost } => {
                21u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Ninjutsu (discriminant 22) -- CR 702.49
            AbilityDefinition::Ninjutsu { cost } => {
                22u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // CommanderNinjutsu (discriminant 23) -- CR 702.49d
            AbilityDefinition::CommanderNinjutsu { cost } => {
                23u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Aftermath (discriminant 24) -- CR 702.127
            AbilityDefinition::Aftermath {
                name,
                cost,
                card_type,
                effect,
                targets,
            } => {
                24u8.hash_into(hasher);
                name.hash_into(hasher);
                cost.hash_into(hasher);
                card_type.hash_into(hasher);
                effect.hash_into(hasher);
                targets.hash_into(hasher);
            }
            // Impending (discriminant 32) -- CR 702.176
            AbilityDefinition::Impending { cost, count } => {
                32u8.hash_into(hasher);
                cost.hash_into(hasher);
                count.hash_into(hasher);
            }
            // Emerge (discriminant 33) -- CR 702.119
            AbilityDefinition::Emerge { cost } => {
                33u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Spectacle (discriminant 34) -- CR 702.137
            AbilityDefinition::Spectacle { cost } => {
                34u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Surge (discriminant 35) -- CR 702.117
            AbilityDefinition::Surge { cost } => {
                35u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Replicate (discriminant 36) -- CR 702.56
            AbilityDefinition::Replicate { cost } => {
                36u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Cleave (discriminant 37) -- CR 702.148
            AbilityDefinition::Cleave { cost } => {
                37u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Splice (discriminant 38) -- CR 702.47
            AbilityDefinition::Splice {
                cost,
                onto_subtype,
                effect,
            } => {
                38u8.hash_into(hasher);
                cost.hash_into(hasher);
                onto_subtype.0.hash_into(hasher);
                effect.hash_into(hasher);
            }
            // Entwine (discriminant 39) -- CR 702.42
            AbilityDefinition::Entwine { cost } => {
                39u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Escalate (discriminant 40) -- CR 702.120
            AbilityDefinition::Escalate { cost } => {
                40u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Vanishing (discriminant 41) -- CR 702.63
            AbilityDefinition::Vanishing { count } => {
                41u8.hash_into(hasher);
                count.hash_into(hasher);
            }
            // Fading (discriminant 42) -- CR 702.32
            AbilityDefinition::Fading { count } => {
                42u8.hash_into(hasher);
                count.hash_into(hasher);
            }
            // Echo (discriminant 43) -- CR 702.30
            AbilityDefinition::Echo { cost } => {
                43u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // CumulativeUpkeep (discriminant 44) -- CR 702.24
            AbilityDefinition::CumulativeUpkeep { cost } => {
                44u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Recover (discriminant 45) -- CR 702.59
            AbilityDefinition::Recover { cost } => {
                45u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Forecast (discriminant 46) -- CR 702.57
            AbilityDefinition::Forecast { cost, effect } => {
                46u8.hash_into(hasher);
                cost.hash_into(hasher);
                effect.hash_into(hasher);
            }
            // Scavenge (discriminant 47) -- CR 702.97
            AbilityDefinition::Scavenge { cost } => {
                47u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Outlast (discriminant 48) -- CR 702.107
            AbilityDefinition::Outlast { cost } => {
                48u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Champion (discriminant 49) -- CR 702.72
            AbilityDefinition::Champion { filter } => {
                49u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            // Soulbond (discriminant 50) -- CR 702.95
            AbilityDefinition::Soulbond { grants } => {
                50u8.hash_into(hasher);
                grants.hash_into(hasher);
            }
            // Fuse (discriminant 51) -- CR 702.102
            AbilityDefinition::Fuse {
                name,
                cost,
                card_type,
                effect,
                targets,
            } => {
                51u8.hash_into(hasher);
                name.hash_into(hasher);
                cost.hash_into(hasher);
                card_type.hash_into(hasher);
                effect.hash_into(hasher);
                targets.hash_into(hasher);
            }
            // Bloodrush (discriminant 52) -- CR 207.2c (ability word)
            AbilityDefinition::Bloodrush {
                cost,
                power_boost,
                toughness_boost,
                grants_keyword,
            } => {
                52u8.hash_into(hasher);
                cost.hash_into(hasher);
                power_boost.hash_into(hasher);
                toughness_boost.hash_into(hasher);
                if let Some(kw) = grants_keyword {
                    1u8.hash_into(hasher);
                    kw.hash_into(hasher);
                } else {
                    0u8.hash_into(hasher);
                }
            }
            // CollectEvidence (discriminant 53) -- CR 701.59a
            AbilityDefinition::CollectEvidence {
                threshold,
                mandatory,
            } => {
                53u8.hash_into(hasher);
                threshold.hash_into(hasher);
                mandatory.hash_into(hasher);
            }
            // Squad (discriminant 54) -- CR 702.157a
            AbilityDefinition::Squad { cost } => {
                54u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Offspring (discriminant 55) -- CR 702.175a
            AbilityDefinition::Offspring { cost } => {
                55u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Gift (discriminant 56) -- CR 702.174a
            AbilityDefinition::Gift { gift_type } => {
                56u8.hash_into(hasher);
                gift_type.hash_into(hasher);
            }
            // Cipher (discriminant 57) -- CR 702.99a
            AbilityDefinition::Cipher => 57u8.hash_into(hasher),
            // Reconfigure (discriminant 58) -- CR 702.151a
            AbilityDefinition::Reconfigure { cost } => {
                58u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // MutateCost (discriminant 59) -- CR 702.140a
            AbilityDefinition::MutateCost { cost } => {
                59u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Disturb (discriminant 60) -- CR 702.146a
            AbilityDefinition::Disturb { cost } => {
                60u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Craft (discriminant 61) -- CR 702.167a
            AbilityDefinition::Craft { cost, materials } => {
                61u8.hash_into(hasher);
                cost.hash_into(hasher);
                materials.hash_into(hasher);
            }
            // Morph -- CR 702.37a — discriminant 62
            AbilityDefinition::Morph { cost } => {
                62u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Megamorph -- CR 702.37b — discriminant 63
            AbilityDefinition::Megamorph { cost } => {
                63u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // Disguise -- CR 702.168a — discriminant 64
            AbilityDefinition::Disguise { cost } => {
                64u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            // SuppressCreatureETBTriggers -- IG-2 (CR 614.16a) — discriminant 65
            AbilityDefinition::SuppressCreatureETBTriggers { filter } => {
                65u8.hash_into(hasher);
                filter.hash_into(hasher);
            }
            AbilityDefinition::LoyaltyAbility {
                cost,
                effect,
                targets,
            } => {
                66u8.hash_into(hasher);
                cost.hash_into(hasher);
                effect.hash_into(hasher);
                targets.hash_into(hasher);
            }
            // SagaChapter (discriminant 67) -- CR 714.2
            AbilityDefinition::SagaChapter {
                chapter,
                effect,
                targets,
            } => {
                67u8.hash_into(hasher);
                chapter.hash_into(hasher);
                effect.hash_into(hasher);
                targets.hash_into(hasher);
            }
            // ClassLevel (discriminant 68) -- CR 716.2
            AbilityDefinition::ClassLevel {
                level,
                cost,
                abilities,
            } => {
                68u8.hash_into(hasher);
                level.hash_into(hasher);
                cost.hash_into(hasher);
                abilities.hash_into(hasher);
            }
            // StaticRestriction (discriminant 69) -- PB-18 stax
            AbilityDefinition::StaticRestriction { restriction } => {
                69u8.hash_into(hasher);
                restriction.hash_into(hasher);
            }
        }
    }
}

impl HashInto for crate::cards::card_definition::CraftMaterials {
    fn hash_into(&self, hasher: &mut Hasher) {
        use crate::cards::card_definition::CraftMaterials;
        match self {
            CraftMaterials::Artifacts(n) => {
                0u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            CraftMaterials::Creatures(n) => {
                1u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            CraftMaterials::Lands(n) => {
                2u8.hash_into(hasher);
                n.hash_into(hasher);
            }
            CraftMaterials::AnyCards(n) => {
                3u8.hash_into(hasher);
                n.hash_into(hasher);
            }
        }
    }
}

impl HashInto for SoulbondGrant {
    fn hash_into(&self, hasher: &mut Hasher) {
        self.layer.hash_into(hasher);
        self.modification.hash_into(hasher);
    }
}

impl HashInto for crate::cards::card_definition::GiftType {
    fn hash_into(&self, hasher: &mut Hasher) {
        use crate::cards::card_definition::GiftType;
        match self {
            GiftType::Food => 0u8.hash_into(hasher),
            GiftType::Card => 1u8.hash_into(hasher),
            GiftType::TappedFish => 2u8.hash_into(hasher),
            GiftType::Treasure => 3u8.hash_into(hasher),
            GiftType::Octopus => 4u8.hash_into(hasher),
            GiftType::ExtraTurn => 5u8.hash_into(hasher),
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
    /// Excludes:
    /// - Event history (O(n) in game length)
    /// - Hand contents and library contents (hidden information)
    /// - `loop_detection_hashes` (M9.4 CR 104.4b): this field is metadata used by the
    ///   loop-detection algorithm, not actual game state. Different engine instances may
    ///   accumulate different hash histories depending on when their mandatory-action
    ///   sequences began, so including it in the public hash would cause false mismatches.
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
        // M9.4: trigger_doublers (CR 603.2d) — Panharmonicon-style doubling
        self.trigger_doublers.hash_into(&mut hasher);
        // IG-2: etb_suppressors (CR 614.16a) — Torpor Orb-style ETB suppression
        self.etb_suppressors.hash_into(&mut hasher);
        // PB-18: active restrictions (Rule of Law, Propaganda, etc.)
        self.restrictions.hash_into(&mut hasher);
        self.stack_objects.hash_into(&mut hasher);

        // 6. Combat state
        self.combat.hash_into(&mut hasher);

        // 7. Gravestorm counter (CR 702.69a)
        self.permanents_put_into_graveyard_this_turn
            .hash_into(&mut hasher);

        // 8. Echo payment choices (CR 702.30a)
        for (player, oid, cost) in self.pending_echo_payments.iter() {
            player.hash_into(&mut hasher);
            oid.hash_into(&mut hasher);
            cost.hash_into(&mut hasher);
        }

        // 9. Cumulative upkeep payment choices (CR 702.24a)
        for (player, oid, cost) in self.pending_cumulative_upkeep_payments.iter() {
            player.hash_into(&mut hasher);
            oid.hash_into(&mut hasher);
            cost.hash_into(&mut hasher);
        }

        // 10. Recover payment choices (CR 702.59a)
        for (player, oid, cost) in self.pending_recover_payments.iter() {
            player.hash_into(&mut hasher);
            oid.hash_into(&mut hasher);
            cost.hash_into(&mut hasher);
        }

        // 11. Forecast once-per-turn tracking (CR 702.57b)
        for card_id in self.forecast_used_this_turn.iter() {
            card_id.hash_into(&mut hasher);
        }

        // 12. Day/Night designation (CR 730.1) and previous turn spell count (CR 730.2)
        match self.day_night {
            None => 0u8.hash_into(&mut hasher),
            Some(crate::state::DayNight::Day) => 1u8.hash_into(&mut hasher),
            Some(crate::state::DayNight::Night) => 2u8.hash_into(&mut hasher),
        }
        self.previous_turn_spells_cast.hash_into(&mut hasher);

        // 13. Dungeon state (CR 309.4) and initiative (CR 725.1)
        (self.dungeon_state.len() as u64).hash_into(&mut hasher);
        for (player_id, ds) in &self.dungeon_state {
            player_id.hash_into(&mut hasher);
            ds.hash_into(&mut hasher);
        }
        self.has_initiative.hash_into(&mut hasher);
        self.monarch.hash_into(&mut hasher);

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
