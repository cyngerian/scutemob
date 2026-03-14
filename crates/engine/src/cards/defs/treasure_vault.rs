// Treasure Vault — Artifact Land, {T}: Add {C}; {X}{X},{T},Sacrifice: Create X Treasures (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("treasure-vault"),
        name: "Treasure Vault".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Artifact, CardType::Land], &[]),
        oracle_text: "{T}: Add {C}.\n{X}{X}, {T}, Sacrifice this land: Create X Treasure tokens.".to_string(),
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
            // TODO: {X}{X}, {T}, Sacrifice: Create X Treasure tokens — PB-9 (X costs)
            // Cost::SacrificeSelf available; blocked on X-cost + X-scaled token creation
        ],
        ..Default::default()
    }
}
