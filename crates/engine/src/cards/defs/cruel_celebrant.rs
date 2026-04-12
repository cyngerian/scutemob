// Cruel Celebrant — {W}{B}, Creature — Vampire 1/2
// Whenever this creature or another creature or planeswalker you control dies, each
// opponent loses 1 life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cruel-celebrant"),
        name: "Cruel Celebrant".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever Cruel Celebrant or another creature or planeswalker you control dies, each opponent loses 1 life and you gain 1 life.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // CR 603.10a: "Whenever Cruel Celebrant or another creature or planeswalker
            // you control dies, each opponent loses 1 life and you gain 1 life."
            // Note: "or planeswalker" portion not covered — WheneverCreatureDies only
            // fires on creature deaths, not planeswalker deaths. Known DSL gap.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                                filter: None,
            },
                effect: Effect::DrainLife {
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
