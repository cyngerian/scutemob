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
        completeness: Completeness::known_wrong(
            "Enters with ONE too many +1/+1 counters. The def is correct (EntersWithCounters, \
             count = PermanentCount{ has_subtype: Human, controller: You, exclude_self: true }), \
             but the engine's PermanentCount resolver (effects/mod.rs:6749) does not honor \
             `exclude_self` — unlike the sibling AttackingCreatureCount (:7032) and \
             TappedCreatureCount (:7066) resolvers, which apply `obj.id != ctx.source`. Since the \
             self-ETB replacement resolves with ctx.source = Éomer AFTER Éomer is on the \
             battlefield (a Human you control), it counts itself: a 2/2 with no other Humans \
             enters as a 3/3. Blocker filed as W-PB2 engine finding EF-W-PB2-1 (one-line fix). \
             DoubleStrike + the two-target ETB (BecomeMonarch + DealDamage PowerOf) are correct.",
        ),
        ..Default::default()
    }
}
