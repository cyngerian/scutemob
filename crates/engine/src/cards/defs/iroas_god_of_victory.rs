// Iroas, God of Victory — {2}{R}{W}, Legendary Enchantment Creature — God 7/4
// Indestructible
// As long as your devotion to red and white is less than seven, Iroas isn't a creature.
// Creatures you control have menace.
// Prevent all damage that would be dealt to attacking creatures you control.
//
// Indestructible and menace grant are implemented.
//
// TODO: DSL gap — devotion-based "isn't a creature" requires a conditional type-removal
// continuous effect (Layer 4) parameterized on devotion count — no such effect in DSL.
//
// TODO: DSL gap — "Prevent all damage that would be dealt to attacking creatures you control"
// is a blanket prevention replacement effect scoped to attacking creatures — no such
// replacement pattern exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("iroas-god-of-victory"),
        name: "Iroas, God of Victory".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Enchantment, CardType::Creature],
            &["God"],
        ),
        oracle_text: "Indestructible\nAs long as your devotion to red and white is less than seven, Iroas isn't a creature.\nCreatures you control have menace.\nPrevent all damage that would be dealt to attacking creatures you control.".to_string(),
        power: Some(7),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // "Creatures you control have menace." — CR 604.2 / CR 613.1f: Layer 6 grant.
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
