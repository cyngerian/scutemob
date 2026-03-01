# Ability Review: Aftermath

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.127
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 783-793)
- `crates/engine/src/cards/card_definition.rs` (lines 279-305)
- `crates/engine/src/state/stack.rs` (lines 142-151)
- `crates/engine/src/state/hash.rs` (lines 517-518, 1592-1593, 3087-3101)
- `crates/engine/src/rules/command.rs` (lines 191-199)
- `crates/engine/src/rules/engine.rs` (lines 97-126)
- `crates/engine/src/rules/casting.rs` (lines 72, 231-287, 351-360, 645-705, 771-780, 975-1007, 1333-1358, 1730-1773)
- `crates/engine/src/rules/resolution.rs` (lines 157-183, 1477-1479)
- `crates/engine/src/rules/copy.rs` (lines 196-197, 376-377)
- `crates/engine/src/rules/abilities.rs` (7 sites, all `cast_with_aftermath: false`)
- `crates/engine/src/testing/replay_harness.rs` (lines 836-860, plus 10 other CastSpell sites with `cast_with_aftermath: false`)
- `crates/engine/src/effects/mod.rs` (line 805, cast_with_flashback exile path)
- `crates/engine/tests/aftermath.rs` (12 tests, 1482 lines)
- `tools/replay-viewer/src/view_model.rs` (line 727)
- `tools/tui/src/play/input.rs` (line 112)

## Verdict: clean

