// Corrupted Conviction — {B} Instant
// As an additional cost to cast this spell, sacrifice a creature.
// Draw two cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("corrupted-conviction"),
        name: "Corrupted Conviction".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a creature.\nDraw two cards.".to_string(),
        // CR 118.8: Mandatory sacrifice of a creature as additional cost.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature],
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
