# Ability Review: Horsemanship

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.31
**Files reviewed**:
- `crates/engine/src/state/types.rs:600-604` (enum variant)
- `crates/engine/src/state/hash.rs:459-460` (discriminant 71)
- `tools/replay-viewer/src/view_model.rs:680` (display arm)
- `crates/engine/src/rules/combat.rs:503-518` (block validation)
- `crates/engine/tests/horsemanship.rs` (7 tests, 371 lines)

## Verdict: clean

The Horsemanship implementation is correct. All three CR subrules (702.31a, 702.31b, 702.31c) are properly handled. The critical unidirectionality distinction from Shadow is implemented correctly: only the `attacker_has && !blocker_has` direction is restricted. The enforcement logic, hash discriminant, view model arm, and all 7 tests are sound. No HIGH or MEDIUM findings.

## Findings

No HIGH or MEDIUM findings. No LOW findings either -- the implementation is clean.

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| (none) | -- | -- | No findings. |

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.31a (Horsemanship is an evasion ability) | Yes | Yes | Categorized correctly as evasion; enforcement is in `validate_blocker_legality` alongside Flying, Fear, Intimidate, Shadow. |
| 702.31b sentence 1 (creature with horsemanship can't be blocked by creatures without horsemanship) | Yes | Yes | `test_702_31_horsemanship_creature_cannot_be_blocked_by_non_horsemanship` -- asserts `is_err()`. |
| 702.31b sentence 2 (creature with horsemanship CAN block creature with or without horsemanship) | Yes | Yes | `test_702_31_non_horsemanship_can_be_blocked_by_horsemanship` -- asserts `is_ok()`. This is the critical unidirectionality test. |
| 702.31b (both have horsemanship -- block legal) | Yes | Yes | `test_702_31_horsemanship_creature_can_be_blocked_by_horsemanship` -- asserts `is_ok()`. |
| 702.31b (neither has horsemanship -- baseline) | Yes | Yes | `test_702_31_non_horsemanship_can_block_non_horsemanship` -- asserts `is_ok()`. |
| 702.31c (multiple instances redundant) | Yes | Implicit | OrdSet deduplication handles this automatically via `Ord` derive on `KeywordAbility`. No explicit test, but the mechanism is the same as all other keyword abilities. |

## Additional Coverage (beyond CR 702.31)

| Scenario | Tested? | Notes |
|----------|---------|-------|
| Flying does not satisfy horsemanship (ruling 2009-10-01) | Yes | `test_702_31_horsemanship_does_not_interact_with_flying` -- flying blocker cannot block horsemanship attacker. |
| Horsemanship + Flying dual evasion (negative) | Yes | `test_702_31_horsemanship_plus_flying_both_must_be_satisfied` -- horsemanship-only blocker fails against horsemanship+flying attacker. |
| Horsemanship + Flying dual evasion (positive) | Yes | `test_702_31_horsemanship_plus_flying_satisfied_by_horsemanship_flying` -- blocker with both keywords succeeds. |

## Detailed Analysis

### 1. CR Correctness

The enforcement logic at `combat.rs:503-518` is a direct translation of CR 702.31b:

```
CR 702.31b: "A creature with horsemanship can't be blocked by creatures without
horsemanship. A creature with horsemanship can block a creature with or without
horsemanship."
```

The code checks `attacker_has_horsemanship && !blocker_has_horsemanship`, which captures exactly the first sentence. The second sentence (creature with horsemanship CAN block anything) is satisfied by the absence of any reverse check -- unlike Shadow's `attacker_has != blocker_has` bidirectional test.

Reach is correctly NOT referenced in the Horsemanship check. Reach only interacts with Flying (CR 702.17), confirming the independence stated in the 2009-10-01 ruling.

### 2. Hash Discriminant

Discriminant 71 is unique within the `KeywordAbility` match in `hash.rs`. The only other `71u8.hash_into` in the file (line 2003) is in the separate `GameEvent` impl block (`CardCycled` variant), so there is no collision.

### 3. Multiplayer Correctness

No special multiplayer handling needed. Block validation is per-attacker/per-blocker within `validate_blocker_legality`, which is called for each individual blocker assignment. Works correctly for N players.

### 4. System Interactions

- **Layer system**: Horsemanship is a standard keyword ability. Granting/removing it via continuous effects works through the generic `calculate_characteristics` Layer 6 handling. No special layer logic needed.
- **Replacement effects**: Not applicable (static evasion ability).
- **SBAs**: Not applicable (no state-based action component).
- **Stack**: Not applicable (static ability, does not use the stack).

### 5. Test Quality

All 7 tests follow the established Shadow test pattern. Each test:
- Uses `GameStateBuilder` correctly
- Sets up `CombatState` with attacker registration
- Sends `Command::DeclareBlockers` through `process_command`
- Asserts the correct `is_ok()` or `is_err()` result
- Has proper doc comments citing CR rules
- Has descriptive assertion messages

The test file includes a comprehensive module-level doc comment listing all test scenarios with CR citations.

### 6. Code Quality

- CR citations present in all doc comments (types.rs, combat.rs, test file).
- Hash coverage present for the new variant (discriminant 71).
- Match exhaustiveness: `hash.rs` and `view_model.rs` both have the new arm. No other files have exhaustive matches on `KeywordAbility` that would need updating (confirmed via grep).
- No `.unwrap()` in engine library code (only in test helper `find_object`, which is appropriate).
- Consistent with `memory/conventions.md` conventions.
- No over-engineering -- the implementation is minimal and correct.
