// Wrenn and Realmbreaker — {2}{G}{G}{G}, Legendary Planeswalker — Wrenn
// Lands you control have "{T}: Add one mana of any color."
// +1: Up to two target lands become 3/3 Elemental creatures with trample, haste, and indestructible
//     until end of turn. They're still lands.
// −2: Return target permanent card from your graveyard to your hand.
// −7: You get an emblem with "You may play lands and cast permanent spells from your graveyard."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wrenn-and-realmbreaker"),
        name: "Wrenn and Realmbreaker".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 3,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Wrenn"],
        ),
        oracle_text: "Lands you control have \"{T}: Add one mana of any color.\"\n+1: Up to two target lands become 3/3 Elemental creatures with trample, haste, and indestructible until end of turn. They're still lands.\n\u{2212}2: Return target permanent card from your graveyard to your hand.\n\u{2212}7: You get an emblem with \"You may play lands and cast permanent spells from your graveyard.\"".to_string(),
        abilities: vec![
            // Static: "Lands you control have '{T}: Add one mana of any color.'"
            // TODO: Granting arbitrary mana abilities to permanents is a complex DSL
            // pattern (AnyColor mana production from lands). Known DSL gap.

            // +1: Up to two target lands become 3/3 Elemental creatures until EOT.
            // TODO: Complex type-changing + P/T-setting continuous effect for this turn.
            // Known DSL gap for loyalty ability effects.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![]),
                targets: vec![
                    TargetRequirement::TargetLand,
                    TargetRequirement::TargetLand,
                ],
            },
            // −2: Return target permanent card from your graveyard to your hand.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    controller_override: None,
                },
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    ..Default::default()
                })],
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
        starting_loyalty: Some(7),
        meld_pair: None,
        ..Default::default()
    }
}
