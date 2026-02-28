# Ability Review: Regenerate

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.19
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (lines 480-484)
- `crates/engine/src/state/replacement_effect.rs` (full file)
- `crates/engine/src/state/hash.rs` (discriminants 5, 7, 37, 83, 84)
- `crates/engine/src/rules/replacement.rs` (lines 300-304, 1609-1694)
- `crates/engine/src/rules/sba.rs` (lines 330-430)
- `crates/engine/src/effects/mod.rs` (lines 520-544, 1457-1500)
- `crates/engine/src/rules/events.rs` (lines 800-824)
- `crates/engine/tests/regenerate.rs` (full file, 811 lines)

## Verdict: clean

The Regenerate implementation correctly handles the core CR 701.19a mechanic: one-shot
regeneration shields as replacement effects that intercept destruction, remove damage,
tap the permanent, and remove it from combat. The architecture (dedicated
`WouldBeDestroyed` trigger + `Regenerate` modification + two interception sites in SBA
and DestroyPermanent) is clean and avoids polluting the existing `WouldChangeZone`
pipeline. All 10 tests pass and cover the key scenarios: spell-based destruction,
SBA lethal damage, SBA deathtouch, one-shot consumption, multiple shields, combat
attacker/blocker removal, zero-toughness non-destruction, shield creation event, and
indestructible priority. Hash coverage is complete for all new enum variants. No HIGH
or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `effects/mod.rs:1486` | **is_self_replacement misclassification.** Regeneration shields are not self-replacement effects per CR 614.15. **Fix:** Change `is_self_replacement: true` to `false`. |
| 2 | LOW | `tests/regenerate.rs` | **Missing end-of-turn expiration test.** Plan listed `test_regenerate_shield_expires_at_end_of_turn` but implementation replaced it with shield-created event test. **Fix:** Add a test verifying that shields are removed during cleanup step. |
| 3 | LOW | `replacement.rs:1646-1648` | **Redundant double get_mut.** `apply_regeneration` calls `state.objects.get_mut(&object_id)` twice in sequence (lines 1646 and 1652) when a single mutable borrow could do both operations. **Fix:** Merge into a single `if let Some(obj) = state.objects.get_mut(&object_id) { obj.damage_marked = 0; obj.deathtouch_damage = false; obj.status.tapped = true; }`. |
| 4 | LOW | N/A | **"Can't be regenerated" not implemented.** Cards like Wrath of God say "They can't be regenerated" (CR 701.19c, 608.2c). No `cant_regenerate` flag on `DestroyPermanent`. Acknowledged as deferred in the plan. **Fix:** Track as future enhancement. |

### Finding Details

#### Finding 1: is_self_replacement misclassification

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:1486`
**CR Rule**: 614.15 -- "Some replacement effects are not continuous effects. Rather, they are an effect of a resolving spell or ability that replace part or all of that spell or ability's own effect(s)."
**Issue**: Regeneration shields are flagged as `is_self_replacement: true`, but they do not meet the CR 614.15 definition. A regeneration shield created by "{B}: Regenerate ~" does not replace part of that ability's own effect -- it replaces a future destruction event from a different source. Self-replacement effects per 614.15 replace "that spell or ability's own effect(s)," not unrelated events.
**Impact**: Currently harmless. The `check_regeneration_shield()` helper calls `find_applicable()` and filters for `Regenerate` modifications, so the ordering provided by self-replacement priority is irrelevant. The flag would only matter if another `WouldBeDestroyed` replacement effect existed and the ordering pipeline had to choose between them.
**Fix**: Change `is_self_replacement: true` to `is_self_replacement: false` at `effects/mod.rs:1486` and in the test helper `regen_shield()` at `tests/regenerate.rs:62`. Also update the comment from "CR 614.15: self-replacement" to "not self-replacement per CR 614.15".

#### Finding 2: Missing end-of-turn expiration test

**Severity**: LOW
**File**: `crates/engine/tests/regenerate.rs`
**CR Rule**: 701.19a -- "The next time [permanent] would be destroyed this turn" / 514.2 -- "all 'until end of turn' and 'this turn' effects end"
**Issue**: The plan (step 4, test 9) specified `test_regenerate_shield_expires_at_end_of_turn` to verify shields are cleaned up during the cleanup step. The implementation replaced this with `test_regenerate_effect_emits_shield_created_event`, which tests a different behavior. No test verifies end-of-turn expiration of regeneration shields.
**Impact**: Low risk. The `expire_end_of_turn_effects()` function in `layers.rs` removes ALL `UntilEndOfTurn` replacement effects generically and is well-tested for other effect types (prevention shields, continuous effects). Regeneration shields use the same `EffectDuration::UntilEndOfTurn` tag.
**Fix**: Add a test that creates a regeneration shield, advances to the cleanup step (or directly calls `expire_end_of_turn_effects`), and asserts the shield is removed from `state.replacement_effects`.

#### Finding 3: Redundant double get_mut in apply_regeneration

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1646-1654`
**Issue**: `apply_regeneration` performs two sequential `state.objects.get_mut(&object_id)` calls to (a) clear damage and (b) tap the permanent. These can be merged into a single mutable borrow since they operate on the same object in sequence with no intervening operations.
**Fix**: Merge lines 1646-1654 into:
```rust
if let Some(obj) = state.objects.get_mut(&object_id) {
    obj.damage_marked = 0;
    obj.deathtouch_damage = false;
    obj.status.tapped = true;
}
```

