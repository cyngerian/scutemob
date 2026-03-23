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
            },
            // TODO: Remove a charge counter from this artifact: Add one mana of any color.
            //   (Cost enum lacks RemoveCounter variant — only Effect::RemoveCounter exists)

        ],
        ..Default::default()
    }
}
