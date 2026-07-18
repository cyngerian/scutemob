// Kalastria Highborn — {B}{B}, Creature — Vampire Shaman 2/2
// Whenever this creature or another Vampire you control dies, you may pay {B}. If you do,
// target player loses 2 life and you gain 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kalastria-highborn"),
        name: "Kalastria Highborn".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Whenever Kalastria Highborn or another Vampire you control dies, you may \
                      pay {B}. If you do, target player loses 2 life and you gain 2 life."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // "Whenever this creature or another Vampire you control dies, you may pay {B}.
            // If you do, target player loses 2 life and you gain 2 life." exclude_self stays
            // false — Kalastria is herself a Vampire and matches her own death.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Vampire".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Mana(ManaCost {
                        black: 1,
                        ..Default::default()
                    }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::Sequence(vec![
                        Effect::LoseLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(2),
                        },
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                    ])),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPlayer],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
