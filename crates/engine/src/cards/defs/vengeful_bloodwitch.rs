// Vengeful Bloodwitch — {1}{B}, Creature — Vampire Warlock 1/1
// Whenever this creature or another creature you control dies, target opponent loses 1
// life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vengeful-bloodwitch"),
        name: "Vengeful Bloodwitch".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Warlock"]),
        oracle_text: "Whenever Vengeful Bloodwitch or another creature you control dies, target opponent loses 1 life and you gain 1 life.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.10a: "Whenever this creature or another creature you control dies,
            // target opponent loses 1 life and you gain 1 life."
            // PB-23: controller_you filter applied via DeathTriggerFilter.
            // "Target opponent" is approximated as DeclaredTarget { index: 0 }.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                },
                effect: Effect::Sequence(vec![
                    Effect::LoseLife {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPlayer],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
