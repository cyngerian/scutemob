// Hermes, Overseer of Elpis — {3}{U}, Legendary Creature — Elder Wizard 2/4
// Whenever you cast a noncreature spell, create a 1/1 blue Bird creature token
//   with flying and vigilance.
// Whenever you attack with one or more Birds, scry 2.
//
// TODO: "Whenever you cast a noncreature spell" — WheneverYouCastSpell lacks a spell type
//   filter (noncreature). Cannot express without overbroad trigger (wrong game state).
// TODO: "Whenever you attack with one or more Birds" — no trigger condition for
//   attacking with creatures of a subtype.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hermes-overseer-of-elpis"),
        name: "Hermes, Overseer of Elpis".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Wizard"],
        ),
        oracle_text: "Whenever you cast a noncreature spell, create a 1/1 blue Bird creature token with flying and vigilance.\nWhenever you attack with one or more Birds, scry 2.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![],
        ..Default::default()
    }
}
