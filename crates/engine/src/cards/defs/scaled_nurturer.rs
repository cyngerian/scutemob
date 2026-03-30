// Scaled Nurturer — {1}{G}, Creature — Dragon Druid 0/2
// {T}: Add {G}. When you spend this mana to cast a Dragon creature spell, you gain 2 life.
//
// TODO: "When you spend this mana to cast a Dragon creature spell, you gain 2 life."
//   DSL gap: no "when you spend this mana to cast [filter] spell" trigger mechanism exists.
//   Implementing only the base {T}: Add {G} ability per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scaled-nurturer"),
        name: "Scaled Nurturer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Dragon", "Druid"]),
        oracle_text: "{T}: Add {G}. When you spend this mana to cast a Dragon creature spell, you gain 2 life.".to_string(),
        power: Some(0),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
