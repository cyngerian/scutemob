// Brave the Sands — {1}{W}, Enchantment
// Creatures you control have vigilance.
// Each creature you control can block an additional creature each combat.
//
// CR 604.2: Static ability functions while on the battlefield.
// CR 613.1f: Layer 6 ability-granting effect scoped to source controller.
// TODO: DSL gap — "Each creature you control can block an additional creature
//   each combat." (additional blocker assignment not supported in card DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brave-the-sands"),
        name: "Brave the Sands".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have vigilance.\nEach creature you control can block an additional creature each combat.".to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Vigilance),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: "Each creature you control can block an additional creature each combat"
            // requires combat rules infrastructure for additional blockers.
        ],
        ..Default::default()
    }
}
