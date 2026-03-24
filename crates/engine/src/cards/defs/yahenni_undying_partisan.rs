// Yahenni, Undying Partisan — {2}{B}, Legendary Creature — Aetherborn Vampire 2/2
// Haste
// Whenever a creature an opponent controls dies, put a +1/+1 counter on Yahenni.
// Sacrifice another creature: Yahenni gains indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("yahenni-undying-partisan"),
        name: "Yahenni, Undying Partisan".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Aetherborn", "Vampire"],
        ),
        oracle_text: "Haste\nWhenever a creature an opponent controls dies, put a +1/+1 counter on Yahenni.\nSacrifice another creature: Yahenni gains indestructible until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 603.10a: "Whenever a creature an opponent controls dies, put a +1/+1
            // counter on Yahenni." — controller_opponent filter on WheneverCreatureDies.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::Opponent),
                    exclude_self: false,
                    nontoken_only: false,
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Indestructible),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
