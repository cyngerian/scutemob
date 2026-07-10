// Indomitable Archangel — {2}{W}{W}, Creature — Angel 4/4
// Flying
// Metalcraft — Artifacts you control have shroud as long as you control three or more artifacts.
//
// CR 702.45a (Metalcraft): The condition checks that you control 3+ artifacts.
// CR 613.1f (Layer 6): Static ability — grants shroud to all artifacts you control.
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
            // CR 613.1f (Layer 6): "Artifacts you control have shroud as long as you
            // control three or more artifacts." (Metalcraft condition)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Shroud),
                    filter: EffectFilter::ArtifactsYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::YouControlNOrMoreWithFilter {
                        count: 3,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            ..Default::default()
                        },
                    }),
                },
            },
        ],
        ..Default::default()
    }
}
