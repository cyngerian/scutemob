// Teferi, Master of Time — {2}{U}{U}, Legendary Planeswalker — Teferi
// You may activate loyalty abilities of Teferi on any player's turn any time
// you could cast an instant.
// +1: Draw a card, then discard a card.
// −3: Target creature you don't control phases out.
// −10: Take two extra turns after this one.
//
// TODO: "Activate on any player's turn" — instant-speed loyalty not in DSL.
// TODO: "Discard a card" after draw — WheneverYouDiscard gap. Using draw-only.
// TODO: "Take two extra turns" — extra turn effect not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferi-master-of-time"),
        name: "Teferi, Master of Time".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Teferi"],
        ),
        oracle_text: "You may activate loyalty abilities of Teferi, Master of Time on any player's turn any time you could cast an instant.\n+1: Draw a card, then discard a card.\n\u{2212}3: Target creature you don't control phases out.\n\u{2212}10: Take two extra turns after this one.".to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // +1: Draw a card, then discard a card.
            // TODO: "then discard a card" — forced discard on self not easily expressible.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
            },
            // −3: Target creature you don't control phases out.
            // TODO: Phase-out target effect — no Effect::PhaseOut variant.
            // −10: Take two extra turns.
            // TODO: Extra turn effect not in DSL.
        ],
        ..Default::default()
    }
}
