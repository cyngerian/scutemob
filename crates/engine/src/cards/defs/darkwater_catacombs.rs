// Darkwater Catacombs — Land, {1}, {T}: Add {U}{B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darkwater-catacombs"),
        name: "Darkwater Catacombs".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{1}, {T}: Add {U}{B}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
