// Beastmaster Ascension — {2}{G}, Enchantment
// Whenever a creature you control attacks, you may put a quest counter on this enchantment.
// As long as this enchantment has seven or more quest counters on it, creatures you control
// get +5/+5.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("beastmaster-ascension"),
        name: "Beastmaster Ascension".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, you may put a quest counter on this enchantment.\nAs long as this enchantment has seven or more quest counters on it, creatures you control get +5/+5.".to_string(),
        abilities: vec![
            // CR 508.1m: "Whenever a creature you control attacks, put a quest counter on this."
            // PB-23: WheneverCreatureYouControlAttacks.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Custom("quest".to_string()),
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "As long as this has 7+ quest counters, creatures you control get +5/+5."
            // Condition-gated static ability not expressible in current DSL.
        ],
        ..Default::default()
    }
}
