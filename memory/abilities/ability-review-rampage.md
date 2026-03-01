# Ability Review: Rampage

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.23
**Files reviewed**:
- `crates/engine/src/state/types.rs:649-659` (KeywordAbility::Rampage variant)
- `crates/engine/src/state/hash.rs:476-480` (KeywordAbility hash, discriminant 78)
- `crates/engine/src/state/hash.rs:1108-1110` (PendingTrigger rampage fields hash)
- `crates/engine/src/state/hash.rs:1165-1166` (TriggerEvent::SelfBecomesBlocked hash, discriminant 18)
- `crates/engine/src/state/hash.rs:1411-1419` (StackObjectKind::RampageTrigger hash, discriminant 20)
- `crates/engine/src/state/stack.rs:449-468` (StackObjectKind::RampageTrigger variant)
- `crates/engine/src/state/stubs.rs:256-268` (PendingTrigger rampage fields)
- `crates/engine/src/state/game_object.rs:199-204` (TriggerEvent::SelfBecomesBlocked variant)
- `crates/engine/src/state/builder.rs:724-743` (Rampage -> TriggeredAbilityDef wiring)
- `crates/engine/src/rules/abilities.rs:1575-1625` (SelfBecomesBlocked dispatch + Rampage tagging)
- `crates/engine/src/rules/abilities.rs:2425-2432` (Rampage flush arm)
- `crates/engine/src/rules/resolution.rs:1722-1804` (RampageTrigger resolution)
- `crates/engine/src/rules/resolution.rs:1914` (counter-spell passthrough)
- `tools/replay-viewer/src/view_model.rs:479-481` (RampageTrigger view model arm)
- `tools/tui/src/play/panels/stack_view.rs:86-88` (RampageTrigger TUI arm)
- `crates/engine/tests/rampage.rs` (8 tests)

## Verdict: needs-fix

The Rampage implementation is largely correct and well-documented. All CR 702.23 subrules
are implemented and tested. However, there is one MEDIUM finding: the resolution code
creates two separate ContinuousEffects (ModifyPower + ModifyToughness) instead of using
the existing `ModifyBoth(i32)` variant, which is the established pattern used by Flanking
(the immediately preceding ability in resolution.rs). This is inconsistent, wastes a
ContinuousEffect slot and timestamp, and creates a subtle correctness risk if any future
code counts effects or relies on a single effect representing a single P/T modification.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `resolution.rs:1757-1796` | **Rampage uses two ContinuousEffects instead of ModifyBoth.** Should use `ModifyBoth(bonus)` like Flanking does. **Fix:** Replace the two ModifyPower + ModifyToughness effects with a single ModifyBoth(bonus) effect. |
| 2 | LOW | `resolution.rs:1759` | **EffectId sourced from timestamp_counter, not next_object_id.** Flanking (line 1698) uses `state.next_object_id().0` for EffectId; Rampage uses `state.timestamp_counter`. Pre-existing inconsistency in the codebase (copy.rs also uses timestamp_counter), but the Rampage code should match its immediate neighbor Flanking. |
| 3 | LOW | `game_object.rs:199-204` | **SelfBecomesBlocked doc comment mentions Bushido but not Rampage.** The doc comment says "Used by the Bushido keyword" but Rampage also uses this trigger event. |

### Finding Details

#### Finding 1: Rampage uses two ContinuousEffects instead of ModifyBoth

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:1757-1796`
**CR Rule**: 702.23a -- "it gets +N/+N until end of turn for each creature blocking it beyond the first"
**Issue**: The resolution code creates two separate `ContinuousEffect` entries: one with
`ModifyPower(bonus)` and one with `ModifyToughness(bonus)`. The existing `LayerModification::ModifyBoth(i32)`
variant handles exactly this case (equal power and toughness modification). Flanking's resolution
at line 1701-1713 uses `ModifyBoth(-1)` for its -1/-1 effect. Using two effects instead of one
is functionally correct for the current tests but is inconsistent with the established pattern,
creates an extra ContinuousEffect in the state (doubling memory for this ability), uses an extra
timestamp, and could cause subtle issues if future code relies on effect count or assumes a
single +N/+N maps to a single ContinuousEffect. The test at line 238-241 (`test_702_23a_rampage_blocked_by_one_no_bonus`)
checks `state.continuous_effects.is_empty()` for the no-bonus case, but no test checks the
effect count for the bonus case -- a test for effect count would fail if ModifyBoth were used later.
**Fix**: Replace the two-effect pattern (lines 1754-1796) with a single `ModifyBoth(bonus)` ContinuousEffect,
matching the Flanking pattern. Use `state.next_object_id().0` for the EffectId to also match
Flanking's ID generation. The single effect approach should look like:
```rust
let eff_id = state.next_object_id().0;
let ts = state.timestamp_counter;
state.timestamp_counter += 1;
state.continuous_effects.push_back(ContinuousEffect {
    id: EffectId(eff_id),
    source: None,
    timestamp: ts,
    layer: EffectLayer::PtModify,
    duration: EffectDuration::UntilEndOfTurn,
    filter: EffectFilter::SingleObject(source_object),
    modification: LayerModification::ModifyBoth(bonus),
    is_cda: false,
});
```

#### Finding 2: EffectId sourced from timestamp_counter

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1759`
**CR Rule**: N/A (architecture consistency)
**Issue**: Rampage uses `EffectId(ts_power)` where `ts_power = state.timestamp_counter`, meaning
the same value is used for both the EffectId and the timestamp. Flanking (immediately above in
the same file) uses `EffectId(state.next_object_id().0)` with a separate timestamp value. While
functionally both work (EffectIds are only compared with other EffectIds), co-opting the
timestamp value as an ID is conceptually confusing and inconsistent with the neighboring code.
This will be automatically resolved if Finding 1 is fixed using the suggested code.
**Fix**: Resolved by Finding 1's fix (use `state.next_object_id().0` for EffectId).

