// Explore — {1}{G}, Sorcery
// You may play an additional land this turn. Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("explore"),
        name: "Explore".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You may play an additional land this turn.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 305.2: Grant one additional land play this turn, then draw a card.
            effect: Effect::Sequence(vec![
                Effect::AdditionalLandPlay,
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
