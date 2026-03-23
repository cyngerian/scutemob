// Priest of Forgotten Gods — {1}{B}, Creature — Human Cleric 1/2
// {T}, Sacrifice two other creatures: Any number of target players each lose 2 life and
// sacrifice a creature of their choice. You add {B}{B} and draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("priest-of-forgotten-gods"),
        name: "Priest of Forgotten Gods".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "{T}, Sacrifice two other creatures: Any number of target players each lose 2 life and sacrifice a creature of their choice. You add {B}{B} and draw a card.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Tap, Sacrifice two other creatures" cost — sacrificing self instead of
            //   two others is wrong game state. Complex multi-target effect not expressible.
        ],
        ..Default::default()
    }
}
