// Fervent Charge — {1}{R}{W}{B}, Enchantment
// Whenever a creature you control attacks, it gets +2/+2 until end of turn.
//
// PB-EF4: the continuous P/T grant targets the attacking creature via
// EffectFilter::TriggeringCreature (CR 611.2a). No filter restriction on the trigger — any
// attacking creature you control qualifies.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fervent-charge"),
        name: "Fervent Charge".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            white: 1,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, it gets +2/+2 until end of turn."
            .to_string(),
        abilities: vec![
            // CR 508.1m / CR 611.2a: "Whenever a creature you control attacks, it gets
            // +2/+2 until end of turn." EffectFilter::TriggeringCreature aims the grant at
            // the attacking creature.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: None,
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(2),
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
