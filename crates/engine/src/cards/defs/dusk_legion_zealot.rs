// Dusk Legion Zealot — {1}{B}, Creature — Vampire Soldier 1/1
// When this creature enters, you draw a card and you lose 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dusk-legion-zealot"),
        name: "Dusk Legion Zealot".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Soldier"]),
        oracle_text: "When this creature enters, you draw a card and you lose 1 life.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
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
