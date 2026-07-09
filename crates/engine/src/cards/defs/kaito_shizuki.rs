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
            // ENGINE-BLOCKED: "if Kaito Shizuki entered this turn, he phases out" — needs an
            // entered-this-turn condition on an end-step trigger plus a self-phase-out effect.
            //
            // +1 is fully expressible as of PB-AC6 (Condition::YouAttackedThisTurn, CR 508.1):
            // the discard is mandatory unless the raid condition is met.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::Conditional {
                        condition: Condition::YouAttackedThisTurn,
                        if_true: Box::new(Effect::Nothing),
                        if_false: Box::new(Effect::DiscardCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(1),
                        }),
                    },
                ]),
                targets: vec![],
            },
            // ENGINE-BLOCKED: −2 creates a 1/1 blue Ninja token with "This token can't be
            // blocked." TokenSpec carries keywords only, and unblockable is a static ability,
            // not a keyword. Left UNAUTHORED rather than declared as a LoyaltyAbility with
            // Effect::Nothing — that shape let a player pay 2 loyalty for no effect, which is
            // wrong game state.
            // ENGINE-BLOCKED: −7 emblem with combat damage → search library. Not expressible.
        ],
        ..Default::default()
    }
}
