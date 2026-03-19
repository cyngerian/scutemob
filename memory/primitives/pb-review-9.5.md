# Primitive Batch Review: PB-9.5 -- Architecture Cleanup

**Date**: 2026-03-17
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 603.3 (triggered abilities), CR 605.3b (mana abilities don't use stack)
**Engine files reviewed**: `crates/engine/src/rules/engine.rs`, `crates/engine/src/rules/combat.rs`
**Card defs reviewed**: 0 (engine-only batch)

## Verdict: needs-fix

PB-9.5 partially achieved its goals. Fix B (test file CardDefinition defaults) is fully
complete -- all 114 test files with CardDefinition constructions use `..Default::default()`.
Fix A (trigger flush discipline) took a different approach than planned: instead of a single
post-match call, it extracted the 4-line pattern into a `check_and_flush_triggers` helper
called from within each match arm that needs it. This is functionally correct and a clear
improvement, but it leaves combat.rs with the old inline pattern (duplicated logic), and
three commands that perform zone changes (ForetellCard, PlotCard, SuspendCard) lack trigger
flush. One finding is MEDIUM (missing trigger flush on zone-change commands), one is LOW
(combat.rs still uses the old inline pattern instead of the helper).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `engine.rs:325-351` | **Missing trigger flush on ForetellCard, PlotCard, SuspendCard.** These commands exile cards from hand, which could trigger "whenever a card is exiled" or similar abilities. **Fix:** Add `check_and_flush_triggers(&mut state, &mut events);` to all three command arms (requires changing `let events =` to `let mut events =`). |
| 2 | **LOW** | `combat.rs:487-494,1109-1119` | **Duplicate inline trigger flush in combat.rs.** DeclareAttackers and DeclareBlockers handlers use the old 4-line inline pattern instead of calling `check_and_flush_triggers`. Functionally correct but inconsistent. **Fix:** Import and call `check_and_flush_triggers` from `combat.rs` (requires making it `pub(super)` or `pub(crate)`), or leave as-is since it works. |
| 3 | **LOW** | `gotchas-infra.md:81-85` | **Stale documentation.** The trigger flush gotcha still references the old 2-call pattern. Should mention `check_and_flush_triggers` helper. **Fix:** Update the gotcha text to reference the helper function. |

### Finding Details

#### Finding 1: Missing trigger flush on ForetellCard, PlotCard, SuspendCard

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/engine.rs:325-351`
**CR Rule**: CR 603.3 -- "Once an ability has triggered, its controller puts it on the stack as an object that's not a card the next time a player would receive priority."
**Issue**: `ForetellCard` (line 325), `PlotCard` (line 334), and `SuspendCard` (line 345) all perform zone changes (hand to exile) that emit events. If any permanent on the battlefield has a triggered ability watching for exile events (e.g., "whenever a card is exiled from anywhere"), the trigger would be silently dropped because `check_and_flush_triggers` is never called. While no current card definitions trigger on hand-to-exile zone changes, this is a correctness gap that could manifest as cards are authored. The plan specifically called out DeclareAttackers/DeclareBlockers as having this problem, but those are handled internally by combat.rs -- these three commands have the same gap.
**Fix**: In each of the three command arms, change `let events =` to `let mut events =` and add `check_and_flush_triggers(&mut state, &mut events);` before `all_events.extend(events);`.

#### Finding 2: Duplicate inline trigger flush in combat.rs

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:487-494,1109-1119`
**CR Rule**: CR 603.3
**Issue**: `handle_declare_attackers` and `handle_declare_blockers` use the old 4-line inline pattern (`check_triggers` + for-loop + `flush_pending_triggers`) instead of calling the centralized `check_and_flush_triggers` helper. The logic is identical and correct, but maintaining two copies of the same pattern is technical debt. The engine.rs match arms for these commands correctly skip `check_and_flush_triggers` since combat.rs handles it internally, so there is no double-flush bug.
**Fix**: Either refactor combat.rs to call `check_and_flush_triggers` (requires making it `pub(crate)`) or document why combat.rs uses the inline pattern (it needs to flush triggers before granting priority, which is done as part of the same function). LOW priority -- no functional impact.

#### Finding 3: Stale gotcha documentation

**Severity**: LOW
**File**: `memory/gotchas-infra.md:81-85`
**Issue**: The trigger flush gotcha still says "must call `check_triggers()` + `flush_pending_triggers()` after its action" but the centralized helper `check_and_flush_triggers` now exists for this purpose.
**Fix**: Update the gotcha to reference `check_and_flush_triggers` as the preferred approach.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 603.3 (triggered ability queueing) | Yes | Yes (indirect) | Covered by existing test suite across 100+ test files |
| CR 605.3b (mana abilities don't use stack) | Yes | Yes | TapForMana correctly skips trigger flush |

## Card Def Summary

N/A -- PB-9.5 is engine-only with 0 card definitions.

## Fix A Assessment: Trigger Flush Discipline

**Approach taken**: Helper function `check_and_flush_triggers` called per-arm (25 call sites).
**Planned approach**: Single post-match call.
**Assessment**: The per-arm approach is actually more correct than the planned post-match approach. A post-match call would flush triggers for commands that should NOT flush (PassPriority, TapForMana, Concede, OrderBlockers, OrderReplacements, mulligan commands). The per-arm approach allows selective application. The helper function successfully eliminates the copy-paste pattern within engine.rs.

Commands correctly WITHOUT trigger flush:
- PassPriority (no state changes)
- Concede (player removal, no trigger-producing events)
- TapForMana (CR 605.3b: mana abilities resolve immediately)
- DeclareAttackers/DeclareBlockers (handled internally in combat.rs)
- OrderBlockers (pure ordering, no events)
- OrderReplacements (pure ordering, no events)
- ReturnCommanderToCommandZone (zone change, but SBA-driven)
- LeaveCommanderInZone (clears pending state only)
- TakeMulligan/KeepHand (pre-game)
- BringCompanion (zone change, but pre-game special action)
- ChooseDungeonRoom (no-op)

Commands questionably WITHOUT trigger flush (Finding 1):
- ForetellCard (exiles from hand)
- PlotCard (exiles from hand)
- SuspendCard (exiles from hand with counters)

## Fix B Assessment: Test File CardDefinition Defaults

**Status**: COMPLETE.
All 114 test files containing `CardDefinition { ... }` constructions now use `..Default::default()`. Zero remaining explicit-only constructions found. Future CardDefinition field additions only need to update `impl Default for CardDefinition` in `card_definition.rs`.
