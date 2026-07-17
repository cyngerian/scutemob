// Torment of Hailfire — {X}{B}{B}, Sorcery
// Repeat the following process X times. Each opponent loses 3 life unless that player
// sacrifices a nonland permanent of their choice or discards a card.
//
// TODO: Complex repeated choice — each opponent independently chooses to sacrifice/discard/
//   lose 3 life, X times. Needs player choice infrastructure (M10) and repeat-N-times effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("torment-of-hailfire"),
        name: "Torment of Hailfire".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Repeat the following process X times. Each opponent loses 3 life unless \
                      that player sacrifices a nonland permanent of their choice or discards a \
                      card."
            .to_string(),
        abilities: vec![],
        completeness: Completeness::inert(
            "Effect::Repeat with XValue covers 'repeat X times'. Still blocked: 'unless that \
             player sacrifices a nonland permanent OR discards a card' is a two-option cost \
             election — MayPayOrElse takes a single Cost and its resolution (effects/mod.rs:3196) \
             always applies or_else without ever offering payment. Needs interactive choice \
             (M10+) plus an OR-cost variant.",
        ),
        ..Default::default()
    }
}
