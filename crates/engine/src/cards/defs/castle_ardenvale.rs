// Castle Ardenvale — This land enters tapped unless you control a Plains. {T}: Add {W}.
// {2}{W}{W}, {T}: Create a 1/1 white Human creature token. (complex activated ability — TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("castle-ardenvale"),
        name: "Castle Ardenvale".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Plains.\n{T}: Add {W}.\n{2}{W}{W}, {T}: Create a 1/1 white Human creature token.".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType("Plains".to_string())])),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: Activated — {2}{W}{W}, {T}: Create a 1/1 white Human creature token.
        ],
        ..Default::default()
    }
}
