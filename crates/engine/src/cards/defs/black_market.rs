// Black Market — {3}{B}{B}, Enchantment
// Whenever a creature dies, put a charge counter on this enchantment.
// At the beginning of your first main phase, add {B} for each charge counter on this
// enchantment.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("black-market"),
        name: "Black Market".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature dies, put a charge counter on Black Market.\nAt the beginning of your first main phase, add {B} for each charge counter on Black Market.".to_string(),
        abilities: vec![
            // "Whenever a creature dies" = any creature, no filter needed.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: None },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Charge,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: DSL gap — "At the beginning of your first main phase, add {B} for each
            // charge counter." Needs main phase trigger + counter-scaled mana production.
        ],
        ..Default::default()
    }
}
