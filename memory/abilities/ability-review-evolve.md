# Ability Review: Evolve

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.100
**Files reviewed**:
- `crates/engine/src/state/stack.rs` (EvolveTrigger variant, lines 274-287)
- `crates/engine/src/state/stubs.rs` (PendingTrigger fields, lines 157-171)
- `crates/engine/src/state/hash.rs` (StackObjectKind disc 12, lines 1243-1251; PendingTrigger fields, lines 1024-1026)
- `crates/engine/src/rules/abilities.rs` (trigger dispatch, lines 900-1043; flush, lines 1751-1759)
- `crates/engine/src/rules/resolution.rs` (EvolveTrigger resolution, lines 1012-1092; counter_spell, line 1194)
- `tools/replay-viewer/src/view_model.rs` (display arm, line 452)
- `crates/engine/tests/evolve.rs` (9 tests)
- All PendingTrigger construction sites (abilities.rs, turn_actions.rs, miracle.rs, effects/mod.rs)

## Verdict: needs-fix

One MEDIUM finding: a missing test for the CR 603.4 intervening-if resolution-time re-check, which is the most distinctive aspect of evolve's rules behavior. The implementation logic itself is correct, but the planned intervening-if-fails-at-resolution test was not written. One LOW finding: a typo in a CR citation comment (703.4 vs 603.4).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tests/evolve.rs` | **Missing test for intervening-if failing at resolution (CR 603.4).** The plan specified test 7 but it was not implemented. **Fix:** Add a test where the trigger fires but the condition fails at resolution. |
| 2 | LOW | `abilities.rs:959` | **Typo in CR citation: "CR 703.4" should be "CR 603.4".** Incorrect rule number in comment. **Fix:** Change "CR 703.4" to "CR 603.4". |

### Finding Details

#### Finding 1: Missing test for intervening-if failing at resolution

**Severity**: MEDIUM
**File**: `crates/engine/tests/evolve.rs` (missing test)
**CR Rule**: 603.4 -- "If the ability triggers, it checks the stated condition again as it resolves. If the condition isn't true at that time, the ability is removed from the stack and does nothing."
**Issue**: The plan explicitly specified `test_evolve_intervening_if_fails_at_resolution` (plan Test 7) as a key test case. The resolution code at `resolution.rs:1050-1063` correctly implements the re-check, but there is no test that exercises the path where the trigger fires at trigger time but the condition is no longer true at resolution time. The multiple-instances test (Test 8) partially validates this -- the second trigger re-checks at resolution -- but that test only covers the case where the condition *still holds* after the first counter. No test covers the fizzle path.

The intervening-if pattern is the single most important rules behavior that distinguishes evolve from a simple ETB counter ability. Without a test, there is no regression protection for this critical code path.

**Fix:** Add a test where:
- P1 controls a 1/1 evolve creature and a 3/3 creature in hand.
- Cast the 3/3. Evolve trigger fires (3 > 1 for both P and T).
- Before the evolve trigger resolves, manually add +1/+1 counters to the evolve creature (e.g., 3 counters, making it 4/4 via raw counters).
- Resolve the trigger. The re-check compares entering 3/3 vs evolve 4/4 -- neither stat is greater.
- Assert: no counter added. The evolve creature still has exactly the manually-added counters, not one more.

Alternatively, if direct counter manipulation is simpler than pump effects, place 3 +1/+1 counters on the evolve creature between cast resolution and trigger resolution (by modifying `state.objects` directly before `pass_all`).

#### Finding 2: Typo in CR citation comment

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:959`
**CR Rule**: 603.4 -- "A triggered ability may read 'When/Whenever/At [trigger event], if [condition], [effect].' ..."
**Issue**: The comment reads `// CR 703.4: Intervening-if check at trigger time.` but the correct rule is **CR 603.4** (triggered abilities), not CR 703.4 (which does not exist -- CR 703 is "Turn-Based Actions"). This could mislead future developers looking up the rule.
**Fix:** Change `CR 703.4` to `CR 603.4` at line 959 of `abilities.rs`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.100a (trigger condition) | Yes | Yes | Tests 1-5 cover positive and negative trigger-time checks |
| 702.100a (OR condition) | Yes | Yes | Test 3 (`test_evolve_greater_power_only_or_condition`) covers power-only |
| 702.100a (same controller) | Yes | Yes | Tests 7, 9 cover opponent/multiplayer |
| 702.100a (counter placement) | Yes | Yes | Tests 1, 2, 3, 8 verify counter after resolution |
| 702.100b (evolves definition) | N/A | N/A | Definitional only; no engine impact |
| 702.100c (noncreature) | Yes | Yes | Test 6 (`test_evolve_noncreature_does_not_trigger`) |
| 702.100d (multiple instances) | Yes | Yes | Test 8 (`test_evolve_multiple_instances`) |
| 603.4 (intervening-if at trigger) | Yes | Yes | Tests 4, 5 (condition false at trigger time, no trigger fires) |
| 603.4 (intervening-if at resolution) | Yes | **No** | Logic at `resolution.rs:1050-1063` correct but no test for fizzle path |
| Layer-aware P/T comparison | Yes | Partial | Tests use raw P/T from ObjectSpec; layer calc invoked but not stressed by continuous effects |
| Last-known information (ruling) | Partial | No | Fallback to `state.objects.get()` present; but dead ObjectId returns None (pre-existing LKI limitation, not evolve-specific) |

## Known Limitations (LOW, pre-existing)

These are not findings against the evolve implementation but are noted for completeness:

1. **Raw keyword check (Humility interaction)**: The trigger dispatch at `abilities.rs:950-953` checks `obj.characteristics.keywords` (raw, not layer-calculated). If Humility removes evolve via layer 6, the trigger still fires incorrectly. This is a pre-existing pattern across all keyword-driven triggers (e.g., Exploit, Dethrone) and is not specific to evolve. LOW, address when the layer-aware keyword check pattern is established project-wide.

2. **Last-known information**: When the entering creature leaves the battlefield before resolution, `calculate_characteristics` returns `None` (dead ObjectId per CR 400.7). The fallback `state.objects.get()` also returns `None`. The condition defaults to `false`. Per ruling 2013-04-15, the correct behavior is to use last-known P/T, but this requires a LKI system that does not yet exist. LOW, pre-existing engine limitation.

3. **Self-entering creature optimization**: The trigger dispatch excludes `obj.id != *object_id` to prevent a creature from triggering its own evolve on entry. While the intervening-if would fail anyway (a creature is never greater than itself), this optimization is correct and harmless.
