// 19a. Dimir Guildgate — Land — Gate; enters the battlefield tapped.
// {T}: Add {U} or {B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dimir-guildgate"),
        name: "Dimir Guildgate".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Gate"]),
        oracle_text: "Dimir Guildgate enters the battlefield tapped.\n{T}: Add {U} or {B}."
            .to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement effect — this permanent enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {U} or {B} (CR 106.6: player chooses color).
            // M9.4: uses Effect::Choose between AddMana blue and AddMana black.
            // Deterministic fallback executes the first option (blue).
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
            },
        ],
        ..Default::default()
    }
}
