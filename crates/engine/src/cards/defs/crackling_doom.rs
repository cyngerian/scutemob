// Crackling Doom — {R}{W}{B}, Instant
// Crackling Doom deals 2 damage to each opponent. Each opponent sacrifices a creature
// with the greatest power among creatures that player controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crackling-doom"),
        name: "Crackling Doom".to_string(),
        mana_cost: Some(ManaCost { red: 1, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Crackling Doom deals 2 damage to each opponent. Each opponent sacrifices a creature with the greatest power among creatures that player controls.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // Deal 2 damage to each opponent.
            // TODO: Second part — "Each opponent sacrifices a creature with the greatest power"
            // requires per-opponent greatest-power filtering + forced sacrifice. Partial implementation.
            effect: Effect::ForEach {
                over: ForEachTarget::EachOpponent,
                effect: Box::new(Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                }),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
