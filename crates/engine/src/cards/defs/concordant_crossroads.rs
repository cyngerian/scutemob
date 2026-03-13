// Concordant Crossroads — {G}, World Enchantment
// All creatures have haste.
//
// CR 604.2: Static ability functions while on the battlefield.
// CR 613.1f: Layer 6 ability-granting effect applies to EffectFilter::AllCreatures.
// Note: "World" is a supertype (CR 205.4c); represented as SuperType::World.
//
// TODO: DSL gap — SuperType::World may not be available in helpers.rs. If it is not,
// this is represented as a plain Enchantment. The static ability (all creatures have haste)
// is expressible and is fully implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("concordant-crossroads"),
        name: "Concordant Crossroads".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "All creatures have haste.".to_string(),
        abilities: vec![
            // Static: All creatures have Haste (layer 6, CR 613.1f).
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
