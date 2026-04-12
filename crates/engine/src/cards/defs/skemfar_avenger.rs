// Skemfar Avenger — {1}{B}, Creature — Elf Berserker 3/1
// Whenever another nontoken Elf or Berserker you control dies, you draw a card
// and you lose 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skemfar-avenger"),
        name: "Skemfar Avenger".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Berserker"]),
        oracle_text: "Whenever another nontoken Elf or Berserker you control dies, you draw a card and you lose 1 life.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            // CR 603.10a: "Whenever another nontoken Elf or Berserker you control dies."
            // PB-23: controller_you + exclude_self + nontoken_only via DeathTriggerFilter.
            // TODO: Elf/Berserker subtype filter not yet in DSL — over-triggers on other creature types.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: true, nontoken_only: true, filter: None,
},
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
