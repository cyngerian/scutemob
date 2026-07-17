// Blooming Marsh — This land enters tapped unless you control two or fewer other lands. {T}: Add {B
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blooming-marsh"),
        name: "Blooming Marsh".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control two or fewer other lands.\n{T}: \
                      Add {B} or {G}."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlAtMostNOtherLands(2)),
            },
            // SR-33 (CR 605.1a/605.3b): the printed "or" is one ability per
            // colour. A mana ability never uses the stack, so the mode choice is
            // made at activation — `TapForMana { ability_index }` selects the
            // colour. Modelling it as `Effect::Choose` registered zero mana
            // abilities and only ever produced the first colour.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
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
