// Footbottom Feast — {2}{B}, Instant
// Put any number of target creature cards from your graveyard on top of your library.
// Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("footbottom-feast"),
        name: "Footbottom Feast".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Put any number of target creature cards from your graveyard on top of your library.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Multi-target graveyard-to-library not expressible. Draw only.
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
