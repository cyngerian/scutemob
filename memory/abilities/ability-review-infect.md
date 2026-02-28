# Ability Review: Infect

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.90
**Files reviewed**:
- `crates/engine/src/state/types.rs:511-520` (enum variant)
- `crates/engine/src/state/hash.rs:434-435` (KeywordAbility hash discriminant 63)
- `crates/engine/src/state/hash.rs:1981-1991` (GameEvent::PoisonCountersGiven hash discriminant 81)
- `crates/engine/src/rules/events.rs:359-371` (PoisonCountersGiven event variant)
- `crates/engine/src/rules/combat.rs:860-1091` (combat damage application with infect)
- `crates/engine/src/effects/mod.rs:143-289` (non-combat damage with infect)
- `tools/replay-viewer/src/view_model.rs:657` (display string)
- `crates/engine/tests/keywords.rs:2601-3323` (9 unit tests)

## Verdict: clean

The Infect implementation is correct, well-structured, and faithfully implements all
subrules of CR 702.90 and the supporting rules CR 120.3a-f. All enforcement paths
(combat damage and non-combat DealDamage effect) are handled for both creature targets
(-1/-1 counters via the existing wither path) and player targets (poison counters instead
of life loss). Hash coverage, event emission, and match-arm exhaustiveness are all
addressed. The 9 unit tests cover the full range of planned scenarios with proper CR
citations and meaningful assertions. Two LOW findings exist: a missing test for the
infect+lifelink interaction explicitly called out by multiple card rulings, and a
pre-existing event asymmetry that is not caused by this change.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/keywords.rs:2601-3323` | **Missing infect+lifelink test.** Multiple card rulings (Contagious Nim, Skithiryx, etc.) emphasize that lifelink works with infect. No test validates this. **Fix:** Add `test_702_90_infect_lifelink_controller_gains_life` -- creature with Infect+Lifelink deals combat damage to a player, assert: poison counters given, no life loss for defender, controller gains life equal to damage dealt, LifeGained event emitted. |
| 2 | LOW | `rules/combat.rs:1016-1020` | **Pre-existing: no LifeLost event for non-infect combat damage to players.** The non-infect combat path reduces `life_total` but does not emit `LifeLost`. The non-combat path in `effects/mod.rs:187-189` does emit it. This is a pre-existing asymmetry not introduced by infect, but it means combat damage has different event semantics from effect damage. Not an infect bug. **Fix:** (deferred) Consider emitting `LifeLost` in the combat non-infect player damage path for consistency, or document that `CombatDamageDealt` is the sole event for combat life changes. |

### Finding Details

#### Finding 1: Missing infect+lifelink test

**Severity**: LOW
**File**: `crates/engine/tests/keywords.rs:2601-3323`
**CR Rule**: 120.3f -- "Damage dealt by a source with lifelink causes that source's controller to gain that much life, in addition to the damage's other results."
**Rulings**: Every infect card in Scryfall has a ruling: "Damage from a source with infect is damage in all respects. If the source with infect also has lifelink, damage dealt by that source also causes its controller to gain that much life."
**Issue**: The plan identified infect+lifelink as an edge case (plan section "Infect + Lifelink") and the implementation correctly handles it (combat.rs lines 1063-1067 apply lifelink gains regardless of infect), but no test validates the interaction. This is the most commonly cited infect ruling across all infect cards.
**Fix**: Add a test `test_702_90_infect_lifelink_controller_gains_life` -- p1 has a creature with both Infect and Lifelink that attacks p2 unblocked. Assert: p2 gets poison counters, p2's life total is unchanged, p1's life total increases by the damage amount, LifeGained event is emitted for p1.

#### Finding 2: Pre-existing combat LifeLost event asymmetry

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:1016-1020`
**CR Rule**: 119.2 -- "Damage dealt to a player normally causes that player to lose that much life."
**Issue**: This is NOT an infect-specific bug. Before infect was added, combat damage to players already reduced `life_total` without emitting `LifeLost`. The effects/mod.rs path emits `LifeLost` for non-combat damage. This asymmetry predates the infect change. The infect implementation correctly mirrors the existing pattern: infect combat damage emits `PoisonCountersGiven` (new event), while non-infect combat damage emits nothing (pre-existing). For infect specifically, `PoisonCountersGiven` IS emitted in both combat and non-combat paths, so the infect event coverage is actually better than the pre-existing non-infect path.
**Fix**: Deferred. Not an infect bug. If addressed, emit `LifeLost` in the combat non-infect player damage branch for consistency with effects/mod.rs. This would be a separate change.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.90a (static ability) | Yes | Yes | No trigger wiring needed; correctly categorized |
| 702.90b (player damage -> poison) | Yes | Yes | `test_702_90_infect_combat_damage_gives_poison_counters_to_player`, `test_702_90_infect_noncombat_damage_player_gives_poison` |
| 702.90c (creature damage -> -1/-1 counters) | Yes | Yes | `test_702_90_infect_combat_damage_places_minus_counters_on_creature`, `test_702_90_infect_noncombat_damage_creature_places_counters` |
| 702.90d (last known information) | Yes | N/A | Uses `calculate_characteristics` at damage-application time (same as wither); no separate test but implicitly covered by all tests using the same path |
| 702.90e (functions from any zone) | Yes | Yes | Non-combat tests exercise from-any-zone (no zone restriction in keyword check); combat tests exercise from battlefield |
| 702.90f (redundant instances) | Yes | Yes | `test_702_90_infect_redundant_instances` + OrdSet deduplication |
| 120.3a (normal damage -> life loss) | Yes | Yes | Non-infect path preserved in both combat.rs and effects/mod.rs |
| 120.3b (infect damage -> poison) | Yes | Yes | Both combat and non-combat paths tested |
| 120.3c (planeswalker damage -> loyalty) | Yes | Yes | `test_702_90_infect_does_not_affect_planeswalker_damage` -- infect does NOT modify PW damage |
| 120.3d (wither/infect -> -1/-1 counters) | Yes | Yes | `test_702_90_infect_wither_overlap_creature` -- wither+infect fires once |
| 120.3e (normal creature damage marking) | Yes | Yes | Preserved in else branch; attacker receives normal damage in `test_702_90_infect_combat_damage_places_minus_counters_on_creature` |
| 120.3f (lifelink applies additionally) | Yes | No | Implementation correct (combat.rs:1063-1067 independent of infect), but no test. See Finding 1 |
| 704.5c (10+ poison = lose) | Pre-existing | Yes | `test_702_90_infect_kills_via_poison_sba` |
| 903.10a (commander damage tracking) | Yes | Yes | `test_702_90_infect_commander_damage_still_tracks` |

## Implementation Quality Notes

**Strengths**:
- Clean code reuse: the creature damage path correctly uses `source_wither || source_infect` instead of duplicating the -1/-1 counter logic (combat.rs:978, effects/mod.rs:249).
- Both combat and non-combat paths handle infect consistently.
- Separate `PoisonCountersGiven` event provides proper observability without conflating poison with life loss.
- Hash coverage complete for both `KeywordAbility::Infect` (discriminant 63) and `GameEvent::PoisonCountersGiven` (discriminant 81, all 3 fields hashed).
- Commander damage tracking correctly preserved for infect -- combat damage counts regardless of life/poison result (combat.rs:1022-1052).
- All tests cite CR rules in both doc comments and assertion messages.
- No `.unwrap()` in engine library code; tests use `.unwrap()` appropriately.
- `OrdSet` for keywords inherently handles CR 702.90f (redundant instances).
- Planeswalker damage correctly unaffected by infect (separate match arm in combat.rs:1054-1060, explicit type check in effects/mod.rs:219).
