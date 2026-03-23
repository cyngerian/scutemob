// Mirari's Wake — {3}{G}{W}, Enchantment
// Creatures you control get +1/+1.
// Whenever you tap a land for mana, add one mana of any type that land produced.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("miraris-wake"),
        name: "Mirari's Wake".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control get +1/+1.\nWhenever you tap a land for mana, add one mana of any type that land produced.".to_string(),
        abilities: vec![
            // CR 613.1c / Layer 7c: "Creatures you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: DSL gap — "Whenever you tap a land for mana, add one mana of any type
            // that land produced." Mana doubling triggered ability not in DSL.
        ],
        ..Default::default()
    }
}
