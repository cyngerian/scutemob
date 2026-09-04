// Retreat to Coralhelm — {2}{U}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • You may tap or untap target creature.
// • Scry 1.
//
// CR 700.2b / PB-35: Modal triggered ability. Bot fallback: mode 0 (untap target creature).
// Note: "tap or untap" is approximated as "untap" (mode 0). The bot always untaps.
//
// PB-DX35 (2026-09, OOS-DX4-2): the mode-0 target used to be declared FLAT, so it applied
// to BOTH modes -- with no creature on the battlefield "Scry 1" (mode 1, which needs no
// target) was unreachable, the whole trigger being removed from the stack instead
// (CR 603.3d). The target now lives in `mode_targets`, scoped to mode 0 alone;
// `trigger_modal_plan` (`rules/abilities.rs`) picks the first CR 700.2b-legal mode, so with
// no creature it now correctly falls through to mode 1. The unrelated tap/untap
// approximation this def's `known_wrong` marker names is NOT touched by this batch.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("retreat-to-coralhelm"),
        name: "Retreat to Coralhelm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n• You may tap \
                      or untap target creature.\n• Scry 1."
            .to_string(),
        abilities: vec![
            // CR 700.2b / PB-35: Landfall modal triggered ability.
            // Mode 0: You may tap or untap target creature (approximated as untap).
            // Mode 1: Scry 1.
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
                effect: Effect::Nothing,
                intervening_if: None,
                // PB-DX35 (OOS-DX4-2): the mode-0 target now lives in `mode_targets`
                // below, scoped to mode 0 alone -- mode 1 (Scry 1) needs no target
                // at all.
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: Untap target creature (approximation of "tap or untap").
                        //
                        // **A SECOND, unrecorded deviation lives on this mode and is filed as
                        // `OOS-DX35-7`** (found by PB-DX35's `/review`): the printed clause is
                        // "You **may** tap or untap target creature", so even the untap is
                        // optional — and once mode 0 is chosen this def performs it
                        // unconditionally. That is audit §5's DP-12 class (a costless "may")
                        // reached through a MODE rather than through an effect, so it is
                        // invisible to every axis keyed on `Effect::MayPayThenEffect` or on
                        // `LookAtTopThenPlace.optional`, PB-DX35's own included.
                        // `ModeSelection` has no per-mode optionality field. The marker below
                        // records only the tap/untap half; this comment records the other.
                        Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        },
                        // Mode 1: Scry 1.
                        Effect::Scry {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(1),
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
        // PB-DX35 (OOS-DX4-2): the mode-target half of this def is now correct (see the
        // module doc); this marker's blocker is the unrelated "tap or untap" approximated
        // as "untap" only, which this batch does not touch.
        completeness: Completeness::known_wrong(
            "'tap or untap' is modeled as 'untap' only — the tap mode is unavailable",
        ),
        ..Default::default()
    }
}
