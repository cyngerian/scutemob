// Badlands — ({T}: Add {B} or {R}.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("badlands"),
        name: "Badlands".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp", "Mountain"]),
        oracle_text: "({T}: Add {B} or {R}.)".to_string(),
        abilities: vec![
                        // SR-33 (CR 605.1a/605.3b): the printed "or" is one ability per
            // colour. A mana ability never uses the stack, so the mode choice is
            // made at activation — `TapForMana { ability_index }` selects the
            // colour. Modelling it as `Effect::Choose` registered zero mana
            // abilities and only ever produced the first colour.
AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 0, 0) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 1, 0, 0) },
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
