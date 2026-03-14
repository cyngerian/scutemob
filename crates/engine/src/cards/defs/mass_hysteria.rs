// Mass Hysteria — {R}, Enchantment
// All creatures have haste.
//
// CR 604.2: Static ability functions while on the battlefield.
// CR 613.1f: Layer 6 ability-granting effect applies to all creatures.
// Note: "All creatures" = AllCreatures filter (both players), like Concordant Crossroads.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mass-hysteria"),
        name: "Mass Hysteria".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "All creatures have haste.".to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::AllCreatures,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
        ],
        ..Default::default()
    }
}
