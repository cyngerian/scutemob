// Dragonlord Ojutai — {3}{W}{U}, Legendary Creature — Elder Dragon 5/4
// Flying
// Dragonlord Ojutai has hexproof as long as it's untapped.
// Whenever Dragonlord Ojutai deals combat damage to a player, look at top 3 cards,
// put one in hand and rest on bottom in any order.
// TODO: DSL gap — conditional hexproof (only while untapped) requires a static ability with
// a game-state condition; no Condition::WhileUntapped or Condition::WhileSelfUntapped exists.
// TODO: DSL gap — combat damage trigger that puts top cards of library into hand/bottom
// requires Effect::LookAtTopCards with choice, not yet in DSL.
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
            // TODO: static — hexproof while untapped.
            // DSL gap: no Condition::WhileSelfUntapped for static keyword grants.
            // TODO: triggered — combat damage to player → look at top 3, put 1 in hand, rest on bottom.
            // DSL gap: no Effect::LookAtTopCards with put-to-hand/bottom choice.
        ],
        ..Default::default()
    }
}
