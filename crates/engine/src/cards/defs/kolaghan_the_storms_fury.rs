// Kolaghan, the Storm's Fury — {3}{B}{R}, Legendary Creature — Dragon 4/5
// Flying
// Whenever a Dragon you control attacks, creatures you control get +1/+0 until end of turn.
// Dash {3}{B}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kolaghan-the-storms-fury"),
        name: "Kolaghan, the Storm's Fury".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever a Dragon you control attacks, creatures you control get +1/+0 until end of turn.\nDash {3}{B}{R}".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 508.1m: "Whenever a Dragon you control attacks, creatures you control get +1/+0."
            // PB-23: WheneverCreatureYouControlAttacks.
            // TODO: Dragon subtype filter not yet in DSL — over-triggers on non-Dragon attackers.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(1),
                        filter: EffectFilter::CreaturesYouControl,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Keyword(KeywordAbility::Dash),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Dash,
                cost: ManaCost { generic: 3, black: 1, red: 1, ..Default::default() },
                details: None,
            },
        ],
        ..Default::default()
    }
}
