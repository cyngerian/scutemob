// Druids' Repository — {1}{G}{G}, Enchantment
// Whenever a creature you control attacks, put a charge counter on this enchantment.
// Remove a charge counter from this enchantment: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("druids-repository"),
        name: "Druids' Repository".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, put a charge counter on this enchantment.\nRemove a charge counter from this enchantment: Add one mana of any color.".to_string(),
        abilities: vec![
            // CR 508.1m: "Whenever a creature you control attacks, put a charge counter on this."
            // PB-23: WheneverCreatureYouControlAttacks.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Charge,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 602.2: Remove a charge counter: Add one mana of any color.
            // Note: Technically a mana ability (CR 605.1). Implemented as regular activated
            // ability for this batch. Mana-ability classification deferred to PB-37.
            AbilityDefinition::Activated {
                cost: Cost::RemoveCounter { counter: CounterType::Charge, count: 1 },
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
