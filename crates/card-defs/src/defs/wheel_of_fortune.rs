// Wheel of Fortune — {2}{R}, Sorcery
// Each player discards their hand, then draws seven cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wheel-of-fortune"),
        name: "Wheel of Fortune".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player discards their hand, then draws seven cards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.9 / 121.1: each player discards their entire hand, then draws seven.
            effect: Effect::WheelHand {
                player: PlayerTarget::EachPlayer,
                disposal: WheelDisposal::Discard,
                draw: WheelDraw::Fixed(7),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
