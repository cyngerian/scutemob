// Archetype of Imagination — {4}{U}{U}, Enchantment Creature — Human Wizard 3/2
// Creatures you control have flying.
// Creatures your opponents control lose flying and can't have or gain flying.
//
// CR 604.2 / CR 613.1f: Static ability — Layer 6 keyword grant to creatures you control.
// TODO: DSL gap — "Creatures your opponents control lose flying and can't have or gain flying"
// requires EffectFilter::CreaturesOpponentsControl and a RemoveKeyword + prevention effect,
// neither of which exist in the DSL. Removal half omitted until those filters are added.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archetype-of-imagination"),
        name: "Archetype of Imagination".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Human", "Wizard"]),
        oracle_text: "Creatures you control have flying.\nCreatures your opponents control lose flying and can't have or gain flying.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // "Creatures you control have flying." — grant half implemented.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
