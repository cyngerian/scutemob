// Temple of the Dragon Queen
// As this land enters, you may reveal a Dragon card from your hand. This land enters
// tapped unless you revealed a Dragon card this way or you control a Dragon.
// As this land enters, choose a color.
// {T}: Add one mana of the chosen color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temple-of-the-dragon-queen"),
        name: "Temple of the Dragon Queen".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, you may reveal a Dragon card from your hand. This land enters tapped unless you revealed a Dragon card this way or you control a Dragon.\nAs this land enters, choose a color.\n{T}: Add one mana of the chosen color.".to_string(),
        abilities: vec![
            // CR 614.1c: enters tapped unless you revealed a Dragon or control a Dragon.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::Or(
                    Box::new(Condition::CanRevealFromHandWithSubtype(vec![SubType(
                        "Dragon".to_string(),
                    )])),
                    Box::new(Condition::ControlCreatureWithSubtype(SubType(
                        "Dragon".to_string(),
                    ))),
                )),
            },
            // CR 614.12 / CR 614.12a: "As this land enters, choose a color."
            // Replacement effect — NOT a triggered ability (PB-X C1 lesson).
            // Default: White (arbitrary; deterministic fallback overrides at ETB time).
            // CR 616.1: Multiple ETB replacements (EntersTapped + ChooseColor); controller
            // chooses order. Both fire on WouldEnterBattlefield — engine applies in order listed.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseColor(Color::White),
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add one mana of the chosen color.
            // Effect::AddManaOfChosenColor reads chosen_color from this object at execution time.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaOfChosenColor {
                    player: PlayerTarget::Controller,
                    amount: 1,
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
