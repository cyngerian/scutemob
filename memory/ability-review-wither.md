# Ability Review: Wither

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.80
**Files reviewed**:
- `crates/engine/src/state/types.rs:475-481` (enum variant)
- `crates/engine/src/state/hash.rs:421-422` (hash discriminant 58)
- `tools/replay-viewer/src/view_model.rs:643` (display string)
- `crates/engine/src/rules/combat.rs:860-1055` (combat damage path)
- `crates/engine/src/effects/mod.rs:206-248` (non-combat damage path)
- `crates/engine/tests/keywords.rs:2147-2504` (5 unit tests)

## Verdict: needs-fix

The core enforcement logic is correct for both combat and non-combat damage paths.
CR 702.80a (counter placement), 702.80c (any zone), and 702.80d (redundancy) are
properly handled. The combat path correctly integrates with lifelink, deathtouch,
and damage prevention. However, there is a MEDIUM finding: the non-combat damage
path (effects/mod.rs) is missing a test entirely (plan Test 4), which means the
`DealDamage` effect handler's wither branch has zero test coverage. There is also
a MEDIUM finding for CR correctness: the `else` branch in effects/mod.rs applies
wither to ANY non-planeswalker object, including battles (CR 120.3h) and other
permanent types, not just creatures. Wither should only affect creatures per
CR 702.80a.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `effects/mod.rs:206` | **Wither applies to non-creature, non-planeswalker permanents.** The `else` branch catches all non-planeswalker objects including battles. **Fix:** add a creature type check. |
| 2 | **MEDIUM** | `keywords.rs` | **Missing non-combat wither test.** Plan Test 4 was not implemented; the `DealDamage` wither branch has zero test coverage. **Fix:** add test. |
| 3 | LOW | `effects/mod.rs:209-214` | **702.80b last-known info not implemented for non-combat sources.** `calculate_characteristics` returns `None` if the source changed zones; `unwrap_or(false)` drops wither. Pre-existing architectural gap. **Fix:** deferred -- requires general last-known-info system. |
| 4 | LOW | `keywords.rs:2445-2504` | **Redundancy test is vacuous.** `OrdSet<KeywordAbility>` deduplicates entries, so two `.with_keyword(Wither)` calls produce one entry. The test passes but doesn't exercise a code path that could fail. **Fix:** none needed -- OrdSet prevents the bug by design. |

### Finding Details

#### Finding 1: Wither applies to non-creature, non-planeswalker permanents

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:206`
**CR Rule**: 702.80a -- "Damage dealt to a creature by a source with wither isn't marked on that creature."
**Issue**: The non-combat damage handler in `effects/mod.rs` branches as:

```rust
if card_types.contains(&CardType::Planeswalker) {
    // loyalty counter removal
} else {
    // wither check + counter placement OR damage marking
}
```

The `else` branch catches ALL non-planeswalker targets: creatures, battles, and any
other permanent type. Per CR 702.80a, wither only modifies damage dealt to **creatures**.
Damage to a battle (CR 120.3h) should remove defense counters, not place -1/-1 counters.
While the engine may not yet support battle permanents, the code as written would
incorrectly apply wither to any future non-planeswalker permanent type. More importantly,
if a non-creature, non-planeswalker permanent somehow receives damage via `DealDamage`,
it would get -1/-1 counters when it should get `damage_marked`.

**Fix**: Add an explicit creature type check:

```rust
} else if card_types.contains(&CardType::Creature) {
    // CR 120.3d/e: check source for wither keyword.
    let source_has_wither = ...;
    if source_has_wither { /* place counters */ } else { /* mark damage */ }
} else {
    // Non-creature, non-planeswalker: mark damage normally (CR 120.3e).
    if let Some(obj) = state.objects.get_mut(&id) {
        obj.damage_marked += final_dmg;
    }
}
```

Note: The combat.rs path is not affected because combat damage targets are already
typed as `CombatDamageTarget::Creature`, `::Player`, or `::Planeswalker`.

#### Finding 2: Missing non-combat wither test

**Severity**: MEDIUM
**File**: `crates/engine/tests/keywords.rs` (missing test)
**CR Rule**: 702.80c -- "The wither rules function no matter what zone an object with wither deals damage from."
**Issue**: The plan specified Test 4 (`test_702_80_wither_noncombat_damage_places_counters`)
to verify the `DealDamage` effect handler's wither branch. This test was not implemented.
The non-combat path in `effects/mod.rs:206-241` has zero test coverage. This is the only
code path that exercises wither from spells (e.g., Puncture Blast), and it contains
Finding 1's bug.

**Fix**: Add a test that constructs an `EffectContext` with a wither source and executes
`Effect::DealDamage` against a creature target. Verify:
- Target creature has -1/-1 counters equal to damage amount
- Target creature has `damage_marked == 0`
- `CounterAdded` event was emitted
- `DamageDealt` event was also emitted

The simplest approach: use `GameStateBuilder` to place a creature with Wither on the
battlefield, set it as `ctx.source`, create a target creature, and call
`execute_effect()` with `Effect::DealDamage`. Alternatively, define a card with Wither
and a damage effect, cast it, and verify the result.

#### Finding 3: CR 702.80b last-known information gap

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:209-214`
**CR Rule**: 702.80b -- "If an object changes zones before an effect causes it to deal damage, its last known information is used to determine whether it had wither."
**Issue**: In the non-combat damage path, `calculate_characteristics(state, ctx.source)`
returns `None` if the source object has changed zones (CR 400.7: new identity). The
`unwrap_or(false)` fallback means wither is silently dropped. Per CR 702.80b, the engine
should use the source's last-known characteristics to determine wither.

