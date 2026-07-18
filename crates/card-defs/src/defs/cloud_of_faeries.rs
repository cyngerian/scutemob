// Cloud of Faeries — {1}{U}, Creature — Faerie 1/1
// Flying
// When this creature enters, untap up to two lands.
// Cycling {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cloud-of-faeries"),
        name: "Cloud of Faeries".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Faerie"]),
        oracle_text: "Flying\nWhen this creature enters, untap up to two lands.\nCycling {2}"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 601.2c / card_definition.rs:2799-2822: UpToN contributes its declared
            // targets at consecutive indices starting where the prior requirement's
            // indices end. This is the only requirement here, so declared lands occupy
            // indices 0..2; an undeclared slot resolves to a no-op via
            // resolve_effect_target_list returning empty (CR 608.2b).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::UntapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::UntapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::UpToN {
                    count: 2,
                    inner: Box::new(TargetRequirement::TargetLand),
                }],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}
