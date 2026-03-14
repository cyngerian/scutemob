//! Continuous effects and the layer system type definitions (CR 613).
//!
//! Continuous effects modify game objects and apply in a strict order across eight
//! layers (CR 613.1). Within each layer, effects apply in timestamp order (CR 613.7)
//! unless a dependency relationship overrides the timestamp (CR 613.8).

use im::OrdSet;
use serde::{Deserialize, Serialize};

use super::game_object::ObjectId;
use super::player::PlayerId;
use super::types::{CardType, Color, KeywordAbility, SubType, SuperType};

/// Unique identifier for a continuous effect instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EffectId(pub u64);

/// Which layer a continuous effect applies in (CR 613.1).
///
/// Effects must be applied in layer order (Copy → Control → Text → TypeChange →
/// ColorChange → Ability → PtCda → PtSet → PtModify → PtSwitch).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EffectLayer {
    /// Layer 1a: copy effects (CR 613.1a, 707).
    Copy,
    /// Layer 2: control-changing effects (CR 613.1b).
    Control,
    /// Layer 3: text-changing effects (CR 613.1c).
    Text,
    /// Layer 4: type-changing effects — card type, supertype, subtype (CR 613.1d).
    TypeChange,
    /// Layer 5: color-changing effects (CR 613.1e).
    ColorChange,
    /// Layer 6: ability-adding and ability-removing effects (CR 613.1f).
    Ability,
    /// Layer 7a: P/T from characteristic-defining abilities (CR 613.4a).
    PtCda,
    /// Layer 7b: P/T-setting effects ("base power and toughness") (CR 613.4b).
    PtSet,
    /// Layer 7c: P/T-modifying effects including counters (CR 613.4c).
    PtModify,
    /// Layer 7d: P/T-switching effects (CR 613.4d).
    PtSwitch,
}

/// How long a continuous effect lasts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectDuration {
    /// Active as long as the source permanent is on the battlefield (CR 611.2b).
    /// The most common duration for effects from static abilities of permanents.
    WhileSourceOnBattlefield,
    /// Expires at the next cleanup step (CR 514.2).
    /// Used for "until end of turn" effects from instants, sorceries, and abilities.
    UntilEndOfTurn,
    /// Never expires on its own (e.g., certain spell effects with no stated duration).
    Indefinite,
    /// Active as long as both ObjectIds are on the battlefield and paired with each other
    /// (CR 702.95a "for as long as both remain creatures on the battlefield under your control").
    /// Used for soulbond "as long as paired" grants registered at SoulbondTrigger resolution.
    WhilePaired(ObjectId, ObjectId),
}

/// Which objects a continuous effect applies to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectFilter {
    /// Applies to a specific object by ID (e.g., an Aura targeting a creature).
    SingleObject(ObjectId),
    /// Applies to all creature permanents on the battlefield.
    AllCreatures,
    /// Applies to all land permanents on the battlefield.
    AllLands,
    /// Applies to all nonbasic land permanents (no Basic supertype).
    AllNonbasicLands,
    /// Applies to all enchantment permanents on the battlefield.
    AllEnchantments,
    /// Applies to all non-Aura enchantment permanents (for Opalescence).
    AllNonAuraEnchantments,
    /// Applies to all permanents (any permanent type) on the battlefield.
    AllPermanents,
    /// Applies to all cards in any graveyard (for Yixlid Jailer-style effects).
    AllCardsInGraveyards,
    /// Applies to all permanents controlled by a specific player.
    ControlledBy(PlayerId),
    /// Applies to all creature permanents controlled by a specific player.
    CreaturesControlledBy(PlayerId),
    /// Applies to the creature that the source Equipment or Aura is attached to.
    ///
    /// Resolved at characteristic-calculation time: the source object's `attached_to`
    /// field points to the target creature. Used for Equipment static abilities such
    /// as Lightning Greaves ("equipped creature has haste and shroud").
    AttachedCreature,
    /// Applies to the land that the source Fortification is attached to.
    ///
    /// Resolved at characteristic-calculation time: the source object's `attached_to`
    /// field points to the target land. Used for Fortification static abilities such
    /// as Darksteel Garrison ("fortified land has indestructible").
    AttachedLand,
    /// Placeholder filter for effects whose target is declared at resolution time.
    ///
    /// When `Effect::ApplyContinuousEffect` is executed, any `DeclaredTarget` filter
    /// is replaced at runtime with `SingleObject(resolved_id)` using the declared
    /// target at the given `index`. Used for activated abilities like Rogue's Passage
    /// ("{4},{T}: target creature can't be blocked this turn").
    DeclaredTarget {
        /// Index into the declared targets list (0-indexed).
        index: usize,
    },
    /// Applies to the source object of the effect (e.g., "this creature gets +1/+1").
    ///
    /// Resolved at `ApplyContinuousEffect` execution time to `SingleObject(ctx.source)`.
    /// Used by keyword abilities like Prowess where the effect targets the source creature.
    Source,
    /// Applies to all creature permanents controlled by the source's controller.
    ///
    /// Resolved dynamically at layer-application time using `effect.source` to determine
    /// the controller. Used for CardDef static abilities like Fervor ("Creatures you
    /// control have haste.") where the controller isn't known at definition time.
    CreaturesYouControl,
    /// Applies to all creature permanents controlled by the source's controller, excluding
    /// the source object itself.
    ///
    /// Used for "lord" effects like Dragonlord Kolaghan ("Other creatures you control
    /// have haste.") where the source doesn't benefit from its own static ability.
    OtherCreaturesYouControl,
    /// Applies to creature permanents controlled by the source's controller that have the
    /// specified subtype, excluding the source object itself.
    ///
    /// Used for tribal lords like Markov Baron ("Other Vampires you control get +1/+1.")
    /// where only creatures of a specific type benefit.
    OtherCreaturesYouControlWithSubtype(SubType),
}

