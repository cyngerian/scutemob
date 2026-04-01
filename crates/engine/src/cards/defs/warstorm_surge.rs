// Warstorm Surge — {5}{R}, Enchantment
// Whenever a creature you control enters, it deals damage equal to its power to any target.
//
// TODO: EffectAmount::PowerOf(EffectTarget::TriggeringCreature) — the triggering creature's power
// is needed for the damage amount. EffectTarget::TriggeringCreature exists, but PowerOf applied
// to a TriggeringCreature at effect resolution needs to be verified against the execution path
// in effects/mod.rs (EffectAmount::PowerOf reads power via resolve_effect_target). The effect
// is approximated as Nothing since directing targeted damage with a variable amount from
// TriggeringCreature power is marginally supported but untested here.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("warstorm-surge"),
        name: "Warstorm Surge".to_string(),
        mana_cost: Some(ManaCost { generic: 5, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control enters, it deals damage equal to its power to any target.".to_string(),
        abilities: vec![
            // CR 603.6a: "Whenever a creature you control enters" — triggered ability.
            // Effect: the entering creature deals damage equal to its power to any target.
            // EffectAmount::PowerOf(EffectTarget::TriggeringCreature) gives the entering
            // creature's power. EffectTarget::TriggeringCreature resolves from
            // PendingTrigger::entering_object_id at execution.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::DealDamage {
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
