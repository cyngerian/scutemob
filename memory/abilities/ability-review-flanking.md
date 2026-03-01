# Ability Review: Flanking

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.25
**Files reviewed**:
- `crates/engine/src/state/types.rs:639-643`
- `crates/engine/src/state/hash.rs:469-470, 1095-1097, 1387-1395`
- `crates/engine/src/state/stubs.rs:243-255`
- `crates/engine/src/state/stack.rs:428-447`
- `crates/engine/src/rules/abilities.rs:1468-1559, 2341-2348`
- `crates/engine/src/rules/resolution.rs:1675-1719, 1828`
- `tools/tui/src/play/panels/stack_view.rs:83-85`
- `tools/replay-viewer/src/view_model.rs:476-478, 691`
- `crates/engine/tests/flanking.rs` (entire file, 820 lines)

## Verdict: clean

The Flanking implementation correctly models CR 702.25a and CR 702.25b. The trigger fires during the declare blockers step, checks that the blocker lacks flanking, counts multiple instances from the card definition, and resolves by applying a Layer 7c continuous effect (ModifyBoth(-1), UntilEndOfTurn). All hash fields are covered. All match arms are present. Seven tests cover the basic case, negative case (flanking blocker), SBA kill of 0-toughness creature, multiple instances, multiple blockers, end-of-turn expiry, and multiplayer scenarios. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:1526` | **Misleading triggering_event.** Uses `SelfBlocks` but flanking triggers on the attacker being blocked. **Fix:** Add a `SelfBecomesBlocked` variant to TriggerEvent or use `None`; defer until a future refactor since the field is only used for Panharmonicon-style doubling filters which already exclude this event. |
| 2 | LOW | `abilities.rs:1500-1517` | **Externally-granted flanking instances not counted.** Multiple-instance count comes from card definition, not runtime state. A creature granted extra flanking by Cavalry Master or Flanking Licid would only trigger once (from the `.max(1)` fallback), not twice. Systemic limitation shared with Ingest (CR 702.115b). **Fix:** Track keyword instance count in runtime state or scan continuous effects adding Flanking. Defer -- no cards currently grant flanking in the engine. |
| 3 | LOW | `abilities.rs:2347` | **Dangerous fallback in flush handler.** `unwrap_or(trigger.source)` falls back to the attacker ObjectId if `flanking_blocker_id` is None, which would apply -1/-1 to the attacker. Precondition guarantees this never happens, but the fallback hides bugs silently. Matches Ingest pattern (line 2339). **Fix:** Consider `debug_assert!(trigger.flanking_blocker_id.is_some())` before the unwrap_or. Defer -- consistent with existing convention. |
| 4 | LOW | `tests/flanking.rs` | **Missing negative test: flanking creature as blocker.** No test verifies that a flanking creature blocking another creature does NOT trigger flanking. The code is structurally correct (only checks attackers), but an explicit test would guard against regressions. **Fix:** Add `test_702_25_flanking_creature_as_blocker_no_trigger`. |

### Finding Details

#### Finding 1: Misleading triggering_event

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:1526`
**CR Rule**: 702.25a -- "Whenever this creature becomes blocked by a creature without flanking..."
**Issue**: The PendingTrigger for flanking uses `triggering_event: Some(TriggerEvent::SelfBlocks)`, but `SelfBlocks` is defined as "Triggers when this creature blocks" (game_object.rs:140-141). Flanking triggers on the attacker BECOMING BLOCKED, not on the blocker blocking. The trigger source is the attacker, making `SelfBlocks` semantically wrong. This has no behavioral impact because: (1) the `is_flanking_trigger` flag routes the trigger to `FlankingTrigger` StackObjectKind, and (2) trigger doublers only match `AnyPermanentEntersBattlefield`, not `SelfBlocks`.
**Fix**: Either use `triggering_event: None` (since the flanking-specific routing handles everything) or add a `SelfBecomesBlocked` variant to TriggerEvent. Defer until a general trigger-event refactor.

