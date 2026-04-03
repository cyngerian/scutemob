// Growth Spiral — {G}{U}, Instant
// Draw a card. You may put a land card from your hand onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("growth-spiral"),
        name: "Growth Spiral".to_string(),
        mana_cost: Some(ManaCost { green: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw a card. You may put a land card from your hand onto the battlefield.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 608.2: Draw a card, then you may put a land from your hand onto the battlefield.
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                Effect::PutLandFromHandOntoBattlefield { tapped: false },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
