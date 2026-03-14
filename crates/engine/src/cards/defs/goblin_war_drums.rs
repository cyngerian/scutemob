// Goblin War Drums — {2}{R}, Enchantment
// Creatures you control have menace.
//
// CR 604.2: Static ability functions while on the battlefield.
// CR 613.1f: Layer 6 ability-granting effect scoped to source controller.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-war-drums"),
        name: "Goblin War Drums".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have menace. (They can't be blocked except by two or more creatures.)".to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Menace),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
        ],
        ..Default::default()
    }
}
