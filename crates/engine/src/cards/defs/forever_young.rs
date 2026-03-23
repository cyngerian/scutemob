// Forever Young — {1}{B}, Sorcery
// Put any number of target creature cards from your graveyard on top of your library.
// Draw a card.
//
// TODO: "Put creature cards from graveyard on top of library" — multi-target
//   graveyard-to-library move. Implementing draw only.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forever-young"),
        name: "Forever Young".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Put any number of target creature cards from your graveyard on top of your library.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Graveyard-to-library multi-target not expressible. Draw only.
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
