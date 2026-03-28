// Retreat to Kazandu — {2}{G}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • Put a +1/+1 counter on target creature.
// • You gain 2 life.
//
// CR 700.2b / PB-35: Modal triggered ability. Bot fallback: mode 0 (+1/+1 counter).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("retreat-to-kazandu"),
        name: "Retreat to Kazandu".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n• Put a +1/+1 counter on target creature.\n• You gain 2 life.".to_string(),
        abilities: vec![
            // CR 700.2b / PB-35: Landfall modal triggered ability.
            // Mode 0: Put a +1/+1 counter on target creature.
            // Mode 1: You gain 2 life.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                // Main effect is a placeholder; modal resolution uses modes field.
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![
                    // Mode 0 target: target creature (auto-selected by bot).
                    TargetRequirement::TargetCreature,
                ],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: Put a +1/+1 counter on target creature.
                        Effect::AddCounter {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            counter: CounterType::PlusOnePlusOne,
                            count: 1,
                        },
                        // Mode 1: You gain 2 life.
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                    ],
                    allow_duplicate_modes: false,
                    mode_costs: None,
                }),
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