#### Finding 4: "Can't be regenerated" not implemented

**Severity**: LOW
**File**: N/A (missing feature)
**CR Rule**: 701.19c -- "Effects that say that a permanent can't be regenerated [...] cause regeneration shields to not be applied." / 608.2c -- "Destroy target creature. It can't be regenerated"
**Issue**: No mechanism exists to suppress regeneration when a destruction effect includes "can't be regenerated" text (e.g., Wrath of God, Terminate). Currently, all destruction events check for regeneration shields without exception. The plan explicitly documents this as deferred.
**Fix**: Track as future enhancement. When implemented, add a `cant_regenerate: bool` field to `Effect::DestroyPermanent` and a parameter to the SBA destruction path. The `check_regeneration_shield()` call sites should skip the check when this flag is set.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.19a (resolving spell/ability creates shield) | Yes | Yes | test_regenerate_effect_emits_shield_created_event |
| 701.19a (one-shot, next time this turn) | Yes | Yes | test_regenerate_shield_is_one_shot |
| 701.19a (remove all damage) | Yes | Yes | test_regenerate_shield_prevents_destruction_by_spell, test_regenerate_shield_prevents_sba_lethal_damage |
| 701.19a (tap the permanent) | Yes | Yes | All destruction tests check is_tapped |
| 701.19a (remove from combat if attacking/blocking) | Yes | Yes | test_regenerate_removes_from_combat_attacker, test_regenerate_removes_from_combat_blocker |
| 701.19a (shield until end of turn) | Yes | No | Missing end-of-turn expiration test (Finding 2); relies on existing expire_end_of_turn_effects infrastructure |
| 701.19a (multiple shields stack) | Yes | Yes | test_regenerate_multiple_shields |
| 701.19b (static regeneration) | No | No | Uncommon pattern; only 701.19a (resolving spell/ability) is implemented |
| 701.19c (can't be regenerated) | No | No | Deferred (Finding 4) |
| 614.8 (destruction-replacement) | Yes | Yes | Architecture matches: regeneration replaces destruction, not zone change |
| 704.5f (zero toughness -- not destruction) | Yes | Yes | test_regenerate_does_not_prevent_zero_toughness |
| 704.5g (lethal damage -- can be regenerated) | Yes | Yes | test_regenerate_shield_prevents_sba_lethal_damage |
| 704.5h (deathtouch damage -- can be regenerated) | Yes | Yes | test_regenerate_shield_prevents_sba_deathtouch_damage |
| 702.12a (indestructible priority) | Yes | Yes | test_regenerate_not_applied_when_indestructible |
| 514.2 (cleanup removes "this turn" effects) | Yes (by infrastructure) | No (for regeneration specifically) | Finding 2 |
