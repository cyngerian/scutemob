// Wonder — {3}{U}, Creature — Incarnation 2/2
// Flying
// As long as this card is in your graveyard and you control an Island, creatures you control have flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wonder"),
        name: "Wonder".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: creature_types(&["Incarnation"]),
        oracle_text: "Flying\nAs long as this card is in your graveyard and you control an Island, creatures you control have flying.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: Static ability — grant flying to creatures you control while this is in graveyard
            // and you control an Island. DSL gap: no graveyard-zone static effect with land-type condition.
        ],
        ..Default::default()
    }
}
