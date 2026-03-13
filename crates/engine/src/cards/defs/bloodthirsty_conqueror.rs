// Bloodthirsty Conqueror — {3}{B}{B}, Creature — Vampire Knight 5/5
// Flying, deathtouch
// Whenever an opponent loses life, you gain that much life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodthirsty-conqueror"),
        name: "Bloodthirsty Conqueror".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Knight"]),
        oracle_text: "Flying, deathtouch\nWhenever an opponent loses life, you gain that much life. (Damage causes loss of life.)".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // TODO: Triggered ability — whenever an opponent loses life, you gain that much life.
            // DSL gap: no "whenever opponent loses life" trigger with life-amount mirror gain.
        ],
        ..Default::default()
    }
}
