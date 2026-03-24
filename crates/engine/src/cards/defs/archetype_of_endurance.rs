// Archetype of Endurance — {6}{G}{G}, Enchantment Creature — Boar 6/5
// Creatures you control have hexproof.
// Creatures your opponents control lose hexproof and can't have or gain hexproof.
//
// CR 604.2 / CR 613.1f: Static abilities — Layer 6.
// The "can't have or gain hexproof" prevention is not expressible in the current DSL
// (keyword prevention/lock is a separate engine feature). The RemoveKeyword half IS
// implemented; the prevention sub-clause is left as a TODO.
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
            // CR 613.1f (Layer 6): "Creatures you control have hexproof."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Hexproof),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.1f (Layer 6): "Creatures your opponents control lose hexproof."
            // NOTE: "can't have or gain hexproof" prevention not yet expressible — TODO.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveKeyword(KeywordAbility::Hexproof),
                    filter: EffectFilter::CreaturesOpponentsControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
