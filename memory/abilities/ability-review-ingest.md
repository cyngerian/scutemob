# Ability Review: Ingest

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.115
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 629-638)
- `crates/engine/src/state/hash.rs` (lines 467-468, 1090-1092, 1373-1381)
- `crates/engine/src/state/stubs.rs` (lines 230-242)
- `crates/engine/src/state/stack.rs` (lines 407-427)
- `crates/engine/src/rules/abilities.rs` (lines 1757-1834, 2214-2222)
- `crates/engine/src/rules/resolution.rs` (lines 1634-1671)
- `tools/tui/src/play/panels/stack_view.rs` (lines 80-82)
- `tools/replay-viewer/src/view_model.rs` (lines 473-475, 687)
- `crates/engine/tests/ingest.rs` (all 619 lines)

## Verdict: needs-fix

The implementation is correct in its core CR enforcement: triggers fire only for
player damage (not planeswalker/battle), empty library is a safe no-op, the damaged
player identity is threaded correctly through the trigger pipeline, and multiplayer
targeting works. Hash discriminants (75 for KeywordAbility, 18 for StackObjectKind)
are unique with no collisions. All PendingTrigger construction sites (18 total) include
the two new fields. The TUI and replay viewer match arms are exhaustive.

However, there is one MEDIUM finding (test does not actually exercise CR 702.115b --
multiple instances on a single creature) and one LOW finding (unreachable!() in engine
library code).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `crates/engine/tests/ingest.rs:375-495` | **Test does not exercise CR 702.115b (multiple instances on one creature).** The test uses two separate creatures with one Ingest each, which tests CR 702.115a (two separate triggers), not 702.115b (one creature with multiple instances triggering separately). **Fix:** Add a test with a single creature that has a CardDefinition containing two `AbilityDefinition::Keyword(Ingest)` entries, register it in the card registry, and verify two triggers fire from one creature. |
| 2 | LOW | `crates/engine/src/rules/abilities.rs:1770` | **`unreachable!()` in engine library code.** While safely guarded by `if matches!(assignment.target, CombatDamageTarget::Player(_))` on line 1748, an `unreachable!()` macro will panic if reached. The conventions say "never `unwrap()` or `expect()` in engine logic" and `unreachable!()` is similarly panic-inducing. **Fix:** Replace the `match` + `unreachable!()` with `if let CombatDamageTarget::Player(pid) = &assignment.target { *pid } else { continue; }` or extract the player from the outer `if matches!` guard using `if let`. |

### Finding Details

#### Finding 1: Test does not exercise CR 702.115b

**Severity**: MEDIUM
**File**: `crates/engine/tests/ingest.rs:375-495`
**CR Rule**: 702.115b -- "If a creature has multiple instances of ingest, each triggers separately."
**Issue**: The test `test_702_115_ingest_multiple_instances_trigger_separately` creates two
separate creatures (Ingest Creature A and Ingest Creature B), each with one instance of
Ingest. This tests that two different creatures each trigger independently, which is just
CR 702.115a applied twice. CR 702.115b specifically covers the case where a SINGLE creature
has multiple instances of Ingest (e.g., granted by two separate effects), and each instance
triggers separately.

The implementation in `abilities.rs:1775-1793` correctly counts Ingest instances from the
CardDefinition and creates that many triggers per creature. However, because the test uses
ObjectSpec creatures without a CardDefinition (card_id is None), the count defaults to 1
via `.unwrap_or(1)`. The actual multi-instance counting code path is never exercised by
any test.

Additionally, the runtime `keywords` field is an `OrdSet<KeywordAbility>` (a set), so it
can only hold one instance of Ingest regardless. The multi-instance counting relies entirely
on the CardDefinition, which is the correct approach, but needs a test to validate it.

**Fix**: Add a test `test_702_115b_ingest_single_creature_multiple_instances` that:
1. Creates a CardDefinition with two `AbilityDefinition::Keyword(KeywordAbility::Ingest)` entries.
2. Registers it in the card registry.
3. Creates a creature ObjectSpec with `.with_card_id(...)` pointing to that definition.
4. Has the creature deal unblocked combat damage to a player.
5. Asserts that TWO triggers fire from the single creature.
6. After resolving both, asserts that two cards are exiled from the damaged player's library.

Rename the existing test to `test_702_115a_ingest_two_creatures_each_trigger` to clarify it
tests the basic case of multiple creatures, not 702.115b.

#### Finding 2: unreachable!() in engine library code

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:1770`
**CR Rule**: N/A (code quality / convention)
**Issue**: Line 1770 uses `_ => unreachable!()` inside a `match &assignment.target` block.
While this is safely guarded by the `if matches!(assignment.target, CombatDamageTarget::Player(_))`
check on line 1748, the `unreachable!()` macro produces a panic path in the engine library.
The project conventions say "never `unwrap()` or `expect()` in engine logic" -- `unreachable!()`
is equivalently panic-inducing. There is one other pre-existing instance in `effects/mod.rs:1328`,
but adding more should be avoided.

**Fix**: Replace lines 1768-1771:
```rust
let damaged_player = match &assignment.target {
    CombatDamageTarget::Player(pid) => *pid,
    _ => unreachable!(),
};
```
with a safe extraction from the outer guard:
```rust
// Already guaranteed by the `if matches!(..., Player(_))` guard above.
let CombatDamageTarget::Player(damaged_player) = &assignment.target else {
    continue;
};
let damaged_player = *damaged_player;
```
Or restructure the outer block to use `if let CombatDamageTarget::Player(pid) = &assignment.target`
instead of the `if matches!()` guard, avoiding the redundant inner match entirely.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.115a (trigger definition) | Yes | Yes | test_basic, test_blocked, test_empty_library, test_multiplayer |
| 702.115a (player only, not planeswalker) | Yes | No | Guarded by `CombatDamageTarget::Player(_)` check; no negative test for planeswalker damage. Acceptable -- engine does not yet generate planeswalker combat damage events. |
| 702.115a (uses the stack) | Yes | Yes | test_basic verifies stack has 1 item before resolution |
| 702.115a (empty library no-op) | Yes | Yes | test_empty_library -- ruling 2015-08-25 |
| 702.115a (face-up exile) | Yes | N/A | Engine default is face-up; no special code needed; confirmed in doc comments |
| 702.115a (multiplayer targeting) | Yes | Yes | test_multiplayer -- P4 library untouched |
| 702.115a (damage == 0 no trigger) | Yes | Yes | test_blocked (blocked creature deals 0 to player) |
| 702.115b (multiple instances) | Yes (code path exists) | **No** | **MEDIUM**: test uses two creatures, not one creature with 2 instances |
| 702.115a (creature on battlefield) | Yes | Implicit | `obj.zone == ZoneId::Battlefield` check at line 1762 |
| Ruling: card exiled face up | Yes | N/A | Engine default behavior |
| Ruling: empty library = nothing happens | Yes | Yes | test_empty_library |

## Previous Findings (re-review only)

N/A -- first review.
