// Creeping Tar Pit
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("creeping-tar-pit"),
        name: "Creeping Tar Pit".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {U} or {B}.\n{1}{U}{B}: Until end of turn, this land becomes a 3/2 blue and black Elemental creature. It's still a land. It can't be blocked this turn.".to_string(),
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
            // {T}: Add {U} or {B}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Choose {
                    prompt: "Add {U} or {B}?".to_string(),
                    choices: vec![
                        Effect::AddMana {
                            player: PlayerTarget::Controller,
                            mana: mana_pool(0, 1, 0, 0, 0, 0),
                        },
                        Effect::AddMana {
                            player: PlayerTarget::Controller,
                            mana: mana_pool(0, 0, 1, 0, 0, 0),
                        },
                    ],
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: Activated — {1}{U}{B}: land animation (becomes 3/2 blue/black Elemental
            // creature with "can't be blocked this turn"). DSL gap: land animation effect.
        ],
        ..Default::default()
    }
}
