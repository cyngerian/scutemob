// Creeping Bloodsucker — {1}{B}, Creature — Vampire 1/2
// At the beginning of your upkeep, this creature deals 1 damage to each opponent.
// You gain life equal to the damage dealt this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("creeping-bloodsucker"),
        name: "Creeping Bloodsucker".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "At the beginning of your upkeep, this creature deals 1 damage to each opponent. You gain life equal to the damage dealt this way.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // Upkeep trigger: drain 1 from each opponent.
            // DrainLife handles "deal damage to each opponent, gain life equal to damage dealt".
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::DrainLife { amount: EffectAmount::Fixed(1) },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
