// Winds of Change — {R}, Sorcery
// Each player shuffles the cards from their hand into their library, then draws
// that many cards.
//
// TODO: "shuffle hand into library, draw that many" — wheel effect. No Effect to
// shuffle hand into library exists. Approximated as Effect::Nothing.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("winds-of-change"),
        name: "Winds of Change".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player shuffles the cards from their hand into their library, then draws that many cards.".to_string(),
        abilities: vec![
            // TODO: Wheel effect — "each player shuffles hand into library, draws that many."
            // Needs Effect::ShuffleHandIntoLibrary + DrawCards(hand_size) for each player.
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
