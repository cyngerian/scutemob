// Dragon Tempest — {1}{R}, Enchantment
// Whenever a creature you control with flying enters, it gains haste until end of turn.
// Whenever a Dragon you control enters, it deals X damage to any target, where X is the
// number of Dragons you control.
//
// PB-EF4: both clauses target "it" = the entering creature, never Dragon Tempest itself.
// (1) "it gains haste" uses EffectFilter::TriggeringCreature on ContinuousEffectDef.filter
// to grant Haste to the entering flyer until end of turn (CR 611.2a). (2) "it deals X
// damage" sources the damage from the entering Dragon via
// Effect::DealDamage.source: Some(EffectTarget::TriggeringCreature) (CR 119.3). X counts
// all Dragons you control, including the just-entered one (PermanentCount, no
// exclude_self — the entering Dragon is already on the battlefield when the trigger
// resolves).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragon-tempest"),
        name: "Dragon Tempest".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control with flying enters, it gains haste until \
                      end of turn.\nWhenever a Dragon you control enters, it deals X damage to \
                      any target, where X is the number of Dragons you control."
            .to_string(),
        abilities: vec![
            // CR 603.6a / CR 611.2a: "Whenever a creature you control with flying enters, it
            // gains haste until end of turn." EffectFilter::TriggeringCreature aims the
            // continuous haste grant at the entering flyer.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_keywords: [KeywordAbility::Flying].into_iter().collect(),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                        filter: EffectFilter::TriggeringCreature,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 603.6a / CR 119.3: "Whenever a Dragon you control enters, it deals X damage
            // to any target, where X is the number of Dragons you control." The entering
            // Dragon is the damage source (Some(TriggeringCreature)); X counts all Dragons
            // you control including the entering one (no exclude_self on the count filter).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::DealDamage {
                    source: Some(EffectTarget::TriggeringCreature),
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Dragon".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
