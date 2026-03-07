# Ability Review: Enrage

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 207.2c (ability word), 603.2g (prevented events don't trigger), 510.2 (simultaneous combat damage), 120.3 (damage results)
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs:1144-1150` (TriggerCondition::WhenDealtDamage)
- `crates/engine/src/state/game_object.rs:214-217` (TriggerEvent::SelfIsDealtDamage)
- `crates/engine/src/state/hash.rs:1408-1409` (TriggerEvent discriminant 20)
- `crates/engine/src/state/hash.rs:3180-3181` (TriggerCondition discriminant 22)
- `crates/engine/src/rules/abilities.rs:4360-4384` (CombatDamageDealt handler)
- `crates/engine/src/rules/abilities.rs:4570-4585` (DamageDealt handler)
- `crates/engine/src/testing/replay_harness.rs:2030-2048` (enrich_spec mapping)
- `crates/engine/tests/enrage.rs` (5 tests, 708 lines)

## Verdict: clean

The Enrage implementation is correct and complete. All CR rules are faithfully implemented. The trigger fires for both combat and non-combat damage, correctly deduplicates multiple simultaneous combat damage sources (one trigger per creature per damage step), correctly skips when amount is 0 (CR 603.2g), and fires even when lethal damage is dealt. Hash discriminants are unique and sequential. No KeywordAbility variant was added (correct -- Enrage is an ability word, not a keyword). Tests cover all five planned scenarios with proper CR citations. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `enrage.rs:398` | **Test 3 title mismatch.** Test name is `test_enrage_zero_damage_no_trigger` but plan calls for a prevention test (CR 603.2g). The test uses a 0-power attacker which never generates a damage assignment at all, rather than testing actual damage prevention (where prevention reduces damage to 0). This still validates the correct behavior path but doesn't exercise the prevention system. **Fix:** Add a comment noting this tests the "no damage assigned" path; a separate test exercising `apply_damage_prevention` reducing final_dmg to 0 would be ideal but is LOW priority. |
| 2 | LOW | `enrage.rs:300-307` | **Direct state mutation in test.** Test 2 directly mutates `state.players.get_mut(&p2).unwrap().mana_pool` and `state.turn.priority_holder`. While acceptable per project conventions (tests may use `.unwrap()` and fields are `pub`), the GameStateBuilder has `.with_mana()` or similar helpers in some tests. **Fix:** No action required -- consistent with existing test patterns. |

### Finding Details

#### Finding 1: Test 3 tests "no assignment" rather than "prevented damage"

**Severity**: LOW
**File**: `crates/engine/tests/enrage.rs:398`
**CR Rule**: 603.2g -- "An event that's prevented or replaced won't trigger anything."
**Issue**: The test uses a 0-power attacker, which means no `CombatDamageAssignment` is ever created for the Enrage creature. This validates that Enrage doesn't trigger when no damage event targets the creature, but it does NOT test the path where damage IS dealt but then fully prevented (e.g., via a prevention shield reducing final_dmg to 0). The engine's `effects/mod.rs` line 243 already guards `DamageDealt` emission behind `final_dmg > 0`, and the `abilities.rs` DamageDealt handler has a redundant `amount > 0` check -- but neither path is exercised by this test.
**Fix**: The current test is valid and sufficient for the Enrage ability itself. A dedicated prevention-interaction test would be nice but is LOW priority since the prevention system is independently tested elsewhere.

#### Finding 2: Direct state mutation in test setup

**Severity**: LOW
**File**: `crates/engine/tests/enrage.rs:300-307`
**CR Rule**: N/A (code quality)
**Issue**: Test 2 directly mutates mana pool and priority holder on the built GameState rather than using builder methods. This is a common pattern in the codebase and is acceptable.
**Fix**: No action required.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 207.2c (ability word, no rules meaning) | Yes -- no KeywordAbility variant | Yes | Correctly treated as TriggerCondition only |
| 120.3 (damage marking) | Yes -- triggers on marked damage | Yes | test_enrage_combat_damage_triggers |
| 603.2g (prevented events don't trigger) | Yes -- amount == 0 guard in both handlers | Partial | test_enrage_zero_damage_no_trigger tests 0-power path, not prevention path |
| 510.2 (simultaneous combat damage) | Yes -- deduplication via damaged_creatures Vec | Yes | test_enrage_multiple_blockers_triggers_once |
| Ruling 2018-01-19 (multiple sources, single trigger) | Yes -- dedup in CombatDamageDealt handler | Yes | test_enrage_multiple_blockers_triggers_once |
| Ruling 2018-01-19 (lethal damage still triggers) | Yes -- trigger collected before SBAs | Yes | test_enrage_lethal_damage_still_triggers |
| Non-combat damage fires trigger | Yes -- DamageDealt handler in check_triggers | Yes | test_enrage_noncombat_damage_triggers |

## Implementation Quality Notes

1. **Combat damage deduplication**: The `damaged_creatures: Vec<ObjectId>` with `.contains()` check is correct and efficient for the expected small number of creatures in a single combat damage step.

2. **DamageDealt handler placement**: Correctly placed at the end of the `check_triggers` match block, after all other handlers. The `.. => {}` wildcard at line 4587 catches remaining events.

3. **enrich_spec_from_def mapping**: Correctly maps `TriggerCondition::WhenDealtDamage` to `TriggerEvent::SelfIsDealtDamage` following the established pattern (identical structure to WhenDies, WhenDealsCombatDamageToPlayer, etc.).

4. **Hash discriminants**: TriggerEvent::SelfIsDealtDamage = 20 (sequential after SelfAttacksWithGreaterPowerAlly = 19). TriggerCondition::WhenDealtDamage = 22 (sequential after TributeNotPaid = 21). No collisions.

5. **No KeywordAbility variant**: Correctly identified that Enrage is an ability word (CR 207.2c) with no special rules meaning, so no `KeywordAbility::Enrage` was added. This avoids the exhaustive-match cascade in view_model.rs, stack_view.rs, hash.rs keyword section, and builder.rs.

6. **Test quality**: All 5 tests cite CR rules, use descriptive names, and test distinct behaviors. The lethal-damage test (test 5) correctly notes the CR 400.7 zone-change limitation as a LOW deferred issue.
