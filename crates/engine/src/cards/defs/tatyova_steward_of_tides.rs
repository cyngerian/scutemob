// Tatyova, Steward of Tides — {G}{G}{U}, Legendary Creature — Merfolk Druid 3/3
// Land creatures you control have flying; Landfall (7+ lands): animate target land 3/3 Elemental haste
// TODO: grant flying to land-creatures (continuous effect with card type filter) and landfall animate-land not in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tatyova-steward-of-tides"),
        name: "Tatyova, Steward of Tides".to_string(),
        mana_cost: Some(ManaCost {
            green: 2,
            blue: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Merfolk", "Druid"],
        ),
        oracle_text: "Land creatures you control have flying.\nWhenever a land you control enters, if you control seven or more lands, up to one target land you control becomes a 3/3 Elemental creature with haste. It's still a land.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: Continuous effect granting flying to land creatures requires a filter on
            // card types (Land + Creature) which is not expressible as an EffectFilter.
            // CR 613.1d/f: Landfall — Whenever a land enters, if you control 7+ lands,
            // target land becomes a 3/3 Elemental creature with haste until end of turn.
            // (Approximation: "7+ lands" → ControlAtLeastNOtherLands(6) intervening-if)
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddCardTypes(
                                [CardType::Creature].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
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
                            duration: EffectDuration::UntilEndOfTurn,
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
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [KeywordAbility::Haste].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                intervening_if: Some(Condition::ControlAtLeastNOtherLands(6)),
                targets: vec![TargetRequirement::TargetLand],
            },
        ],
        ..Default::default()
    }
}
