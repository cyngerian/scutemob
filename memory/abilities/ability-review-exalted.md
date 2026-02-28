# Ability Review: Exalted

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.83
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 256-262)
- `crates/engine/src/state/game_object.rs` (line 141-146)
- `crates/engine/src/state/stubs.rs` (line 66-74)
- `crates/engine/src/state/hash.rs` (lines 349-350, 904-905, 946-947)
- `crates/engine/src/state/builder.rs` (lines 397-416)
- `crates/engine/src/rules/abilities.rs` (lines 667-697, 983-990)
- `crates/engine/tests/exalted.rs` (full file, 617 lines)
- `tools/replay-viewer/src/view_model.rs` (line 598)

## Verdict: clean

The Exalted implementation is correct, well-structured, and closely follows the CR rule
text. The trigger dispatch correctly checks `attackers.len() == 1` for the "attacks alone"
condition (CR 702.83b), iterates ALL permanents controlled by the attacking player to fire
triggers (not just creatures), and tags each trigger with the lone attacker's ObjectId so
the `DeclaredTarget { index: 0 }` filter resolves to the correct creature at effect
execution time. The hash coverage is complete: `KeywordAbility::Exalted` (discriminant 34),
`TriggerEvent::ControllerCreatureAttacksAlone` (discriminant 11), and
`PendingTrigger.exalted_attacker_id` are all hashed. The 8 tests cover all planned
scenarios including multiplayer and edge cases. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `crates/engine/tests/exalted.rs` | **Missing test for exalted on non-creature permanent.** Plan edge case 7 identifies that exalted triggers on ALL permanents (not just creatures), but no test uses a non-creature permanent (e.g., enchantment) with exalted. Test 4 uses a non-attacking creature, which is close but not the same thing. **Fix:** Add a test where an enchantment ObjectSpec with `with_keyword(KeywordAbility::Exalted)` triggers when a creature attacks alone. |
| 2 | LOW | `crates/engine/tests/exalted.rs` | **Missing test for creature with exalted attacking alone itself.** The rulings explicitly state "each exalted ability on each permanent you control (including, perhaps, the attacking creature itself) will trigger." No test verifies a creature with exalted that IS the lone attacker gets +1/+1 from its own exalted. **Fix:** Add a test where a creature with exalted attacks alone and verify it receives the +1/+1 from its own exalted trigger. |

### Finding Details

#### Finding 1: Missing test for exalted on non-creature permanent

**Severity**: LOW
**File**: `crates/engine/tests/exalted.rs`
**CR Rule**: 702.83a -- "Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn."
**Rulings**: "each exalted ability on each permanent you control" (Akrasan Squire, Bant, etc.)
**Issue**: The plan identifies edge case 7: "Exalted triggers on ALL permanents you control, not just creatures." The existing tests use only creature ObjectSpecs with exalted. While the dispatch code correctly iterates all battlefield permanents (not filtering by creature type), there is no test verifying that a non-creature permanent (enchantment, artifact, land) with exalted actually triggers. Test 4 verifies a non-attacking creature, which is semantically different.
**Fix**: Add a test using `ObjectSpec::new(p1, "Exalted Enchantment").with_card_type(CardType::Enchantment).with_keyword(KeywordAbility::Exalted).in_zone(ZoneId::Battlefield)` that verifies the exalted trigger fires from the enchantment.

#### Finding 2: Missing test for self-exalted creature

**Severity**: LOW
**File**: `crates/engine/tests/exalted.rs`
**CR Rule**: 702.83a
**Rulings**: "each exalted ability on each permanent you control (including, perhaps, the attacking creature itself) will trigger" (Akrasan Squire ruling)
**Issue**: None of the 8 tests verifies the case where the lone attacker itself has exalted. This is explicitly called out in the official rulings. While the code handles it correctly (the dispatch iterates ALL permanents, including the attacker), a test confirming this ruling would improve confidence.
**Fix**: Add a test where a single creature with exalted attacks alone, and verify it receives +1/+1 from its own exalted ability.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.83a (triggered ability, +1/+1 until EOT) | Yes | Yes | test_exalted_basic_attacks_alone_gives_plus_one |
| 702.83a (bonus targets attacker, not source) | Yes | Yes | test_exalted_on_non_attacker_permanent_targets_attacker |
| 702.83a ("you control" -- only attacking player) | Yes | Yes | test_exalted_does_not_trigger_on_opponent_attack, test_exalted_multiplayer_only_attacker_controller_triggers |
| 702.83a (multiple instances stack) | Yes | Yes | test_exalted_multiple_instances_stack |
| 702.83a (until end of turn duration) | Yes | Yes | test_exalted_bonus_expires_at_end_of_turn |
| 702.83b (attacks alone = exactly one declared) | Yes | Yes | test_exalted_does_not_trigger_with_multiple_attackers |
| 702.83b (zero attackers = no trigger) | Yes | Yes | test_exalted_with_zero_attackers_does_not_trigger |
| 506.5 (attacks alone definition) | Yes | Yes | Checked via attackers.len() == 1 |
| Ruling: non-creature permanents trigger | Yes | No (LOW) | Dispatch code iterates all permanents, but no test uses a non-creature with exalted |
| Ruling: attacking creature's own exalted triggers | Yes | No (LOW) | Code handles it, no dedicated test |
| Ruling: additional combat phases | Yes (infra) | No | UntilEndOfTurn duration already handles this; no explicit multi-combat test |

## Previous Findings (re-review only)

N/A -- this is the initial review.
