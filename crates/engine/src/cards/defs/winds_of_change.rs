// Winds of Change — {R}, Sorcery
// Each player shuffles the cards from their hand into their library, then draws
// that many cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("winds-of-change"),
        name: "Winds of Change".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player shuffles the cards from their hand into their library, then draws that many cards.".to_string(),
        abilities: vec![
            // CR 701.24 / 121.1: each player shuffles their hand into their library, then
            // draws that many cards. `Effect::WheelHand` snapshots the hand size before
            // the shuffle-in.
            AbilityDefinition::Spell {
                effect: Effect::WheelHand {
                    player: PlayerTarget::EachPlayer,
                    disposal: WheelDisposal::ShuffleHandIntoLibrary,
                    draw: WheelDraw::ThatMany,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
