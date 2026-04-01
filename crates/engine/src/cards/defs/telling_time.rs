// Telling Time — {1}{U}, Instant
// Look at the top three cards of your library. Put one of those cards into your
// hand, one on top of your library, and one on the bottom of your library.
//
// TODO: Interactive "choose 1 of 3" — M10 player choice. Approximated as DrawCards(1).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("telling-time"),
        name: "Telling Time".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Look at the top three cards of your library. Put one of those cards into your hand, one on top of your library, and one on the bottom of your library.".to_string(),
        abilities: vec![
            // TODO: Interactive top-3 selection deferred to M10.
            // Approximated as DrawCards(1) — puts one card in hand.
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
