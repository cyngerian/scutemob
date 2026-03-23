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
            // TODO: "Another nontoken Elf or Berserker you control dies" — WheneverCreatureDies
            //   is overbroad (all creatures, not just Elf/Berserker, nontoken, yours, another).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You) },
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
            },
        ],
        ..Default::default()
    }
}
