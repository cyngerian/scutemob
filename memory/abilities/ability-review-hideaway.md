# Ability Review: Hideaway

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.75
**Files reviewed**:
- `crates/engine/src/state/types.rs:544` (KeywordAbility::Hideaway)
- `crates/engine/src/state/hash.rs:440-444` (KeywordAbility hash)
- `crates/engine/src/state/hash.rs:1318-1326` (StackObjectKind::HideawayTrigger hash)
- `crates/engine/src/state/hash.rs:2115-2127` (GameEvent::HideawayExiled hash)
- `crates/engine/src/state/hash.rs:2689-2692` (Effect::PlayExiledCard hash)
- `crates/engine/src/state/hash.rs:604-607` (GameObject exiled_by_hideaway hash)
- `crates/engine/src/state/hash.rs:1014-1058` (PendingTrigger hash -- MISSING hideaway fields)
- `crates/engine/src/state/game_object.rs:408-420` (exiled_by_hideaway field)
- `crates/engine/src/state/stack.rs:354-373` (StackObjectKind::HideawayTrigger)
- `crates/engine/src/state/stubs.rs:204-216` (PendingTrigger hideaway fields)
- `crates/engine/src/state/mod.rs:294-295` (exiled_by_hideaway cleared on zone change)
- `crates/engine/src/state/mod.rs:381-382` (exiled_by_hideaway cleared in move_to_bottom)
- `crates/engine/src/cards/card_definition.rs:532-544` (Effect::PlayExiledCard)
- `crates/engine/src/rules/events.rs:842-858` (GameEvent::HideawayExiled)
- `crates/engine/src/rules/resolution.rs:1467-1556` (HideawayTrigger resolution arm)
- `crates/engine/src/rules/abilities.rs:921-972` (ETB trigger generation)
- `crates/engine/src/rules/abilities.rs:2011-2018` (flush_pending_triggers HideawayTrigger branch)
- `crates/engine/src/effects/mod.rs:1592-1696` (Effect::PlayExiledCard execution)
- `crates/engine/tests/hideaway.rs` (7 tests, 675 lines)
- `tools/replay-viewer/src/view_model.rs:467-469,672` (Hideaway keyword + HideawayTrigger stack kind)

## Verdict: needs-fix

One HIGH finding (missing hash fields on PendingTrigger for Hideaway), one MEDIUM finding (incorrect CR citation), and several LOW findings. The core implementation logic is sound and follows the CR correctly for the ETB trigger and resolution. The HIGH finding is a straightforward omission that will cause hash non-determinism when a Hideaway PendingTrigger is in flight.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `hash.rs:1058` | **Missing PendingTrigger hash fields for Hideaway.** `is_hideaway_trigger` and `hideaway_count` not hashed. **Fix:** Add two hash_into calls. |
| 2 | MEDIUM | `abilities.rs:926` | **Incorrect CR citation: 702.75c does not exist.** Comment references non-existent subrule. **Fix:** Remove or correct the citation. |
| 3 | LOW | `resolution.rs:1533` | **Silently discarding move errors.** `let _ =` suppresses errors from `move_object_to_bottom_of_zone`. **Fix:** Log or handle the error. |
| 4 | LOW | `effects/mod.rs:1639,1674` | **Owner overwritten to controller.** `obj.owner = controller` is incorrect per CR 108.3. **Fix:** Remove `obj.owner = controller;` lines. |
| 5 | LOW | `events.rs:848` | **HideawayExiled event not private.** Doc says "private to the controller" but no `private_to` method exists yet. **Fix:** Document as deferred to M10 (network layer). |
| 6 | LOW | `tests/hideaway.rs` | **No test for partial library (fewer than N cards).** Empty library is tested but not 2 cards with Hideaway(4). **Fix:** Add `test_hideaway_partial_library`. |
| 7 | LOW | `tests/hideaway.rs:182-237` | **Test 1 does not test trigger generation.** It manually pushes a trigger onto the stack and asserts it's there. Does not test the ETB path. **Fix:** Use process_command to play the permanent and verify trigger appears. |
| 8 | LOW | `resolution.rs:1519-1528` | **Inline PRNG without crate reuse.** Fisher-Yates with LCG constants duplicated (same pattern could be a shared utility). **Fix:** Extract to shared helper (opportunistic). |

