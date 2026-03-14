// Flamekin Village
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flamekin-village"),
        name: "Flamekin Village".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, you may reveal an Elemental card from your hand. If you don't, this land enters tapped.\n{T}: Add {R}.\n{R}, {T}: Target creature gains haste until end of turn.".to_string(),
        abilities: vec![
            // CR 614.1c: enters tapped unless you reveal an Elemental card from your hand.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::CanRevealFromHandWithSubtype(vec![SubType("Elemental".to_string())])),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 1, 0, 0) },
                timing_restriction: None,
            },
            // TODO: Activated — {R}, {T}: Target creature gains haste until end of turn.
        ],
        ..Default::default()
    }
}
