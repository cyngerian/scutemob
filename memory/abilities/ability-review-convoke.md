# Ability Review: Convoke

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.51
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 231-237)
- `crates/engine/src/state/hash.rs` (line 341-342)
- `crates/engine/src/rules/command.rs` (line 58-72)
- `crates/engine/src/rules/casting.rs` (line 254-270 integration, line 499-620 apply_convoke_reduction)
- `crates/engine/src/rules/engine.rs` (line 70-80)
- `crates/engine/src/testing/replay_harness.rs` (line 202, 220-231)
- `crates/engine/src/testing/script_schema.rs` (line 218-222)
- `crates/engine/tests/convoke.rs` (12 tests)
- `tools/replay-viewer/src/view_model.rs` (line 594)
- `tools/replay-viewer/src/replay.rs` (line 204, 216)

## Verdict: clean

The Convoke implementation is correct against CR 702.51a-d. All four subrules are
addressed. The cost reduction logic correctly handles colored mana matching in WUBRG
order, generic mana fallback, all five validation checks (battlefield, controller,
untapped, creature type, uniqueness), and the "too many creatures" rejection. The
ordering (total cost determination BEFORE convoke, per CR 702.51b) is correct. The
commander tax integration is validated by test. The harness, replay viewer, and all
existing CastSpell call sites have been updated with `convoke_creatures: vec![]`. No
HIGH or MEDIUM findings. Three LOW findings noted for completeness.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `replay_harness.rs:221-224` | **Duplicate-name convoke creatures in harness.** Harness uses `find_on_battlefield` which returns only the first match; duplicate names in the convoke array yield duplicate ObjectIds. **Fix:** Not blocking -- document limitation or add index-based lookup. |
| 2 | LOW | `convoke.rs` (all tests) | **Missing test for duplicate ObjectId rejection.** The validation code at `casting.rs:508-516` rejects duplicate creature IDs, but no test covers this path. **Fix:** Add a small test that passes the same ObjectId twice and asserts an error. |
| 3 | LOW | `casting.rs:568` | **Convoke on spell with `mana_cost: None`.** If a spell has Convoke but no mana cost, `cost.unwrap_or_default()` produces all-zero ManaCost; any creature then hits "too many creatures." Technically correct (nothing to reduce), but the error message is misleading. **Fix:** Not blocking -- add a guard or clearer error for None-cost convoke if desired. |

### Finding Details

#### Finding 1: Duplicate-name convoke creatures in harness

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs:221-224`
**CR Rule**: 702.51a
**Issue**: The convoke name resolution loop calls `find_on_battlefield(state, player, name)` for
each entry in `convoke_names`. If a script lists the same creature name twice (e.g.,
`["Saproling Token", "Saproling Token"]`), `find_on_battlefield` returns the same `ObjectId`
both times, producing a duplicate in `convoke_ids`. The engine's `apply_convoke_reduction`
then rejects it ("duplicate creature"). This means game scripts cannot use convoke with
multiple identically-named tokens unless each has a unique name (e.g., "Saproling Token 1").
This is a known limitation pattern (see the `find_on_battlefield_by_name` doc comment at
line 721-725 which acknowledges the same issue for blockers).
**Fix**: Document in `gotchas-infra.md` under Script Harness Gotchas that convoke scripts must
use unique creature names. Optionally, add index-based lookup in a future enhancement.

#### Finding 2: Missing test for duplicate ObjectId rejection

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/convoke.rs`
**CR Rule**: N/A (validation code at `casting.rs:508-516`)
**Issue**: The engine correctly validates uniqueness of `convoke_creatures` entries and returns
an error for duplicates. However, no test exercises this validation path. While the code is
correct, an untested code path is a minor gap.
**Fix**: Add `test_convoke_reject_duplicate_creature` that passes the same ObjectId twice and
asserts the error contains "duplicate".

#### Finding 3: Convoke on spell with no mana cost

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs:568`
**CR Rule**: 702.51a -- "For each colored mana in this spell's total cost..."
**Issue**: If a spell has Convoke and `mana_cost: None`, `cost.unwrap_or_default()` produces
`ManaCost { generic: 0, white: 0, ... }`. Any convoke creature then fails at the "too many
creatures" branch with "exceeds total cost." This is technically correct (there is no cost to
reduce), but the error message is misleading -- it sounds like the player provided too many
creatures, when the real issue is the spell has no cost. In practice, spells with no mana cost
and Convoke essentially don't exist in Magic, so this is purely theoretical.
**Fix**: Optionally add an early-return with a clearer error like "spell has no mana cost; convoke
has no effect" when `cost.is_none() && !convoke_creatures.is_empty()`. Not blocking.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.51a (static ability, colored/generic reduction) | Yes | Yes | Tests 1, 2, 3, 12 cover colored matching and generic fallback |
| 702.51a (untapped creature requirement) | Yes | Yes | Test 5 (reject tapped) |
| 702.51a (creature type requirement) | Yes | Yes | Test 6 (reject non-creature) |
| 702.51a (controller requirement) | Yes | Yes | Test 7 (reject opponent's creature) |
| 702.51a (too many creatures) | Yes | Yes | Test 8 (reject too many) |
| 702.51b (applies after total cost determined) | Yes | Yes | Test 9 (commander tax + convoke) |
| 702.51b (not additional/alternative cost) | Yes | Yes | Code order: base -> tax/flashback -> convoke -> pay |
| 702.51c (creature "convoked" that spell) | N/A | N/A | Flavor text; no mechanical implication in engine |
| 702.51d (multiple instances redundant) | Yes | N/A | Boolean presence check; no special handling needed |
| Ruling: summoning sickness no effect | Yes | Yes | Test 10 |
| Ruling: multicolored creature picks one color | Yes | Yes | Test 12 (Selesnya creature) |
| Ruling: no convoke = normal cast | Yes | Yes | Test 11 (zero creatures) |
| Ruling: no keyword = rejected | Yes | Yes | Test 4 |

## Additional Verification Notes

1. **Hash coverage**: `KeywordAbility::Convoke` is assigned discriminant 30 in
   `state/hash.rs:341-342`. No payload to hash (bare variant). Correct.

2. **Match arm sweep**: Verified exhaustive matches at `hash.rs` (HashInto),
   `view_model.rs` (format_keyword). Both include the Convoke arm.

3. **Command field propagation**: `CastSpell { convoke_creatures }` is destructured in
   `engine.rs:70-74` and passed to `handle_cast_spell`. All existing test files pass
   `convoke_creatures: vec![]` (verified via grep -- 100+ occurrences across 15 test files).

4. **Replay harness**: `translate_player_action` signature updated to accept `convoke_names`
   parameter. Script schema `PlayerAction` has `convoke: Vec<String>` with `#[serde(default)]`.
   Replay viewer's `replay.rs` destructures and passes `convoke` correctly.

5. **Event ordering**: Convoke `PermanentTapped` events are collected in `convoke_events`
   first, then emitted AFTER `ManaCostPaid` (line 294-296). This matches CR 601.2h ordering:
   all cost payments happen together, with tapping as part of cost payment.

6. **Multiplayer correctness**: The controller check (`obj.controller != player`) correctly
   prevents using any opponent's creature in N-player games. No APNAP ordering issues since
   convoke is part of a single player's casting action.

7. **Layer correctness**: The creature type check at `casting.rs:550-561` uses
   `calculate_characteristics` which respects continuous effects. A permanent that is
   currently a creature due to an animation effect can legally convoke.

verdict: clean
