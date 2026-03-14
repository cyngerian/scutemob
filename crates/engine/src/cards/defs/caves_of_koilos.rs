// Caves of Koilos — Land; {T}: Add {C}; {T}: Add {W} or {B} (deals 1 damage to you).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("caves-of-koilos"),
        name: "Caves of Koilos".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {W} or {B}. This land deals 1 damage to you.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(1, 0, 1, 0, 0, 0),
                    },
                    Effect::DealDamage {
                        target: EffectTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
