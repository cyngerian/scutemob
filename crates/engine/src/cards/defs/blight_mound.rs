// Blight Mound — {2}{B}, Enchantment
// Attacking Pests you control get +1/+0 and have menace.
// Whenever a nontoken creature you control dies, create a 1/1 black and green Pest creature
// token with "When this token dies, you gain 1 life."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blight-mound"),
        name: "Blight Mound".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking Pests you control get +1/+0 and have menace.\nWhenever a nontoken creature you control dies, create a 1/1 black and green Pest creature token with \"When this token dies, you gain 1 life.\"".to_string(),
        abilities: vec![
            // CR 613.4c (Layer 7c): "Attacking Pests you control get +1/+0."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(
                        SubType("Pest".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.1f (Layer 6): "Attacking Pests you control have menace."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Menace),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(
                        SubType("Pest".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "Whenever a nontoken creature you control dies, create a 1/1 Pest token."
            // Blocked: nontoken filter on WheneverCreatureDies not in DSL.
            // Also blocked: token's nested triggered ability ("When this token dies, you gain 1 life")
            // not expressible as a TokenSpec ability.
        ],
        ..Default::default()
    }
}
