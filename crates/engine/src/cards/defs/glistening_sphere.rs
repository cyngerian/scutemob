// Glistening Sphere
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glistening-sphere"),
        name: "Glistening Sphere".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "This artifact enters tapped.\nWhen this artifact enters, proliferate.\n{T}: Add one mana of any color.\nCorrupted — {T}: Add three mana of any one color. Activate only if an opponent has three or more poison counters.".to_string(),
        abilities: vec![
            // This artifact enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // When this artifact enters, proliferate.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // Corrupted — {T}: Add three mana of any one color. Activate only if an opponent has 3+ poison counters.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaChoice { player: PlayerTarget::Controller, count: EffectAmount::Fixed(3) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::OpponentHasPoisonCounters(3)),
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
