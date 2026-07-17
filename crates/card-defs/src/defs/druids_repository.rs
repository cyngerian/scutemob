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
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks { filter: None },
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
                once_per_turn: false,
            },
        ],
        completeness: Completeness::known_wrong("CR 106.1b: 'Remove a charge counter: Add one mana of any color' adds one COLORLESS mana (probed with 5 charge counters: +1 colorless). Also a CR 605.1a/605.3b violation — it is a mana ability but registers as a stack-using activated ability, because mana_ability_cost_components refuses Cost::RemoveCounter (ManaAbility has no counter-cost field and handle_tap_for_mana has no counter payment path). Note this ability has NO tap component at all, and every try_as_tap_mana_ability return site hardcodes requires_tap: true — the requires_tap: false path is unexercised in the whole corpus. The color bug is the reason for known_wrong rather than partial."),
        ..Default::default()
    }
}
