// Fog — {G}, Instant
// Prevent all combat damage that would be dealt this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fog"),
        name: "Fog".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Prevent all combat damage that would be dealt this turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::PreventAllCombatDamage,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
