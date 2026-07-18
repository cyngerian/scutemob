// Éomer, King of Rohan — {3}{R}{W}, Legendary Creature — Human Noble 2/2
// Double strike; enters with a +1/+1 counter per other Human you control; ETB: monarch +
// deal damage equal to power.
//
// CR 614.1c: "enters with" is a replacement effect, not a triggered ability —
// ReplacementModification::EntersWithCounters (see master_biomancer.rs / ingenious_prodigy.rs
// for the pattern).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eomer-king-of-rohan"),
        name: "Éomer, King of Rohan".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Noble"],
        ),
        oracle_text: "Double strike\nÉomer enters with a +1/+1 counter on it for each other Human \
                      you control.\nWhen Éomer enters, target player becomes the monarch. Éomer \
                      deals damage equal to its power to any target."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::DoubleStrike),
            // "Éomer enters with a +1/+1 counter on it for each other Human you control."
            // CR 614.1c replacement effect, not a trigger.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersWithCounters {
                    counter: CounterType::PlusOnePlusOne,
                    count: Box::new(EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Human".to_string())),
                            controller: TargetController::You,
                            exclude_self: true,
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    }),
                },
                is_self: true,
                unless_condition: None,
            },
            AbilityDefinition::Triggered {
                once_per_turn: false,
                // CR 603.5: "When ~ enters" — ETB trigger.
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    // "target player becomes the monarch" — CR 724.1.
                    Effect::BecomeMonarch {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                    },
                    // "Éomer deals damage equal to its power to any target" — CR 120.1.
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        amount: EffectAmount::PowerOf(EffectTarget::Source),
                    },
                ]),
                intervening_if: None,
                targets: vec![
                    // target 0: a player that becomes the monarch
                    TargetRequirement::TargetPlayer,
                    // target 1: any target (creature, planeswalker, or player) for damage
                    TargetRequirement::TargetAny,
                ],

                modes: None,
                trigger_zone: None,
            },
        ],
        // PB-EF1 (scutemob-99): the PermanentCount resolver now honors `exclude_self`
        // (effects/mod.rs, CR 109.1), so "for each OTHER Human you control" no longer
        // counts Éomer itself. A 2/2 with no other Humans enters as a 2/2. Complete.
        ..Default::default()
    }
}
