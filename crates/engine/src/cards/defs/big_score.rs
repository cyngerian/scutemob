// Big Score — {3}{R}, Instant
// As an additional cost to cast this spell, discard a card.
// Draw two cards and create two Treasure tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("big-score"),
        name: "Big Score".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, discard a card.\nDraw two cards and create two Treasure tokens.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Discard additional cost not expressible. Draw + treasure.
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                Effect::CreateToken {
                    spec: treasure_token_spec(2),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
