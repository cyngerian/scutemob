// Ugin, the Spirit Dragon — {8}, Legendary Planeswalker — Ugin (loyalty 7)
// +2: Ugin deals 3 damage to any target.
// −X: Exile each permanent with mana value X or less that's one or more colors.
// −10: You gain 7 life, draw seven cards, then put up to seven permanent cards
//      from your hand onto the battlefield.
//
// NOTE: −X requires X as a variable removed loyalty cost and filtering by mana value + colored.
// No such dynamic loyalty cost (Minus(X)) exists in the DSL. Omitted per W5 policy.
// NOTE: −10 "put up to seven permanent cards from your hand onto the battlefield" requires
// effect to iterate hand, present choices, and put multiple cards onto battlefield — not
// expressible in the DSL. Partially: GainLife + DrawCards are implementable but the hand
// put-onto-battlefield is not. Omitted per W5 (partial = wrong).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ugin-the-spirit-dragon"),
        name: "Ugin, the Spirit Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 8, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Ugin"],
        ),
        oracle_text: "+2: Ugin deals 3 damage to any target.\n\u{2212}X: Exile each permanent with mana value X or less that's one or more colors.\n\u{2212}10: You gain 7 life, draw seven cards, then put up to seven permanent cards from your hand onto the battlefield.".to_string(),
        starting_loyalty: Some(7),
        abilities: vec![
            // +2: Ugin deals 3 damage to any target.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(2),
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetAny],
            },
            // CR 606.6 / CR 107.3k: −X: Exile each permanent with mana value X or less that's
            // one or more colors. LoyaltyCost::MinusX is now wired and x_value passes through
            // the activate_loyalty_ability harness action.
            // TODO: "mana value X or less" filter — TargetFilter has no dynamic MV <= X predicate.
            // TODO: "one or more colors" filter — TargetFilter has no is-colored predicate.
            // Effect remains Nothing until these filters are added.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::MinusX,
                effect: Effect::Nothing,
                targets: vec![],
            },
            // −10: You gain 7 life, draw seven cards, then put up to seven permanent cards
            // from your hand onto the battlefield.
            // TODO: "Put up to seven permanent cards from your hand onto the battlefield"
            // requires interactive hand selection and zone-change effect for multiple cards.
            // Not expressible in the DSL. GainLife + DrawCards are implementable but the
            // put-onto-battlefield portion is not. Omitted to avoid wrong game state.
        ],
        ..Default::default()
    }
}
