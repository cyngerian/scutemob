// Ancient Tomb — Land.
// "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you."
// CR 305.6: Land activated ability; produces {C}{C} but also deals 2 damage.
// Modeled as a regular Activated ability (not a pure mana ability) because it
// has a non-mana side effect. Engine timing: resolves normally (not mana-ability fast).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-tomb"),
        name: "Ancient Tomb".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 2),
                    },
                    Effect::DealDamage {
                        target: EffectTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
