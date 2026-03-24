// Rising of the Day — {2}{R}, Enchantment
// Creatures you control have haste.
// Legendary creatures you control get +1/+0.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rising-of-the-day"),
        name: "Rising of the Day".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have haste.\nLegendary creatures you control get +1/+0.".to_string(),
        abilities: vec![
            // CR 613.1f / Layer 6: "Creatures you control have haste."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: DSL gap — "Legendary creatures you control get +1/+0."
            // No EffectFilter for creatures you control with a supertype (Legendary).
        ],
        ..Default::default()
    }
}
