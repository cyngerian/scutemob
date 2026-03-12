// Oathsworn Vampire
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oathsworn-vampire"),
        name: "Oathsworn Vampire".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Knight", "Vampire"]),
        oracle_text: "This creature enters tapped.\nYou may cast this card from your graveyard if you gained life this turn.".to_string(),
        abilities: vec![
            // TODO: This creature enters tapped.
            // TODO: You may cast this card from your graveyard if you gained life this turn.
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}
