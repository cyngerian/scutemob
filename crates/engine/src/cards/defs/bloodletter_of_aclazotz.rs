// Bloodletter of Aclazotz — {1}{B}{B}{B}, Creature — Vampire Demon 2/4
// Flying
// If an opponent would lose life during your turn, they lose twice that much life instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodletter-of-aclazotz"),
        name: "Bloodletter of Aclazotz".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 3, ..Default::default() }),
        types: creature_types(&["Vampire", "Demon"]),
        oracle_text: "Flying\nIf an opponent would lose life during your turn, they lose twice that much life instead. (Damage causes loss of life.)".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: Replacement effect — if an opponent would lose life during your turn, they lose
            // twice that much life instead.
            // DSL gap: no life-loss-doubling replacement effect conditioned on whose turn it is.
        ],
        ..Default::default()
    }
}
