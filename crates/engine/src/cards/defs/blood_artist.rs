// Blood Artist — {1}{B}, Creature — Vampire 0/1.
// "Whenever Blood Artist or another creature dies, target player loses 1 life
// and you gain 1 life."
// CR 603.2: Triggered by WheneverCreatureDies (self included via any-creature trigger).
// Simplification: targets all opponents (no declared-target support for triggers).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blood-artist"),
        name: "Blood Artist".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever Blood Artist or another creature dies, target player loses 1 life and you gain 1 life.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            // WheneverCreatureDies — any creature dying triggers this (self included).
            // "Target player loses 1 life" simplified to each opponent; controller gains 1.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies,
                effect: Effect::Sequence(vec![
                    Effect::LoseLife {
                        player: PlayerTarget::EachOpponent,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
            },
        ],
    }
}
