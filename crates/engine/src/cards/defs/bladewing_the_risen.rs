// Bladewing the Risen — {3}{B}{B}{R}{R}, Legendary Creature — Zombie Dragon 4/4
// Flying
// When Bladewing enters, you may return target Dragon permanent card from your
// graveyard to the battlefield.
// {B}{R}: Dragon creatures get +1/+1 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bladewing-the-risen"),
        name: "Bladewing the Risen".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 2,
            red: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Zombie", "Dragon"],
        ),
        oracle_text: "Flying\nWhen Bladewing enters, you may return target Dragon permanent card from your graveyard to the battlefield.\n{B}{R}: Dragon creatures get +1/+1 until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.1: ETB trigger — return target Dragon permanent from your GY to BF.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_subtypes: vec![SubType("Dragon".to_string())],
                    // "permanent card" = has at least one permanent card type (CR 110.4a).
                    has_card_types: vec![
                        CardType::Creature,
                        CardType::Artifact,
                        CardType::Enchantment,
                        CardType::Land,
                        CardType::Planeswalker,
                    ],
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
            // CR 613.4c: "{B}{R}: Dragon creatures get +1/+1 until end of turn."
            // AllCreaturesWithSubtype — no controller restriction, affects all players' Dragons.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { black: 1, red: 1, ..Default::default() }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(1),
                        filter: EffectFilter::AllCreaturesWithSubtype(
                            SubType("Dragon".to_string()),
                        ),
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
