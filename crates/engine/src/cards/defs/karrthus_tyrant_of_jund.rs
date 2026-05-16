// Karrthus, Tyrant of Jund — {4}{B}{R}{G}, Legendary Creature — Dragon 7/7
// Flying, haste
// When Karrthus enters, gain control of all Dragons, then untap all Dragons.
// Other Dragon creatures you control have haste.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("karrthus-tyrant-of-jund"),
        name: "Karrthus, Tyrant of Jund".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying, haste\nWhen Karrthus enters, gain control of all Dragons, then untap all Dragons.\nOther Dragon creatures you control have haste.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 613.1f / CR 701.10a: "When Karrthus enters, gain control of all Dragons,
            // then untap all Dragons." — ETB triggered ability. Gain control is indefinite
            // (no stated duration). Untap follows immediately (Sequence).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::GainControl {
                        target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter {
                            has_subtype: Some(SubType("Dragon".to_string())),
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        })),
                        duration: EffectDuration::Indefinite,
                    },
                    Effect::UntapPermanent {
                        target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter {
                            has_subtype: Some(SubType("Dragon".to_string())),
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        })),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 613.1f / Layer 6: "Other Dragon creatures you control have haste."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Dragon".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
