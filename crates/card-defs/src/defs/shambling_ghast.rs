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
// Mode order is left as authored (Treasure first) even though the printed bullets run the
// other way (-1/-1 first, Treasure second). A mode's identity in Magic is its text, not its
// index, so the order is not itself an oracle deviation — but it is not invisible either
// (fix cycle, review Finding 10): anything that renders `ModeSelection.modes` positionally,
// or that reports "mode 0" to a player, shows this card's two modes in the opposite order
// from the printed card. Reordering would also silently repoint every existing
// `modes_chosen: [0]` in tests and scripts, so it is recorded here rather than done in a
// batch whose subject is markers.
//
// PB-DX35 (2026-09, OOS-DX4-2): the mode-1 target now lives in
// `ModeSelection.mode_targets`, scoped to mode 1 alone, closing the deviation the PB-DX4
// note above described. `trigger_modal_plan` (`rules/abilities.rs`) is now a genuine
// consumer of `mode_targets` on the TRIGGER path (previously every consumer was on the
// casting path only) and picks the first CR 700.2b-legal mode: with no opponent creature on
// the battlefield, mode 1 (whose target requirement has no legal candidate) can't be chosen,
// so mode 0 (Create a Treasure token, which needs no target) is — the printed "choose one"
// is honoured instead of the whole trigger being removed from the stack.
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
                // PB-DX35 (OOS-DX4-2): the mode-1 target now lives in
                // `mode_targets` below, scoped to mode 1 alone -- mode 0
                // (Create a Treasure token) needs no target at all. See the
                // module doc for the CR 700.2b consequence.
                targets: vec![],
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
                    // PB-DX35 (CR 700.2c/700.2f / OOS-DX4-2): mode 0 (Treasure) needs
                    // no target; mode 1's opponent-creature requirement is scoped to
                    // mode 1 alone. Mode 1's `EffectFilter::DeclaredTarget { index: 0 }`
                    // above already reads slot 0 of ITS OWN mode's slice, so no index
                    // change was needed here.
                    mode_targets: Some(vec![
                        vec![],
                        vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                            controller: TargetController::Opponent,
                            ..Default::default()
                        })],
                    ]),
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
        // PB-DX4 (2026-08-01, OOS-DP10-8): Complete -> partial, for the mode-1 target
        // being declared FLAT on the trigger instead of scoped to mode 1.
        //
        // PB-DX35 (2026-09, OOS-DX4-2): partial -> Complete. The blocker PB-DX4 filed is
        // closed: mode 1's target now lives in `mode_targets`, scoped to mode 1 alone, and
        // `trigger_modal_plan` (`rules/abilities.rs`) makes the CR 700.2b-legal choice on
        // the trigger path -- with no opponent creature on the battlefield mode 1 can't be
        // chosen, so mode 0 (Create a Treasure token, no target needed) is, matching the
        // printed "choose one" instead of removing the whole trigger from the stack
        // (CR 603.3d). The three PB-DX4 oracle defects (phantom Decayed keyword, permanent
        // -1/-1 counter for a printed "until end of turn", and a stored oracle_text naming
        // Decayed and "enters" against a WhenDies trigger) were already fixed by PB-DX4.
        completeness: Completeness::Complete,
    }
}
