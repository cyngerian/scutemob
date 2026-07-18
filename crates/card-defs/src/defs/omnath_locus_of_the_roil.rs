// Omnath, Locus of the Roil — {1}{G}{U}{R}, Legendary Creature — Elemental 3/3
// When Omnath enters, it deals damage to any target equal to the number of Elementals you
// control.
// Landfall — Whenever a land you control enters, put a +1/+1 counter on target Elemental you
// control. If you control eight or more lands, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("omnath-locus-of-the-roil"),
        name: "Omnath, Locus of the Roil".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            red: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental"],
        ),
        oracle_text: "When Omnath, Locus of the Roil enters, it deals damage to any target equal \
                      to the number of Elementals you control.\nLandfall \u{2014} Whenever a land \
                      you control enters, put a +1/+1 counter on target Elemental you control. If \
                      you control eight or more lands, draw a card."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // When Omnath enters, it deals damage to any target equal to the number of
            // Elementals you control. CR 611.2c: amount is calculated as the ability resolves
            // (Omnath, if still on the battlefield, counts itself — TargetController::You +
            // has_subtype Elemental over PermanentCount naturally includes Omnath).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_subtype: Some(SubType("Elemental".to_string())),
                            controller: TargetController::You,
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                trigger_zone: None,
            },
            // Landfall — Whenever a land you control enters, put a +1/+1 counter on target
            // Elemental you control. If you control eight or more lands, draw a card.
            // The counter placement is mandatory (a legal target must exist, or the ability
            // doesn't resolve per 2019-07-12 ruling); the draw is conditional on land count.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::Conditional {
                        condition: Condition::YouControlNOrMoreWithFilter {
                            count: 8,
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Land),
                                ..Default::default()
                            },
                        },
                        if_true: Box::new(Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(1),
                        }),
                        if_false: Box::new(Effect::Nothing),
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Elemental".to_string())),
                    controller: TargetController::You,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
