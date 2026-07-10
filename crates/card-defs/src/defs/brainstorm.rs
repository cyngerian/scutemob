// 39. Brainstorm — {U}, Instant; draw 3, then put 2 cards from hand on top of library.
// (CR 701.20 "put on top": deterministic M7 — takes first 2 by ObjectId ascending.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brainstorm"),
        name: "Brainstorm".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw three cards, then put two cards from your hand on top of your library in any order.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                Effect::PutOnLibrary {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                    from: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
