// Dark Ritual — {B}, Instant
// Add {B}{B}{B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dark-ritual"),
        name: "Dark Ritual".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Add {B}{B}{B}.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: ManaPool { black: 3, ..Default::default() },
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
