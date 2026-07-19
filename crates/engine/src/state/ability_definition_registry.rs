//! AbilityDefinition dispatch registry (SR-15).
//!
//! Sibling of [`super::keyword_registry`] (SR-5), applying the same machine gate
//! to the other genuine dispatch table in the card DSL: [`AbilityDefinition`].
//!
//! Unlike [`super::keyword_registry`]'s subject, `AbilityDefinition` is never read
//! through `keywords.contains(..)`. It is *dispatched on* — lowered by
//! `enrich_spec_from_def` into runtime structures and pulled apart by dozens of
//! `if let AbilityDefinition::X { .. }` / `filter_map(|a| match a { .. })` sites.
//! There is **no single exhaustive `match`** anywhere in the engine that names every
//! variant, so a newly added variant compiles everywhere and is silently inert: it
//! is lowered by nothing, read by nothing, and does nothing. That is exactly the
//! hazard SR-5 feared for keywords (and that it found on the *wrong* enum — see
//! `docs/sr-5-keyword-catchall-audit.md`).
//!
//! [`handling`] closes it. It is an exhaustive `match`, so a new variant cannot
//! compile until it is classified, and `tests/core/ability_definition_registry.rs`
//! then checks the classification against the actual source tree in both directions.
//!
//! Two classifications exist:
//!
//! * [`AbilityHandling::Handled`] — engine code branches on this exact variant. The
//!   declared `sites` must equal the set of scanned source files that mention
//!   `AbilityDefinition::<Variant>` outside a comment or string literal. Deleting the
//!   last read of a variant fails the test; adding a read in a new file fails the test.
//!
//!   Sites are **workspace-relative** paths, and the scanned tree spans `crates/engine/`
//!   (dispatch) and `crates/card-types/` (the DSL crate) — the same two roots the
//!   keyword registry scans. `crates/card-defs/` is deliberately not scanned: a def
//!   naming a variant is card data, not engine behavior.
//!
//! * [`AbilityHandling::Marker`] — the variant carries no dispatch of its own. Its
//!   payload is a *duplicate* of a [`KeywordAbility`] twin that the engine reads
//!   instead (e.g. the count for `AbilityDefinition::Vanishing { count }` is read
//!   from `KeywordAbility::Vanishing(n)`, never from this variant). The test asserts
//!   no engine file branches on such a variant. `carrier` names the twin and `cr`
//!   cites the rule, so "no dispatch needed" is an argued position, not an omission.
//!
//! The audit that produced this table is `docs/sr-15-dispatch-enum-catchall-audit.md`.

use crate::cards::card_definition::{
    AbilityDefinition, ContinuousEffectDef, Cost, CraftMaterials, Effect, EffectAmount, GiftType,
    LoyaltyCost, TriggerCondition,
};
use crate::state::continuous_effect::{
    EffectDuration, EffectFilter, EffectLayer, LayerModification,
};
use crate::state::game_object::ManaCost;
use crate::state::replacement_effect::{PlayerFilter, ReplacementModification, ReplacementTrigger};
use crate::state::stubs::{
    ETBSuppressFilter, FlashGrantFilter, GameRestriction, PlayFromTopFilter, TriggerDoublerFilter,
};
use crate::state::types::{
    AltCostKind, CardType, ChampionFilter, CounterType, CumulativeUpkeepCost, KeywordAbility,
    SubType,
};
use crate::state::zone::ZoneType;

/// Where an [`AbilityDefinition`] variant's behavior lives. See the module docs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbilityHandling {
    /// Engine code reads this exact variant. `sites` are workspace-relative paths
    /// checked for exact set equality against the scanned source tree.
    Handled { sites: &'static [&'static str] },
    /// The variant is an inert presence/data marker: its payload duplicates a
    /// [`KeywordAbility`] twin that the engine reads instead. `carrier` names the
    /// twin construct; `cr` cites the rule that defines the keyword.
    Marker {
        carrier: &'static str,
        cr: &'static str,
    },
}

