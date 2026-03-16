// Archetype of Endurance — {6}{G}{G}, Enchantment Creature — Boar 6/5
// Creatures you control have hexproof.
// Creatures your opponents control lose hexproof and can't have or gain hexproof.
//
// CR 604.2 / CR 613.1f: Static ability — Layer 6 keyword grant to creatures you control.
// TODO: DSL gap — "Creatures your opponents control lose hexproof and can't have or gain hexproof"
// requires EffectFilter::CreaturesOpponentsControl and a RemoveKeyword + prevention effect,
// neither of which exist in the DSL. Removal half omitted until those filters are added.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archetype-of-endurance"),
        name: "Archetype of Endurance".to_string(),
        mana_cost: Some(ManaCost { generic: 6, green: 2, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Enchantment, CardType::Creature],
            &["Boar"],
        ),
        oracle_text: "Creatures you control have hexproof.\nCreatures your opponents control lose hexproof and can't have or gain hexproof.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            // "Creatures you control have hexproof." — grant half implemented.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Hexproof),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
        ],
        ..Default::default()
    }
}
