// Massive Raid — {1}{R}{R}, Instant
// Massive Raid deals damage equal to the number of creatures you control to target
// creature, player, or planeswalker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("massive-raid"),
        name: "Massive Raid".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Massive Raid deals damage equal to the number of creatures you control to target creature, player, or planeswalker.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
