// Zurgo and Ojutai — {2}{U}{R}{W}, Legendary Creature — Orc Dragon 4/4
// Flying, haste; hexproof as long as it entered this turn;
// whenever one or more Dragons deal combat damage to player/battle: look top 3, put one in hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zurgo-and-ojutai"),
        name: "Zurgo and Ojutai".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, red: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Orc", "Dragon"],
        ),
        oracle_text: "Flying, haste\nZurgo and Ojutai has hexproof as long as it entered this turn.\nWhenever one or more Dragons you control deal combat damage to a player or battle, look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order. You may return one of those Dragons to its owner's hand.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: DSL gap — conditional hexproof "as long as it entered this turn" requires
            // a duration-tracked continuous effect tied to ETB timestamp; not expressible.
            // TODO: DSL gap — "whenever one or more Dragons you control deal combat damage"
            // trigger with look-top-3 + optional return-to-hand effect not expressible.
        ],
        ..Default::default()
    }
}
