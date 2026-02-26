# Ability Review: Delve

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.66
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 238-243)
- `crates/engine/src/state/hash.rs` (lines 343-344)
- `crates/engine/src/rules/command.rs` (lines 72-80)
- `crates/engine/src/rules/engine.rs` (lines 70-87)
- `crates/engine/src/rules/casting.rs` (lines 47-55, 278-318, 644-722)
- `crates/engine/src/testing/replay_harness.rs` (lines 194-237)
- `crates/engine/src/testing/script_schema.rs` (lines 223-227)
- `crates/engine/tests/script_replay.rs` (lines 135-178)
- `crates/engine/tests/delve.rs` (all 886 lines, 10 tests)
- `tools/replay-viewer/src/view_model.rs` (line 595)

## Verdict: clean

The Delve implementation is correct, complete, and well-tested. All three CR 702.66
subrules are faithfully implemented. The `apply_delve_reduction` function validates
uniqueness, graveyard ownership, and generic mana limits before exiling cards and
reducing the cost. The implementation correctly follows the established `apply_convoke_reduction`
pattern with the appropriate differences: graveyard instead of battlefield, exile instead
of tap, generic-only instead of colored+generic. Hash coverage is correct (discriminant 31).
All 20 test files with `Command::CastSpell` constructions were updated with the
`delve_cards` field (145 occurrences match convoke_creatures exactly). No `.unwrap()`
in engine library code. No HIGH or MEDIUM findings. Three LOW findings noted below
for opportunistic improvement.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `replay_harness.rs:227-230` | **Duplicate graveyard names silently resolve to same ObjectId.** Scripts with multiple copies of a card in the graveyard cannot delve more than one copy. Pre-existing pattern from convoke. |
| 2 | LOW | `casting.rs:278-291` | **Flashback+Delve self-exile produces unhelpful error.** Theoretical interaction only (no real card has both). Engine rejects correctly but error message is confusing. |
| 3 | LOW | `delve.rs:101-106` | **`graveyard_card` helper always creates Instants.** Minor style: the docstring says "any type" but the card is always Instant. Does not affect test correctness. |

### Finding Details

#### Finding 1: Duplicate graveyard names silently resolve to same ObjectId

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:227-230`
**CR Rule**: N/A (replay harness infrastructure, not a rules issue)
**Issue**: The `find_in_graveyard` function returns the first object matching a name
in the player's graveyard. If a script lists `"delve": ["Mountain", "Mountain"]` and
the player has two Mountains in the graveyard, both lookups return the same ObjectId,
causing `apply_delve_reduction` to reject the cast with a "duplicate card" error. This
is the same pre-existing limitation as convoke's `find_on_battlefield` with duplicate
creature names.
**Fix**: Defer. If needed, add a `find_in_graveyard_excluding(state, player, name, &exclude_set)`
variant that skips already-matched ObjectIds. Address alongside the convoke equivalent.

#### Finding 2: Flashback+Delve self-exile produces unhelpful error

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:278-321`
**CR Rule**: 601.2a -- "To propose the casting of a spell, a player first moves that
card from where it is to the stack."
**Issue**: The engine pays costs (including delve exile) before moving the spell to the
stack (line 321). Per CR 601.2a, the spell should move to the stack first, then costs are
determined and paid. For a hypothetical flashback+delve card, a malformed command could
include the spell's own ObjectId in `delve_cards`. The spell would be exiled, then the
`move_object_to_zone(card, ZoneId::Stack)` at line 321 would fail with `ObjectNotFound`
-- a confusing error. No real MTG card has both flashback and delve, so this is theoretical.
The cost-before-move ordering is a pre-existing engine pattern shared with convoke.
**Fix**: Defer. If flashback+delve is ever needed, add a validation check in
`apply_delve_reduction` that rejects `delve_cards` containing the spell's own ObjectId.

#### Finding 3: graveyard_card helper always creates Instants

**Severity**: LOW
**File**: `crates/engine/tests/delve.rs:101-106`
**CR Rule**: 702.66a -- "you may exile a card from your graveyard" (any card type)
**Issue**: The `graveyard_card` helper creates all cards as `CardType::Instant`. The
docstring says "any type -- lands, instants, creatures, etc." but the implementation
always uses `Instant`. This does not affect test correctness since delve has no card-type
restriction on exiled cards, but it would be more illustrative to vary the types (e.g.,
one land, one creature, one instant) in the basic test.
**Fix**: Optional. Vary card types in `test_delve_basic_exile_cards_reduce_generic_cost`
by using `.with_types(vec![CardType::Land])` for some cards. Low priority.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.66a (static ability, exile from graveyard, pay generic) | Yes | Yes | Tests 1-3, 5-8 |
| 702.66a ("your graveyard") | Yes | Yes | Test 7 (opponent graveyard rejection) |
| 702.66a (generic mana only) | Yes | Yes | Test 1 (full delve), Test 2 (partial delve) |
| 702.66a (cannot exceed generic) | Yes | Yes | Test 5 (too many cards rejection) |
| 702.66b (not additional/alternative cost) | Yes | Yes | Test 10 (commander tax applies first, then delve reduces) |
| 702.66b (applies after total cost determined) | Yes | Yes | Ordering in casting.rs: base -> tax -> flashback -> convoke -> delve -> pay |
| 702.66c (multiple instances redundant) | Yes (implicit) | No | No explicit test, but `.contains()` is boolean -- multiple instances have same effect |
| CR 400.7 (zone change = new ObjectId) | Yes | Yes | Test 3 (old IDs retired, new IDs in exile) |
| Treasure Cruise ruling (mana value unchanged) | Yes (implicit) | No | `mana_value()` reads from card cost, not from payment; delve modifies the local `reduced` variable, not the card |
| Duplicate cards rejected | Yes | Yes | Test 8 |
| Zero delve cards (normal cast) | Yes | Yes | Test 9 |
| ObjectExiled events emitted | Yes | Yes | Test 3 (3 events for 3 cards) |

verdict: clean
