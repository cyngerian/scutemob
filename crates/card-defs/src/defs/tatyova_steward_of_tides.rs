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
        oracle_text: "Land creatures you control have flying.\nWhenever a land you control \
                      enters, if you control seven or more lands, up to one target land you \
                      control becomes a 3/3 Elemental creature with haste. It's still a land."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: Continuous effect granting flying to land creatures requires a filter on
            // card types (Land + Creature) which is not expressible as an EffectFilter.
            // CR 613.1d/f: Landfall — Whenever a land enters, if you control 7+ lands,
            // target land becomes a 3/3 Elemental creature with haste until end of turn.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
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
                // CR 613.1d/f: "if you control seven or more lands." `ctx.source` is Tatyova
                // herself (a Merfolk Druid, not a land), so `ControlAtLeastNOtherLands`'s
                // source exclusion removes nothing from the land count — the correct argument
                // for a non-land source is the printed number itself, 7.
                intervening_if: Some(Condition::ControlAtLeastNOtherLands(7)),
                targets: vec![TargetRequirement::TargetLand],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "DSL gap — no EffectFilter for 'land creatures you control' (EffectFilter cannot \
             intersect card types), so the flying grant is unimplemented. The landfall trigger IS \
             implemented, but its target should be UpToN{1, TargetLandWithFilter(controller: \
             You)}, not bare TargetLand. PB-DX1 (2026-08-01): the 'you control seven or more \
             lands' intervening-if on this trigger is now actually evaluated at both CR 603.4 \
             checkpoints (previously silently dropped by the runtime lowering, so the animate \
             effect could fire regardless of land count); review Finding 5 additionally caught \
             the threshold itself reading 6 instead of 7 (ctx.source is Tatyova, a non-land, so \
             the 'other lands' exclusion removed nothing and the value must equal the printed \
             number) -- fixed to 7 in the same batch. Neither repair resolves either blocker \
             named above; marker stays partial.",
        ),
        ..Default::default()
    }
}
