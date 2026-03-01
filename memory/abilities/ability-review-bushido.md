# Ability Review: Bushido

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.45
**Files reviewed**:
- `crates/engine/src/state/types.rs:639-648`
- `crates/engine/src/state/hash.rs:471-475` (KeywordAbility), `1157-1158` (TriggerEvent)
- `crates/engine/src/state/game_object.rs:199-204`
- `crates/engine/src/state/builder.rs:687-722`
- `crates/engine/src/rules/abilities.rs:1561-1580`
- `tools/replay-viewer/src/view_model.rs:692`
- `crates/engine/tests/bushido.rs` (722 lines, 7 tests)

## Verdict: clean

The Bushido implementation is correct and complete. All CR subrules are handled properly.
The trigger mechanism correctly separates "blocks" (SelfBlocks) from "becomes blocked"
(SelfBecomesBlocked), ensuring a creature gets exactly one Bushido trigger per combat.
The dedup logic for multi-blocker scenarios correctly implements CR 509.3c. The layer
system usage (PtModify / ModifyBoth) is correct for +N/+N effects. Multiple instances
trigger separately per CR 702.45b. All 7 tests are well-structured, cite CR rules, and
cover the documented edge cases. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/bushido.rs:491` | **Direct state mutation in test.** Uses `let mut state = state;` + `expire_end_of_turn_effects(&mut state)` which bypasses immutable state architecture (Invariant #2). Acceptable for test code but inconsistent with other tests that use Commands. **Fix:** No fix required -- this is the established pattern for testing expiry (see exalted tests). |
| 2 | LOW | `tests/bushido.rs:524` | **Menace keyword used without explicit test of menace enforcement.** Test 6 adds Menace to the Bushido attacker to require 2 blockers, which is a dependency on Menace working correctly. If Menace enforcement ever regresses, this test would fail for the wrong reason. **Fix:** Add a comment noting the Menace dependency, e.g., `// Menace requires >= 2 blockers, ensuring multi-blocker scenario`. Already partially documented at line 550. |

### Finding Details

#### Finding 1: Direct state mutation in test

**Severity**: LOW
**File**: `crates/engine/tests/bushido.rs:491`
**Architecture Invariant**: #2 -- "Game state is immutable"
**Issue**: The test directly mutates the game state to call `expire_end_of_turn_effects`. This is a test-only concern and follows the established pattern used by other ability tests (e.g., Exalted). The engine's public API does not expose a Command for cleanup, so direct mutation is the only way to test expiry in isolation.
**Fix**: No fix needed. This is the established testing convention for UntilEndOfTurn expiry verification.

#### Finding 2: Menace dependency in multi-blocker test

**Severity**: LOW
**File**: `crates/engine/tests/bushido.rs:524`
**CR Rule**: 702.45a, 509.3c
**Issue**: Test 6 relies on Menace to create a legal multi-blocker scenario. The Bushido behavior being tested (trigger fires once) is independent of Menace. If a future change breaks Menace enforcement, this test would fail with a misleading error.
**Fix**: Add a clarifying comment at line 524: `// Menace (CR 702.111) requires >= 2 blockers, used here to create a legal multi-blocker scenario for CR 509.3c testing`. The existing comment at line 550 partially covers this.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.45a (blocks) | Yes | Yes | test_702_45a_bushido_blocker_gets_bonus |
| 702.45a (becomes blocked) | Yes | Yes | test_702_45a_bushido_attacker_becomes_blocked |
| 702.45a (until end of turn) | Yes | Yes | test_702_45a_bushido_bonus_expires_eot |
| 702.45b (multiple instances) | Yes | Yes | test_702_45b_bushido_multiple_instances |
| 509.3c (once per attacker) | Yes | Yes | test_702_45a_bushido_attacker_blocked_by_multiple |
| 509.3a (once per blocker) | Yes | Yes | Implicitly covered -- SelfBlocks fires once per blocker_id in the loop |
| Multiplayer (N players) | Yes | Yes | test_702_45a_bushido_multiplayer (4-player) |
| No double-trigger (block vs. become blocked) | Yes | Yes | test_702_45a_bushido_does_not_double_trigger |
| Hash coverage (KeywordAbility) | Yes | N/A | hash.rs:471-475, discriminant 77, includes N param |
| Hash coverage (TriggerEvent) | Yes | N/A | hash.rs:1157-1158, discriminant 18 |
| View model string | Yes | N/A | view_model.rs:692, "Bushido {n}" |

## Implementation Quality Notes

### Correct Decisions

1. **Two separate TriggeredAbilityDef entries per Bushido instance**: One for SelfBlocks, one for SelfBecomesBlocked. This correctly models CR 702.45a's "blocks or becomes blocked" as two distinct trigger conditions. A creature is either attacking (may become blocked) or blocking (blocks), never both, so exactly one fires per combat.

2. **Reusing `collect_triggers_for_event` instead of manual PendingTrigger construction**: Unlike Flanking (which needs special fields like `flanking_blocker_id`), Bushido's effect is self-contained as a standard `ApplyContinuousEffect`. The generic trigger path handles it cleanly.

3. **CEFilter::Source for the continuous effect**: At resolution time, `CEFilter::Source` resolves to `CEFilter::SingleObject(ctx.source)`, correctly targeting the creature with Bushido (not the opponent's creature). This matches CR 702.45a: "**it** gets +N/+N".

4. **PtModify layer (Layer 7c)**: Correct for +N/+N effects that modify power and toughness. Bushido's bonus does not set P/T, it modifies it -- PtModify is the right layer.

5. **Dedup of blocked attackers**: `blocked_attackers.sort(); blocked_attackers.dedup();` correctly ensures an attacker blocked by multiple creatures only triggers SelfBecomesBlocked once (CR 509.3c).

6. **SelfBecomesBlocked as shared infrastructure**: The new TriggerEvent variant is not Bushido-specific -- it will be reusable by Rampage and card-specific "becomes blocked" triggers, avoiding future duplication.

### Ruling Coverage

| Ruling | Covered? | Notes |
|--------|----------|-------|
| Shape Stealer (2005-06-01) | N/A | Variable-N Bushido; standard fixed-N calculates at resolution which is always the same value |
| Fumiko the Lowblood (2005-02-01) | N/A | Variable Bushido; not relevant to fixed-N implementation |
| Curtain of Light (2005-06-01) | Not tested | "Curtain of Light does trigger bushido abilities" -- effects that cause a creature to become blocked would also trigger SelfBecomesBlocked. Engine does not yet support "becomes blocked by effect" (CR 509.3c second sentence). No action needed now. |
| Sensei Golden-Tail (2004-12-01) | Yes | "Multiple instances of the bushido ability each trigger separately" -- covered by test_702_45b_bushido_multiple_instances |
