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
            // +1: Draw (conditional discard simplified)
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
            },
            // −2: Create 1/1 unblockable Ninja
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Ninja".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Ninja".to_string())].into_iter().collect(),
                        colors: [Color::Blue].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                targets: vec![],
            },
            // −7: Emblem — too complex for DSL.
        ],
        ..Default::default()
    }
}
