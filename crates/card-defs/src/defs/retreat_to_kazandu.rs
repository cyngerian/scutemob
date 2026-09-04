// Retreat to Kazandu — {2}{G}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • Put a +1/+1 counter on target creature.
// • You gain 2 life.
//
// CR 700.2b / PB-35: Modal triggered ability. Bot fallback: mode 0 (+1/+1 counter).
//
// PB-DX35 (2026-09, OOS-DX4-2): the mode-0 target used to be declared FLAT, so it applied
// to BOTH modes -- with no creature on the battlefield the whole trigger was removed from
// the stack (CR 603.3d) and "You gain 2 life" (mode 1, which needs no target) was
// unreachable. The target now lives in `mode_targets`, scoped to mode 0 alone;
// `trigger_modal_plan` (`rules/abilities.rs`) picks the first CR 700.2b-legal mode, so with
// no creature it now correctly falls through to mode 1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("retreat-to-kazandu"),
        name: "Retreat to Kazandu".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n• Put a +1/+1 \
                      counter on target creature.\n• You gain 2 life."
            .to_string(),
        abilities: vec![
            // CR 700.2b / PB-35: Landfall modal triggered ability.
            // Mode 0: Put a +1/+1 counter on target creature.
            // Mode 1: You gain 2 life.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                // Main effect is a placeholder; modal resolution uses modes field.
                effect: Effect::Nothing,
                intervening_if: None,
                // PB-DX35 (OOS-DX4-2): the mode-0 target now lives in `mode_targets`
                // below, scoped to mode 0 alone -- mode 1 (You gain 2 life) needs no
                // target at all.
                targets: vec![],
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
                    // PB-DX35 (CR 700.2c/700.2f / OOS-DX4-2): mode 0's target creature
                    // requirement scoped to mode 0 alone; mode 1 needs no target. Mode
                    // 0's `EffectTarget::DeclaredTarget { index: 0 }` above already
                    // reads slot 0 of ITS OWN mode's slice, so no index change was
                    // needed here.
                    mode_targets: Some(vec![vec![TargetRequirement::TargetCreature], vec![]]),
                }),
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
