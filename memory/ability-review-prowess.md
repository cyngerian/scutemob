# Ability Review: Prowess

**Date**: 2026-02-25
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.108
**Files reviewed**:
- `crates/engine/src/state/types.rs` (enum variant at line 123)
- `crates/engine/src/state/hash.rs` (hash arms at lines 282, 548, 879)
- `crates/engine/src/state/continuous_effect.rs` (EffectFilter::Source at lines 98-102)
- `crates/engine/src/state/game_object.rs` (TriggerEvent::ControllerCastsNoncreatureSpell at lines 118-122)
- `crates/engine/src/effects/mod.rs` (Source resolution at line 898)
- `crates/engine/src/rules/abilities.rs` (SpellCast trigger dispatch at lines 314-361)
- `crates/engine/src/rules/layers.rs` (Source filter fallback at line 220)
- `crates/engine/src/state/builder.rs` (Prowess keyword-to-TriggeredAbilityDef at lines 378-395)
- `crates/engine/src/testing/replay_harness.rs` (enrich_spec_from_def at lines 340-409)
- `crates/engine/src/rules/resolution.rs` (TriggeredAbility resolution at lines 277-331)
- `crates/engine/src/rules/engine.rs` (check_triggers + flush_pending_triggers at lines 78-87)
- `crates/engine/tests/prowess.rs` (8 tests)

## Verdict: clean

The Prowess implementation correctly follows CR 702.108a and 702.108b. The trigger dispatch properly checks that the cast spell lacks `CardType::Creature`, filters by controller ("you"), and generates a `TriggeredAbilityDef` that applies a `PtModify` / `ModifyBoth(1)` / `UntilEndOfTurn` continuous effect to the source creature via the new `EffectFilter::Source` mechanism. All three new hash arms are present (KeywordAbility::Prowess = 16, EffectFilter::Source = 12, TriggerEvent::ControllerCastsNoncreatureSpell = 7). The `enrich_spec_from_def` dual-site concern is not an issue because keywords added there flow through `GameStateBuilder::build()` which handles keyword-to-trigger expansion. Tests cover all planned scenarios across 8 tests including positive, negative, edge-case, and 4-player multiplayer cases. No HIGH or MEDIUM findings. Three LOW observations below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `crates/engine/tests/prowess.rs:617-619` | **Direct call to expire function instead of full turn cycle.** Test 6 calls `expire_end_of_turn_effects` directly rather than advancing through the full turn cycle with `pass_all`. Pragmatic but slightly less rigorous. |
| 2 | LOW | `crates/engine/tests/prowess.rs` | **No test for prowess creature leaving battlefield before trigger resolves.** If the creature is destroyed in response to the prowess trigger, the trigger still resolves but has no effect (correct behavior per resolution.rs lines 310-313). A test would document this edge case. |
| 3 | LOW | `crates/engine/src/state/builder.rs:349-350` | **CR 702.108b relies on Vec iteration order, not OrdSet.** The triggered ability generation iterates `spec.keywords` (Vec) at line 350, not `spec_keywords` (OrdSet) at line 349. This means duplicate Prowess entries in the Vec correctly produce multiple TriggeredAbilityDefs per CR 702.108b. However, the stored `characteristics.keywords` (OrdSet) deduplicates them. This is correct for current usage but the implicit contract (Vec preserves duplicates for trigger generation, OrdSet deduplicates for display) is subtle and undocumented. |

### Finding Details

#### Finding 1: Direct call to expire function instead of full turn cycle

**Severity**: LOW
**File**: `crates/engine/tests/prowess.rs:617-619`
**CR Rule**: 514.2 -- "At the beginning of the cleanup step, ... all 'until end of turn' ... effects end."
**Issue**: Test 6 (`test_prowess_until_end_of_turn_expires`) calls `mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state)` directly rather than advancing through the full turn cycle. This tests the expiry mechanism in isolation but does not verify integration with the turn engine's cleanup step. The gotchas file notes that 1-player `start_game` doesn't reach cleanup, and the test correctly uses 2 players, so a full-cycle test would work.
**Fix**: Optional. Either add a comment explaining the design choice, or add a separate integration test that advances through the full turn and verifies expiry. Low priority.

