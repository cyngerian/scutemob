// Grim Haruspex — {2}{B}, Creature — Human Wizard 3/2
// Morph {B}
// Whenever another nontoken creature you control dies, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grim-haruspex"),
        name: "Grim Haruspex".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Morph {B} (You may cast this card face down as a 2/2 creature for {3}. Turn it face up any time for its morph cost.)\nWhenever another nontoken creature you control dies, draw a card.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Morph),
            AbilityDefinition::Morph {
                cost: ManaCost { black: 1, ..Default::default() },
            },
            // TODO: "Whenever another nontoken creature you control dies, draw a card"
            //   — WheneverCreatureDies lacks controller filter + exclude-self + nontoken.
            //   Overbroad trigger draws on all deaths (opponents' too) = wrong game state.
        ],
        ..Default::default()
    }
}
