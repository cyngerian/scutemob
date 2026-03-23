// Welcoming Vampire — {2}{W}, Creature — Vampire 2/3
// Flying
// Whenever one or more other creatures you control with power 2 or less enter,
// draw a card. This ability triggers only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("welcoming-vampire"),
        name: "Welcoming Vampire".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Flying\nWhenever one or more other creatures you control with power 2 or less enter, draw a card. This ability triggers only once each turn.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Once each turn" trigger + power<=2 filter not in DSL.
            //   Overbroad trigger removed to avoid wrong game state.
        ],
        ..Default::default()
    }
}
