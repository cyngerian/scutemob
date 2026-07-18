// Spore Frog — {G}, Creature — Frog 1/1
// Sacrifice this creature: Prevent all combat damage that would be dealt this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spore-frog"),
        name: "Spore Frog".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Frog"]),
        oracle_text: "Sacrifice this creature: Prevent all combat damage that would be dealt this \
                      turn."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::SacrificeSelf,
            effect: Effect::PreventAllCombatDamage,
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}
