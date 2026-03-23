// Preacher of the Schism — {2}{B}, Creature — Vampire Cleric 2/4
// Deathtouch
// Whenever this creature attacks the player with the most life or tied for most life,
// create a 1/1 white Vampire creature token with lifelink.
// Whenever this creature attacks while you have the most life or are tied for most life,
// you draw a card and you lose 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("preacher-of-the-schism"),
        name: "Preacher of the Schism".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Cleric"]),
        oracle_text: "Deathtouch\nWhenever this creature attacks the player with the most life or tied for most life, create a 1/1 white Vampire creature token with lifelink.\nWhenever this creature attacks while you have the most life or are tied for most life, you draw a card and you lose 1 life.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // TODO: "Attacks the player with most life" conditional attack trigger not in DSL.
            // TODO: "Attacks while you have most life" conditional attack trigger not in DSL.
            //   Unconditional draw-on-attack would be wrong game state; removed.
        ],
        ..Default::default()
    }
}
