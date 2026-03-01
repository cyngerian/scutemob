# Ability Review: Decayed

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.147
**Files reviewed**:
- `crates/engine/src/state/types.rs:616-628` (enum variant)
- `crates/engine/src/state/game_object.rs:399-408` (flag field)
- `crates/engine/src/state/hash.rs:465-466` (keyword discriminant), `hash.rs:627` (flag hash)
- `crates/engine/src/state/mod.rs:292-293, 381-382` (zone-change reset, both sites)
- `crates/engine/src/state/builder.rs:746` (flag init)
- `crates/engine/src/effects/mod.rs:2461` (flag init in token creation)
- `crates/engine/src/rules/resolution.rs:1233` (flag init in myriad token creation)
- `crates/engine/src/rules/combat.rs:285-301` (attack tagging), `combat.rs:396-402` (blocking restriction)
- `crates/engine/src/rules/turn_actions.rs:606-702` (EOC sacrifice)
- `tools/replay-viewer/src/view_model.rs:683` (keyword display arm)
- `crates/engine/tests/decayed.rs` (8 tests)

## Verdict: clean

The Decayed implementation is correct and complete for CR 702.147a. Both sub-abilities
(static "can't block" and triggered "sacrifice at EOC") are faithfully implemented. The
flag-based tag-on-attack / check-at-EOC pattern correctly handles the 2021-09-24 ruling
that sacrifice persists even after losing the keyword. Zone-change resets, hash coverage,
replacement effect handling, and all exhaustive match arms are accounted for. Eight tests
cover positive cases, negative cases, and the key edge case (keyword removal persistence).
No HIGH or MEDIUM findings. Two LOW findings noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `decayed.rs:390` | **Weak disjunctive assertion.** Test 5 uses `creature_died \|\| is_in_graveyard(...)` instead of asserting both independently. **Fix:** Split into two separate assertions like test 4. |
| 2 | LOW | `turn_actions.rs:615-619` | **Delayed trigger TODO is accurate but shared with Myriad.** The TBA implementation bypasses the stack (cannot be Stifled). Documented. No fix needed for V1. |

### Finding Details

#### Finding 1: Weak Disjunctive Assertion in Test 5

**Severity**: LOW
**File**: `crates/engine/tests/decayed.rs:390`
**CR Rule**: 702.147a -- "When this creature attacks, sacrifice it at end of combat."
**Issue**: The assertion `creature_died || is_in_graveyard(...)` would pass if only one
condition is true. In contrast, test 4 (`test_702_147_decayed_creature_sacrificed_at_eoc`)
correctly asserts both `CreatureDied` event emission AND graveyard presence independently.
Test 5 should do the same to fully validate that the sacrifice occurs with proper event
emission even after keyword removal.
**Fix:** Replace the disjunctive assertion at line 390 with two independent assertions:
```rust
assert!(creature_died, "Ruling 2021-09-24: CreatureDied event should fire even after losing decayed");
assert!(is_in_graveyard(&state, "Decayed Attacker", p1), "Ruling 2021-09-24: Creature should be in graveyard");
```

#### Finding 2: Delayed Trigger TODO (Informational)

**Severity**: LOW
**File**: `crates/engine/src/rules/turn_actions.rs:615-619`
**CR Rule**: 702.147a / 603.7 -- The EOC sacrifice is technically a delayed triggered ability.
**Issue**: The TODO correctly documents that the current TBA implementation does not use the
stack and therefore cannot be Stifled. This is the same caveat as the Myriad EOC exile
(turn_actions.rs:566-571). Both should be refactored together when delayed trigger
infrastructure is expanded.
**Fix:** No fix needed for V1. The TODO is well-documented and consistent with the existing
Myriad pattern.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.147a "can't block" (static) | Yes | Yes | test_1 (blocking rejected), test_7 (non-decayed baseline) |
| 702.147a "sacrifice at EOC" (triggered) | Yes | Yes | test_4 (full EOC lifecycle), test_3 (flag set on attack) |
| 702.147a redundant instances | Yes (implicit) | No | Keywords are in OrdSet (auto-dedup); flag is bool. Multiple instances have identical effect. No explicit test needed. |
| Ruling: sacrifice persists after keyword removal | Yes | Yes | test_5 (removes keyword, verifies sacrifice still occurs) |
| Ruling: not forced to attack | Yes | Yes | test_6 (non-attacking decayed creature survives EOC) |
| Ruling: no haste | Yes | Yes | test_8 (summoning sickness blocks attack) |
| Ruling: can attack normally | Yes | Yes | test_2 (DeclareAttackers succeeds) |
| CR 400.7: flag reset on zone change | Yes | No | Flag reset in both `move_object_to_zone` sites. Implicit in test_6 (no zone change, no sacrifice). Explicit zone-change flag reset test not written but low risk. |
| CR 701.17a: sacrifice != destruction | Yes (implicit) | No | Sacrifice uses `move_object_to_zone` to graveyard, not the destroy path. Indestructible is never checked. No explicit test but correctly handled by design. |
| CR 614: replacement effects on sacrifice | Yes | No | `check_zone_change_replacement` called before move. No test for Rest in Peace / commander redirect interaction, but same pattern as Myriad (tested there). |
| CR 508.4: put onto battlefield attacking | N/A | N/A | Engine does not support "put onto battlefield attacking" yet. Decayed correctly only tags in `handle_declare_attackers`, which aligns with CR 508.4. |

## Previous Findings (re-review only)

N/A -- this is the first review.
