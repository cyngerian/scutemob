//! Keyword dispatch registry (SR-5).
//!
//! Every [`KeywordAbility`] variant must declare, here, *where its behavior lives*.
//! [`handling`] is an exhaustive `match`, so a newly added variant cannot compile
//! until it is classified — this is the machine gate that replaces the process
//! guarantee "remember to wire up your new keyword."
//!
//! Two classifications exist, and `tests/keyword_registry.rs` checks both against
//! the actual engine source, in both directions:
//!
//! * [`KeywordHandling::Handled`] — engine code branches on this exact variant. The
//!   declared `sites` must equal the set of engine source files that mention
//!   `KeywordAbility::<Variant>` outside a comment. Deleting the last read of a
//!   keyword fails the test; adding a read in a new file fails the test.
//!
//! * [`KeywordHandling::Marker`] — the variant is a *presence marker* carrying no
//!   dispatch of its own; the rules text it names is implemented by the `carrier`
//!   construct instead (an `AbilityDefinition`, `Effect`, or `Command` variant).
//!   The test asserts **no** engine file branches on such a variant. Each entry
//!   cites the Comprehensive Rule that defines the keyword as shorthand for that
//!   construct, so "no dispatch needed" is an argued position rather than an
//!   omission.
//!
//! The audit that produced this table is `docs/sr-5-keyword-catchall-audit.md`.

use super::game_object::ManaCost;
use super::types::{
    AffinityTarget, BlockingExceptionFilter, Color, CumulativeUpkeepCost, EnchantTarget,
    KeywordAbility, LandwalkType, ProtectionQuality,
};

/// Where a keyword's behavior lives. See the module docs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeywordHandling {
    /// Engine code reads this exact variant. `sites` are crate-relative paths
    /// (relative to `crates/engine/`) and are checked for exact set equality
    /// against the source tree.
    Handled { sites: &'static [&'static str] },
    /// The variant exists only so `keywords.contains(..)` can answer "does this
    /// object have <keyword>?". No engine code branches on it; `carrier` names
    /// the construct that actually implements the rules text, and `cr` cites the
    /// rule that licenses the substitution.
    Marker {
        carrier: &'static str,
        cr: &'static str,
    },
}

