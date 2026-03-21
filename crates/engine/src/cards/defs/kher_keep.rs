// Kher Keep — Legendary Land, {T}: Add {C}. {1}{R}, {T}: Create a 0/1 Kobold token (TODO hybrid-adjacent cost).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kher-keep"),
        name: "Kher Keep".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}{R}, {T}: Create a 0/1 red Kobold creature token named Kobolds of Kher Keep.".to_string(),
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
            // TODO: {1}{R}, {T}: Create a 0/1 red Kobold creature token named Kobolds of Kher Keep.
            // DSL gap: no named token spec for "Kobolds of Kher Keep" (TokenSpec only supports
            // predefined token types). Also needs Cost::Sequence([Cost::Mana({1}{R}), Cost::Tap]).
        ],
        ..Default::default()
    }
}
