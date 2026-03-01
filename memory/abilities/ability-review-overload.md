# Ability Review: Overload

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.96
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 591-599)
- `crates/engine/src/cards/card_definition.rs` (lines 255-263, 816-821)
- `crates/engine/src/state/stack.rs` (lines 127-134)
- `crates/engine/src/rules/command.rs` (lines 157-166)
- `crates/engine/src/rules/casting.rs` (lines 68, 485-535, 591-600, 737-749, 974-975, 1080, 1123, 1343-1362)
- `crates/engine/src/rules/engine.rs` (lines 96, 117)
- `crates/engine/src/effects/mod.rs` (lines 66-69, 81, 98, 1341, 1359, 2759-2760)
- `crates/engine/src/state/hash.rs` (lines 457-458, 1413-1414, 2462-2463, 2885-2888)
- `crates/engine/src/rules/resolution.rs` (lines 179-186, 1434-1435)
- `crates/engine/src/rules/copy.rs` (lines 192-193, 369)
- `crates/engine/src/rules/abilities.rs` (lines 344, 565, 738, 2035, 2136, 2488)
- `crates/engine/src/testing/replay_harness.rs` (lines 285, 312, 336, 360, 385, 410, 440, 664, 668-690)
- `tools/replay-viewer/src/view_model.rs` (line 679)
- `crates/engine/tests/overload.rs` (all 827 lines)

## Verdict: needs-fix

