// Falkenrath Noble — {3}{B}, Creature — Vampire Noble 2/2
// Flying
// Whenever this creature or another creature dies, target player loses 1 life and you
// gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("falkenrath-noble"),
        name: "Falkenrath Noble".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Noble"]),
        oracle_text: "Flying\nWhenever Falkenrath Noble or another creature dies, target player loses 1 life and you gain 1 life.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // "this creature or another creature dies" = any creature dies.
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
