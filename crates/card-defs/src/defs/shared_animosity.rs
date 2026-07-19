// Shared Animosity — {2}{R}, Enchantment
// Whenever a creature you control attacks, it gets +1/+0 until end of turn for each
// other attacking creature that shares a creature type with it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shared-animosity"),
        name: "Shared Animosity".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, it gets +1/+0 until end of turn \
                      for each other attacking creature that shares a creature type with it."
            .to_string(),
        abilities: vec![
            // CR 508.1m / CR 205.3m / CR 613.1d / CR 611.2a: "Whenever a creature you control
            // attacks, it gets +1/+0 until end of turn for each other attacking creature that
            // shares a creature type with it." PB-OS5 (OOS-EF4-1) closes the count-amount gap:
            // EffectAmount::OtherAttackersSharingCreatureType counts OTHER attacking creatures
            // (any controller — ruling 2008-04-01) whose layer-resolved creature-type set
            // intersects the triggering creature's. relative_to: TriggeringCreature both
            // scopes the count and (via EffectFilter::TriggeringCreature below) aims the pump.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: None,
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPowerDynamic {
                            amount: Box::new(EffectAmount::OtherAttackersSharingCreatureType {
                                relative_to: EffectTarget::TriggeringCreature,
                            }),
                            negate: false,
                        },
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
        ],
        ..Default::default()
    }
}
