// Goblin Bombardment — {1}{R} Enchantment
// Sacrifice a creature: This enchantment deals 1 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-bombardment"),
        name: "Goblin Bombardment".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Sacrifice a creature: Goblin Bombardment deals 1 damage to any target.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![TargetRequirement::TargetAny],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
