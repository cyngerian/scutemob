// Slayers' Stronghold — Land, {T}: Add {C}. {R}{W},{T}: Target creature +2/+0 vigilance haste (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("slayers-stronghold"),
        name: "Slayers' Stronghold".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{R}{W}, {T}: Target creature gets +2/+0 and gains vigilance and haste until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {R}{W},{T}: Target creature +2/+0, gains vigilance and haste until EOT — targeted pump not in DSL for activated abilities
        ],
        ..Default::default()
    }
}
