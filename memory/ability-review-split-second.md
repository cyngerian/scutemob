# Ability Review: Split Second

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.61
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 249-255: KeywordAbility::SplitSecond)
- `crates/engine/src/state/hash.rs` (lines 347-348: discriminant 33)
- `tools/replay-viewer/src/view_model.rs` (line 597: display string)
- `crates/engine/src/rules/casting.rs` (lines 66-72: CastSpell gate; lines 1051-1074: has_split_second_on_stack helper)
- `crates/engine/src/rules/abilities.rs` (lines 63-70: ActivateAbility gate; lines 388-395: CycleCard gate)
- `crates/engine/tests/split_second.rs` (8 tests, 796 lines)
- `crates/engine/src/rules/engine.rs` (command dispatcher -- checked for completeness)
- `crates/engine/src/rules/command.rs` (all Command variants -- checked for coverage)
- `crates/engine/src/rules/mana.rs` (confirmed no split second check -- correct per CR 702.61b)
- `crates/engine/src/rules/lands.rs` (confirmed no split second check -- correct, self-blocking via empty stack requirement)

## Verdict: clean

The Split Second implementation is correct and complete for the current engine scope. All three CR subrules (702.61a, 702.61b, 702.61c) are properly handled. The enforcement gates are placed in the right locations (CastSpell, ActivateAbility, CycleCard), mana abilities and special actions are correctly exempted, and the helper function uses `calculate_characteristics` to respect the layer system. All 8 tests cover the key behaviors with proper CR citations. The one known gap (cascade + split second interaction) is documented in the plan and is genuinely niche -- it does not warrant a MEDIUM finding since it requires two uncommon mechanics interacting simultaneously and is correctly identified as deferred.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `casting.rs:1063-1067` | **Fallback chain in has_split_second_on_stack masks potential data integrity issue.** See details. |
| 2 | LOW | `tests/split_second.rs:553` | **Test 6 missing CardDefinition for second instant.** Card is a "naked object" without a CardDefinition in the registry. |
| 3 | LOW | `copy.rs:317-345` | **Cascade bypasses split second check (documented/deferred).** The plan correctly identifies this as deferred. |

### Finding Details

#### Finding 1: Fallback chain in has_split_second_on_stack

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:1063-1067`
**CR Rule**: 702.61a -- "As long as this spell is on the stack, players can't cast other spells or activate abilities that aren't mana abilities."
**Issue**: The `has_split_second_on_stack` function has a two-level fallback: if `calculate_characteristics` returns `None` (object not found in state), it falls back to `state.object()` (which should also fail for missing objects), and then to `Characteristics::default()` (no keywords). This means a corrupted stack (stack object references a nonexistent source object) would silently report "no split second" rather than flagging the data integrity problem. In practice this is extremely unlikely since the engine maintains strong invariants between `stack_objects` and `objects`, but the fallback to `unwrap_or_default()` could mask bugs during development.
**Fix**: Consider logging or debug-asserting when the fallback path is taken. Not blocking -- this is a defensive coding style issue, not a correctness bug.

#### Finding 2: Test 6 naked object

**Severity**: LOW
**File**: `crates/engine/tests/split_second.rs:553`
**CR Rule**: Architecture invariant 9 -- "Every card in a game must have a CardDefinition before the game starts."
**Issue**: Test 6 (`test_split_second_blocks_caster_too`) creates "Second Instant" as a naked `ObjectSpec` without a corresponding `CardDefinition` in the registry. While this works in tests because the cast fails before the card definition is needed (the split second check blocks it), it violates the documented architecture invariant. Other tests in this file (tests 1, 3, 7) correctly register `plain_instant_def()` in the registry.
**Fix**: Add `plain_instant_def()` to the registry in test 6 and use `.with_card_id(CardId("plain-instant".to_string()))` on the second instant's ObjectSpec. Alternatively, since the test asserts the cast is blocked, the naked object is functionally correct -- the fix is only for consistency.

#### Finding 3: Cascade bypasses split second check (deferred)

**Severity**: LOW
**File**: `crates/engine/src/rules/copy.rs:317-345`
**CR Rule**: 702.61a -- Cascade is a triggered ability that resolves and casts a spell. Per Krosan Grip rulings (2021-03-19), spells cast via cascade are still blocked by split second if a split second spell is also on the stack.
**Issue**: `resolve_cascade` directly creates a `StackObject` and pushes it onto the stack without checking `has_split_second_on_stack`. If cascade and split second ever co-exist on the stack, the cascade-found card would be "cast" in violation of CR 702.61a. This is documented in the plan as a deferred interaction since it requires a cascade spell to have been cast BEFORE the split second spell (cascade triggers on cast, then the split second spell is cast in response). This scenario is extremely rare.
**Fix**: When cascade support is tested with split second, add a `has_split_second_on_stack` check before line 319 in `resolve_cascade`. For now, document in the ability coverage doc that this is a known gap.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.61a (spell casting blocked) | Yes | Yes | test_split_second_blocks_casting_spells (test 1) |
| 702.61a (ability activation blocked) | Yes | Yes | test_split_second_blocks_activated_abilities (test 2) |
| 702.61a (cycling blocked) | Yes | Yes | test_split_second_blocks_cycling (test 3) |
| 702.61a (applies to caster too) | Yes | Yes | test_split_second_blocks_caster_too (test 6) |
| 702.61a (only while on stack) | Yes | Yes | test_split_second_restriction_ends_after_resolution (test 7) |
| 702.61b (mana abilities allowed) | Yes | Yes | test_split_second_allows_mana_abilities (test 4) |
| 702.61b (special actions allowed) | Yes (implicit) | No | PlayLand self-blocks via empty stack requirement; BringCompanion self-blocks via empty stack requirement. No explicit test, but correct by construction. |
| 702.61b (triggered abilities fire) | Yes | Yes | test_split_second_triggered_abilities_still_fire (test 8) |
| 702.61b (pass priority allowed) | Yes | Yes | test_split_second_allows_pass_priority (test 5) |
| 702.61c (multiple instances redundant) | Yes (OrdSet) | No | Automatic via OrdSet deduplication of keywords. No explicit test needed. |

## Implementation Quality Notes

- **CR citations**: All doc comments on the helper function, enum variant, and gate checks cite CR 702.61a/b. The test file header has a comprehensive rules summary. All 8 tests have CR citations.
- **Hash coverage**: Discriminant 33 added in `hash.rs` -- matches exhaustive match, no gaps.
- **view_model.rs**: Display string "Split Second" added -- correct rendering.
- **No .unwrap() in library code**: The `has_split_second_on_stack` function uses `unwrap_or_else` / `unwrap_or_default` (safe fallback). No panic paths in engine logic.
- **Multiplayer correctness**: The check is player-agnostic -- it scans all stack objects regardless of who cast them, and the gate is checked for ALL players attempting to cast/activate. Correct for N players.
- **Layer system integration**: Uses `calculate_characteristics` to respect continuous effects that might grant or remove split second (e.g., a hypothetical effect that gives a spell split second). This is thorough and correct.
- **Gate placement**: All three gates (CastSpell, ActivateAbility, CycleCard) are placed immediately after the priority check and before any other validation. This ensures early, clear error reporting.
