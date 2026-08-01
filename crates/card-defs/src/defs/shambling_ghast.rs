// Shambling Ghast — {B}, Creature — Zombie 1/1.
// When this creature dies, choose one —
//   • Target creature an opponent controls gets -1/-1 until end of turn.
//   • Create a Treasure token.
//
// CR 700.2b / PB-35: Modal WhenDies triggered ability. Bot fallback: mode 0 (Treasure token).
//
// PB-DX4 (2026-08-01, OOS-DP10-8 triage): three oracle deviations corrected here. This def
// had (1) `KeywordAbility::Decayed`, which the printed card does not have AT ALL — MCP
// reports keywords `["Treasure"]` only, and Decayed (CR 702.147a) would have made the Ghast
// unable to block and self-sacrificing after any attack; (2) mode 1 as a PERMANENT
// `CounterType::MinusOneMinusOne` counter where the card says "-1/-1 **until end of turn**"
// — a permanent counter persists past cleanup, is proliferate-able, and pairs off against
// +1/+1 counters under CR 122.3; and (3) a stored `oracle_text` asserting a Decayed reminder
// and "When Shambling Ghast **enters**" against this def's own `TriggerCondition::WhenDies`.
// Mode order is left as authored (Treasure first) — a mode's identity in Magic is its text,
// not its index, so index order is an engine artifact and not an oracle deviation.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shambling-ghast"),
        name: "Shambling Ghast".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Zombie"]),
        oracle_text: "When this creature dies, choose one —\n• Target creature an opponent \
                      controls gets -1/-1 until end of turn.\n• Create a Treasure token. (It's an \
                      artifact with \"{T}, Sacrifice this token: Add one mana of any color.\")"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 700.2b / PB-35: Modal WhenDies trigger.
            // Mode 0: Create a Treasure token.
            // Mode 1: Target creature an opponent controls gets -1/-1 until end of turn.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::Nothing,
                intervening_if: None,
                // Mode 1 target: opponent's creature. Declared FLAT rather than per-mode.
                //
                // PB-DX4: `ModeSelection.mode_targets` (PB-AC4, the golgari_charm idiom) would
                // be the CR 601.2c-faithful encoding -- a target should be declared only for a
                // mode that was chosen. It is deliberately NOT used here: every consumer of
                // `mode_targets` lives on the CASTING path (`rules/casting.rs`, plus the
                // read-only `rules/queries.rs`), and nothing on the triggered-ability path
                // reads it, so moving these targets into `mode_targets` would silently drop
                // the target requirement rather than scope it. Residual deviation recorded as
                // OOS-DX4-2: choosing mode 0 (Treasure) still requires a legal opponent
                // creature to target, so with no opponent creature on the battlefield the
                // trigger is removed from the stack (CR 603.3d/608.2b) instead of making a
                // Treasure. Closing it needs `mode_targets` honoured on the trigger path --
                // an engine change, out of scope for this card-def-only batch.
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: Create a Treasure token.
                        Effect::CreateToken {
                            spec: treasure_token_spec(1),
                        },
                        // Mode 1: Target creature an opponent controls gets -1/-1 UNTIL END OF
                        // TURN (CR 613.1e, layer 7c). Authored as the shipped until-EOT idiom
                        // -- `ApplyContinuousEffect` + `EffectFilter::DeclaredTarget` +
                        // `EffectDuration::UntilEndOfTurn` -- exactly as `drown_in_ichor.rs`
                        // does for its printed "-4/-4 until end of turn". NOT a
                        // `CounterType::MinusOneMinusOne` counter: a counter would persist past
                        // cleanup, be proliferate-able, and annihilate against +1/+1 counters
                        // under CR 122.3, none of which the printed card does.
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyBoth(-1),
                                filter: EffectFilter::DeclaredTarget { index: 0 },
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                    ],
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    mode_targets: None,
                }),
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        // PB-DX4 (2026-08-01, OOS-DP10-8): Complete -> partial.
        //
        // The three deviations this batch found (phantom `Decayed`, permanent -1/-1 counter,
        // wrong stored `oracle_text`) are all FIXED above. This marker is for the fourth,
        // which the fix surfaced rather than introduced: the mode-1 target is declared FLAT
        // on the trigger, so it is required whichever mode is chosen. With no opponent
        // creature on the battlefield, the trigger is removed from the stack (CR 603.3d) and
        // the controller gets NOTHING -- where the printed card lets them simply choose
        // "Create a Treasure token". That is reachable in ordinary play, not a corner case.
        //
        // Not authorable today: `ModeSelection.mode_targets` (PB-AC4) is the CR 601.2c-correct
        // scoping, but every consumer of it is on the CASTING path (`rules/casting.rs`, plus
        // read-only `rules/queries.rs`) and nothing on the triggered-ability path reads it --
        // so moving the target there would DROP the requirement rather than scope it. Filed
        // as OOS-DX4-2; closing it is an engine change, out of scope for this card-def batch.
        completeness: Completeness::partial(
            "Modal WhenDies trigger declares its mode-1 target flat (ModeSelection.mode_targets \
             is honoured only on the casting path, never for triggered abilities), so the 'target \
             creature an opponent controls' requirement applies to BOTH modes: with no opponent \
             creature the trigger is removed from the stack (CR 603.3d) instead of creating a \
             Treasure. The three PB-DX4 oracle defects (phantom Decayed keyword, permanent -1/-1 \
             counter for a printed 'until end of turn', and a stored oracle_text naming Decayed \
             and 'enters' against a WhenDies trigger) are fixed.",
        ),
    }
}