/// Classify a keyword. Exhaustive by construction: adding a `KeywordAbility`
/// variant without adding an arm here is a compile error.
pub fn handling(keyword: &KeywordAbility) -> KeywordHandling {
    use KeywordAbility as K;
    match keyword {
        K::Deathtouch => KeywordHandling::Handled {
            sites: &[
                "src/effects/mod.rs",
                "src/rules/combat.rs",
                "src/state/dungeon.rs",
            ],
        },
        K::Defender => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::DoubleStrike => KeywordHandling::Handled {
            sites: &[
                "src/rules/combat.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Enchant(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
                "src/rules/sba.rs",
            ],
        },
        K::Equip => KeywordHandling::Marker {
            carrier: "Effect::AttachEquipment / Effect::DetachEquipment, activated through AbilityDefinition::Activated",
            cr: "702.6a",
        },
        K::FirstStrike => KeywordHandling::Handled {
            sites: &[
                "src/rules/combat.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Flash => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/suspend.rs",
            ],
        },
        K::Flying => KeywordHandling::Handled {
            sites: &[
                "src/rules/combat.rs",
                "src/state/builder.rs",
            ],
        },
        K::Haste => KeywordHandling::Handled {
            sites: &[
                "src/effects/mod.rs",
                "src/rules/abilities.rs",
                "src/rules/combat.rs",
                "src/rules/mana.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Hexproof => KeywordHandling::Handled {
            sites: &[
                "src/rules/mod.rs",
                "src/rules/protection.rs",
            ],
        },
        K::Indestructible => KeywordHandling::Handled {
            sites: &[
                "src/effects/mod.rs",
                "src/rules/sba.rs",
            ],
        },
        K::Intimidate => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Landwalk(..) => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Lifelink => KeywordHandling::Handled {
            sites: &[
                "src/effects/mod.rs",
                "src/rules/combat.rs",
            ],
        },
        K::Menace => KeywordHandling::Handled {
            sites: &[
                "src/rules/combat.rs",
                "src/rules/layers.rs",
                "src/state/dungeon.rs",
            ],
        },
        K::ProtectionFrom(..) => KeywordHandling::Handled { sites: &["src/rules/protection.rs"] },
        K::Prowess => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Reach => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Shroud => KeywordHandling::Handled {
            sites: &[
                "src/rules/mod.rs",
                "src/rules/protection.rs",
            ],
        },
        K::Trample => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Vigilance => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Ward(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/layers.rs",
                "src/state/builder.rs",
            ],
        },
        K::Partner => KeywordHandling::Handled { sites: &["src/rules/commander.rs"] },
        K::PartnerWith(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/commander.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::FriendsForever => KeywordHandling::Handled { sites: &["src/rules/commander.rs"] },
        K::ChooseABackground => KeywordHandling::Handled { sites: &["src/rules/commander.rs"] },
        K::DoctorsCompanion => KeywordHandling::Handled { sites: &["src/rules/commander.rs"] },
        K::NoMaxHandSize => KeywordHandling::Handled { sites: &["src/rules/turn_actions.rs"] },
        K::CantBeBlocked => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Storm => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Cascade => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Flashback => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Cycling => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::Dredge(..) => KeywordHandling::Handled { sites: &["src/rules/replacement.rs"] },
        K::Convoke => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Delve => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Kicker => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Kicker { cost, is_multikicker }",
            cr: "702.33a",
        },
        K::SplitSecond => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Exalted => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Annihilator(..) => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Persist => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Undying => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Changeling => KeywordHandling::Handled { sites: &["src/rules/layers.rs"] },
        K::Evoke => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Crew(..) => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::BattleCry => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Afterlife(..) => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Extort => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Improvise => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Affinity(..) => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Undaunted => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Dethrone => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Bestow => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Bestow { cost } + AltCostKind::Bestow",
            cr: "702.103a",
        },
        K::Fear => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::LivingWeapon => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Madness => KeywordHandling::Handled {
            sites: &[
                "src/effects/mod.rs",
                "src/rules/abilities.rs",
                "src/rules/casting.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Miracle => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/miracle.rs",
            ],
        },
        K::Escape => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Foretell => KeywordHandling::Handled { sites: &["src/rules/foretell.rs"] },
        K::Unearth => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Riot => KeywordHandling::Handled { sites: &["src/rules/resolution.rs"] },
        K::Exploit => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Wither => KeywordHandling::Handled {
            sites: &[
                "src/effects/mod.rs",
                "src/rules/combat.rs",
            ],
        },
        K::Modular(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
                "src/state/builder.rs",
            ],
        },
        K::Evolve => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Buyback => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Buyback { cost } + AltCostKind::Buyback",
            cr: "702.27a",
        },
        K::Ascend => KeywordHandling::Handled {
            sites: &[
                "src/rules/resolution.rs",
                "src/rules/sba.rs",
            ],
        },
        K::Infect => KeywordHandling::Handled {
            sites: &[
                "src/effects/mod.rs",
                "src/rules/combat.rs",
            ],
        },
        K::Myriad => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
                "src/state/builder.rs",
            ],
        },
        K::Suspend => KeywordHandling::Handled { sites: &["src/rules/suspend.rs"] },
        K::Hideaway(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Adapt(..) => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Activated with Effect::Conditional(SourceHasNoCountersOfType) -> Effect::AddCounter",
            cr: "701.46a",
        },
        K::Shadow => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Overload => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Overload { cost } + AltCostKind::Overload",
            cr: "702.96a",
        },
        K::Horsemanship => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Skulk => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Devoid => KeywordHandling::Handled { sites: &["src/rules/layers.rs"] },
        K::Decayed => KeywordHandling::Handled {
            sites: &[
                "src/cards/card_definition.rs",
                "src/rules/combat.rs",
            ],
        },
        K::Ingest => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Flanking => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Bushido(..) => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Rampage(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
                "src/state/builder.rs",
            ],
        },
        K::Provoke => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
                "src/state/builder.rs",
            ],
        },
        K::Afflict(..) => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Renown(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Training => KeywordHandling::Handled { sites: &["src/state/builder.rs"] },
        K::Melee => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
                "src/state/builder.rs",
            ],
        },
        K::Poisonous(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Toxic(..) => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::Enlist => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/combat.rs",
                "src/rules/resolution.rs",
                "src/state/builder.rs",
            ],
        },
        K::Ninjutsu => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::CommanderNinjutsu => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::Retrace => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::JumpStart => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Aftermath => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Embalm => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::Eternalize => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::Encore => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Dash => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Blitz => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Plot => KeywordHandling::Handled { sites: &["src/rules/plot.rs"] },
        K::Prototype => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Prototype + AltCostKind::Prototype",
            cr: "702.160a",
        },
        K::Impending => KeywordHandling::Handled {
            sites: &[
                "src/rules/resolution.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Bargain => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Emerge => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Emerge { cost } + AltCostKind::Emerge",
            cr: "702.119a",
        },
        K::Spectacle => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Surge => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Casualty(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Assist => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Replicate => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Gravestorm => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Cleave => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Cleave { cost } + AltCostKind::Cleave",
            cr: "702.148a",
        },
        K::Splice => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Entwine => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Escalate => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Recover => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Vanishing(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/lands.rs",
                "src/rules/resolution.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Fading(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/lands.rs",
                "src/rules/resolution.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Echo(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/lands.rs",
                "src/rules/resolution.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::CumulativeUpkeep(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/resolution.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Forecast => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::Phasing => KeywordHandling::Handled { sites: &["src/rules/turn_actions.rs"] },
        K::Graft(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/lands.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Scavenge => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::Outlast => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Outlast { cost }",
            cr: "702.107a",
        },
        K::Amplify(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/lands.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Bloodthirst(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/lands.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Devour(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Backup(..) => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Champion => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
                "src/testing/replay_harness.rs",
            ],
        },
        K::UmbraArmor => KeywordHandling::Handled { sites: &["src/rules/replacement.rs"] },
        K::LivingMetal => KeywordHandling::Handled { sites: &["src/rules/layers.rs"] },
        K::Soulbond => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
                "src/testing/replay_harness.rs",
            ],
        },
        K::Fortify => KeywordHandling::Marker {
            carrier: "Effect::AttachFortification",
            cr: "702.67a",
        },
        K::Tribute(..) => KeywordHandling::Handled { sites: &["src/rules/resolution.rs"] },
        K::Fabricate(..) => KeywordHandling::Handled { sites: &["src/rules/replacement.rs"] },
        K::Fuse => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Spree => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Ravenous => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Discover => KeywordHandling::Marker {
            carrier: "Effect::Discover { player, n }",
            cr: "701.57a",
        },
        K::Squad => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Offspring => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Gift => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/casting.rs",
                "src/rules/resolution.rs",
                "src/testing/replay_harness.rs",
            ],
        },
        K::Saddle(..) => KeywordHandling::Handled { sites: &["src/rules/abilities.rs"] },
        K::Cipher => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
                "src/testing/replay_harness.rs",
            ],
        },
        K::Haunt => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/resolution.rs",
            ],
        },
        K::Reconfigure => KeywordHandling::Handled {
            sites: &[
                "src/effects/mod.rs",
                "src/testing/replay_harness.rs",
            ],
        },
        K::Mutate => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Transform => KeywordHandling::Marker {
            carrier: "Command::Transform + Effect::TransformPermanent",
            cr: "701.27a",
        },
        K::Daybound => KeywordHandling::Handled {
            sites: &[
                "src/rules/engine.rs",
                "src/rules/resolution.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Nightbound => KeywordHandling::Handled {
            sites: &[
                "src/rules/engine.rs",
                "src/rules/resolution.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Disturb => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Disturb { cost } + AltCostKind::Disturb",
            cr: "702.146a",
        },
        K::Craft => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Craft + Command::ActivateCraft",
            cr: "702.167a",
        },
        K::Morph => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Megamorph => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Disguise => KeywordHandling::Handled { sites: &["src/rules/casting.rs"] },
        K::Manifest => KeywordHandling::Marker {
            carrier: "Effect::Manifest { player }",
            cr: "701.40a",
        },
        K::Cloak => KeywordHandling::Marker {
            carrier: "Effect::Cloak { player }",
            cr: "701.58a",
        },
        K::MustAttackEachCombat => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::HexproofPlayer => KeywordHandling::Handled {
            sites: &[
                "src/rules/abilities.rs",
                "src/rules/casting.rs",
            ],
        },
        K::CantBlock => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::CantBeBlockedExceptBy(..) => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
        K::DoesNotUntap => KeywordHandling::Handled { sites: &["src/rules/turn_actions.rs"] },
        K::Warp => KeywordHandling::Handled {
            sites: &[
                "src/rules/resolution.rs",
                "src/rules/turn_actions.rs",
            ],
        },
        K::Transmute => KeywordHandling::Marker {
            carrier: "AbilityDefinition::Activated (Cost::DiscardSelf + Effect::SearchLibrary)",
            cr: "702.53a",
        },
        K::Exert => KeywordHandling::Handled { sites: &["src/rules/combat.rs"] },
    }
}