The Aftermath implementation correctly enforces CR 702.127a's three static abilities:
(a) permission to cast the aftermath half from graveyard, (b) restriction against
casting from non-graveyard zones, and (c) exile replacement when the spell leaves
the stack. The implementation cleanly reuses the existing `cast_with_flashback`
exile-on-departure mechanism, which covers all 4 stack departure paths (resolution,
fizzle, counter in resolution.rs, counter via Effect::CounterSpell in effects/mod.rs).
Cost lookup, type-based timing validation, target requirement selection, and effect
resolution all correctly branch on `casting_with_aftermath`. Hash coverage is complete
for all new fields. No `.unwrap()` in engine library code. All 12 tests pass and cover
the core positive, negative, and lifecycle scenarios with proper CR citations.
No HIGH or MEDIUM findings. Three LOW findings noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/aftermath.rs` | **Missing fizzle exile test.** Plan specified `test_aftermath_exile_on_fizzle` but it was not implemented. |
| 2 | LOW | `tests/aftermath.rs` | **Missing instant-speed aftermath timing test.** No test verifies an Instant aftermath half can be cast at instant speed (CR 709.3a). |
| 3 | LOW | `tests/aftermath.rs` | **Missing mutual exclusion test.** No test verifies that combining aftermath with another alternative cost (e.g., flashback) is rejected per CR 118.9a. |

### Finding Details

#### Finding 1: Missing fizzle exile test

**Severity**: LOW
**File**: `crates/engine/tests/aftermath.rs`
**CR Rule**: 702.127a -- "exile it instead of putting it anywhere else any time it would leave the stack"
**Issue**: The plan explicitly specified `test_aftermath_exile_on_fizzle` (when all targets become illegal before resolution, the aftermath spell is exiled rather than going to graveyard). This test was not implemented. The behavior is correct because it reuses `cast_with_flashback` which is tested for fizzle in the flashback tests, but the aftermath-specific fizzle path is untested.
**Fix**: Add a test where the aftermath half targets a creature (would need an aftermath card with targets), the target becomes illegal, and the card is exiled on fizzle. Alternatively, accept coverage through the flashback fizzle test since the mechanism is shared.

#### Finding 2: Missing instant-speed aftermath timing test

**Severity**: LOW
**File**: `crates/engine/tests/aftermath.rs`
**CR Rule**: 709.3a -- "Only the chosen half is evaluated to see if it can be cast."
**Issue**: The implementation correctly recalculates `is_instant_speed` based on `get_aftermath_card_type()` at casting.rs:355-360, but no test verifies that an Instant aftermath half can be cast at instant speed (e.g., during another player's turn or with the stack non-empty). All test cards use Sorcery aftermath halves.
**Fix**: Add a test card with an Instant aftermath half (e.g., "Spring // Mind" from the plan) and verify it can be cast at instant speed. Alternatively, accept this as a low-priority gap since the code path is structurally identical to normal instant-speed checking.

#### Finding 3: Missing mutual exclusion test

**Severity**: LOW
**File**: `crates/engine/tests/aftermath.rs`
**CR Rule**: 118.9a -- "Only one alternative cost can be applied to any one spell as it's being cast."
**Issue**: The alternative cost mutual exclusion code at casting.rs:645-705 correctly rejects combining aftermath with flashback, evoke, bestow, madness, miracle, escape, foretell, and overload. However, no test validates this behavior. The code follows the same proven pattern as other alternative cost exclusion blocks.
**Fix**: Add at least one negative test: a card with both Aftermath and Flashback keywords, casting with both `cast_with_aftermath: true` and `cast_with_flashback: true`, verifying the engine returns an error. Alternatively, accept coverage through the structural similarity with other tested mutual exclusion blocks.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.127a (permission: cast from graveyard) | Yes | Yes | test_aftermath_cast_second_half_from_graveyard |
| 702.127a (restriction: only from graveyard) | Yes | Yes | test_aftermath_cannot_cast_second_half_from_hand |
| 702.127a (exile on resolution) | Yes | Yes | test_aftermath_exile_on_resolution |
| 702.127a (exile on counter) | Yes | Yes | test_aftermath_exile_on_counter |
| 702.127a (exile on fizzle) | Yes | No | Covered by cast_with_flashback reuse; no aftermath-specific test |
| 709.3 (first half cast normally from hand) | Yes | Yes | test_aftermath_basic_cast_first_half_from_hand |
| 709.3a (chosen half evaluated for casting) | Yes | Yes (partial) | Timing for sorcery aftermath tested; instant aftermath untested |
| 709.3b (only chosen half's characteristics on stack) | Yes (effect selection) | Yes | test_aftermath_uses_aftermath_effect |
| 118.9a (alternative cost mutual exclusion) | Yes | No | Code follows proven pattern; no dedicated test |
| 118.9 (aftermath cost paid as base cost) | Yes | Yes | test_aftermath_pays_aftermath_cost |
| 608.2n (first half to graveyard on resolution) | Yes | Yes | test_aftermath_first_half_goes_to_graveyard |
| Full lifecycle (hand -> graveyard -> aftermath -> exile) | Yes | Yes | test_aftermath_full_lifecycle |
| CR 400.7 (zone-change identity) | Yes | Yes | test_aftermath_full_lifecycle re-finds object after zone change |
| Multiplayer (each opponent loses life) | Yes | Yes | test_aftermath_uses_aftermath_effect (3 players) |

## Architecture Notes

- **Split card representation**: Modeled as single `CardDefinition` with `AbilityDefinition::Aftermath` carrying the second half's data. This is an acceptable simplification; combined characteristics (CR 709.4) are explicitly deferred as a known gap. No real game interaction requires combined MV/color/name in non-stack zones for the initial implementation.
- **Flashback reuse**: `cast_with_flashback` on `StackObject` is set to `true` for both flashback AND aftermath casts (casting.rs:1335). This is a clean reuse of existing infrastructure covering all 4 departure paths without duplication. The separate `cast_with_aftermath` bool on `StackObject` is used only for effect selection at resolution time.
- **No new StackObjectKind**: Aftermath correctly uses `StackObjectKind::Spell` -- it is a spell, not a triggered ability. No new SOK variant was needed.
- **Escape auto-detection gap**: The escape auto-detection at casting.rs:519-523 does not exclude `casting_with_aftermath`, meaning a hypothetical card with both Aftermath and Escape could trigger a false mutual-exclusion rejection. No real MTG card has both keywords, so this is not a practical concern.
