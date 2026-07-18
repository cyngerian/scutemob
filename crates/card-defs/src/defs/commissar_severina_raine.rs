// Commissar Severina Raine — {1}{W}{B}, Legendary Creature — Human Soldier 2/2
// Whenever Commissar Severina Raine attacks, each opponent loses X life, where X
// is the number of other attacking creatures.
// {2}, Sacrifice another creature: You gain 2 life and draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("commissar-severina-raine"),
        name: "Commissar Severina Raine".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Leading from the Front — Whenever Commissar Severina Raine attacks, each \
                      opponent loses X life, where X is the number of other attacking \
                      creatures.\nSummary Execution — {2}, Sacrifice another creature: You gain 2 \
                      life and draw a card."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // Leading from the Front — Whenever Commissar Severina Raine attacks, each
            // opponent loses X life, where X is the number of OTHER attacking creatures.
            // CR 508.1: AttackingCreatureCount honors filter.exclude_self (effects/mod.rs),
            // so Commissar herself is excluded from the count.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::LoseLife {
                    player: PlayerTarget::EachOpponent,
                    amount: EffectAmount::AttackingCreatureCount {
                        controller: PlayerTarget::EachPlayer,
                        filter: Some(TargetFilter {
                            exclude_self: true,
                            ..Default::default()
                        }),
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // Summary Execution — {2}, Sacrifice another creature: You gain 2 life and
            // draw a card.
            // PB-EF1 (CR 109.1): "Sacrifice ANOTHER creature" — Cost::Sacrifice carries
            // TargetFilter.exclude_self, lowered onto ActivationCost.sacrifice_exclude_self
            // and enforced in handle_activate_ability, so Commissar cannot pay by sacrificing
            // herself.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 2,
                        ..Default::default()
                    }),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    }),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        // PB-EF1 (scutemob-99): both clauses authored. The attack trigger uses
        // AttackingCreatureCount with exclude_self ("other attacking creatures"); the
        // "{2}, Sacrifice another creature" cost is enforced via
        // ActivationCost.sacrifice_exclude_self (CR 109.1). Complete.
        ..Default::default()
    }
}