This is a pre-existing architectural limitation: the engine has no general last-known-info
system. The only last-known info tracked is `pre_death_counters` for persist/undying.

**Practical impact**: Very narrow. For combat damage, the source is pre-extracted before
application. For spell damage, the spell is still on the stack during resolution. The gap
only matters for triggered abilities from a wither source that left the battlefield before
the trigger resolves -- a rare scenario with no printed cards that would trigger it in
practice.

**Fix**: Deferred. When a general last-known-info system is implemented, ensure the
wither check uses it. No action required for this review.

#### Finding 4: Redundancy test is vacuous due to OrdSet deduplication

**Severity**: LOW
**File**: `crates/engine/tests/keywords.rs:2445-2504`
**CR Rule**: 702.80d -- "Multiple instances of wither on the same object are redundant."
**Issue**: The test calls `.with_keyword(KeywordAbility::Wither).with_keyword(KeywordAbility::Wither)`
but `keywords` is stored as `OrdSet<KeywordAbility>`, which deduplicates entries. The
test's creature has exactly 1 Wither keyword internally. The test passes correctly but
doesn't actually exercise any code that could produce incorrect behavior with multiple
instances -- the data model prevents it.

**Fix**: None needed. The `OrdSet` representation inherently enforces CR 702.80d. The
test serves as documentation of the expected behavior. Consider adding a comment noting
that `OrdSet` makes redundancy impossible at the data model level.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.80a (counter placement) | Yes | Yes | `test_702_80_wither_combat_damage_places_minus_counters`, `test_702_80_wither_combat_kills_creature_via_toughness_sba` |
| 702.80a (only creatures) | Yes (combat), Partial (non-combat -- see Finding 1) | Yes (combat: `test_702_80_wither_does_not_affect_player_damage`) | Non-combat path lacks creature type guard |
| 702.80b (last-known info) | Partial (combat pre-extract covers most cases) | No | Pre-existing arch gap; see Finding 3 |
| 702.80c (any zone) | Yes | No (no non-combat test -- Finding 2) | `effects/mod.rs` uses `calculate_characteristics` which works for any zone |
| 702.80d (redundancy) | Yes (inherent via OrdSet) | Yes (vacuous -- Finding 4) | `test_702_80_wither_redundant_instances` |
| 120.3d (wither/infect counters) | Yes | Yes | Counter placement verified in 3 tests |
| 120.3e (normal damage marking) | Yes | Yes | Verified by contrast in counter tests |
| 120.3f (lifelink compat) | Yes (combat path) | No | No wither+lifelink test, but lifelink gain is unconditional in combat loop |
| 704.5f (toughness SBA) | Pre-existing | Yes | `test_702_80_wither_combat_kills_creature_via_toughness_sba` |
| 704.5q (counter annihilation) | Pre-existing | Not for wither+existing counters | Pre-existing SBA; no specific wither+annihilation test |
| CR 702.79 (persist interaction) | Yes | Yes | `test_702_80_wither_persist_interaction` |

## Previous Findings (re-review only)

N/A -- first review.
