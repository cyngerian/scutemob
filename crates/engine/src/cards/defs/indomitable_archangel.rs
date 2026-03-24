// Indomitable Archangel — {2}{W}{W}, Creature — Angel 4/4
// Flying
// Metalcraft — Artifacts you control have shroud as long as you control three or more artifacts.
//
// Flying is implemented.
// TODO: Metalcraft — condition is now expressible: Condition::YouControlNOrMoreWithFilter
//   { count: 3, filter: artifact_filter }. BLOCKED on EffectFilter for "artifacts you control"
//   (PB-25 scope: no EffectFilter::ArtifactsYouControl variant). The shroud grant to all your
//   artifacts cannot be expressed without a filter that scopes to controlled artifacts.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("indomitable-archangel"),
        name: "Indomitable Archangel".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 2, ..Default::default() }),
        types: creature_types(&["Angel"]),
        oracle_text: "Flying\nMetalcraft \u{2014} Artifacts you control have shroud as long as you control three or more artifacts. (An artifact with shroud can't be the target of spells or abilities.)".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: Metalcraft — BLOCKED on EffectFilter::ArtifactsYouControl (PB-25 scope).
            // Condition is expressible (YouControlNOrMoreWithFilter { count: 3, artifact_filter })
            // but the target filter "artifacts you control" is not yet a supported EffectFilter.
        ],
        ..Default::default()
    }
}
