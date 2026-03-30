// Gemstone Array — {4} Artifact
// {2}: Put a charge counter on this artifact.
// Remove a charge counter from this artifact: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gemstone-array"),
        name: "Gemstone Array".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{2}: Put a charge counter on this artifact.\nRemove a charge counter from this artifact: Add one mana of any color.".to_string(),
        abilities: vec![
            // {2}: Put a charge counter on this artifact.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Charge,
                    count: 1,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // CR 602.2: Remove a charge counter: Add one mana of any color.
            // Note: Technically a mana ability (CR 605.1) but implemented as regular
            // activated ability for this batch. Mana-ability classification deferred to PB-37.
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
        ..Default::default()
    }
}
