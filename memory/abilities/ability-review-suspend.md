# Ability Review: Suspend

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.62
**Files reviewed**:
- `crates/engine/src/state/types.rs:531-543`
- `crates/engine/src/cards/card_definition.rs:246-254`
- `crates/engine/src/state/hash.rs` (KeywordAbility:438-439, GameObject:599-601, StackObjectKind:1286-1305, StackObject:1359-1361, PendingTrigger:1007-1046, GameEvent:2081-2093, AbilityDefinition:2791-2799)
- `crates/engine/src/state/game_object.rs:402-407` (is_suspended field)
- `crates/engine/src/state/stubs.rs:189-203` (PendingTrigger suspend fields)
- `crates/engine/src/state/stack.rs:116-126,329-351` (was_suspended, SuspendCounterTrigger, SuspendCastTrigger)
- `crates/engine/src/state/mod.rs:292-293` (CR 400.7 zone-change reset)
- `crates/engine/src/rules/suspend.rs` (full, 197 lines)
- `crates/engine/src/rules/command.rs:345-356` (SuspendCard command)
- `crates/engine/src/rules/events.rs:778-793` (CardSuspended event)
- `crates/engine/src/rules/engine.rs:303-310` (handler dispatch)
- `crates/engine/src/rules/turn_actions.rs:33-97` (upkeep_actions)
- `crates/engine/src/rules/abilities.rs:1923-1940` (flush_pending_triggers)
- `crates/engine/src/rules/resolution.rs:266-274,1278-1458` (counter/cast resolution, haste grant)
- `crates/engine/tests/suspend.rs` (full, 9 tests, 781 lines)
- `tools/replay-viewer/src/view_model.rs:461-465,668` (display arms)

## Verdict: needs-fix

The Suspend implementation is well-structured and correctly handles the core flow: special-action exile from hand, upkeep counter-removal triggers, last-counter cast trigger, and creature haste grant. The CR 400.7 zone-change identity reset, intervening-if checks (CR 603.4), and multiplayer scoping are all correct. However, there is one HIGH finding (missing hash fields for PendingTrigger) and two MEDIUM findings (a vacuous test assertion and a missing priority-reset after the suspend cast). These must be addressed before the implementation can be considered clean.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `hash.rs:1045` | **Missing PendingTrigger hash fields.** Three new suspend fields not hashed. **Fix:** add hash lines. |
| 2 | MEDIUM | `tests/suspend.rs:543` | **Vacuous assertion in test 5.** `spent_on_cast = false` is always true. **Fix:** check ManaCostPaid events. |
| 3 | MEDIUM | `resolution.rs:1428` | **Priority not reset after suspend cast.** Spell placed on stack but `players_passed` not cleared. **Fix:** add priority reset. |
| 4 | LOW | `resolution.rs:1447` | **Silent error swallowing.** `Err(_) => {}` hides move_object_to_zone failures. **Fix:** log or emit event. |
| 5 | LOW | `suspend.rs:106-119` | **Cast-prohibition effects not checked.** CR 702.62c requires considering "effects that would prohibit" casting. **Fix:** deferred; note in code comment. |
| 6 | LOW | `resolution.rs:1445` | **Unused `is_creature` binding.** `let _ = is_creature` is dead code; haste is handled via `was_suspended` flag at ETB. **Fix:** remove dead code. |

### Finding Details

