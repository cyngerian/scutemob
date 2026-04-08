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

            // CR 613.1d/613.4b / CR 611.2b: +1: Target land becomes 3/3 Elemental with vigilance,
            // hexproof, and haste until your next turn. PlayerId(0) resolves to the controller
            // at execution time (same pattern as lightning_army_of_one, teferi_time_raveler).
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddCardTypes(
                                [CardType::Creature].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilYourNextTurn(PlayerId(0)),
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddSubtypes(
                                [SubType("Elemental".to_string())].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilYourNextTurn(PlayerId(0)),
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtSet,
                            modification: LayerModification::SetPowerToughness {
                                power: 3,
                                toughness: 3,
                            },
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilYourNextTurn(PlayerId(0)),
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [
                                    KeywordAbility::Vigilance,
                                    KeywordAbility::Hexproof,
                                    KeywordAbility::Haste,
                                ]
                                .into_iter()
                                .collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilYourNextTurn(PlayerId(0)),
                            condition: None,
                        }),
                    },
                ]),
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
            // from your graveyard." (CR 114.1-114.4, CR 601.3, CR 305.1)
            // PB-B: PlayFromGraveyardPermission with PermanentsAndLands filter.
            // The emblem is the permission source; since emblems never leave the command zone,
            // this permission is permanent for the rest of the game.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(7),
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![],
                    static_effects: vec![],
                    play_from_graveyard: Some(PlayFromTopFilter::PermanentsAndLands),
                },
                targets: vec![],
            },
        ],
        starting_loyalty: Some(4),
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        ..Default::default()
    }
}
