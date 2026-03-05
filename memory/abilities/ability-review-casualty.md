# Ability Review: Casualty

**Date**: 2026-03-05
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.153
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Casualty)
- `crates/engine/src/state/stack.rs` (StackObjectKind::CasualtyTrigger, was_casualty_paid)
- `crates/engine/src/state/hash.rs` (discriminants 104, 34; StackObject field hash)
- `crates/engine/src/rules/command.rs` (Command::CastSpell casualty_sacrifice field)
- `crates/engine/src/rules/engine.rs` (destructure + pass)
- `crates/engine/src/rules/casting.rs` (validation, sacrifice execution, trigger creation)
- `crates/engine/src/rules/resolution.rs` (CasualtyTrigger resolution via copy_spell_on_stack)
- `crates/engine/src/rules/copy.rs` (was_casualty_paid = false on copies)
- `crates/engine/src/testing/replay_harness.rs` (cast_spell_casualty action type)
- `crates/engine/src/testing/script_schema.rs` (casualty_sacrifice field)
- `tools/tui/src/play/panels/stack_view.rs` (CasualtyTrigger match arm)
- `tools/replay-viewer/src/view_model.rs` (CasualtyTrigger serialization + KeywordAbility display)
- `crates/engine/tests/casualty.rs` (9 tests)

## Verdict: clean

The implementation is correct and thorough. It faithfully implements CR 702.153a
(the static additional-cost ability and the triggered copy ability), uses layer-resolved
characteristics for the power check, correctly marks copies with `is_copy: true` so they
do not increment `spells_cast_this_turn`, gracefully handles the case where the original
spell is no longer on the stack at trigger resolution time, and covers all relevant
negative cases in tests. Hash coverage is complete for new fields and enum variants.
No HIGH or MEDIUM findings. Two LOW findings noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `memory/ability-wip.md:4` | **WIP references wrong CR number.** `ability-wip.md` says `cr: 702.154` but the correct CR is 702.153. The plan file has the right number. **Fix:** Change `cr: 702.154` to `cr: 702.153` in `ability-wip.md`. |
| 2 | LOW | `crates/engine/tests/casualty.rs` | **Missing test for countered-original edge case.** No test verifies the graceful no-op when the original spell is countered before the CasualtyTrigger resolves (the `Err(_)` branch at resolution.rs:817). **Fix:** Add a test that casts with casualty, counters the original spell, then resolves the CasualtyTrigger and verifies no copy is created and no panic occurs. |

### Finding Details

#### Finding 1: WIP references wrong CR number

**Severity**: LOW
**File**: `memory/ability-wip.md:4`
**CR Rule**: 702.153 -- Casualty
**Issue**: The `ability-wip.md` file lists `cr: 702.154` (which is Enlist, not Casualty). The plan file correctly identifies 702.153 in its header. This is a metadata-only issue with no functional impact.
**Fix**: Change line 4 of `memory/ability-wip.md` from `cr: 702.154` to `cr: 702.153`.

#### Finding 2: Missing test for countered-original edge case

**Severity**: LOW
**File**: `crates/engine/tests/casualty.rs`
**CR Rule**: 702.153a -- "When you cast this spell, if a casualty cost was paid for it, copy it."
**Issue**: The resolution handler at `resolution.rs:817` has a graceful `Err(_)` branch for when the original spell is no longer on the stack (e.g., was countered by an opponent before the CasualtyTrigger resolves). This branch is correct but has no dedicated test coverage. The code path is low-risk (it simply emits `AbilityResolved` without creating a copy), but a test would prevent regressions and document the interaction.
**Fix**: Add `test_casualty_original_countered_no_copy` that casts with casualty, uses a `CounterSpell` command (or direct stack manipulation) to remove the original, then resolves the CasualtyTrigger and asserts: (a) no copy on the stack, (b) no panic, (c) `AbilityResolved` event emitted.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.153a (static: additional cost) | Yes | Yes | test_casualty_basic_copy, test_casualty_power_threshold, test_casualty_optional_no_sacrifice |
| 702.153a (triggered: copy spell) | Yes | Yes | test_casualty_basic_copy (trigger + copy resolution) |
| 702.153a (power >= N) | Yes | Yes | test_casualty_power_threshold, test_casualty_higher_power_accepted |
| 702.153a (optional) | Yes | Yes | test_casualty_optional_no_sacrifice |
| 702.153a (creature only) | Yes | Yes | test_casualty_not_a_creature |
| 702.153a (caster's creature) | Yes | Yes | test_casualty_wrong_controller |
| 702.153a (battlefield only) | Yes | Yes | test_casualty_creature_not_on_battlefield |
| 702.153a (copy not cast) | Yes | Yes | test_casualty_copy_is_not_cast |
| 702.153a (LIFO: copy resolves first) | Yes | Yes | test_casualty_basic_copy (life gain sequence) |
| 702.153a (layer-resolved power) | Yes | Indirect | Power check uses `calculate_characteristics`; no test with P/T modifying effects |
| 702.153a (countered trigger = no copy) | Yes | No | LOW Finding 2 |
| 702.153b (multiple instances) | No (deferred) | No | No cards have multiple casualty instances; documented as deferred |
| 601.2b/f-h (additional cost rules) | Yes | Yes | Sacrifice during cost payment, before spell goes to stack |
