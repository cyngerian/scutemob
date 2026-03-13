// Indomitable Archangel — {2}{W}{W}, Creature — Angel 4/4
// Flying
// Metalcraft — Artifacts you control have shroud as long as you control three or more artifacts.
//
// Flying is implemented.
// TODO: DSL gap — Metalcraft conditional static (requires controlling 3+ artifacts) grants shroud
//   to all your artifacts. Needs Condition::ControlsNOrMoreArtifacts(3) + EffectFilter::CreaturesYouControl
//   filtered to artifacts — not in DSL.
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
            // TODO: Metalcraft — conditional shroud grant to all your artifacts (count threshold static)
        ],
        ..Default::default()
    }
}
