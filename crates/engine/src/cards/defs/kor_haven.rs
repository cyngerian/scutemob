// Kor Haven — Legendary Land, {T}: Add {C}. {1}{W},{T}: Prevent combat damage from target attacker (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kor-haven"),
        name: "Kor Haven".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Legendary], &[CardType::Land], &[]),
        oracle_text: "{T}: Add {C}.\n{1}{W}, {T}: Prevent all combat damage that would be dealt by target attacking creature this turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: {1}{W},{T}: Prevent all combat damage from target attacking creature — prevention effect not in DSL
        ],
        ..Default::default()
    }
}
