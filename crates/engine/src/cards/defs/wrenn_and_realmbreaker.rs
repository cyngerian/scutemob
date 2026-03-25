// Wrenn and Realmbreaker — {1}{G}{G}, Legendary Planeswalker — Wrenn
// Oracle text (Scryfall verified):
// Lands you control have "{T}: Add one mana of any color."
// +1: Up to one target land you control becomes a 3/3 Elemental creature with vigilance,
//     hexproof, and haste until your next turn. It's still a land.
// −2: Mill three cards. You may put a permanent card from among the milled cards into your hand.
// −7: You get an emblem with "You may play lands and cast permanent spells from your graveyard."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wrenn-and-realmbreaker"),
        name: "Wrenn and Realmbreaker".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Wrenn"],
        ),
        oracle_text: "Lands you control have \"{T}: Add one mana of any color.\"\n+1: Up to one target land you control becomes a 3/3 Elemental creature with vigilance, hexproof, and haste until your next turn. It's still a land.\n\u{2212}2: Mill three cards. You may put a permanent card from among the milled cards into your hand.\n\u{2212}7: You get an emblem with \"You may play lands and cast permanent spells from your graveyard.\"".to_string(),
        abilities: vec![
            // Static: "Lands you control have '{T}: Add one mana of any color.'"
            // TODO: Granting arbitrary mana abilities to permanents is a complex DSL
            // pattern (AnyColor mana production from lands). Known DSL gap.

            // +1: Up to one target land you control becomes a 3/3 Elemental creature with
            // vigilance, hexproof, and haste until your next turn. It's still a land.
            // TODO: Type-changing (add Elemental creature type) + P/T-setting continuous
            // effect + keyword grants (vigilance, hexproof, haste) until next turn is a
            // multi-layer continuous effect. Known DSL gap for loyalty ability effects.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![]),
                targets: vec![TargetRequirement::TargetLand],
            },
            // −2: Mill three cards. You may put a permanent card from among the milled
            // cards into your hand.
            // TODO: Mill + conditional return from milled cards requires tracking which
            // cards were milled and offering a player choice from among them. Known DSL gap.
            // The MoveZone approximation below does not match the oracle text — the -2
            // mills first, then allows putting a permanent card from among the milled cards
            // into hand (not a target from the existing graveyard). Placeholder only.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
            // −7: You get an emblem with "You may play lands and cast permanent spells
            // from your graveyard." (CR 114.1-114.4)
            // TODO: "Play lands from graveyard" and "cast permanent spells from graveyard"
            // are game-rules-modifying static abilities that require changes to legal_actions.rs
            // to scan the graveyard for castable cards. This is a separate primitive gap
            // beyond emblem infrastructure — similar to Crucible of Worlds / Yawgmoth's Will.
            // The emblem is created correctly; the rule modification is deferred.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(7),
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![],
                    static_effects: vec![],
                    // TODO: Static rule-modifying effect for "play lands/permanents from graveyard".
                    // This requires a new DSL primitive or a custom legal_actions.rs hook.
                },
                targets: vec![],
            },
        ],
        starting_loyalty: Some(4),
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
        ..Default::default()
    }
}