The overload engine mechanism is correctly implemented across all core files: alternative cost
selection, mutual exclusion with other alternative costs (CR 118.9a), target suppression
(CR 702.96b), `was_overloaded` flag propagation through StackObject/EffectContext/ForEach
contexts, hash coverage, copy handling, and replay harness integration. The implementation
follows established patterns (Evoke for alternative costs, Kicker for conditional effect
dispatch). However, the test suite has three MEDIUM gaps: the mock card definition does not
filter by controller (making tests less realistic), the commander tax test from the plan was
dropped, and no test covers 3+ player scenarios despite this being a Commander-first engine
where overload is one of the most impactful mechanics.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tests/overload.rs:100-102` | **Mock Vandalblast does not restrict to opponent's artifacts.** Test 2 only has P2 artifacts, hiding the gap. |
| 2 | MEDIUM | `tests/overload.rs` (missing) | **No commander tax + overload test (CR 118.9d).** Plan specified this as test 8; was replaced. |
| 3 | MEDIUM | `tests/overload.rs` (missing) | **No multiplayer (3+ player) test.** All tests use 2 players. Overload is Commander's most impactful mechanic. |
| 4 | LOW | `tests/overload.rs:729-826` | **Test 8 duplicates test 1.** Both test normal cast destroying a single target with WasOverloaded=false. |

### Finding Details

#### Finding 1: Mock Vandalblast does not restrict to opponent's artifacts

**Severity**: MEDIUM
**File**: `crates/engine/tests/overload.rs:100-102`
**CR Rule**: 702.96a -- "change its text by replacing all instances of the word 'target' with the word 'each'"
**Issue**: The mock Vandalblast's overloaded branch uses `ForEachTarget::EachPermanentMatching(TargetFilter { has_card_type: Some(CardType::Artifact), ..Default::default() })`. The default `TargetController::Any` means the overloaded spell destroys ALL artifacts, including the caster's own. The real Vandalblast says "Destroy target artifact you don't control" / "Destroy each artifact you don't control." Test 2 (`test_702_96_overloaded_cast_destroys_all_matching`) only places artifacts under P2's control, so this bug is hidden. If a P1-controlled artifact were present, the test would pass even though it would be incorrectly destroyed.

Note: The root cause is that `collect_for_each` / `matches_filter` do not check `TargetFilter.controller`. This is a pre-existing system gap (not introduced by overload). However, the test should still include P1's own artifact to document the expected behavior and catch future regressions when the controller filter is implemented.

**Fix**: In `mock_vandalblast_def()`, set `controller: TargetController::Opponent` on the overloaded TargetFilter (even though it's not enforced yet -- documents intent). In `test_702_96_overloaded_cast_destroys_all_matching`, add an artifact under P1's control and assert it survives. When `matches_filter` is updated to check `controller` (a separate LOW remediation item), this test will validate the fix.

#### Finding 2: No commander tax + overload test (CR 118.9d)

**Severity**: MEDIUM
**File**: `crates/engine/tests/overload.rs` (missing test)
**CR Rule**: 118.9d -- "If an alternative cost is being paid to cast a spell, any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost."
**Issue**: The plan specified test 8 as `test_overload_commander_tax_applies` -- cast a commander with overload from the command zone, verify that overload cost + commander tax is paid. This tests CR 118.9d, which is a critical interaction for Commander format. The test was not implemented; instead, test 8 is a condition-false test that largely duplicates test 1.

**Fix**: Add a test `test_702_96_commander_tax_applies_to_overload_cost` that:
1. Creates a mock commander sorcery with Overload.
2. Places it in ZoneId::CommandZone with commander tax of {2} (one prior cast).
3. Casts it with overload from the command zone.
4. Verifies that mana consumed is overload cost + {2} commander tax.
5. Also test: insufficient mana for overload+tax is rejected.

#### Finding 3: No multiplayer (3+ player) test

**Severity**: MEDIUM
**File**: `crates/engine/tests/overload.rs` (missing test)
**CR Rule**: 702.96a/b -- overloaded spell replaces "target" with "each", affecting all valid objects.
**Issue**: All 8 tests use exactly 2 players (P1 and P2). The project is Commander-first (architecture invariant 5: "Multiplayer-first... 1v1 is N=2, not a special case"). Overloaded spells like Cyclonic Rift are the most powerful cards in Commander specifically because they affect ALL opponents' permanents. A 2-player test cannot verify that "each" means all opponents, not just one. The plan itself called out this edge case (edge case 7: "Overloaded Cyclonic Rift returns EACH nonland permanent the caster doesn't control to its owner's hand -- across all opponents").

**Fix**: Add a test `test_702_96_overloaded_affects_all_opponents_multiplayer` with 4 players:
1. P1 casts overloaded Mock Vandalblast.
2. P2 has 2 artifacts, P3 has 1 artifact, P4 has 1 artifact, P1 has 1 artifact.
3. Resolve. Assert: P2's, P3's, and P4's artifacts are all destroyed; P1's artifact survives (once controller filter is working; for now, note it's destroyed due to Finding 1's gap).
4. This validates multiplayer correctness and documents the expected controller-filtering behavior.

#### Finding 4: Test 8 duplicates test 1

**Severity**: LOW
**File**: `crates/engine/tests/overload.rs:729-826`
**CR Rule**: 702.96a
**Issue**: Test 8 (`test_702_96_condition_was_overloaded_false_when_not_overloaded`) does the same thing as test 1 (`test_702_96_normal_cast_targets_single`): cast Mock Vandalblast normally for {R}, targeting Sol Ring, verify only Sol Ring is destroyed while Arcane Signet survives, and check `was_overloaded == false`. Test 1 already checks `!state.stack_objects[0].was_overloaded` at line 218-221. This test provides minimal incremental value. The slot would be better used for the commander tax test (Finding 2).

**Fix**: Replace test 8 with the commander tax test from Finding 2. Alternatively, keep it but add a unique assertion (e.g., verify `Condition::WasOverloaded` is false in the event stream, or test that the `if_false` branch effect executed via event inspection).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.96a (alternative cost, pay overload cost) | Yes | Yes | Tests 2, 6 verify cost payment |
| 702.96a (text change: "target" -> "each") | Yes (via Condition::WasOverloaded) | Yes | Tests 1, 2, 4, 8 verify branching |
| 702.96a (follows CR 601.2b, 601.2f-h) | Yes | Yes | Casting pipeline handles this |
| 702.96b (no targets when overloaded) | Yes | Yes | Test 7 (targets rejected), Test 3 (cannot fizzle) |
| 702.96b (may affect non-targetable objects) | Yes | Yes | Test 4 (hexproof bypassed) |
| 702.96c (text-changing effect per CR 612) | Yes (semantic, not literal) | No | Modeled via conditional dispatch; no literal text change needed |
| 118.9a (only one alternative cost) | Yes | Partial | Test 5 checks evoke+overload; other combinations not individually tested |
| 118.9c (mana value unchanged) | Yes (implicit) | No | Engine reads printed mana cost for MV; no test verifies this |
| 118.9d (additional costs on top of alt cost) | Yes (code path) | **No** | Commander tax code path exists but is untested for overload |
| Multiplayer (all opponents affected) | Yes (via ForEach) | **No** | No 3+ player test |
| Copy handling (was_overloaded=false on copies) | Yes | No | copy.rs correctly sets false; no test |

## Notes

### Pre-existing System Gaps (not introduced by overload)

- `matches_filter()` and `collect_for_each()` in `effects/mod.rs` do not check `TargetFilter.controller`. This means `TargetController::Opponent` is semantically correct but not enforced. This affects ALL cards using `ForEachTarget::EachPermanentMatching` with controller constraints, not just overload cards. This is a known LOW from the remediation tracker.

### Implementation Quality

- All 14 `StackObject` construction sites across `casting.rs`, `copy.rs`, `abilities.rs`, and `resolution.rs` correctly include `was_overloaded`.
- All `Command::CastSpell` construction sites in 44 test files include `cast_with_overload: false`.
- All 10 `Command::CastSpell` construction sites in `replay_harness.rs` include `cast_with_overload: false`.
- `ForEach` inner context propagation correctly passes `was_overloaded` (both player-based and object-based paths).
- Hash coverage is complete: `StackObject.was_overloaded` (line 1414), `Condition::WasOverloaded` (discriminant 9, line 2463), `AbilityDefinition::Overload` (discriminant 21, line 2886), `KeywordAbility::Overload` (discriminant 70, line 458).
- CR citations are present on all new code.
- The `#[serde(default)]` annotations ensure backward compatibility.
- Copy handling (`was_overloaded: false`) is correct per CR 707.10 -- copies are not cast.
