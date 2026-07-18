// Culling the Weak — {B}, Instant
// As an additional cost to cast this spell, sacrifice a creature.
// Add {B}{B}{B}{B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("culling-the-weak"),
        name: "Culling the Weak".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a creature.\nAdd \
                      {B}{B}{B}{B}."
            .to_string(),
        // CR 118.8: Mandatory sacrifice of a creature as additional cost.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature],
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 4, 0, 0, 0),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