### Finding Details

#### Finding 1: Missing PendingTrigger hash fields for Hideaway

**Severity**: HIGH
**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs:1054-1058`
**CR Rule**: Architecture invariant -- all state fields must be hashed for deterministic verification.
**Issue**: The `HashInto for PendingTrigger` implementation ends at line 1058 after hashing `suspend_card_id`. The two new Hideaway fields -- `is_hideaway_trigger: bool` (stubs.rs:211) and `hideaway_count: Option<u32>` (stubs.rs:216) -- are not included in the hash. This means two `PendingTrigger` values that differ only in their Hideaway fields will produce identical hashes, breaking deterministic state verification (Tier 1 of the network security model).

Every other PendingTrigger boolean/payload pair (exploit, modular, evolve, myriad, suspend) has its hash entries at lines 1043-1057. The Hideaway fields were added to the struct (stubs.rs:204-216) but not to the hash impl.

**Fix**: In `crates/engine/src/state/hash.rs`, after line 1057 (`self.suspend_card_id.hash_into(hasher);`), before the closing `}` at line 1058, add:
```rust
        // CR 702.75a: is_hideaway_trigger -- hideaway ETB trigger marker
        self.is_hideaway_trigger.hash_into(hasher);
        self.hideaway_count.hash_into(hasher);
