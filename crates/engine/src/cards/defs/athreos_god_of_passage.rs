// Athreos, God of Passage — {1}{W}{B}, Legendary Enchantment Creature — God 5/4
// Indestructible
// As long as your devotion to white and black is less than seven, Athreos isn't a creature.
// Whenever another creature you own dies, return it to your hand unless target opponent
// pays 3 life.
//
// CR 700.5 / CR 604.2 / CR 613.1d (Layer 4): "As long as your devotion to white and black
// is less than seven, Athreos isn't a creature." Multi-color devotion counts mana symbols
// of EITHER color (CR 700.5: each matching symbol counts once).
//
// TODO: "Whenever another creature you own dies, return it to your hand unless target opponent
// pays 3 life." Requires a death trigger with opponent-choice (pay 3 life or allow return).
// DSL gap: no mechanic for opponent paying life as an alternative to an effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("athreos-god-of-passage"),
        name: "Athreos, God of Passage".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Enchantment, CardType::Creature],
            &["God"],
        ),
        oracle_text: "Indestructible\nAs long as your devotion to white and black is less than seven, Athreos isn't a creature.\nWhenever another creature you own dies, return it to your hand unless target opponent pays 3 life.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // CR 700.5 / CR 613.1d (Layer 4): "As long as your devotion to white and black
            // is less than seven, Athreos isn't a creature." Multi-color devotion: counts
            // any mana symbol that is white OR black (including {W/B} hybrid symbols).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::RemoveCardTypes(
                        [CardType::Creature].into_iter().collect(),
                    ),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::DevotionToColorsLessThan {
                        colors: vec![Color::White, Color::Black],
                        threshold: 7,
                    }),
                },
            },
            // TODO: "Whenever another creature you own dies, return it to your hand unless
            // target opponent pays 3 life." DSL gap: no opponent-pays-life alternative.
        ],
        ..Default::default()
    }
}
