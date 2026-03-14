// Emeria, the Sky Ruin
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("emeria-the-sky-ruin"),
        name: "Emeria, the Sky Ruin".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "This land enters tapped.\nAt the beginning of your upkeep, if you control seven or more Plains, you may return target creature card from your graveyard to the battlefield.\n{T}: Add {W}.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // TODO: Triggered — At the beginning of your upkeep, if you control seven or more Plains,
            // return target creature card from your graveyard to the battlefield.
            // DSL gap: count_threshold + targeted_trigger + return_from_graveyard.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