/// What a continuous effect does when applied.
///
/// Each variant corresponds to a specific type of game modification within a layer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerModification {
    // --- Layer 1: Copy effects ---
    /// Sets the copiable values of the object to those of another object (CR 707).
    CopyOf(ObjectId),

    // --- Layer 2: Control-changing ---
    /// Changes the controller of a permanent to a specific player (CR 613.1b).
    SetController(PlayerId),

    // --- Layer 4: Type-changing ---
    /// Sets the complete type line: replaces all supertypes, card types, and subtypes.
    ///
    /// Used by Blood Moon ("Nonbasic lands are Mountains" → type becomes Land — Mountain).
    /// This overrides all prior type-adding effects when applied after them.
    SetTypeLine {
        supertypes: OrdSet<SuperType>,
        card_types: OrdSet<CardType>,
        subtypes: OrdSet<SubType>,
    },
    /// Adds card types without removing existing ones.
    ///
    /// Used by effects like Opalescence ("each other non-Aura enchantment is a creature
    /// in addition to its other types").
    AddCardTypes(OrdSet<CardType>),
    /// Adds subtypes without removing existing ones.
    ///
    /// Used by Urborg, Tomb of Yawgmoth ("Each land is a Swamp in addition to its
    /// other types").
    AddSubtypes(OrdSet<SubType>),
    /// Removes all subtypes from the object.
    LoseAllSubtypes,
    /// Adds every creature type from CR 205.3m to the object's subtypes.
    ///
    /// Used by Changeling CDA (CR 702.73a) and effects like Maskwood Nexus
    /// ("creatures you control are every creature type").
    /// No payload needed — the engine's `ALL_CREATURE_TYPES` constant supplies the list.
    AddAllCreatureTypes,

    // --- Layer 5: Color-changing ---
    /// Replaces all colors with the given set.
    SetColors(OrdSet<Color>),
    /// Adds colors without removing existing ones.
    AddColors(OrdSet<Color>),
    /// Makes the object colorless (removes all colors).
    BecomeColorless,

    // --- Layer 6: Ability-adding/removing ---
    /// Grants a single keyword ability (CR 702).
    AddKeyword(KeywordAbility),
    /// Grants multiple keyword abilities.
    AddKeywords(OrdSet<KeywordAbility>),
    /// Removes all abilities: keywords, mana abilities, activated abilities, triggered
    /// abilities (CR 613.1f). Used by Humility ("All creatures lose all abilities").
    RemoveAllAbilities,
    /// Removes a specific keyword ability.
    RemoveKeyword(KeywordAbility),

    // --- Layer 7a: P/T from characteristic-defining abilities ---
    /// Sets P/T via a CDA to fixed values.
    ///
    /// Used by creatures like Tarmogoyf whose P/T is determined by a CDA
    /// that evaluates to a specific value at calculation time (pre-computed by
    /// the caller before the effect is constructed).
    SetPtViaCda { power: i32, toughness: i32 },
    /// Sets P/T to the object's mana value (for both power and toughness).
    ///
    /// Used by Opalescence-style effects: "has base power and toughness each equal
    /// to its mana value." The mana value is taken from the object's printed mana cost.
    SetPtToManaValue,

    // --- Layer 7b: P/T-setting effects ---
    /// Sets base power and toughness to specific values (CR 613.4b).
    ///
    /// Used by Humility ("All creatures have base power and toughness 1/1").
    SetPowerToughness { power: i32, toughness: i32 },

    // --- Layer 7c: P/T-modifying effects ---
    /// Adds to power only (e.g., "+1/+0" effects).
    ModifyPower(i32),
    /// Adds to toughness only (e.g., "+0/+2" effects).
    ModifyToughness(i32),
    /// Adds equally to both power and toughness (e.g., "+2/+2" effects).
    ModifyBoth(i32),

    // --- Layer 7d: P/T-switching ---
    /// Switches power and toughness values (e.g., Inside Out, Behind the Scenes).
    SwitchPowerToughness,
}

/// A single continuous effect active in the game (CR 611).
///
/// Continuous effects apply to game objects through the layer system (CR 613).
/// Effects are gathered from:
/// - Static abilities of battlefield permanents (most common)
/// - Resolved spells and activated/triggered abilities with durations
///
/// The layer system processes all active effects in layer order and timestamp order
/// (with dependency overrides) to determine each object's current characteristics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuousEffect {
    /// Unique identifier for this effect instance.
    pub id: EffectId,
    /// The source object generating this effect, or `None` for spell-based effects.
    ///
    /// Used to determine activity for `WhileSourceOnBattlefield` duration: the effect
    /// is active only while this object is on the battlefield.
    pub source: Option<ObjectId>,
    /// Timestamp for ordering within a layer (CR 613.7).
    ///
    /// Effects with earlier timestamps apply before effects with later timestamps,
    /// unless a dependency relationship overrides this (CR 613.8).
    pub timestamp: u64,
    /// Which layer this effect applies in (CR 613.1).
    pub layer: EffectLayer,
    /// How long this effect lasts.
    pub duration: EffectDuration,
    /// Which objects this effect applies to.
    pub filter: EffectFilter,
    /// What this effect does when applied.
    pub modification: LayerModification,
    /// True if this effect comes from a characteristic-defining ability (CDA).
    ///
    /// CDAs apply before other effects within the same layer (CR 613.3).
    /// Note: CDAs and non-CDAs cannot depend on each other (CR 613.8a(c)).
    pub is_cda: bool,
}
