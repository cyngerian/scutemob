// Brainsurge — {2}{U}, Instant (note: actually {3}{U})
// Draw four cards, then put two cards from your hand on top of your library in
// any order.
//
// TODO: "put two cards from hand on top of library" — interactive card selection
// deferred to M10. Approximated as DrawCards(4) only (net +2 is correct but
// library ordering is wrong).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brainsurge"),
        name: "Brainsurge".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw four cards, then put two cards from your hand on top of your library in any order.".to_string(),
        abilities: vec![
            // Draw 4, then put 2 back on top.
            // TODO: "put 2 cards from hand on top" — needs interactive card selection.
            // Approximated as DrawCards(4) — net card advantage is wrong (+4 vs +2).
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(4),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
