// Goblin Sharpshooter — {2}{R} Creature — Goblin 1/1
// This creature doesn't untap during your untap step.
// Whenever a creature dies, untap this creature.
// {T}: This creature deals 1 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-sharpshooter"),
        name: "Goblin Sharpshooter".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Goblin Sharpshooter doesn't untap during your untap step.\nWhenever a creature dies, untap Goblin Sharpshooter.\n{T}: Goblin Sharpshooter deals 1 damage to any target.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // TODO: "doesn't untap during your untap step" — static restriction not in DSL.
            // CR 603.10a: "Whenever a creature dies, untap Goblin Sharpshooter."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: None,
                    exclude_self: false,
                    nontoken_only: false,
                },
                effect: Effect::UntapPermanent {
                    target: EffectTarget::Source,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // {T}: deal 1 damage to any target.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
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
