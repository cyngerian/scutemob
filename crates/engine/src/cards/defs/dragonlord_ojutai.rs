// Dragonlord Ojutai — {3}{W}{U}, Legendary Creature — Elder Dragon 5/4
// Flying
// Dragonlord Ojutai has hexproof as long as it's untapped.
// Whenever Dragonlord Ojutai deals combat damage to a player, look at top 3 cards,
// put one in hand and rest on bottom in any order.
//
// CR 604.2 / CR 613.1f (Layer 6): The hexproof is conditional on the source being untapped.
// Implemented as a Static ability with Condition::SourceIsUntapped.
//
// TODO: "Whenever Dragonlord Ojutai deals combat damage to a player, look at the top three
// cards of your library. Put one of them into your hand and the rest on the bottom of your
// library in any order." — DSL gap: no Effect::LookAtTopCards with put-to-hand/bottom choice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonlord-ojutai"),
        name: "Dragonlord Ojutai".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Dragon"],
        ),
        oracle_text: "Flying\nDragonlord Ojutai has hexproof as long as it's untapped.\nWhenever Dragonlord Ojutai deals combat damage to a player, look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 604.2 / CR 613.1f (Layer 6): "Dragonlord Ojutai has hexproof as long as
            // it's untapped." The condition is checked at layer-application time each time
            // characteristics are calculated — hexproof vanishes immediately when tapped.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Hexproof),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::SourceIsUntapped),
                },
            },
            // TODO: triggered — combat damage to player → look at top 3, put 1 in hand, rest on bottom.
            // DSL gap: no Effect::LookAtTopCards with put-to-hand/bottom choice.
        ],
        ..Default::default()
    }
}
