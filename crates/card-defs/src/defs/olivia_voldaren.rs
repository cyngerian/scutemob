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
                        source: None,
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
                modes: None,
            },
            // CR 613.1b / 611.2b/c: {3}{B}{B}: Gain control of target Vampire for as long as
            // you control Olivia Voldaren. EffectDuration::WhileYouControlSource(PlayerId(0))
            // is the placeholder form; Effect::GainControl resolves PlayerId(0) to the
            // activating player's controller at execution time (PB-EF9).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 3,
                    black: 2,
                    ..Default::default()
                }),
                effect: Effect::GainControl {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::WhileYouControlSource(PlayerId(0)),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Vampire".to_string())),
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
