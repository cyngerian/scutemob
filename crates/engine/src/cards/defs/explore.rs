// Explore — {1}{G}, Sorcery
// You may play an additional land this turn. Draw a card.
//
// TODO: "Additional land this turn" one-shot effect not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("explore"),
        name: "Explore".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You may play an additional land this turn.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Additional land play not expressible. Draw only.
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
