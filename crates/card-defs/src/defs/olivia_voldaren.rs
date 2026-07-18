// Olivia Voldaren — {2}{B}{R}, Legendary Creature — Vampire 3/3
// Flying
// {1}{R}: Olivia Voldaren deals 1 damage to another target creature. That creature becomes
// a Vampire in addition to its other types. Put a +1/+1 counter on Olivia Voldaren.
// {3}{B}{B}: Gain control of target Vampire for as long as you control Olivia Voldaren.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("olivia-voldaren"),
        name: "Olivia Voldaren".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            red: 1,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Vampire"]),
        oracle_text: "Flying\n{1}{R}: Olivia Voldaren deals 1 damage to another target creature. \
                      That creature becomes a Vampire in addition to its other types. Put a +1/+1 \
                      counter on Olivia Voldaren.\n{3}{B}{B}: Gain control of target Vampire for \
                      as long as you control Olivia Voldaren."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 602.2: {1}{R}: Olivia Voldaren deals 1 damage to another target creature.
            // That creature becomes a Vampire in addition to its other types (no stated
            // duration -> CR 611.2c indefinite). Put a +1/+1 counter on Olivia Voldaren.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 1,
                    red: 1,
                    ..Default::default()
                }),
                effect: Effect::Sequence(vec![
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddSubtypes(
                                [SubType("Vampire".to_string())].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    exclude_self: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // CR 613.1b: {3}{B}{B}: Gain control of target Vampire for as long as you control
            // Olivia Voldaren (WhileSourceOnBattlefield approximation).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 3,
                    black: 2,
                    ..Default::default()
                }),
                effect: Effect::GainControl {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Vampire".to_string())),
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        completeness: Completeness::partial(
            "The {1}{R} ability is fully correct (1 damage to another target creature, it becomes \
             a Vampire in addition, +1/+1 counter on Olivia). The {3}{B}{B} gain-control ability \
             approximates \"for as long as you control Olivia Voldaren\" with \
             EffectDuration::WhileSourceOnBattlefield — no EffectDuration variant expresses \
             \"while you control source\" (continuous_effect.rs L44-64), so the borrowed creature \
             would NOT return if an opponent gains control of Olivia while she stays on the \
             battlefield (wrong game state under gain-control). Blocked on a missing duration \
             primitive — W-PB2 engine finding EF-W-PB2-5.",
        ),
        ..Default::default()
    }
}
