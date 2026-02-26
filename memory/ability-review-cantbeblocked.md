# Ability Review: CantBeBlocked

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 509.1b (blocking restrictions), 113.12 (quality statement, not ability)
**Files reviewed**:
- `crates/engine/src/state/types.rs:168-172`
- `crates/engine/src/state/hash.rs:309`
- `crates/engine/src/rules/combat.rs:427-451`
- `crates/engine/src/cards/definitions.rs:400-437` (Rogue's Passage)
- `crates/engine/src/cards/definitions.rs:1370-1408` (Whispersilk Cloak)
- `crates/engine/tests/keywords.rs:1510-1787` (5 new tests)
- `crates/engine/tests/card_def_fixes.rs:572-622` (pre-existing test)

## Verdict: clean

The CantBeBlocked implementation is correct. The blocking restriction at `combat.rs:441-451`
faithfully implements CR 509.1b -- any blocker assignment against a creature with the
CantBeBlocked keyword is rejected. The Whispersilk Cloak fix correctly grants CantBeBlocked
via a static continuous effect in Layer::Ability to the AttachedCreature, mirroring the
existing Shroud grant. All 5 new tests are well-structured, cite CR rules, and cover the
important cases including the end-to-end activated-ability-to-layer-calculation pipeline.
No HIGH or MEDIUM findings. Two LOW findings related to CR citation precision and check
ordering are noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `types.rs:168` | **Imprecise CR citation.** Comment says "CR 509.1" but the specific restriction rule is 509.1b. **Fix:** Change to "CR 509.1b". |
| 2 | LOW | `combat.rs:441` | **Imprecise CR citation.** Comment says "CR 509.1" but should say "CR 509.1b". **Fix:** Change to "CR 509.1b". |
| 3 | LOW | `combat.rs:427-451` | **Check ordering: flying before CantBeBlocked.** A creature with both CantBeBlocked and Flying attacking against a non-flying/non-reach blocker will get rejected with a "flying" error message rather than "CantBeBlocked". Functionally correct (block is still rejected), but error message is misleading. Not a bug per CR 509.1b (all restrictions are checked; any violation makes it illegal). **Fix:** Move the CantBeBlocked check before the flying check for more accurate error messages, or accept as-is since the outcome is correct. |

### Finding Details

#### Finding 1: Imprecise CR citation in types.rs

**Severity**: LOW
**File**: `crates/engine/src/state/types.rs:168`
**CR Rule**: 509.1b -- "The defending player checks each creature they control to see whether it's affected by any restrictions (effects that say a creature can't block, or that it can't block unless some condition is met). If any restrictions are being disobeyed, the declaration of blockers is illegal."
**Issue**: The doc comment says `CR 509.1: "This creature can't be blocked."` but CR 509.1 is the parent rule about the declare-blockers process. The specific restriction-checking subrule is CR 509.1b.
**Fix**: Change the comment to `CR 509.1b` for precision. The quoted text is also not from the CR itself -- it is descriptive. Consider: `/// CR 509.1b: Blocking restriction — "can't be blocked" quality (see also CR 113.12).`

#### Finding 2: Imprecise CR citation in combat.rs

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:441`
**CR Rule**: 509.1b (same as above)
**Issue**: Comment says `CR 509.1 / KeywordAbility::CantBeBlocked` but should cite `CR 509.1b` specifically.
**Fix**: Change to `CR 509.1b`.

#### Finding 3: Check ordering preference

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:427-451`
**CR Rule**: 509.1b -- All restrictions are checked; any violation makes the declaration illegal.
**Issue**: The flying check (lines 427-439) precedes the CantBeBlocked check (lines 441-451). When a creature has both CantBeBlocked and Flying, and the blocker lacks flying/reach, the flying check fires first. The block is correctly rejected, but the error message says "attacker has flying" instead of "CantBeBlocked keyword". Since CantBeBlocked is a stronger restriction (absolute, no conditions), checking it first would produce clearer diagnostics. Per CR 509.1b, all restrictions are checked as a batch and any violation is sufficient, so this is purely an error-message quality issue.
**Fix**: Optionally reorder: move the CantBeBlocked check (lines 441-451) before the flying check (lines 427-439). Or accept as-is since the result is correct. No functional impact.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 509.1a (defender chooses blockers, must be untapped) | Yes | Yes | Pre-existing in combat.rs:354-367 |
| 509.1b (blocking restrictions -- CantBeBlocked) | Yes | Yes | combat.rs:441-451; tests 1-4 in keywords.rs |
| 509.1h (unblocked creature with no blockers) | Yes | Yes | test_509_1b_cant_be_blocked_allows_no_blockers |
| 113.12 (quality statement, not ability) | Acknowledged (LOW gap) | No | Engine models as keyword; Muraganda Petroglyphs interaction would be incorrect. Documented in plan as known LOW gap. |
| CantBeBlocked via continuous effect (Layer 6) | Yes | Yes | test_509_1b_cant_be_blocked_via_continuous_effect (end-to-end) |
| CantBeBlocked + Flying (absolute restriction) | Yes | Yes | test_509_1b_cant_be_blocked_plus_flying |
| CantBeBlocked + other attacker (creature-specific) | Yes | Yes | test_509_1b_cant_be_blocked_other_attacker_can_be_blocked |
| Whispersilk Cloak (static grant to attached creature) | Yes | No (unit test) | Card def verified correct; would need equip + combat script to fully test. Existing Shroud grant uses identical pattern. |
| Rogue's Passage (activated ability, UntilEndOfTurn) | Yes | Yes | Card def at definitions.rs:400-437; test via continuous_effect test |
| Already-blocked timing (Rogue's Passage ruling) | Yes (by design) | No (explicit test) | The check is only in `handle_declare_blockers`, so granting CantBeBlocked after blockers are declared has no effect. Correct by architecture, not explicitly tested. |

## Notes

- The implementation is pre-existing (enum variant, hash, combat enforcement were already in place). The new work consists of 5 tests in keywords.rs and the Whispersilk Cloak card definition fix.
- The Whispersilk Cloak fix (definitions.rs:1386-1395) correctly mirrors the Shroud grant pattern at definitions.rs:1378-1384. Both use `Static` ability with `AttachedCreature` filter and `WhileSourceOnBattlefield` duration.
- The end-to-end test (test 5) is particularly valuable: it validates the full pipeline from activated ability through stack resolution to continuous effect application in the layer system. This is the test that would catch regressions in `ApplyContinuousEffect` or `DeclaredTarget` resolution.
- The pre-existing test in card_def_fixes.rs:572 provides additional coverage using a different setup pattern.
