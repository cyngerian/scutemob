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
        oracle_text: "Indestructible\nWhenever this creature is dealt damage, it deals that much \
                      damage to target opponent.\n{2}{R}, {T}: This creature fights another \
                      target creature."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // TODO: WhenDealtDamage trigger redirecting that amount to a target opponent
            // requires a targeted_trigger with a damage-amount variable — not in DSL.
            // {2}{R}, {T}: This creature fights another target creature.
            // CR 701.14a: Fight — each creature deals damage equal to its power to the other.
            // PB-XS: CR 109.1 / 601.2c — "another target creature" excludes Brash Taunter.
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
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    exclude_self: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        completeness: Completeness::partial(
            "Blocked on a damage-amount variable for non-combat damage triggers: \
             TriggerCondition::WhenDealtDamage is wired (abilities.rs:5413 -> SelfIsDealtDamage, \
             used by ripjaw_raptor.rs) and targeted triggers exist (PB-5), but the dispatch never \
             populates combat_damage_amount and EffectAmount has no 'damage dealt in the \
             triggering event' variable, so 'it deals THAT MUCH damage to target opponent' is \
             inexpressible.",
        ),
        ..Default::default()
    }
}
