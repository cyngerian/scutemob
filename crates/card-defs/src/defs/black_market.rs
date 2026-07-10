// Black Market — {3}{B}{B}, Enchantment
// Whenever a creature dies, put a charge counter on this enchantment.
// At the beginning of your first main phase, add {B} for each charge counter on this
// enchantment.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("black-market"),
        name: "Black Market".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature dies, put a charge counter on Black Market.\nAt the beginning of your first main phase, add {B} for each charge counter on Black Market.".to_string(),
        abilities: vec![
            // "Whenever a creature dies" = any creature, no filter needed.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: None, exclude_self: false, nontoken_only: false, filter: None,
},
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Charge,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // PB-AC6: "At the beginning of your first main phase, add {B} for each charge
            // counter on Black Market." TriggerCondition::AtBeginningOfFirstMainPhase fires
            // once per turn on Step::PreCombatMain for the active player only (CR 505.1a).
            // Effect::AddManaScaled with EffectAmount::CounterCount reads the charge-counter
            // count on the source at resolution time.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfFirstMainPhase,
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Black,
                    count: EffectAmount::CounterCount {
                        target: EffectTarget::Source,
                        counter: CounterType::Charge,
                    },
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
