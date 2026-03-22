// Bontu's Monument — {3}, Legendary Artifact
// Black creature spells you cast cost {1} less to cast.
// Whenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.
//
// TODO: "Black creature spells" — SpellCostFilter needs compound HasColor+HasCardType filter.
//   HasColor(Black) alone reduces all black spells, not just creatures. Wrong game state.
// TODO: "Whenever you cast a creature spell" — WheneverYouCastSpell lacks a spell type filter.
//   Unfiltered trigger fires on all spells. Wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bontus-monument"),
        name: "Bontu's Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Black creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