/// Classify an [`AbilityDefinition`]. Exhaustive by construction: adding a variant
/// without adding an arm here is a compile error.
pub fn handling(ability: &AbilityDefinition) -> AbilityHandling {
    use AbilityDefinition as A;
    match ability {
        A::Activated { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/testing/replay_harness.rs"],
        },
        A::Triggered { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/abilities.rs",
                "crates/engine/src/rules/mana.rs",
                "crates/engine/src/rules/replacement.rs",
                "crates/engine/src/rules/resolution.rs",
                "crates/engine/src/rules/turn_actions.rs",
                "crates/engine/src/testing/replay_harness.rs",
            ],
        },
        A::Static { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/face.rs",
                "crates/engine/src/rules/replacement.rs",
                "crates/engine/src/rules/resolution.rs",
            ],
        },
        A::Keyword(..) => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/abilities.rs",
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/commander.rs",
                "crates/engine/src/rules/layers.rs",
                "crates/engine/src/rules/replacement.rs",
                "crates/engine/src/rules/resolution.rs",
                "crates/engine/src/testing/replay_harness.rs",
            ],
        },
        A::Spell { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/resolution.rs",
            ],
        },
        A::Replacement { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::OpeningHand => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/engine.rs"],
        },
        A::TriggerDoubling { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::SuppressCreatureETBTriggers { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::AltCastAbility { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/abilities.rs",
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/commander.rs",
                "crates/engine/src/rules/plot.rs",
            ],
        },
        A::Cycling { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/abilities.rs"],
        },
        A::Kicker { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Evoke { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Bestow { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::LoyaltyAbility { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/engine.rs",
                "crates/simulator/src/legal_actions.rs",
            ],
        },
        A::SagaChapter { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/replacement.rs",
                "crates/engine/src/rules/resolution.rs",
                "crates/engine/src/rules/sba.rs",
                "crates/engine/src/rules/turn_actions.rs",
            ],
        },
        A::ClassLevel { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/engine.rs",
                "crates/engine/src/rules/replacement.rs",
                "crates/engine/src/rules/resolution.rs",
            ],
        },
        A::Madness { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/effects/mod.rs",
                "crates/engine/src/rules/abilities.rs",
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/turn_actions.rs",
            ],
        },
        A::Miracle { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/miracle.rs",
            ],
        },
        A::EscapeWithCounter { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/resolution.rs"],
        },
        A::Foretell { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Buyback { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Suspend { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/suspend.rs"],
        },
        A::Overload { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Ninjutsu { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/abilities.rs"],
        },
        A::CommanderNinjutsu { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/abilities.rs"],
        },
        A::Aftermath { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/resolution.rs",
            ],
        },
        A::Impending { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Emerge { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Spectacle { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Surge { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Replicate { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Cleave { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Splice { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Entwine { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Escalate { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        // ── The four inert markers. Each carries a payload that duplicates a
        //    KeywordAbility twin; the engine reads the twin, never this variant.
        A::Vanishing { .. } => AbilityHandling::Marker {
            carrier: "KeywordAbility::Vanishing(n) — the count is read from the keyword",
            cr: "702.63a",
        },
        A::Fading { .. } => AbilityHandling::Marker {
            carrier: "KeywordAbility::Fading(n) — the count is read from the keyword",
            cr: "702.32a",
        },
        A::Echo { .. } => AbilityHandling::Marker {
            carrier: "KeywordAbility::Echo(cost) — the cost is read from the keyword",
            cr: "702.30a",
        },
        A::CumulativeUpkeep { .. } => AbilityHandling::Marker {
            carrier: "KeywordAbility::CumulativeUpkeep(cost) — the cost is read from the keyword",
            cr: "702.24a",
        },
        A::Recover { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/abilities.rs"],
        },
        A::Forecast { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/abilities.rs"],
        },
        A::Scavenge { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/abilities.rs"],
        },
        A::Outlast { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/testing/replay_harness.rs"],
        },
        A::Champion { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/abilities.rs",
                "crates/engine/src/testing/replay_harness.rs",
            ],
        },
        A::Soulbond { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/resolution.rs",
                "crates/engine/src/testing/replay_harness.rs",
            ],
        },
        A::Fuse { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/resolution.rs",
            ],
        },
        A::CollectEvidence { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Squad { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Bloodrush { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/abilities.rs",
                "crates/simulator/src/legal_actions.rs",
            ],
        },
        A::Offspring { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
        A::Gift { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/resolution.rs",
                "crates/engine/src/testing/replay_harness.rs",
            ],
        },
        A::Cipher => AbilityHandling::Handled {
            sites: &["crates/engine/src/testing/replay_harness.rs"],
        },
        A::Reconfigure { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/testing/replay_harness.rs"],
        },
        A::MutateCost { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/simulator/src/legal_actions.rs",
            ],
        },
        A::Disturb { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/resolution.rs",
            ],
        },
        A::Craft { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/engine.rs"],
        },
        A::Morph { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/engine.rs",
                "crates/engine/src/testing/replay_harness.rs",
                "crates/simulator/src/legal_actions.rs",
            ],
        },
        A::Megamorph { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/engine.rs",
                "crates/engine/src/testing/replay_harness.rs",
                "crates/simulator/src/legal_actions.rs",
            ],
        },
        A::Disguise { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/casting.rs",
                "crates/engine/src/rules/engine.rs",
                "crates/engine/src/testing/replay_harness.rs",
                "crates/simulator/src/legal_actions.rs",
            ],
        },
        A::StaticRestriction { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::CdaPowerToughness { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::CdaModifyPowerToughness { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::AdditionalLandPlays { .. } => AbilityHandling::Handled {
            sites: &[
                "crates/engine/src/rules/replacement.rs",
                "crates/engine/src/rules/resolution.rs",
            ],
        },
        A::StaticFlashGrant { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::StaticPlayFromTop { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::StaticPlayFromGraveyard { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/replacement.rs"],
        },
        A::CastSelfFromGraveyard { .. } => AbilityHandling::Handled {
            sites: &["crates/engine/src/rules/casting.rs"],
        },
    }
}

/// One representative value of every [`AbilityDefinition`] variant.
///
/// Payload values are arbitrary — nothing here depends on them; the tests read only
/// the variant name (`Debug` prefix). Rust cannot enumerate an enum's variants, so
/// this list is kept honest by `all_ability_definitions_covers_every_variant`, which
/// parses the enum declaration out of `cards/card_definition.rs` and set-compares.
pub fn all_ability_definitions() -> Vec<AbilityDefinition> {
    use AbilityDefinition as A;
    let mc = ManaCost::default;
    let eff = || Effect::Proliferate;
    let cont = || ContinuousEffectDef {
        layer: EffectLayer::Copy,
        modification: LayerModification::RemoveAllAbilities,
        filter: EffectFilter::AllCreatures,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: None,
    };
    vec![
        A::Activated {
            cost: Cost::Tap,
            effect: eff(),
            timing_restriction: None,
            targets: Vec::new(),
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        },
        A::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: eff(),
            intervening_if: None,
            targets: Vec::new(),
            modes: None,
            trigger_zone: None,
            once_per_turn: false,
        },
        A::Static {
            continuous_effect: cont(),
        },
        A::Keyword(KeywordAbility::Flying),
        A::Spell {
            effect: eff(),
            targets: Vec::new(),
            modes: None,
            cant_be_countered: false,
        },
        A::Replacement {
            trigger: ReplacementTrigger::WouldDraw {
                player_filter: PlayerFilter::Any,
            },
            modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
            is_self: false,
            unless_condition: None,
        },
        A::OpeningHand,
        A::TriggerDoubling {
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        },
        A::SuppressCreatureETBTriggers {
            filter: ETBSuppressFilter::CreaturesOnly,
        },
        A::AltCastAbility {
            kind: AltCostKind::Flashback,
            cost: mc(),
            details: None,
        },
        A::Cycling { cost: mc() },
        A::Kicker {
            cost: mc(),
            is_multikicker: false,
        },
        A::Evoke { cost: mc() },
        A::Bestow { cost: mc() },
        A::LoyaltyAbility {
            cost: LoyaltyCost::Zero,
            effect: eff(),
            targets: Vec::new(),
        },
        A::SagaChapter {
            chapter: 1,
            effect: eff(),
            targets: Vec::new(),
        },
        A::ClassLevel {
            level: 1,
            cost: mc(),
            abilities: Vec::new(),
        },
        A::Madness { cost: mc() },
        A::Miracle { cost: mc() },
        A::EscapeWithCounter {
            counter_type: CounterType::PlusOnePlusOne,
            count: 1,
        },
        A::Foretell { cost: mc() },
        A::Buyback { cost: mc() },
        A::Suspend {
            cost: mc(),
            time_counters: 1,
        },
        A::Overload { cost: mc() },
        A::Ninjutsu { cost: mc() },
        A::CommanderNinjutsu { cost: mc() },
        A::Aftermath {
            name: String::new(),
            cost: mc(),
            card_type: CardType::Instant,
            effect: eff(),
            targets: Vec::new(),
        },
        A::Impending {
            cost: mc(),
            count: 1,
        },
        A::Emerge { cost: mc() },
        A::Spectacle { cost: mc() },
        A::Surge { cost: mc() },
        A::Replicate { cost: mc() },
        A::Cleave { cost: mc() },
        A::Splice {
            cost: mc(),
            onto_subtype: SubType(String::new()),
            effect: Box::new(eff()),
        },
        A::Entwine { cost: mc() },
        A::Escalate { cost: mc() },
        A::Vanishing { count: 1 },
        A::Fading { count: 1 },
        A::Echo { cost: mc() },
        A::CumulativeUpkeep {
            cost: CumulativeUpkeepCost::Life(1),
        },
        A::Recover { cost: mc() },
        A::Forecast {
            cost: mc(),
            effect: eff(),
        },
        A::Scavenge { cost: mc() },
        A::Outlast { cost: mc() },
        A::Champion {
            filter: ChampionFilter::AnyCreature,
        },
        A::Soulbond { grants: Vec::new() },
        A::Fuse {
            name: String::new(),
            cost: mc(),
            card_type: CardType::Instant,
            effect: eff(),
            targets: Vec::new(),
        },
        A::CollectEvidence {
            threshold: 1,
            mandatory: false,
        },
        A::Squad { cost: mc() },
        A::Bloodrush {
            cost: mc(),
            power_boost: 1,
            toughness_boost: 1,
            grants_keyword: None,
        },
        A::Offspring { cost: mc() },
        A::Gift {
            gift_type: GiftType::Food,
        },
        A::Cipher,
        A::Reconfigure { cost: mc() },
        A::MutateCost { cost: mc() },
        A::Disturb { cost: mc() },
        A::Craft {
            cost: mc(),
            materials: CraftMaterials::Artifacts(1),
        },
        A::Morph { cost: mc() },
        A::Megamorph { cost: mc() },
        A::Disguise { cost: mc() },
        A::StaticRestriction {
            restriction: GameRestriction::MaxSpellsPerTurn { max: 1 },
        },
        A::CdaPowerToughness {
            power: EffectAmount::Fixed(1),
            toughness: EffectAmount::Fixed(1),
        },
        A::CdaModifyPowerToughness {
            power: Some(EffectAmount::Fixed(1)),
            toughness: Some(EffectAmount::Fixed(1)),
        },
        A::AdditionalLandPlays { count: 1 },
        A::StaticFlashGrant {
            filter: FlashGrantFilter::AllSpells,
        },
        A::StaticPlayFromTop {
            filter: PlayFromTopFilter::All,
            look_at_top: false,
            reveal_top: false,
            pay_life_instead: false,
            condition: None,
            on_cast_effect: None,
        },
        A::StaticPlayFromGraveyard {
            filter: PlayFromTopFilter::All,
            condition: None,
        },
        A::CastSelfFromGraveyard {
            condition: None,
            alt_mana_cost: None,
            additional_costs: Vec::new(),
            required_alt_cost: None,
        },
    ]
}
