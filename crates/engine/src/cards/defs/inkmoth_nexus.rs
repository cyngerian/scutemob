// Inkmoth Nexus — Land; {T}: Add {C}; {1}: becomes 1/1 Phyrexian Blinkmoth
// artifact creature with flying and infect until end of turn.
// TODO: animation ability ({1}: becomes creature until end of turn) not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("inkmoth-nexus"),
        name: "Inkmoth Nexus".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}: This land becomes a 1/1 Phyrexian Blinkmoth artifact creature with flying and infect until end of turn. It's still a land. (It deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {1}: animate land as 1/1 Phyrexian Blinkmoth artifact creature with flying
            // and infect until end of turn — requires land animation effect not in DSL.
        ],
        ..Default::default()
    }
}