#### Finding 3: SelfBecomesBlocked doc comment incomplete

**Severity**: LOW
**File**: `crates/engine/src/state/game_object.rs:199-204`
**CR Rule**: 702.23a, 702.45a
**Issue**: The doc comment on `TriggerEvent::SelfBecomesBlocked` says "Used by the Bushido keyword"
but does not mention Rampage, which also uses this trigger event. The comment should list both
keywords for discoverability.
**Fix**: Update the doc comment to say "Used by the Bushido keyword (CR 702.45a) and the Rampage
keyword (CR 702.23a)."

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.23a (trigger fires when becomes blocked) | Yes | Yes | test_702_23a_rampage_blocked_by_two_gets_bonus, test_702_23a_rampage_blocked_by_one_no_bonus |
| 702.23a (+N/+N until end of turn) | Yes | Yes | test_702_23a_rampage_bonus_expires_at_end_of_turn |
| 702.23a ("for each creature beyond the first") | Yes | Yes | test_702_23a_rampage_blocked_by_three_scaled_bonus, test_702_23a_rampage_three_blocked_by_four |
| 702.23a (no trigger when not blocked) | Yes | Yes | test_702_23a_rampage_not_blocked_no_trigger |
| 702.23b (bonus calculated at resolution) | Yes | Yes | test_702_23b_bonus_calculated_at_resolution_time |
| 702.23c (multiple instances trigger separately) | Yes | Yes | test_702_23c_multiple_rampage_instances |
| 509.3c (triggers once per combat) | Yes (dedup in dispatch) | Partial | Dedup logic present; no test for "already blocked" re-triggering edge case (low risk -- requires effect-based blocking infrastructure not yet built) |

## Additional Notes

### Correct aspects of the implementation

1. **CR 702.23b compliance**: The blocker count is correctly read from `state.combat.blockers_for(source_object)`
   at resolution time, not at trigger time. This ensures changes to blockers between trigger
   and resolution are captured.

2. **CR 702.23c compliance**: Each `Rampage(n)` keyword generates its own `TriggeredAbilityDef` in
   builder.rs, so multiple instances produce multiple triggers. The test (`test_702_23c_multiple_rampage_instances`)
   verifies two triggers fire and resolve independently.

3. **Bushido + Rampage safety**: The Rampage tagging code in abilities.rs (line 1605) checks
   `ability_def.description.starts_with("Rampage")`. Bushido's triggered abilities have descriptions
   starting with "Bushido", so there is no risk of mis-tagging. Both abilities share the
   `SelfBecomesBlocked` trigger event without conflict.

4. **Hash coverage**: All new fields (`is_rampage_trigger`, `rampage_n` on PendingTrigger;
   `RampageTrigger` on StackObjectKind; `Rampage(u32)` on KeywordAbility; `SelfBecomesBlocked`
   on TriggerEvent) have proper hash implementations with unique discriminants.

5. **Counter-spell passthrough**: `RampageTrigger` is correctly included in the counter-spell
   passthrough match at resolution.rs:1914.

6. **View model and TUI**: Both tools have correct match arms for `RampageTrigger`.

7. **Bonus calculation**: `saturating_sub(1)` correctly handles the edge case of 0 blockers
   (returns 0) and 1 blocker (returns 0). The multiplication by `rampage_n` is safe for all
   non-negative values.

8. **Source alive check**: The resolution correctly checks that the source creature is still
   on the battlefield before applying effects. If it has left, the trigger resolves but does nothing,
   which is correct for continuous effects that require the target to exist.

9. **Test quality**: 8 tests with clear CR citations, covering positive cases (bonus applied),
   negative cases (not blocked, blocked by 1), scaling (blocked by 3, blocked by 4), multiple
   instances, end-of-turn expiry, and resolution timing. All tests follow the established pattern
   from battle_cry.rs.
