// Growth Spiral — {G}{U}, Instant
// Draw a card. You may put a land card from your hand onto the battlefield.
//
// TODO: "Put a land from hand onto battlefield" not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("growth-spiral"),
        name: "Growth Spiral".to_string(),
        mana_cost: Some(ManaCost { green: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw a card. You may put a land card from your hand onto the battlefield.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            // TODO: Put land from hand not expressible.
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
