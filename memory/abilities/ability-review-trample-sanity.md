# Ability Review: Trample (Sanity Check)

**Date**: 2026-03-09
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.19
**Files reviewed**: `crates/engine/src/rules/combat.rs` (lines 1090-1240, 1635-1657), `crates/engine/src/state/combat.rs`, `crates/engine/src/state/types.rs:265`, `crates/engine/tests/combat.rs` (tests 4-8), `crates/engine/src/rules/replacement.rs:1909-1924`, `test-data/generated-scripts/combat/005_trample_assigns_lethal_then_player.json`

## Verdict: needs-fix

The core trample damage assignment logic is correct: lethal damage is assigned to each
blocker in order, and excess goes to the defending player/planeswalker. Deathtouch +
trample interaction (CR 702.2c) correctly treats 1 damage as lethal. However, a pre-existing
combat bug in `is_blocked()` causes the trample-specific code path to be unreachable when
all blockers die before damage, and there are missing tests for key scenarios.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `combat.rs:1109-1156` | **`is_blocked()` returns false when blockers die, violating CR 509.1h.** Non-trample creatures incorrectly deal damage to the player. |
| 2 | MEDIUM | `combat.rs:1167-1168` | **CR citation wrong: 702.19b should be 702.19d** for "blocked but no blockers remain" case. |
| 3 | MEDIUM | `tests/combat.rs` | **No test for trample + multiple blockers.** Missing test for CR 702.19b with 2+ blockers and trample. |
| 4 | MEDIUM | `tests/combat.rs` | **No test for CR 702.19d.** No test for trample when all blockers removed before damage step. |
| 5 | LOW | `combat.rs:1527-1531` | **Planeswalker damage marked instead of removing loyalty counters.** CR 120.3c says damage to a PW removes loyalty counters directly. |
| 6 | LOW | `combat.rs:1168` | **Trample "all blockers gone" path is dead code.** Due to Finding 1, the `has_trample` branch is never reached; the `!was_blocked` branch fires first. |

### Finding Details

#### Finding 1: `is_blocked()` returns false when blockers die (CR 509.1h violation)

**Severity**: HIGH
**File**: `crates/engine/src/rules/combat.rs:1154-1157` + `crates/engine/src/state/combat.rs:109-111` + `crates/engine/src/rules/replacement.rs:1912`
**CR Rule**: 509.1h -- "An attacking creature with one or more creatures declared as blockers for it becomes a blocked creature [...]. It remains a blocked creature even if all the creatures declared as blockers for it are removed from combat."
**Issue**: When a blocker leaves the battlefield (dies from first-strike damage, etc.), `replacement.rs:1912` removes it from `combat.blockers`. The `is_blocked()` method (combat.rs:109-111) only checks the current `blockers` map, so it returns `false` when all blockers have left. In `apply_combat_damage()`, this causes the creature to be treated as "truly unblocked" (line 1159-1166), dealing full damage to the defending player. This violates CR 509.1h + CR 510.1c: a blocked creature with no remaining blockers should assign no damage at all (unless it has trample).

For trample specifically, the result is accidentally correct (damage goes to player), but via the wrong code path -- making Finding 6 (dead code) a consequence. For non-trample creatures, this is a correctness bug.

**Fix**: Add a `blocked_attackers: OrdSet<ObjectId>` field to `CombatState` that is populated when blockers are declared and never cleared (except at end of combat). Use this set in `is_blocked()` instead of scanning the `blockers` map. Alternatively, do not remove blockers from the `blockers` map when they leave the battlefield -- instead, filter by battlefield presence at damage time (which the code already does at lines 1127-1134).

