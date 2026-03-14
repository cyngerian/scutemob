// Isolated Chapel — Land; enters tapped unless you control a Plains or Swamp.
// {T}: Add {W} or {B}.
// TODO: DSL gap — the conditional ETB check "unless you control a Plains or Swamp"
// requires an ObjectFilter matching by subtype, which is not supported. Modeled as
// always entering tapped (safe conservative fallback).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("isolated-chapel"),
        name: "Isolated Chapel".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Isolated Chapel enters the battlefield tapped unless you control a Plains or a Swamp.\n{T}: Add {W} or {B}.".to_string(),
        abilities: vec![
            // TODO: conditional ETB — should check for Plains/Swamp; always tapped for now
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType("Plains".to_string()), SubType("Swamp".to_string())])),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Choose {
                    prompt: "Add {W} or {B}?".to_string(),
                    choices: vec![
                        Effect::AddMana {
                            player: PlayerTarget::Controller,
                            mana: mana_pool(1, 0, 0, 0, 0, 0),
                        },
                        Effect::AddMana {
                            player: PlayerTarget::Controller,
                            mana: mana_pool(0, 0, 1, 0, 0, 0),
                        },
                    ],
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
