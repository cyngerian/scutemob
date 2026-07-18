// Warstorm Surge — {5}{R}, Enchantment
// Whenever a creature you control enters, it deals damage equal to its power to any target.
//
// PB-EF4: the entering creature is both the amount reference (PowerOf(TriggeringCreature))
// AND the damage source (source: Some(TriggeringCreature)) — so its own
// lifelink/deathtouch/protection interactions apply correctly (CR 119.3 / 702.15a).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("warstorm-surge"),
        name: "Warstorm Surge".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control enters, it deals damage equal to its power \
                      to any target."
            .to_string(),
        abilities: vec![
            // CR 603.6a: "Whenever a creature you control enters" — triggered ability.
            // Effect: the entering creature deals damage equal to its power to any target.
            // EffectAmount::PowerOf(EffectTarget::TriggeringCreature) gives the entering
            // creature's power. EffectTarget::TriggeringCreature resolves from
            // PendingTrigger::entering_object_id at execution.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::DealDamage {
                    source: Some(EffectTarget::TriggeringCreature),
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::PowerOf(EffectTarget::TriggeringCreature),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
