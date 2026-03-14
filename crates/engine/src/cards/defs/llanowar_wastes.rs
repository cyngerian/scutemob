// Llanowar Wastes — Land; {T}: Add {C}; {T}: Add {B} or {G} (deals 1 damage to you).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("llanowar-wastes"),
        name: "Llanowar Wastes".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {B} or {G}. This land deals 1 damage to you.".to_string(),
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
                        mana: mana_pool(0, 0, 1, 0, 1, 0),
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
