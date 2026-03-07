# Ability Review: Graft

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.58
**Files reviewed**:
- `crates/engine/src/state/types.rs` (Graft variant)
- `crates/engine/src/state/hash.rs` (KeywordAbility + StackObjectKind + PendingTrigger hashing)
- `crates/engine/src/state/stubs.rs` (PendingTriggerKind::Graft, graft_entering_creature field)
- `crates/engine/src/state/stack.rs` (StackObjectKind::GraftTrigger)
- `crates/engine/src/rules/resolution.rs` (ETB counters + trigger resolution)
- `crates/engine/src/rules/lands.rs` (ETB counters for lands)
- `crates/engine/src/rules/abilities.rs` (trigger collection + flush_pending_triggers)
- `tools/tui/src/play/panels/stack_view.rs` (TUI match arm)
- `tools/replay-viewer/src/view_model.rs` (missing GraftTrigger arm)
- `crates/engine/tests/graft.rs` (9 tests)

## Verdict: needs-fix

One HIGH finding: the replay viewer `view_model.rs` has an exhaustive match on
`StackObjectKind` that is missing the new `GraftTrigger` variant, which will cause
a compile error when building the replay-viewer crate. All CR enforcement logic is
correct. The trigger collection, intervening-if checks, and resolution handler
faithfully implement CR 702.58a/b and CR 603.4. Test coverage is thorough.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `view_model.rs:557` | **Missing GraftTrigger arm in replay viewer.** Exhaustive match on StackObjectKind will fail to compile. **Fix:** add arm. |
| 2 | LOW | `lands.rs:203` | **No CounterAdded event for land ETB counters.** Consistent with Vanishing/Fading pattern in lands.rs -- systemic, not Graft-specific. |
| 3 | LOW | `resolution.rs:2249` | **Phased-out source not checked at resolution.** Intervening-if recheck does not call `is_phased_in()`. Same pattern as Evolve -- systemic, not Graft-specific. |
| 4 | LOW | `graft.rs:11` | **Typo in test module doc.** Line 11 cites "CR 703.4" instead of "CR 603.4". |

### Finding Details

#### Finding 1: Missing GraftTrigger arm in replay viewer view_model.rs

**Severity**: HIGH
**File**: `tools/replay-viewer/src/view_model.rs:557`
**CR Rule**: N/A -- infrastructure invariant (exhaustive match coverage)
**Issue**: The `stack_kind_info` function has an exhaustive match on `StackObjectKind`
(no catch-all `_` arm). The new `GraftTrigger` variant is not handled. This will
cause a compile error when building the replay-viewer crate (`cargo build -p replay-viewer`).
The TUI `stack_view.rs` was updated correctly, but the replay viewer was missed.
**Fix**: Add the following arm before the closing `}` of the match at line 557:
```rust
StackObjectKind::GraftTrigger { source_object, .. } => {
    ("graft_trigger", Some(*source_object))
}
```

#### Finding 2: No CounterAdded event for land ETB counters

**Severity**: LOW
**File**: `crates/engine/src/rules/lands.rs:203`
**CR Rule**: 702.58a -- "This permanent enters with N +1/+1 counters on it"
**Issue**: The `lands.rs` Graft ETB counter placement does not emit a
`GameEvent::CounterAdded` event, while the `resolution.rs` path does. This is
consistent with Vanishing and Fading counter placement in the same file, which also
skip the event. This is a systemic pattern, not Graft-specific. The counters are
placed correctly on the object; only event logging is missing.
**Fix**: No immediate fix needed for Graft. Track as a systemic LOW for all
lands.rs ETB counter placements.

#### Finding 3: Phased-out source not checked at resolution

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:2249`
**CR Rule**: 702.26d -- phased-out permanents are "treated as though not on the battlefield"
**Issue**: The intervening-if recheck at resolution (CR 603.4) checks
`obj.zone == ZoneId::Battlefield` but does not check `obj.is_phased_in()`. If the
Graft source phases out between trigger and resolution, the code would incorrectly
proceed with the counter move. However, this is the same pattern used by Evolve and
other trigger resolutions. The scenario is extremely unlikely (requires a phasing
effect at instant speed while a Graft trigger is on the stack).
**Fix**: No immediate fix needed for Graft. Track as a systemic LOW for all
trigger resolution handlers.

#### Finding 4: Typo in test module doc

**Severity**: LOW
**File**: `crates/engine/tests/graft.rs:11`
**CR Rule**: 603.4 -- intervening-if clause rule
**Issue**: Line 11 says "CR 703.4 intervening-if" but the correct rule number is
CR 603.4 (as correctly cited elsewhere in the same file and in the implementation).
**Fix**: Change "CR 703.4" to "CR 603.4" on line 11 of `graft.rs`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.58a (static: ETB counters) | Yes (resolution.rs:640, lands.rs:179) | Yes | test_graft_etb_counters, test_graft_multiple_instances_etb_counter_sum |
| 702.58a (trigger: another creature enters) | Yes (abilities.rs:2415) | Yes | test_graft_trigger_moves_counter |
| 702.58a (intervening-if at trigger time) | Yes (abilities.rs:2448-2453) | Yes | test_graft_trigger_does_not_fire_without_counters |
| 702.58a (intervening-if at resolution, CR 603.4) | Yes (resolution.rs:2243-2257) | Yes | test_graft_resolution_recheck_intervening_if |
| 702.58a ("another creature" -- self-exclusion) | Yes (abilities.rs:2446) | Yes | test_graft_trigger_does_not_fire_for_self |
| 702.58a ("you may" -- optional) | Yes (auto-accept, resolution.rs:2267) | No | No test for declining; auto-accept is consistent with Evolve pattern |
| 702.58a (fires for any player's creature) | Yes (abilities.rs:2421) | Yes | test_graft_trigger_fires_for_opponents_creatures |
| 702.58a (non-creature does not trigger) | Yes (abilities.rs:2425-2435) | Yes | test_graft_noncreature_does_not_trigger |
| 702.58b (multiple instances work separately) | Yes (abilities.rs:2472-2473) | Yes | test_graft_multiple_instances |
| Phased-out exclusion from trigger scan | Yes (abilities.rs:2447) | No | is_phased_in() check present; no dedicated test |
| SBA 0/0 after counter move (CR 704.5f) | Yes (natural consequence) | Yes | test_graft_trigger_fires_for_opponents_creatures (indirect) |

