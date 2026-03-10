// Crop Rotation — {G} Instant; as an additional cost, sacrifice a land.
// Search your library for a land card, put it onto the battlefield, then shuffle.
// TODO: DSL gap — "sacrifice a land" as additional cost and "search for a land"
// both require targeted type-restricted effects. Only the spell shell is defined.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crop-rotation"),
        name: "Crop Rotation".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a land.\nSearch your library for a land card, put that card onto the battlefield, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(0),
            },
            // TODO: sacrifice a land (additional cost) + search library for a land
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
