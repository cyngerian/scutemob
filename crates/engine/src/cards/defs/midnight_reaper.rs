// Midnight Reaper — {2}{B}, Creature — Zombie Knight 3/2
// Whenever a nontoken creature you control dies, Midnight Reaper deals 1 damage to
// you and you draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("midnight-reaper"),
        name: "Midnight Reaper".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Zombie", "Knight"]),
        oracle_text: "Whenever a nontoken creature you control dies, Midnight Reaper deals 1 damage to you and you draw a card.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // CR 603.10a: "Whenever a nontoken creature you control dies, draw a card and take 1 damage."
            // PB-23: controller_you + nontoken_only filters via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: false, nontoken_only: true },
                effect: Effect::Sequence(vec![
                    Effect::DealDamage {
                        target: EffectTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
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
