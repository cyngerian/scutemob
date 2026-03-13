// Viridescent Bog — Land; {1}, {T}: Add {B}{G}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("viridescent-bog"),
        name: "Viridescent Bog".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{1}, {T}: Add {B}{G}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 1, 0),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
