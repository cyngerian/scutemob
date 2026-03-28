// Everflowing Chalice — {0} Artifact, Multikicker {2}; enters with charge counters per kick;
// {T}: Add {C} for each charge counter.
// TODO: "This artifact enters with a charge counter on it for each time it was kicked." —
// ETB-place-counters based on Multikicker count is a DSL gap (no MultikickerCount EffectAmount
// variant for ETB counter placement).
// The tap ability uses EffectAmount::CounterCount which is supported.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("everflowing-chalice"),
        name: "Everflowing Chalice".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Multikicker {2} (You may pay an additional {2} any number of times as you cast this spell.)\nThis artifact enters with a charge counter on it for each time it was kicked.\n{T}: Add {C} for each charge counter on this artifact.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Kicker),
            AbilityDefinition::Kicker {
                cost: ManaCost { generic: 2, ..Default::default() },
                is_multikicker: true,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