```

#### Finding 2: Incorrect CR citation (702.75c does not exist)

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:926`
**CR Rule**: CR 702.75 has only subrules 702.75a and 702.75b. There is no 702.75c.
**Issue**: The comment at line 926 states `// Multiple instances trigger separately (CR 702.75c).` but CR 702.75c does not exist. The behavior of multiple Hideaway instances triggering separately is implied by the general rules for triggered abilities (each instance is a separate triggered ability per CR 603.2), not by a Hideaway-specific subrule. Fabricated CR citations violate the project convention that "tests cite their rules source" (architecture invariant #8) and erode trust in other citations.

**Fix**: Change the comment at `crates/engine/src/rules/abilities.rs:926` from:
```rust
// Multiple instances trigger separately (CR 702.75c).
```
to:
```rust
// Multiple instances trigger separately (CR 603.2: each keyword instance
// is a separate triggered ability).
```

#### Finding 3: Silently discarding move errors

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:1533`
**CR Rule**: N/A (code quality)
**Issue**: `let _ = state.move_object_to_bottom_of_zone(card_id, lib_zone);` silently discards any error from moving remaining cards to the library bottom. While this is unlikely to fail in practice (the cards are still in the library), per conventions.md the engine should not silently discard errors.
**Fix**: Either propagate the error with `?`, or at minimum add a comment explaining why the error is safe to ignore. Compare with Cascade at `copy.rs:416` which has an explicit comment explaining the error path.

#### Finding 4: Owner overwritten to controller in PlayExiledCard

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs:1639` and `1674`
**CR Rule**: CR 108.3 -- "The owner of a card in the game is the player who started the game with it in their deck."
**Issue**: Lines 1639 and 1674 set `obj.owner = controller;` which overwrites the card's true owner with the Hideaway permanent's current controller. Per CR 108.3, a card's owner never changes. In normal Hideaway usage the owner and controller are the same player, so this has no observable effect. But if control of a Hideaway permanent changes, the exiled card's owner would be incorrectly set. The `move_object_to_zone` function already correctly preserves `owner = old_object.owner`.
**Fix**: Remove the `obj.owner = controller;` lines at 1639 and 1674. The `obj.controller = controller;` lines are correct and sufficient.

#### Finding 5: HideawayExiled event privacy not enforced

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs:848`
**CR Rule**: CR 406.3 -- face-down exiled cards can't be examined by any player except when instructions allow it. Architecture invariant #7 -- hidden information is enforced.
**Issue**: The doc comment says "this event is private to the controller" but the engine has no `private_to` method on `GameEvent`. The network layer (M10) would need to filter this event, but there's currently no mechanism to identify it as private. This is a pre-existing infrastructure gap, not a Hideaway-specific issue.
**Fix**: No action needed now. Document as deferred to M10. When `private_to` is implemented on `GameEvent`, ensure `HideawayExiled` returns `Some(player)`.

#### Finding 6: No test for partial library edge case

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/hideaway.rs`
**CR Rule**: CR 702.75a -- "look at the top N cards"
**Issue**: Test 4 covers the empty library case. There is no test for a library with fewer than N cards (e.g., Hideaway(4) with 2 library cards). The implementation handles this correctly (the `start` calculation at resolution.rs:1490 clamps to 0), but the edge case is untested.
**Fix**: Add a test `test_hideaway_partial_library` that sets up Hideaway(4) with only 2 library cards and verifies that 1 card is exiled and 1 is put on the bottom.

#### Finding 7: Test 1 does not exercise trigger generation path

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/hideaway.rs:182-237`
**CR Rule**: CR 702.75a -- "When this permanent enters"
**Issue**: `test_hideaway_etb_trigger_fires` manually pushes a `HideawayTrigger` stack object and then asserts it's there. This tests resolution but not the trigger generation path in `abilities.rs:921-972`. The test name implies it verifies that the trigger fires on ETB, but it doesn't actually exercise the `check_triggers` or `flush_pending_triggers` code paths. Test 7 (negative test) does check `pending_triggers` on the state but doesn't verify the positive case through `process_command`.
**Fix**: Either rename the test to `test_hideaway_trigger_resolves` (more accurate) or rewrite it to use `process_command` to move a permanent from hand to battlefield and verify a `HideawayTrigger` appears on the stack.

#### Finding 8: Inline PRNG duplication

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:1519-1528`
**CR Rule**: N/A (code quality)
**Issue**: The Fisher-Yates shuffle with LCG constants is implemented inline. If this same pattern is needed elsewhere (or already exists in another module), it should be a shared utility to avoid constant duplication and ensure consistent seeded randomness.
**Fix**: Opportunistic -- extract the seeded shuffle to a utility function if similar patterns exist elsewhere. Low priority.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.75a (ETB trigger: look at top N) | Yes | Yes (partial) | test_1 manually pushes trigger; trigger generation in abilities.rs:921 not exercised in tests |
| 702.75a (exile one face-down) | Yes | Yes | test_2 (exile count + face_down), test_5 (face_down enforcement) |
| 702.75a (put rest on bottom in random order) | Yes | Yes (implicit) | test_2 checks library count; random order via seeded Fisher-Yates |
| 702.75a (exiled card gains "controller may look") | No (documented gap) | No | The granted ability is implicit via `exiled_by_hideaway` tracking; no explicit ability granted to the exiled card object |
| 702.75b (old cards errata: "Hideaway 4" + enters tapped) | Yes (doc only) | No | Plan documents this as a card-definition concern, not keyword concern. Correct. |
| CR 607.2a (linked abilities) | Yes | Yes | test_3 (exiled_by_hideaway tracking), test_6 (PlayExiledCard finds matching card) |
| CR 406.3 (face-down exile, no examination) | Yes (face_down flag) | Yes | test_5; network-level enforcement deferred to M10 |
| CR 406.3a (turn face up before playing) | Yes | Yes (implicit) | PlayExiledCard sets face_down = false at effects/mod.rs:1629 |
| CR 400.7 (zone change clears identity) | Yes | No | Implemented in mod.rs:295 and mod.rs:382; not explicitly tested for Hideaway |
| Edge: empty library | Yes | Yes | test_4 |
| Edge: partial library (fewer than N) | Yes | No | Implementation correct (resolution.rs:1490 clamps); no test |
| Edge: blink Hideaway permanent | Yes (implicit) | No | CR 400.7 gives new ObjectId; old exiled_by_hideaway won't match. No test. |

## Previous Findings (re-review only)

N/A -- this is the initial review.
