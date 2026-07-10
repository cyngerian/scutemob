// Seething Song — {2}{R}, Instant
// Add {R}{R}{R}{R}{R}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("seething-song"),
        name: "Seething Song".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Add {R}{R}{R}{R}{R}.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: ManaPool { red: 5, ..Default::default() },
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
