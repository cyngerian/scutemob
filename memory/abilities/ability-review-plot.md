# Ability Review: Plot

**Date**: 2026-03-02
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.170
**Files reviewed**:
- `crates/engine/src/rules/plot.rs` (NEW -- 163 lines)
- `crates/engine/src/rules/casting.rs` (Plot validation, zone-bypass, mutual exclusion, cost)
- `crates/engine/src/rules/engine.rs` (Command::PlotCard dispatch)
- `crates/engine/src/rules/events.rs` (GameEvent::CardPlotted)
- `crates/engine/src/rules/command.rs` (Command::PlotCard)
- `crates/engine/src/rules/resolution.rs` (was_plotted in StackObject paths, cast_alt_cost)
- `crates/engine/src/rules/copy.rs` (was_plotted: false for copies and cascade)
- `crates/engine/src/rules/mod.rs` (pub mod plot)
- `crates/engine/src/state/types.rs` (KeywordAbility::Plot, AltCostKind::Plot)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Plot { cost })
- `crates/engine/src/state/game_object.rs` (is_plotted, plotted_turn fields)
- `crates/engine/src/state/stack.rs` (was_plotted field on StackObject)
- `crates/engine/src/state/hash.rs` (discriminants: KW=97, AbilDef=30, Event=88, GO fields, SO field)
- `crates/engine/src/state/mod.rs` (zone-change resets at 2 sites)
- `crates/engine/src/state/builder.rs` (is_plotted: false, plotted_turn: 0)
- `crates/engine/src/effects/mod.rs` (token defaults: is_plotted: false, plotted_turn: 0)
- `crates/engine/src/testing/replay_harness.rs` (plot_card, cast_spell_plot, find_plotted_in_exile)
- `crates/engine/tests/plot.rs` (NEW -- 20 tests, 1513 lines)

## Verdict: clean

The Plot implementation is correct and thorough. All six CR 702.170 subrules are properly implemented. The special action handler (`plot.rs`) correctly validates timing (main phase + empty stack), turn ownership, card location (hand), keyword presence, and mana cost. The free-cast pipeline in `casting.rs` correctly enforces sorcery-speed timing even for instants, the "later turn" restriction, zero-cost payment, and mutual exclusion with all 14 other alternative costs. Hash discriminants are unique within their respective enums. Zone-change cleanup properly resets `is_plotted`/`plotted_turn` at both `move_object_to_zone` sites. Tests cover all CR subrules with 20 comprehensive tests including positive cases, negative cases, boundary conditions, and edge cases. All findings are LOW severity.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `rules/plot.rs:119` | **Silent default cost for missing AbilityDefinition::Plot.** If card has keyword but no AbilityDefinition, cost defaults to zero. |
| 2 | LOW | `rules/resolution.rs:284` | **Missing cast_alt_cost transfer for Plot.** Resolved permanent won't have AltCostKind::Plot. |
| 3 | LOW | `rules/casting.rs:205-221` | **No explicit owner check on plot free-cast.** CR 702.170d says "owner"; implicitly enforced by timing. Matches Foretell pattern. |

### Finding Details

#### Finding 1: Silent default cost for missing AbilityDefinition::Plot

**Severity**: LOW
**File**: `crates/engine/src/rules/plot.rs:119`
**CR Rule**: 702.170a -- "Plot [cost] means... you may exile this card from your hand and pay [cost]."
**Issue**: When the card has `KeywordAbility::Plot` but no `AbilityDefinition::Plot { cost }` in the registry (misconfigured card definition), the cost lookup at line 107-117 returns `None`, which is silently converted to `ManaCost::default()` (zero) via `unwrap_or_default()`. This allows a misconfigured card to be plotted for free instead of returning an error. Compare with the blitz validation pattern in `casting.rs:869` which returns `GameStateError::InvalidCommand("spell does not have blitz")` when `get_blitz_cost().is_none()`.
**Fix**: Replace `cost.unwrap_or_default()` with an explicit error check:
```rust
let plot_cost = cost.ok_or_else(|| GameStateError::InvalidCommand(
    "plot: card has Plot keyword but no AbilityDefinition::Plot { cost } defined".into(),
))?;
```

