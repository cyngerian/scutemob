// Ancient Den — {T}: Add {W}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-den"),
        name: "Ancient Den".to_string(),
        mana_cost: None,
        types: types(&[CardType::Artifact, CardType::Land]),
        oracle_text: "{T}: Add {W}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(1, 0, 0, 0, 0, 0) },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
