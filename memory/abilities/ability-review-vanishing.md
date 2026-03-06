# Ability Review: Vanishing

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.63
**Files reviewed**: `crates/engine/src/state/types.rs:1004-1013`, `crates/engine/src/state/hash.rs:562-566,1656-1673,3382-3386`, `crates/engine/src/state/stack.rs:894-906`, `crates/engine/src/state/stubs.rs:74-77`, `crates/engine/src/cards/card_definition.rs:463-471`, `crates/engine/src/rules/turn_actions.rs:100-149`, `crates/engine/src/rules/resolution.rs:448-469,977-1064,1066-1160,3720-3741`, `crates/engine/src/rules/lands.rs:115-134`, `crates/engine/src/rules/abilities.rs:3785-3802`, `tools/tui/src/play/panels/stack_view.rs:141-150`, `crates/engine/tests/vanishing.rs`

## Verdict: needs-fix

One MEDIUM finding: ETB counter placement only processes the first Vanishing instance, violating CR 702.63c (multiple instances work separately). A permanent with Vanishing 3 and Vanishing 2 should enter with 5 time counters, but the current `find_map` approach only places 3. Same issue in both `resolution.rs` and `lands.rs`. All other aspects of the implementation are correct.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `resolution.rs:453-469` | **ETB counter placement ignores multiple Vanishing instances (CR 702.63c).** `find_map` returns only the first match. **Fix:** sum all Vanishing N values. |
| 2 | **MEDIUM** | `lands.rs:120-133` | **Same find_map issue as Finding 1 for lands.** **Fix:** sum all Vanishing N values. |
| 3 | LOW | `turn_actions.rs:115-128` | **Uses raw characteristics instead of layer-resolved.** Pre-existing pattern; Humility/Dress Down would not suppress Vanishing triggers. **Fix:** address holistically when other turn_actions keyword checks are updated. |
| 4 | LOW | `resolution.rs:462-468` | **No CounterAdded event for ETB counter placement.** Impending has the same gap. **Fix:** address holistically with Impending. |

### Finding Details

#### Finding 1: ETB counter placement ignores multiple Vanishing instances

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:453-469`
**CR Rule**: 702.63c -- "If a permanent has multiple instances of vanishing, each works separately."
**Issue**: The ETB counter placement uses `find_map` which returns the N value from only the first `KeywordAbility::Vanishing(n)` found in the keywords set. CR 702.63a says "Vanishing N means 'This permanent enters with N time counters on it'" and CR 702.63c says "each works separately." If a permanent has Vanishing 3 and Vanishing 2 (e.g., via a copy effect gaining an additional instance, or Maelstrom Djinn turning face up twice per its ruling), it should enter with 3 + 2 = 5 time counters. The current code places only 3 (or 2, depending on iteration order).

The upkeep trigger queueing in `turn_actions.rs` correctly handles multiple instances by counting and queueing one trigger per instance. The ETB placement should follow the same pattern.

**Fix**: Replace the `find_map` + single placement with a `filter_map` + `sum`:
```rust
let total_vanishing: u32 = obj.characteristics.keywords.iter()
    .filter_map(|kw| {
        if let KeywordAbility::Vanishing(n) = kw { Some(*n) } else { None }
    })
    .sum();
if total_vanishing > 0 {
    let current = obj.counters.get(&CounterType::Time).copied().unwrap_or(0);
    obj.counters = obj.counters.update(CounterType::Time, current + total_vanishing);
}
```

#### Finding 2: Same find_map issue in lands.rs

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/lands.rs:120-133`
**CR Rule**: 702.63c -- "If a permanent has multiple instances of vanishing, each works separately."
**Issue**: Identical to Finding 1. The lands ETB hook also uses `find_map` which processes only the first Vanishing instance. Apply the same `filter_map` + `sum` fix.
**Fix**: Same pattern as Finding 1 -- replace `find_map` with `filter_map(...).sum()`.

#### Finding 3: Raw characteristics used for keyword detection

**Severity**: LOW
**File**: `crates/engine/src/rules/turn_actions.rs:115-128`
**CR Rule**: 702.63a -- trigger should check whether the permanent currently has Vanishing (per layer-resolved characteristics).
**Issue**: The upkeep trigger queueing reads `obj.characteristics.keywords` directly instead of using `calculate_characteristics`. Under Humility or Dress Down, Vanishing would be removed from a creature, and the trigger should not fire. This is a pre-existing pattern: Madness, NoMaxHandSize, and other keyword checks in `turn_actions.rs` all use raw characteristics. Suspend avoids this because suspended cards are in exile (not subject to layer effects).
**Fix**: Address holistically when updating all `turn_actions.rs` keyword checks to use layer-resolved characteristics. Not Vanishing-specific.

#### Finding 4: Missing CounterAdded event for ETB placement

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:462-468`
**CR Rule**: N/A (engine architecture -- events should reflect all state changes).
**Issue**: The ETB counter placement modifies `obj.counters` but does not emit a `GameEvent::CounterAdded` event. Other ETB counter placements (e.g., +1/+1 from Modular, Undying) do emit `CounterAdded`. However, the Impending ETB counter placement (lines 440-446) has the same gap, so this is a pre-existing pattern.
**Fix**: Address holistically with Impending -- add `CounterAdded` events to both Vanishing and Impending ETB placements.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.63a (ETB counters) | Yes | Yes | `test_vanishing_etb_counters_on_cast` (cast flow) |
| 702.63a (upkeep removal) | Yes | Yes | `test_vanishing_upkeep_removes_counter` |
| 702.63a (sacrifice on last) | Yes | Yes | `test_vanishing_sacrifice_on_last_counter` |
| 702.63a (full lifecycle) | Yes | Yes | `test_vanishing_full_lifecycle` |
| 702.63a ("your upkeep") | Yes | Yes | `test_vanishing_multiplayer_only_active_player` |
| 702.63b (no number) | Yes | Yes | `test_vanishing_without_number_no_etb_counters` |
| 702.63c (multiple instances -- triggers) | Yes | Yes | `test_vanishing_multiple_instances` |
| 702.63c (multiple instances -- ETB counters) | No | No | Finding 1: `find_map` only processes first instance |
| Intervening-if (CR 603.4) | Yes | Partial | Tested via no-counter case (702.63b test); not explicitly tested for removal-before-resolution |
| Countered sacrifice (Dreamtide Whale ruling) | Noted | No | Comments at resolution.rs:3737-3740 document behavior; no test exercises Stifle interaction |
| Replacement effects on sacrifice | Yes | No | Uses `check_zone_change_replacement` correctly; no test |

verdict: needs-fix
