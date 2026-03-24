// Archetype of Imagination — {4}{U}{U}, Enchantment Creature — Human Wizard 3/2
// Creatures you control have flying.
// Creatures your opponents control lose flying and can't have or gain flying.
//
// CR 604.2 / CR 613.1f: Static abilities — Layer 6.
// The "can't have or gain flying" prevention is not expressible in the current DSL
// (keyword prevention/lock is a separate engine feature). The RemoveKeyword half IS
// implemented; the prevention sub-clause is left as a TODO.
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
            // CR 613.1f (Layer 6): "Creatures you control have flying."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.1f (Layer 6): "Creatures your opponents control lose flying."
            // NOTE: "can't have or gain flying" prevention not yet expressible — TODO.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveKeyword(KeywordAbility::Flying),
                    filter: EffectFilter::CreaturesOpponentsControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
