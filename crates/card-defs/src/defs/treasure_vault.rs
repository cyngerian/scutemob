// Treasure Vault — Artifact Land, {T}: Add {C}; {X}{X},{T},Sacrifice: Create X Treasures
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
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // CR 107.3k: {X}{X}, {T}, Sacrifice: Create X Treasure tokens.
            // x_count: 2 means total cost = 2 * x_value generic mana.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { x_count: 2, ..Default::default() }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                // CR 107.3m: Create X Treasures using Repeat + XValue.
                effect: Effect::Repeat {
                    count: EffectAmount::XValue,
                    effect: Box::new(Effect::CreateToken {
                        spec: treasure_token_spec(1),
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
