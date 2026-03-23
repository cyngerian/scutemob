// Blood Artist — {1}{B}, Creature — Vampire 0/1
// Whenever this creature or another creature dies, target player loses 1 life and you
// gain 1 life.
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
            // "this creature or another creature dies" = any creature dies.
            // NOTE: oracle targets a single player but DrainLife drains all opponents.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: None },
                effect: Effect::DrainLife { amount: EffectAmount::Fixed(1) },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
