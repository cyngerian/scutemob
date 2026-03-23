// Kaito Shizuki — {1}{U}{B}, Legendary Planeswalker — Kaito
// At the beginning of your end step, if Kaito entered this turn, he phases out.
// +1: Draw a card. Then discard a card unless you attacked this turn.
// −2: Create a 1/1 blue Ninja creature token with "This token can't be blocked."
// −7: You get an emblem with "Whenever a creature you control deals combat damage to a
//     player, search your library for a blue or black creature card, put it onto the
//     battlefield, then shuffle."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kaito-shizuki"),
        name: "Kaito Shizuki".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Kaito"],
        ),
        oracle_text: "At the beginning of your end step, if Kaito Shizuki entered this turn, he phases out.\n+1: Draw a card. Then discard a card unless you attacked this turn.\n\u{2212}2: Create a 1/1 blue Ninja creature token with \"This token can't be blocked.\"\n\u{2212}7: You get an emblem with \"Whenever a creature you control deals combat damage to a player, search your library for a blue or black creature card, put it onto the battlefield, then shuffle.\"".to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // TODO: "If entered this turn, phases out" — conditional end-step phase-out.
            // TODO: +1 draw + conditional discard ("unless you attacked") not in DSL.
            //   Free unconditional draw is wrong game state.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // TODO: −2 Ninja token needs "can't be blocked" — TokenSpec lacks static
            //   abilities (only keywords). Unblockable is a static, not a keyword.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // TODO: −7 emblem with combat damage → search library. Not expressible.
        ],
        ..Default::default()
    }
}
