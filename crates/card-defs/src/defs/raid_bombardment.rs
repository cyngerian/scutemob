// Raid Bombardment — {2}{R}, Enchantment
// Whenever a creature you control with power 2 or less attacks, this enchantment
//   deals 1 damage to the player or planeswalker that creature is attacking.
//
// PB-EF3 (EF-W-MISS-4): EffectTarget::AttackTarget resolves to the specific attack
// target of the triggering creature (CR 508.4/506.4c).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("raid-bombardment"),
        name: "Raid Bombardment".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control with power 2 or less attacks, this \
                      enchantment deals 1 damage to the player or planeswalker that creature is \
                      attacking."
            .to_string(),
        abilities: vec![
            // CR 508.1m / CR 601.2c: "Whenever a creature you control with power 2 or less
            // attacks, this enchantment deals 1 damage to the player or planeswalker that
            // creature is attacking." The `max_power` filter is applied to the triggering
            // (attacking) creature at trigger-collection time (collect_triggers_for_event).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: Some(TargetFilter {
                        max_power: Some(2),
                        ..Default::default()
                    }),
                },
                effect: Effect::DealDamage {
                    target: EffectTarget::AttackTarget,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
