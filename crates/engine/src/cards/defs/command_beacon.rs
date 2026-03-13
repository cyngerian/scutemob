// Command Beacon — Land, {T}: Add {C}; sacrifice to put commander in hand (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("command-beacon"),
        name: "Command Beacon".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice this land: Put your commander into your hand from the command zone.".to_string(),
        abilities: vec![
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}, Sacrifice: Put your commander into your hand from the command zone
            // — sacrifice cost + command zone targeting not expressible in DSL
        ],
        ..Default::default()
    }
}
