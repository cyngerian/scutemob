// Vampire Cutthroat — {B}, Creature — Vampire Rogue 1/1
// "Skulk (This creature can't be blocked by creatures with greater power.)
// Lifelink (Damage dealt by this creature also causes you to gain that much life.)"
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-cutthroat"),
        name: "Vampire Cutthroat".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Rogue"]),
        oracle_text: "Skulk (This creature can't be blocked by creatures with greater power.)\nLifelink (Damage dealt by this creature also causes you to gain that much life.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Skulk),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
