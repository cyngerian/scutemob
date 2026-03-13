// Blinkmoth Nexus — Land, {T}: Add {C}. {1}: animate (TODO). {1},{T}: pump Blinkmoth (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blinkmoth-nexus"),
        name: "Blinkmoth Nexus".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}: This land becomes a 1/1 Blinkmoth artifact creature with flying until end of turn. It's still a land.\n{1}, {T}: Target Blinkmoth creature gets +1/+1 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {1}: Animate land as 1/1 Blinkmoth artifact creature with flying — land animation not in DSL
            // TODO: {1},{T}: Target Blinkmoth creature gets +1/+1 until EOT — targeted pump for activated abilities not in DSL
        ],
        ..Default::default()
    }
}