#### Finding 2: No test for prowess creature leaving battlefield before trigger resolves

**Severity**: LOW
**File**: `crates/engine/tests/prowess.rs`
**CR Rule**: 702.108a / 400.7 -- The trigger resolves independently, but if the source creature left the battlefield, the effect lookup in `resolution.rs:310-313` returns `None` (CR 400.7: new object identity) and the trigger resolves without effect.
**Issue**: There is no test verifying that if the prowess creature leaves the battlefield after the trigger is on the stack but before it resolves, the trigger resolves without applying the +1/+1. This is handled correctly by the existing resolution code (`state.objects.get(&source_object)` returns `None` for dead ObjectIds), but a test would document this edge case and prevent regressions.
**Fix**: Optional. Add a test: p1 casts noncreature spell (prowess triggers), then destroy the prowess creature (e.g., via another spell or direct state manipulation), then resolve the prowess trigger -- verify no crash and no continuous effect is created.

#### Finding 3: Vec vs OrdSet for CR 702.108b multiple instances

**Severity**: LOW
**File**: `crates/engine/src/state/builder.rs:349-350`
**CR Rule**: 702.108b -- "If a creature has multiple instances of prowess, each triggers separately."
**Issue**: The keyword-to-trigger expansion loop iterates `spec.keywords` (a `Vec<KeywordAbility>`) at line 350, which preserves duplicates. This means two `Prowess` entries in the Vec produce two `TriggeredAbilityDef` entries, correctly implementing CR 702.108b. However, `spec_keywords` (an `OrdSet<KeywordAbility>`) at line 349 deduplicates for storage in `characteristics.keywords`. The implicit contract -- that trigger generation uses Vec (preserves dupes) while keyword storage uses OrdSet (dedupes) -- is correct but subtle. A comment at line 350 noting "Vec iteration preserves duplicates for CR 702.108b" would help future maintainers.
**Fix**: Optional. Add a one-line comment at line 350: `// Iterate Vec (not OrdSet) to preserve duplicate keywords for CR 702.108b (multiple instances trigger separately).`

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.108a (trigger definition) | Yes | Yes | Tests 1, 5, 6, 7 verify the trigger fires and applies +1/+1 UntilEndOfTurn |
| 702.108a (noncreature check) | Yes | Yes | Tests 2, 3 verify creature and artifact-creature spells do NOT trigger |
| 702.108a ("you cast" = controller) | Yes | Yes | Tests 4, 8 verify opponent spells do NOT trigger prowess |
| 702.108a (stack independence) | Yes | Yes | Test 5 verifies prowess resolves while spell remains on stack |
| 702.108a (UntilEndOfTurn) | Yes | Yes | Test 6 verifies expiry via direct expire call |
| 702.108a (additive stacking) | Yes | Yes | Test 7 verifies two spells give +2/+2 |
| 702.108b (multiple instances) | Partial | No | Correct by construction (Vec iteration) but no dedicated test with two Prowess instances |
| Storm copies not cast | Yes (pre-existing) | No (for prowess) | Storm emits SpellCopied not SpellCast; no prowess-specific test but engine architecture is correct |
| Cascade casts trigger prowess | Yes (pre-existing) | No (for prowess) | Cascade emits SpellCast at copy.rs:347; no prowess-specific test |

## Summary

Implementation is correct and well-structured. The `EffectFilter::Source` mechanism is a clean, reusable addition for "this permanent" self-referencing effects. All hash arms are present. Test coverage is thorough across 8 tests covering the key positive, negative, and multiplayer scenarios. The three LOW findings are documentation/test-gap items that can be addressed opportunistically.
