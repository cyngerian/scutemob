// Brash Taunter — {4}{R}, Creature — Goblin 1/1
// Indestructible; when dealt damage deals that much to target opponent; activated fight
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brash-taunter"),
        name: "Brash Taunter".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Indestructible\nWhenever this creature is dealt damage, it deals that much damage to target opponent.\n{2}{R}, {T}: This creature fights another target creature.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // TODO: WhenDealtDamage trigger redirecting that amount to a target opponent
            // requires a targeted_trigger with a damage-amount variable — not in DSL.
            // {2}{R}, {T}: This creature fights another target creature.
            // CR 701.14a: Fight — each creature deals damage equal to its power to the other.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 2,
                        red: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::Fight {
                    attacker: EffectTarget::Source,
                    defender: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
