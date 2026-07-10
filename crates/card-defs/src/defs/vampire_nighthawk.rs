// Vampire Nighthawk — {1}{B}{B}, Creature — Vampire Shaman 2/3
// Flying, Deathtouch, Lifelink
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-nighthawk"),
        name: "Vampire Nighthawk".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Flying\nDeathtouch (Any amount of damage this deals to a creature is enough to destroy it.)\nLifelink (Damage dealt by this creature also causes you to gain that much life.)".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