#### Finding 2: Missing cast_alt_cost transfer for Plot in resolution

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:284-294`
**CR Rule**: 702.170d -- tracking that a permanent was cast via Plot
**Issue**: When a spell resolves and moves to the battlefield, the `cast_alt_cost` assignment chain at line 284 checks `was_evoked`, `was_escaped`, `was_dashed`, `was_blitzed`, then falls to `None`. It does NOT check `was_plotted`. This means a permanent that entered via Plot free-cast will have `cast_alt_cost: None`. Currently no game mechanics depend on knowing a permanent was plot-cast after resolution, so this has no functional impact. But it breaks pattern consistency with other alternative costs.
**Fix**: Add `was_plotted` check to the chain:
```rust
obj.cast_alt_cost = if stack_obj.was_evoked {
    Some(AltCostKind::Evoke)
} else if stack_obj.was_escaped {
    Some(AltCostKind::Escape)
} else if stack_obj.was_dashed {
    Some(AltCostKind::Dash)
} else if stack_obj.was_blitzed {
    Some(AltCostKind::Blitz)
} else if stack_obj.was_plotted {
    Some(AltCostKind::Plot)
} else {
    None
};
```

#### Finding 3: No explicit owner check on plot free-cast

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:205-221`
**CR Rule**: 702.170d -- "A plotted card's **owner** may cast it from exile"
**Issue**: The plot validation block checks zone (exile), is_plotted, and plotted_turn, but does not explicitly verify `card_obj.owner == player`. This is implicitly enforced by the sorcery-speed timing check at line 950 (`state.turn.active_player != player`) -- a non-owner cannot cast during "their main phase" because it's not their turn when the owner is the active player. This matches the existing Foretell pattern (also no explicit owner check). The `find_plotted_in_exile` helper in replay_harness.rs DOES filter by owner. Severity is LOW because it's defense-in-depth only -- the timing check provides the same guarantee in all standard game scenarios.
**Fix**: Add an explicit owner check for defense-in-depth:
```rust
if card_obj.owner != player {
    return Err(GameStateError::InvalidCommand(
        "plot: only the card's owner may cast a plotted card (CR 702.170d)".into(),
    ));
}
```
This fix should also be applied to the Foretell validation block at line 184 for consistency (separate ticket).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.170a (plot special action from hand, pay cost) | Yes | Yes | test_plot_basic_exile_face_up, test_plot_requires_card_in_hand, test_plot_requires_plot_keyword, test_plot_insufficient_mana |
| 702.170a (main phase + empty stack timing) | Yes | Yes | test_plot_requires_main_phase_empty_stack (2 sub-tests) |
| 702.170a (face-up in exile) | Yes | Yes | test_plot_basic_exile_face_up asserts !face_down |
| 702.170b (special action, no stack) | Yes | Yes | test_plot_does_not_use_stack |
| 702.170c (effects can plot) | N/A | N/A | V1 scope: hand only. 702.170f covers zone flexibility. No cards currently need this. |
| 702.170d (free-cast from exile on later turn) | Yes | Yes | test_plot_cast_from_exile_on_later_turn, test_plot_free_cast_costs_zero |
| 702.170d (sorcery-speed for free-cast) | Yes | Yes | test_plot_free_cast_requires_sorcery_timing, test_plot_free_cast_requires_empty_stack, test_plot_free_cast_requires_own_turn |
| 702.170d (any turn AFTER plotted turn) | Yes | Yes | test_plot_cannot_cast_same_turn, test_plot_turn_tracking_boundary (turn 3->4 boundary) |
| 702.170d (cast even without plot keyword in exile) | Yes | Yes | Validation checks is_plotted flag, not keyword presence in exile. test_plot_mutual_exclusion_not_plotted_card tests the inverse. |
| 702.170e (plotting = performing special action) | Yes | N/A | Implicit -- PlotCard command IS the special action |
| 702.170f (zone flexibility for effects) | No | No | V1 scope: hand only. Plan notes this as future work. Acceptable deferral. |
| CR 116.2k (special action timing) | Yes | Yes | Validated in plot.rs handler; test_plot_requires_player_turn |
| CR 116.3 (priority after special action) | Yes | N/A | Comment in engine.rs:318 notes priority is maintained |
| CR 118.9a (mutual exclusion with other alt costs) | Yes | Yes | Plot block checks all 14 other alt costs; test_plot_mutual_exclusion_not_plotted_card |
| CR 118.9c (mana value unchanged) | Yes | Yes | test_plot_mana_value_unchanged_on_stack (mana value 5 on stack) |
| CR 400.7 (new ObjectId on zone change) | Yes | Yes | test_plot_card_identity_tracking; zone-change cleanup in state/mod.rs |
| Normal cast still works | Yes | Yes | test_plot_normal_cast_still_works |
| Both main phases work | Yes | Yes | test_plot_cast_postcombat_main_phase, test_plot_action_postcombat_main_phase |

## Additional Observations

### Strengths
- **Thorough mutual exclusion**: The plot block in casting.rs (Step 1k, lines 879-966) checks all 14 other alternative costs. This is comprehensive.
- **Sorcery-speed enforcement for instants**: CR 702.170d's timing restriction is correctly applied even to instant cards -- tested explicitly in `test_plot_free_cast_requires_sorcery_timing`.
- **Face-up vs face-down**: Correctly distinguishes Plot (face-up, public) from Foretell (face-down, hidden). `reveals_hidden_info()` correctly returns `false` for CardPlotted (via the `_ => false` catch-all).
- **20 tests with CR citations**: Every test cites the relevant CR section. Test coverage is excellent across positive cases, negative cases, and boundary conditions.
- **Replay harness support**: Both `plot_card` and `cast_spell_plot` action types added with appropriate helper (`find_plotted_in_exile` filters by owner).
- **Hash coverage complete**: All 4 hash sites covered (KeywordAbility=97, AbilityDefinition=30, GameObject fields, StackObject field, GameEvent=88). Discriminants unique within their respective enums.
- **Zone-change cleanup**: Both `move_object_to_zone` paths reset `is_plotted: false, plotted_turn: 0`. Builder, token creation, and resolution paths all initialize correctly.
- **Copy safety**: `was_plotted: false` in copy.rs for both copy and cascade paths.
