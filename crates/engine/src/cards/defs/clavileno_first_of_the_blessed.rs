// Clavileño, First of the Blessed — {1}{W}{B}, Legendary Creature — Vampire Cleric 2/2
// Whenever you attack, target attacking Vampire that isn't a Demon becomes a Demon in
// addition to its other types. It gains "When this creature dies, draw a card and create
// a tapped 4/3 white and black Vampire Demon creature token with flying."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("clavileno-first-of-the-blessed"),
        name: "Clavileño, First of the Blessed".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Cleric"],
        ),
        oracle_text: "Whenever you attack, target attacking Vampire that isn't a Demon becomes a Demon in addition to its other types. It gains \"When this creature dies, draw a card and create a tapped 4/3 white and black Vampire Demon creature token with flying.\"".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever you attack" trigger not in DSL (WheneverYouAttack missing).
            // TODO: Type addition + granting triggered abilities to target not expressible.
        ],
        ..Default::default()
    }
}
