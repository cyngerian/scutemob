// Veil of Summer — {G}, Instant
// Draw a card if an opponent has cast a blue or black spell this turn.
// Spells you control can't be countered this turn.
// You and permanents you control gain hexproof from blue and from black until end of turn.
//
// TODO: Conditional draw ("if opponent cast blue/black this turn") — needs cast-tracking.
// TODO: "Spells you control can't be countered" — continuous effect on spells, not in DSL.
// TODO: "Hexproof from blue and from black" — color-specific hexproof not in DSL.
// Implementing the draw only (unconditional approximation).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("veil-of-summer"),
        name: "Veil of Summer".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw a card if an opponent has cast a blue or black spell this turn. Spells you control can't be countered this turn. You and permanents you control gain hexproof from blue and from black until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Should be conditional on opponent casting blue/black spell this turn.
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