#### Finding 1: Missing PendingTrigger hash fields for Suspend

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:1045`
**CR Rule**: Architecture invariant -- every field in a hashed struct must be included in the hash.
**Issue**: The `PendingTrigger` struct gained three new fields in this implementation: `is_suspend_counter_trigger` (stubs.rs:189), `is_suspend_cast_trigger` (stubs.rs:197), and `suspend_card_id` (stubs.rs:203). The `HashInto for PendingTrigger` implementation at hash.rs:1007-1046 ends at `is_myriad_trigger` (line 1045) and does NOT include any of the three new suspend fields. This means two game states with different pending suspend triggers will produce identical hashes, violating the deterministic hash invariant.

Every other trigger-type field added to PendingTrigger (is_evoke_sacrifice, is_madness_trigger, is_miracle_trigger, is_unearth_trigger, is_exploit_trigger, is_modular_trigger, is_evolve_trigger, is_myriad_trigger) has a corresponding line in the hash impl. The suspend fields are the only ones missing.

**Fix**: Add three lines after `self.is_myriad_trigger.hash_into(hasher);` (line 1045):
```rust
// CR 702.62a: is_suspend_counter_trigger -- suspend upkeep trigger marker
self.is_suspend_counter_trigger.hash_into(hasher);
// CR 702.62a: is_suspend_cast_trigger -- suspend cast trigger marker
self.is_suspend_cast_trigger.hash_into(hasher);
self.suspend_card_id.hash_into(hasher);
```

#### Finding 2: Vacuous assertion in test_suspend_cast_without_paying_mana_cost

**Severity**: MEDIUM
**File**: `crates/engine/tests/suspend.rs:543-547`
**CR Rule**: 702.62d -- "Casting a spell as an effect of its suspend ability follows the rules for paying alternative costs"
**Issue**: Test 5 (`test_suspend_cast_without_paying_mana_cost`) is supposed to verify that no mana is spent when the suspend cast trigger resolves. However, the actual assertion is:
```rust
let spent_on_cast = false; // Structural: suspend.rs never calls pay_cost in cast trigger
assert!(!spent_on_cast, "CR 702.62d: no mana should be spent...");
```
This is a hardcoded `false` -- the assertion always passes regardless of engine behavior. It does not test anything at runtime. A regression that accidentally adds mana payment to the suspend cast path would not be caught by this test.

**Fix**: Replace the vacuous assertion with one that checks the events for the absence of a `ManaCostPaid` event during the cast trigger resolution round. For example:
```rust
// Advance Untap -> Upkeep -> counter trigger -> cast trigger resolves
let (state, _) = pass_all(state, &[p1, p2]);
let (state, _) = pass_all(state, &[p1, p2]);
let (state, cast_events) = pass_all(state, &[p1, p2]);

// Verify no ManaCostPaid event was emitted during the cast trigger resolution.
let mana_paid_events: Vec<_> = cast_events
    .iter()
    .filter(|e| matches!(e, GameEvent::ManaCostPaid { .. }))
    .collect();
assert!(
    mana_paid_events.is_empty(),
    "CR 702.62d: no ManaCostPaid event should fire when casting via suspend trigger; found {:?}",
    mana_paid_events,
);
```

#### Finding 3: Priority not reset after suspend cast places spell on stack

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:1428`
**CR Rule**: 702.62a / 116.3b -- After a spell is cast, the active player receives priority. Casting a spell during trigger resolution should reset the priority pass tracking.
**Issue**: When `SuspendCastTrigger` resolves, the spell is pushed onto the stack (line 1428: `state.stack_objects.push_back(suspend_stack_obj)`), but `state.turn.players_passed` is never cleared. Compare with the standard casting flow in `casting.rs` which resets `players_passed = OrdSet::new()` after placing a spell on the stack, and the cycling/channel activation flow in `abilities.rs` which does the same. Without this reset, the priority system may incorrectly believe all players have already passed for the newly-cast spell, potentially auto-resolving it.

In practice this may be partially mitigated by the `resolve_top_of_stack` function resetting priority after the *trigger* resolves (the SuspendCastTrigger itself), but the spell cast DURING that resolution should independently reset the pass tracking so that players get a chance to respond to the newly-cast spell before it resolves.

**Fix**: After `state.stack_objects.push_back(suspend_stack_obj);` (resolution.rs:1428), add:
```rust
// CR 116.3b: Casting a spell resets priority. All players must
// pass again before the newly-cast suspend spell resolves.
state.turn.players_passed = im::OrdSet::new();
```