#### Finding 2: Externally-granted flanking instances not counted

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:1500-1517`
**CR Rule**: 702.25b -- "If a creature has multiple instances of flanking, each triggers separately."
**Issue**: The `flanking_count` is derived from the card definition's `abilities` Vec, not the runtime game state. If a creature is granted an additional instance of flanking by an external source (e.g., Cavalry Master: "Other creatures you control with flanking have flanking"), the granted instance would not be counted because it only exists as a continuous effect in Layer 6, not in the card definition. The `.max(1)` ensures at least one trigger fires, but externally granted instances beyond the first are missed. This is the same systemic limitation as Ingest (CR 702.115b) and is documented in the plan: "The 'multiple instances' behavior comes from the card definition's `abilities` Vec count, not the runtime `OrdSet` (which deduplicates)."
**Fix**: Track keyword instance counts in runtime state (e.g., a `keyword_counts: HashMap<KeywordAbility, usize>` field on the game object) or count continuous effects adding the keyword. Defer -- no cards in the engine currently grant flanking to other creatures.

#### Finding 3: Dangerous fallback in flush handler

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:2347`
**CR Rule**: 702.25a
**Issue**: `trigger.flanking_blocker_id.unwrap_or(trigger.source)` uses the attacker's ObjectId as fallback if `flanking_blocker_id` is None. If the precondition (`is_flanking_trigger => flanking_blocker_id.is_some()`) were ever violated, the attacker would apply -1/-1 to itself -- a silent bug. The Ingest pattern at line 2339 (`unwrap_or(trigger.controller)`) has the same issue. This is a defensive coding pattern where the fallback value is wrong but the precondition should always hold.
**Fix**: Add `debug_assert!(trigger.flanking_blocker_id.is_some(), "flanking trigger must have blocker_id")` before the unwrap_or. Consistent with project convention of using debug_assert for preconditions.

#### Finding 4: Missing negative test for flanking creature as blocker

**Severity**: LOW
**File**: `crates/engine/tests/flanking.rs`
**CR Rule**: 702.25a -- "Whenever THIS CREATURE becomes blocked..."
**Issue**: No test verifies the scenario where a creature with flanking is used as a BLOCKER (not attacker). Flanking should NOT trigger in this case because the trigger condition is "whenever this creature becomes blocked," which only applies to attackers. The code handles this correctly by only iterating over attacker_ids in the BlockersDeclared handler, but an explicit regression test would be valuable.
**Fix**: Add a test `test_702_25_flanking_creature_as_blocker_no_trigger` where a flanking creature blocks a non-flanking attacker and verify no trigger fires.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.25a (trigger condition) | Yes | Yes | test_702_25_flanking_basic_minus_one_minus_one |
| 702.25a ("without flanking" check) | Yes | Yes | test_702_25_flanking_does_not_trigger_on_flanking_blocker |
| 702.25a (-1/-1 continuous effect) | Yes | Yes | test_702_25_flanking_basic_minus_one_minus_one (verifies 1/1 P/T) |
| 702.25a (until end of turn) | Yes | Yes | test_702_25_flanking_effect_expires_at_end_of_turn |
| 702.25a (SBA kills 0-toughness) | Yes | Yes | test_702_25_flanking_kills_1_toughness_blocker |
| 702.25a (per-blocker trigger, 509.3d) | Yes | Yes | test_702_25_flanking_multiple_blockers |
| 702.25a (multiplayer) | Yes | Yes | test_702_25_flanking_multiplayer |
| 702.25b (multiple instances) | Yes | Yes | test_702_25b_flanking_multiple_instances |
| 702.25a (attacker-only, not blocker) | Yes (structural) | No | Code only checks attacker_id; no explicit test. Finding 4. |
| 509.3f (characteristics at declaration) | Yes (implicit) | No | Event processed synchronously at block time; no test for mid-resolution keyword changes. Acceptable -- would require complex Humility interaction test. |
