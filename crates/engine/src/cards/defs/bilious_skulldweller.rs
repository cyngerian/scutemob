// Bilious Skulldweller — {B}, Creature — Phyrexian Insect 1/1
// Deathtouch, Toxic 1
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bilious-skulldweller"),
        name: "Bilious Skulldweller".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Insect"]),
        oracle_text: "Deathtouch\nToxic 1 (Players dealt combat damage by this creature also get a poison counter.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
        ],
        ..Default::default()
    }
}