#### Finding 2: Wrong CR citation for "blocked, no blockers remain" case

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/combat.rs:1168`
**CR Rule**: 702.19d -- "If an attacking creature with trample or trample over planeswalkers is blocked, but there are no creatures blocking it when damage is assigned, its damage is assigned to the defending player and/or planeswalker as though all blocking creatures have been assigned lethal damage."
**Issue**: The comment says "CR 702.19b" but the correct rule for this specific case (blocked, all blockers gone) is CR 702.19d. CR 702.19b covers the general trample overflow rule with blockers still present.
**Fix**: Change comment from `(CR 702.19b)` to `(CR 702.19d)`.

#### Finding 3: No test for trample + multiple blockers

**Severity**: MEDIUM
**File**: `crates/engine/tests/combat.rs`
**CR Rule**: 702.19b -- "The controller of an attacking creature with trample first assigns damage to the creature(s) blocking it. Once all those blocking creatures are assigned lethal damage, any excess damage is assigned [...]"
**Issue**: There is a test for trample with one blocker (`test_702_19_trample_excess_to_player`) and a test for multiple blockers without trample (`test_509_2_multiple_blockers_damage_order`), but no test combining trample with multiple blockers. For example: a 7/7 trampler blocked by [2/2, 2/2] should assign 2 to first blocker, 2 to second blocker, and 3 to the player.
**Fix**: Add a test `test_702_19b_trample_multiple_blockers_excess_to_player` with a large trampler blocked by 2+ creatures, verifying lethal is assigned to each in order and excess goes to the defending player.

#### Finding 4: No test for CR 702.19d (trample, all blockers removed)

**Severity**: MEDIUM
**File**: `crates/engine/tests/combat.rs`
**CR Rule**: 702.19d -- "If an attacking creature with trample [...] is blocked, but there are no creatures blocking it when damage is assigned, its damage is assigned to the defending player [...]"
**Issue**: No test exercises the scenario where a trample creature is blocked, all blockers die before the combat damage step (e.g., from first strike), and the trample creature's full power goes to the defending player. This is an important edge case. Note: due to Finding 1, such a test would pass today but via the wrong code path.
**Fix**: Add a test `test_702_19d_trample_blockers_removed_before_damage` using first strike to kill the blocker before regular damage, then verify the trampler deals full damage to the player.

#### Finding 5: Planeswalker damage uses `damage_marked` instead of removing loyalty counters

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:1527-1531`
**CR Rule**: 120.3c -- "Damage dealt to a planeswalker causes that many loyalty counters to be removed from that planeswalker."
**Issue**: Combat damage to a planeswalker adds to `obj.damage_marked` rather than removing loyalty counters. The comment says "SBA 704.5i handles the loyalty counter check separately" but 704.5i only checks for 0 loyalty and destroys; it doesn't convert damage to loyalty loss. This is a pre-existing planeswalker damage bug not specific to trample, but it would affect trample overflow to a planeswalker attack target.
**Fix**: Out of scope for trample review. File as a separate combat/planeswalker damage bug.

#### Finding 6: Trample "all blockers gone" branch is dead code

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:1167-1174`
**CR Rule**: 702.19d
**Issue**: Due to Finding 1, when all blockers die before damage, `is_blocked()` returns false, so the `!was_blocked` branch at line 1159 fires. The `else if has_trample` branch at line 1167 is never reached. The trample path and the unblocked path both call `push_player_or_pw_damage` with the same args, so the result is correct, but the code is misleading.
**Fix**: Resolving Finding 1 will make this branch reachable. No separate fix needed.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.19a    | Yes         | Implicit | Static ability, no effect when blocking or noncombat. Correctly only checked for attackers. |
| 702.19b    | Yes         | Partial  | Tested with 1 blocker. No test with multiple blockers (Finding 3). |
| 702.19c    | No          | No      | "Trample over planeswalkers" -- no separate keyword. Acceptable deferral. |
| 702.19d    | Buggy*      | No      | Code path exists but is dead code due to Finding 1. No test (Finding 4). |
| 702.19e    | No          | No      | "Trample over planeswalkers" variant. Acceptable deferral. |
| 702.19f    | Yes         | Yes     | Non-trample blocked creature gets no damage -- tests 4, 8 verify. Buggy when all blockers removed (Finding 1). |
| 702.19g    | Yes         | Implicit | Multiple instances redundant -- `has_keyword` is boolean check. |
| 702.2c     | Yes         | Yes     | Deathtouch+trample: `test_702_deathtouch_with_trample` -- 1 damage lethal. |
| 510.1c     | Yes         | Partial  | Multiple blockers tested without trample. Trample+multiple blockers not tested. |
| 510.2      | Yes         | Yes     | Simultaneous damage assignment. |

\* CR 702.19d code exists (line 1167-1174) but is unreachable due to `is_blocked()` bug. Result is accidentally correct via wrong code path.

## Summary of Action Items

1. **(HIGH)** Fix `is_blocked()` to track declaration rather than current presence -- add `blocked_attackers: OrdSet<ObjectId>` to `CombatState`, or stop removing blockers from the map.
2. **(MEDIUM)** Fix CR citation: `702.19b` -> `702.19d` at line 1168.
3. **(MEDIUM)** Add test for trample + multiple blockers.
4. **(MEDIUM)** Add test for trample + all blockers removed before damage (CR 702.19d).
5. **(LOW)** Planeswalker damage bug (out of scope, file separately).