/// One representative value of every `KeywordAbility` variant.
///
/// Payload values are arbitrary — nothing here depends on them. Rust cannot
/// enumerate an enum's variants, so this list is kept honest by
/// `keyword_registry::all_keywords_covers_every_variant`, which parses the enum
/// declaration out of `state/types.rs` and set-compares.
pub fn all_keywords() -> Vec<KeywordAbility> {
    use KeywordAbility as K;
    vec![
        K::Deathtouch,
        K::Defender,
        K::DoubleStrike,
        K::Enchant(EnchantTarget::Creature),
        K::Equip,
        K::FirstStrike,
        K::Flash,
        K::Flying,
        K::Haste,
        K::Hexproof,
        K::Indestructible,
        K::Intimidate,
        K::Landwalk(LandwalkType::Nonbasic),
        K::Lifelink,
        K::Menace,
        K::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
        K::Prowess,
        K::Reach,
        K::Shroud,
        K::Trample,
        K::Vigilance,
        K::Ward(1),
        K::Partner,
        K::PartnerWith(String::new()),
        K::FriendsForever,
        K::ChooseABackground,
        K::DoctorsCompanion,
        K::NoMaxHandSize,
        K::CantBeBlocked,
        K::Storm,
        K::Cascade,
        K::Flashback,
        K::Cycling,
        K::Dredge(1),
        K::Convoke,
        K::Delve,
        K::Kicker,
        K::SplitSecond,
        K::Exalted,
        K::Annihilator(1),
        K::Persist,
        K::Undying,
        K::Changeling,
        K::Evoke,
        K::Crew(1),
        K::BattleCry,
        K::Afterlife(1),
        K::Extort,
        K::Improvise,
        K::Affinity(AffinityTarget::Artifacts),
        K::Undaunted,
        K::Dethrone,
        K::Bestow,
        K::Fear,
        K::LivingWeapon,
        K::Madness,
        K::Miracle,
        K::Escape,
        K::Foretell,
        K::Unearth,
        K::Riot,
        K::Exploit,
        K::Wither,
        K::Modular(1),
        K::Evolve,
        K::Buyback,
        K::Ascend,
        K::Infect,
        K::Myriad,
        K::Suspend,
        K::Hideaway(1),
        K::Adapt(1),
        K::Shadow,
        K::Overload,
        K::Horsemanship,
        K::Skulk,
        K::Devoid,
        K::Decayed,
        K::Ingest,
        K::Flanking,
        K::Bushido(1),
        K::Rampage(1),
        K::Provoke,
        K::Afflict(1),
        K::Renown(1),
        K::Training,
        K::Melee,
        K::Poisonous(1),
        K::Toxic(1),
        K::Enlist,
        K::Ninjutsu,
        K::CommanderNinjutsu,
        K::Retrace,
        K::JumpStart,
        K::Aftermath,
        K::Embalm,
        K::Eternalize,
        K::Encore,
        K::Dash,
        K::Blitz,
        K::Plot,
        K::Prototype,
        K::Impending,
        K::Bargain,
        K::Emerge,
        K::Spectacle,
        K::Surge,
        K::Casualty(1),
        K::Assist,
        K::Replicate,
        K::Gravestorm,
        K::Cleave,
        K::Splice,
        K::Entwine,
        K::Escalate,
        K::Recover,
        K::Vanishing(1),
        K::Fading(1),
        K::Echo(ManaCost::default()),
        K::CumulativeUpkeep(CumulativeUpkeepCost::Life(1)),
        K::Forecast,
        K::Phasing,
        K::Graft(1),
        K::Scavenge,
        K::Outlast,
        K::Amplify(1),
        K::Bloodthirst(1),
        K::Devour(1),
        K::Backup(1),
        K::Champion,
        K::UmbraArmor,
        K::LivingMetal,
        K::Soulbond,
        K::Fortify,
        K::Tribute(1),
        K::Fabricate(1),
        K::Fuse,
        K::Spree,
        K::Ravenous,
        K::Discover,
        K::Squad,
        K::Offspring,
        K::Gift,
        K::Saddle(1),
        K::Cipher,
        K::Haunt,
        K::Reconfigure,
        K::Mutate,
        K::Transform,
        K::Daybound,
        K::Nightbound,
        K::Disturb,
        K::Craft,
        K::Morph,
        K::Megamorph,
        K::Disguise,
        K::Manifest,
        K::Cloak,
        K::MustAttackEachCombat,
        K::HexproofPlayer,
        K::CantBlock,
        K::CantBeBlockedExceptBy(BlockingExceptionFilter::HasAnyKeyword(Vec::new())),
        K::DoesNotUntap,
        K::Warp,
        K::Transmute,
        K::Exert,
    ]
}
