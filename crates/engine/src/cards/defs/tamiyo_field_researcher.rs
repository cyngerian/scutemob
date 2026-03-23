// Tamiyo, Field Researcher — {1}{G}{W}{U}, Legendary Planeswalker — Tamiyo
// +1: Choose up to two target creatures. Until your next turn, whenever either of those
//     creatures deals combat damage, you draw a card.
// −2: Tap up to two target nonland permanents. They don't untap during their controller's
//     next untap step.
// −7: Draw three cards. You get an emblem with "You may cast spells from your hand
//     without paying their mana costs."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tamiyo-field-researcher"),
        name: "Tamiyo, Field Researcher".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, white: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Tamiyo"],
        ),
        oracle_text: "+1: Choose up to two target creatures. Until your next turn, whenever either of those creatures deals combat damage, you draw a card.\n\u{2212}2: Tap up to two target nonland permanents. They don't untap during their controller's next untap step.\n\u{2212}7: Draw three cards. You get an emblem with \"You may cast spells from your hand without paying their mana costs.\"".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +1: TODO: Grant combat-damage draw trigger to targets — not expressible.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // −2: Tap up to two nonland permanents + freeze
            // TODO: "Up to two targets" + "don't untap" not fully expressible.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // −7: Draw 3 + emblem
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(7),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                // TODO: Emblem "cast spells without paying mana costs" not expressible.
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
