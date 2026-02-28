# Ability Review: Annihilator

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.86
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 263-269)
- `crates/engine/src/state/hash.rs` (lines 351-355, 911-912, 2223-2228)
- `crates/engine/src/state/stubs.rs` (lines 75-83)
- `crates/engine/src/state/builder.rs` (lines 418-436)
- `crates/engine/src/cards/card_definition.rs` (lines 344-354)
- `crates/engine/src/effects/mod.rs` (lines 1027-1148)
- `crates/engine/src/rules/abilities.rs` (lines 656-678, 1000-1007)
- `tools/replay-viewer/src/view_model.rs` (line 599)
- `crates/engine/tests/annihilator.rs` (full file, 625 lines, 8 tests)

## Verdict: clean

The Annihilator implementation is correct, well-structured, and faithfully implements CR 702.86a
and 702.86b. The triggered ability auto-generation in `builder.rs` correctly uses
`TriggerEvent::SelfAttacks` and routes the defending player through
`PendingTrigger.defending_player_id`, which is resolved at flush time to
`Target::Player(dp)` at index 0. The `SacrificePermanents` effect correctly bypasses
indestructible (consistent with CR 701.21a), handles fewer-than-N permanents gracefully
via `min(n, count)`, and routes through the replacement effect system for commanders.
The multiplayer defending player resolution in `abilities.rs` (line 670) correctly handles
both `AttackTarget::Player` and `AttackTarget::Planeswalker` per CR 508.5. Hash coverage
is complete for all new fields and variants. The 8 tests cover all planned scenarios
including multiplayer, planeswalker attacks, indestructible, multiple instances, and edge
cases. No HIGH or MEDIUM findings. Two LOW findings relate to incorrect CR citations
(701.17a cited where 701.21a is correct) and a minor semantic event mismatch.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `crates/engine/src/effects/mod.rs:1027` | **Incorrect CR citation for sacrifice.** Comments cite CR 701.17a (Mill) instead of CR 701.21a (Sacrifice). Same error in card_definition.rs:344, hash.rs:2223, and tests/annihilator.rs:9-10,206,248,431,474. **Fix:** Replace all "CR 701.17a" references with "CR 701.21a" in the SacrificePermanents-related comments and test docstrings. |
| 2 | LOW | `crates/engine/src/effects/mod.rs:1099` | **Semantic event mismatch for non-creature sacrifice.** SacrificePermanents emits `PermanentDestroyed` for non-creature permanents, but `PermanentDestroyed` is documented as "A non-creature permanent was **destroyed** by a spell or ability (CR 701.7)." Sacrifice is not destruction (CR 701.21a). No functional impact today since no trigger keys off `PermanentDestroyed`, but a future "whenever a permanent is destroyed" trigger would incorrectly fire on sacrifice. **Fix:** When a dedicated `PermanentSacrificed` event is added (or a general `PermanentLeftBattlefield` event), update SacrificePermanents to use it. Deferred -- no fix needed now. |

### Finding Details

#### Finding 1: Incorrect CR citation for sacrifice

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:1027`, `crates/engine/src/cards/card_definition.rs:344`, `crates/engine/src/state/hash.rs:2223`, `crates/engine/tests/annihilator.rs:9-10,206,248,431,474`
**CR Rule**: 701.21a -- "To sacrifice a permanent, its controller moves it from the battlefield directly to its owner's graveyard. A player can't sacrifice something that isn't a permanent, or something that's a permanent they don't control. Sacrificing a permanent doesn't destroy it, so regeneration or other effects that replace destruction can't affect this action."
**Issue**: All comments and test docstrings cite "CR 701.17a" for sacrifice. CR 701.17 is actually the Mill rule ("For a player to mill a number of cards, that player puts that many cards from the top of their library into their graveyard."). The correct sacrifice rule is CR 701.21a. The plan originally had this wrong and the implementation carried the error forward. The behavior is correct -- only the citation is wrong.
**Fix**: Replace all occurrences of "CR 701.17a" with "CR 701.21a" in:
- `crates/engine/src/effects/mod.rs` lines 1027, 1029, 1051
- `crates/engine/src/cards/card_definition.rs` lines 344, 349
- `crates/engine/src/state/hash.rs` line 2223
- `crates/engine/tests/annihilator.rs` lines 9, 10, 206, 248, 431, 474

#### Finding 2: Semantic event mismatch for non-creature sacrifice

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:1099`
**CR Rule**: 701.21a -- "Sacrificing a permanent doesn't destroy it"
**Issue**: When a non-creature permanent is sacrificed, the effect emits `GameEvent::PermanentDestroyed`, whose documentation says "A non-creature permanent was destroyed by a spell or ability (CR 701.7)." Sacrifice is explicitly not destruction. Currently no trigger handler matches on `PermanentDestroyed`, so this has no functional impact. However, if a future card triggers "whenever a permanent is destroyed," it would incorrectly fire on sacrifice. The `CreatureDied` event used for creatures is less problematic because "dies" in MTG means "is put into a graveyard from the battlefield" (CR 700.4), which includes sacrifice.
**Fix**: Deferred. When a dedicated `PermanentSacrificed` event or a unified `PermanentPutIntoGraveyard` event is added, update the SacrificePermanents effect handler. No immediate fix needed.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.86a (annihilator is a triggered ability) | Yes | Yes | test 1, 2, 3 |
| 702.86a ("Whenever this creature attacks") | Yes | Yes | TriggerEvent::SelfAttacks in builder.rs |
| 702.86a ("defending player sacrifices N permanents") | Yes | Yes | test 1, 2 verify defending player only |
| 702.86b (multiple instances trigger separately) | Yes | Yes | test 4 |
| 508.5 (defending player = player being attacked / PW controller) | Yes | Yes | test 5 (multiplayer), test 7 (planeswalker) |
| 508.5a (multiplayer: each trigger individually determined) | Yes | Yes | test 5 |
| 701.21a (sacrifice ignores indestructible) | Yes | Yes | test 6 |
| 701.21a (fewer than N: sacrifice all controlled) | Yes | Yes | test 3 |
| Edge: zero permanents | Yes | Yes | test 8 |
| Edge: commander sacrifice -> zone-change replacement | Yes | No | Replacement effect system is invoked (lines 1062-1070) but no test covers commander sacrifice specifically |

## Previous Findings (re-review only)

N/A -- first review.
