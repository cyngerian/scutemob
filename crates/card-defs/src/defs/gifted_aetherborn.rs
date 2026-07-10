// Gifted Aetherborn — {B}{B}, Creature — Aetherborn Vampire 2/3
// Deathtouch, lifelink
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gifted-aetherborn"),
        name: "Gifted Aetherborn".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Aetherborn", "Vampire"]),
        oracle_text: "Deathtouch, lifelink".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