#### Finding 4: Silent error swallowing on move_object_to_zone failure

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1447-1449`
**CR Rule**: Architecture invariant -- no silent failures in engine logic
**Issue**: The `SuspendCastTrigger` resolution arm has `Err(_) => { // Card disappeared -- nothing to cast. }` which silently swallows the error from `move_object_to_zone`. While this path is unlikely in normal gameplay, silently ignoring errors can mask bugs. Other resolution arms (e.g., cascade) follow a similar pattern, so this is consistent but still suboptimal.
**Fix**: Consider emitting a diagnostic event or at minimum logging the error in a comment that explains why it's safe to ignore. Low priority since it matches the existing pattern.

#### Finding 5: Cast-prohibition effects not checked (CR 702.62c)

**Severity**: LOW
**File**: `crates/engine/src/rules/suspend.rs:106-119`
**CR Rule**: 702.62c -- "While determining if you could begin to cast a card with suspend, take into consideration any effects that would prohibit that card from being cast."
**Issue**: The timing check in `suspend.rs` validates sorcery-speed constraints (active player, main phase, empty stack) but does not check for cast-prohibition effects (e.g., Iona naming the card's color, Void Winnower preventing even-cost spells, Rule of Law preventing second spells). This is a systemic gap -- the engine does not yet have a centralized "can this card be cast" predicate. All other cast-from-hand paths (normal casting, foretell) share this gap.
**Fix**: Add a code comment noting the gap. Address when a general cast-prohibition system is implemented. No suspend-specific fix needed.

#### Finding 6: Unused is_creature binding (dead code)

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1441-1445`
**CR Rule**: N/A (code quality)
**Issue**: The `is_creature` variable is computed at line 1398-1402 but then explicitly ignored with `let _ = is_creature;` at line 1445. The comment says "used at permanent ETB time via was_suspended flag" but the variable itself is never actually used -- the haste grant is handled entirely via the `was_suspended` flag check at resolution.rs:270-274. This is dead code that creates a false impression of being used.
**Fix**: Remove the `is_creature` binding and the `let _ = is_creature` line. Replace with a comment: `// Haste grant for creature spells is handled at spell resolution time (line ~270) via the was_suspended flag.`

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.62a (static: exile from hand) | Yes | Yes | test_suspend_basic_exile_from_hand |
| 702.62a (triggered: upkeep counter removal) | Yes | Yes | test_suspend_counter_removal_on_upkeep |
| 702.62a (triggered: cast when last counter removed) | Yes | Yes | test_suspend_last_counter_triggers_cast |
| 702.62a (creature haste) | Yes | Yes | test_suspend_creature_gains_haste |
| 702.62a (face up exile) | Yes | Yes | Checked in test 1 (face_down == false) |
| 702.62b (suspended = exile + suspend + time counters) | Yes | Yes | upkeep_actions filter checks all 3; test_suspend_no_longer_suspended_after_cast |
| 702.62c (timing restrictions) | Partial | No | Sorcery-speed timing checked; cast-prohibition effects NOT checked (Finding 5) |
| 702.62d (alternative cost rules) | Yes | Weak | test_suspend_cast_without_paying_mana_cost has vacuous assertion (Finding 2) |
| 116.2f (special action, any time with priority) | Yes | Yes | Priority check in suspend.rs:62 |
| 116.2f (does not use the stack) | Yes | Yes | No stack object created for the special action |
| Multiplayer: only owner's upkeep | Yes | Yes | test_suspend_not_active_player_upkeep_no_trigger (4 players) |
| Intervening-if at resolution (CR 603.4) | Yes | No | Checked in code (counter trigger + cast trigger) but no test for "card left exile before trigger resolves" |
| CR 400.7 (zone change = new object) | Yes | Yes | move_object_to_zone resets is_suspended; test_suspend_no_longer_suspended_after_cast |
| Invalid: not in hand | Yes | Yes | test_suspend_invalid_not_in_hand |
| Invalid: no Suspend keyword | Yes | Yes | test_suspend_invalid_no_keyword |
| Stifle interaction (counter trigger) | No | No | Plan notes it; implementation supports it structurally (trigger on stack can be countered) but no explicit test |
| Stifle interaction (cast trigger) | No | No | Same as above |
| Cards with no mana cost | No | No | Ancestral Vision pattern; deferred per plan |

## Test Summary

The 9 tests cover the core positive and negative paths well. The multiplayer test (test 9) validates owner-only upkeep scoping in a 4-player game. Key gaps are:
1. Test 5 has a vacuous assertion (Finding 2) -- needs a real runtime check.
2. No test for the intervening-if "card left exile before trigger resolves" edge case.
3. No test for Stifle-type interaction (countering the suspend triggers).
4. No test for cards with no mana cost (Ancestral Vision pattern).

Items 2-4 are acceptable deferrals for V1 given the plan's scope. Item 1 must be fixed.
