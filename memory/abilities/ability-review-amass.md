# Ability Review: Amass

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.47
**Files reviewed**: `crates/engine/src/cards/card_definition.rs` (lines 675-686, 1273-1295), `crates/engine/src/effects/mod.rs` (lines 1073-1182), `crates/engine/src/rules/events.rs` (lines 710-725), `crates/engine/src/state/hash.rs` (lines 2753-2763, 3347-3352), `crates/engine/src/rules/abilities.rs` (lines 3411-3419), `crates/engine/src/lib.rs` (line 9), `crates/engine/src/cards/mod.rs` (line 17), `crates/engine/tests/amass.rs` (full file, 612 lines)

## Verdict: needs-fix

Two findings. One MEDIUM: the early `return` on token-creation failure skips the `Amassed` event emission, contradicting both CR 701.47b and the code's own comment. One LOW: `.unwrap()` in engine library code where Bolster uses `let-else` pattern.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `effects/mod.rs:1138` | **Early return skips Amassed event (CR 701.47b).** Comment says "still emit Amassed" but code does `return;`. **Fix:** see details. |
| 2 | LOW | `effects/mod.rs:1144` | **`.unwrap()` in engine library code.** Convention prohibits this. **Fix:** use `let-else`. |

### Finding Details

#### Finding 1: Early return skips Amassed event on token-creation failure

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:1136-1138`
**CR Rule**: 701.47b -- "A player 'amassed' after the process described in rule 701.47a is complete, even if some or all of those actions were impossible."
**Issue**: When `state.add_object()` returns `Err` (token creation failure), the code hits `return;` and never emits `GameEvent::Amassed`. The comment on line 1137 explicitly says "CR 701.47b: still emit Amassed with a sentinel; use a fallback" but the code contradicts this by returning early. While this path is extremely unlikely in practice (add_object failing is near-impossible), the code violates its own stated invariant and the CR rule. More importantly, the comment is actively misleading.
**Fix**: Either (a) emit `Amassed` with a sentinel `army_id` (e.g., `ObjectId(0)`) before returning, or (b) change the comment to acknowledge the early return is intentional for this impossible-in-practice case. Option (a) is CR-correct; option (b) is pragmatic. Recommended: option (a):
```rust
} else {
    // Token creation failed (should not happen in normal play).
    // CR 701.47b: still emit Amassed even if actions were impossible.
    events.push(GameEvent::Amassed {
        player: controller,
        army_id: ObjectId(0),
        count: n,
    });
    return;
}
```

#### Finding 2: `.unwrap()` in engine library code

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:1144`
**CR Rule**: N/A (architecture convention from `memory/conventions.md`: "never `unwrap()` or `expect()` in engine logic")
**Issue**: `*army_ids.iter().min_by_key(|id| id.0).unwrap()` uses `.unwrap()`. While logically safe (this is in the `else` branch where `army_ids` is guaranteed non-empty), Bolster uses `let Some(...) = ... else { continue; }` for the same pattern (line 1046-1053). The convention is strict: no `.unwrap()` in engine code.
**Fix**: Replace with:
```rust
let Some(&chosen) = army_ids.iter().min_by_key(|id| id.0) else {
    return;
};
chosen
```

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.47a (no Army -> create token) | Yes | Yes | test_amass_creates_army_token_when_none_exists |
| 701.47a (choose Army, add counters) | Yes | Yes | test_amass_adds_counters_to_existing_army |
| 701.47a (add subtype if missing) | Yes | Yes | test_amass_adds_subtype_to_existing_army |
| 701.47a (subtype already present) | Yes | Yes | test_amass_subtype_not_duplicated_if_already_present |
| 701.47a (multiple Armies) | Yes | Yes | test_amass_multiple_armies_chooses_smallest_object_id |
| 701.47a (multiplayer - own Armies only) | Yes | Yes | test_amass_multiplayer_only_own_armies |
| 701.47b (amass 0 still completes) | Yes | Yes | test_amass_zero_still_creates_token |
| 701.47b (Amassed event always emitted) | Partial | No | Token-creation-failure path returns without emitting (Finding 1) |
| 701.47c ("the Army you amassed" ref) | N/A | N/A | No card currently uses this phrasing; Amassed event stores army_id for future use |
| 701.47d (errata to "Zombies") | Yes (doc) | N/A | Documented in Effect variant; card definitions will use explicit subtype |
| Token is 0/0 black | Yes | Yes | army_token_spec verified; P/T checked in test 1 |
| Layer-aware Army detection (Changeling) | Yes | No | Uses calculate_characteristics; no Changeling-specific test (acceptable LOW gap) |
| Phased-out exclusion | Yes | No | `is_phased_in()` filter at line 1095; no dedicated test (acceptable LOW gap) |

## Additional Notes

**Correct design decisions verified:**
- No KeywordAbility variant (Amass is a keyword action, not a keyword ability) -- correct
- No AbilityDefinition / StackObjectKind variant needed -- correct
- Effect::Amass hash discriminant 41 -- no collision (separate HashInto impl from KeywordAbility::BattleCry at 41)
- GameEvent::Amassed hash discriminant 98 -- no collision (separate HashInto impl from KeywordAbility::Prototype at 98)
- `army_token_spec()` creates 0/0 black token with correct subtypes -- verified
- `calculate_characteristics()` used for Army detection (layer-aware, Changeling-safe) -- correct
- Subtype addition modifies `obj.characteristics.subtypes` directly (one-shot, not continuous effect) -- correct per CR 701.47a
- `resolve_amount().max(0) as u32` prevents negative counter placement -- correct
- Token emits both `TokenCreated` and `PermanentEnteredBattlefield` events -- consistent with Investigate pattern
- Trigger wiring in abilities.rs is a no-op placeholder -- acceptable, no current card uses "whenever you amass"
- Re-export chain (card_definition.rs -> cards/mod.rs -> lib.rs) includes `army_token_spec` -- verified
- 7 tests, all with CR citations, covering positive/negative/edge cases -- good coverage
- Tests use `calculate_characteristics` to verify layer-aware P/T -- thorough

**Test quality:**
- All 7 tests cite CR rules in doc comments
- Positive cases: token creation, counter addition, subtype addition
- Negative cases: no duplicate subtype, no token when Army exists, opponent's Army ignored
- Edge cases: amass 0, multiple Armies
- Helper functions are clean and reusable
- 4-player test for multiplayer correctness
