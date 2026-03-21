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
                targets: vec![],
                activation_condition: None,
            },
            // {X}{X}, {T}, Sacrifice: Create X Treasure tokens.
            // TODO: X-scaled token creation (EffectAmount::XValue in token count) not yet wired.
            // Cost is correct; effect approximated as creating 1 Treasure.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { x_count: 2, ..Default::default() }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                // TODO: should create X Treasures (EffectAmount::XValue); approximated as 1.
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
