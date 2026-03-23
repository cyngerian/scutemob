// Chandra, Flamecaller — {4}{R}{R}, Legendary Planeswalker — Chandra
// +1: Create two 3/1 red Elemental creature tokens with haste. Exile them at the beginning
//     of the next end step.
// 0: Discard all the cards in your hand, then draw that many cards plus one.
// −X: Chandra deals X damage to each creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chandra-flamecaller"),
        name: "Chandra, Flamecaller".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Chandra"],
        ),
        oracle_text: "+1: Create two 3/1 red Elemental creature tokens with haste. Exile them at the beginning of the next end step.\n0: Discard all the cards in your hand, then draw that many cards plus one.\n\u{2212}X: Chandra deals X damage to each creature.".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +1: Create tokens + delayed exile at next end step
            // TODO: Token creation with delayed exile trigger not expressible in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // 0: Discard hand then draw that many + 1
            // TODO: "Discard all cards then draw that many plus one" — EffectAmount::HandSize
            // not in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::Nothing,
                targets: vec![],
            },
            // −X: Deal X damage to each creature
            // TODO: LoyaltyCost::MinusX not in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
