// Ponder — {U} Sorcery
// Look at the top three cards of your library, then put them back in any order.
// You may shuffle.
// Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ponder"),
        name: "Ponder".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Look at the top three cards of your library, then put them back in any order. You may shuffle.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "Look at top 3, put back in any order, may shuffle" not in DSL.
            // Implementing the draw only.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
