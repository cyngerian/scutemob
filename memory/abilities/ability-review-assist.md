# Ability Review: Assist

**Date**: 2026-03-05
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.132
**Files reviewed**:
- `crates/engine/src/state/types.rs:939-951` (KeywordAbility::Assist)
- `crates/engine/src/state/hash.rs:548-549` (hash discriminant 105)
- `crates/engine/src/rules/command.rs:199-211` (CastSpell fields)
- `crates/engine/src/rules/engine.rs:97-122` (command dispatch)
- `crates/engine/src/rules/casting.rs:55-73,2172-2235` (handle_cast_spell signature + enforcement)
- `crates/engine/src/rules/casting.rs:2285-2287` (assist_events emission)
- `crates/engine/src/testing/replay_harness.rs:261-267,1181-1210` (harness support)
- `crates/engine/src/testing/script_schema.rs:298-308` (schema fields)
- `crates/engine/tests/script_replay.rs:155-183` (schema-to-harness threading)
- `tools/replay-viewer/src/view_model.rs:769` (display string)
- `crates/engine/tests/assist.rs` (11 tests, 814 lines)

## Verdict: clean

The implementation correctly models CR 702.132a. The enforcement logic validates all
required conditions: spell has the Assist keyword, assisting player is not the caster,
assisting player is active (not eliminated), assist amount does not exceed generic mana
in the total cost, and the assisting player has sufficient mana. The cost pipeline order
(assist after convoke/improvise/delve, before caster payment) is correct. All 11 tests
cover the planned scenarios including positive, negative, and interaction cases. No HIGH
or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `types.rs:950` | **Unsourced redundancy claim.** Doc comment says "CR 702.132a: Multiple instances are redundant" but 702.132a does not explicitly state this. **Fix:** Change to "Multiple instances are functionally redundant (inferred -- Assist has no variable parameter)." |
| 2 | LOW | `casting.rs:2207-2218` | **Redundant mana check.** Lines 2207-2210 check `assist_pool_total < assist_amount`, then lines 2216-2218 call `can_pay_cost(assist_pool, &assist_cost)` which performs essentially the same check. One is sufficient. **Fix:** Remove the first `assist_pool_total` check (lines 2207-2210) since `can_pay_cost` is the canonical check and handles colored mana edge cases (though assist only uses generic, the pattern should be consistent with other cost checks). |

### Finding Details

#### Finding 1: Unsourced redundancy claim

**Severity**: LOW
**File**: `crates/engine/src/state/types.rs:950`
**CR Rule**: 702.132a -- the rule only has one subrule and does not mention multiple instances.
**Issue**: The doc comment states "CR 702.132a: Multiple instances are redundant" but this is not an explicit statement in the Comprehensive Rules. It is a correct inference (Assist has no numerical parameter and the CastSpell command only has one assist_player/assist_amount pair), but citing it as "CR 702.132a" is misleading.
**Fix**: Change the comment to: `/// Multiple instances are functionally redundant (inferred — no variable parameter).`

#### Finding 2: Redundant mana check

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:2207-2218`
**CR Rule**: 702.132a -- "the player you chose may pay for any amount of the generic mana"
**Issue**: Two consecutive checks verify the assisting player can pay: first `assist_pool_total < assist_amount` (line 2208), then `can_pay_cost(assist_pool, &assist_cost)` (line 2217). Since `assist_cost` is a pure generic cost (`ManaCost { generic: assist_amount, ..Default::default() }`), both checks are equivalent. The `can_pay_cost` call is the canonical pattern used elsewhere in the casting pipeline.
**Fix**: Remove lines 2207-2210 (the `assist_pool_total` check). Keep the `can_pay_cost` call for consistency with other cost-checking patterns.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.132a "choose another player" | Yes | Yes | test 3 (self-assist rejected), test 11 (P3 assists) |
| 702.132a "any amount of the generic mana" | Yes | Yes | test 1 (partial), test 4 (exceeds), test 6 (all generic) |
| 702.132a "total cost" (after modifications) | Yes | Yes | test 7 (convoke reduces ceiling) |
| 702.132a optional ("you may") | Yes | Yes | test 2 (no assist), test 8 (amount=0) |
| 601.2g-h mana ability ordering | N/A | N/A | Engine pre-resolves mana; no interactive mana activation |
| CR 800.4a eliminated players | Yes | Yes | test 5 (eliminated player rejected) |
| Ruling: only generic mana | Yes | Yes | test 4 (exceeds generic component) |
| Ruling: total cost after modifications | Yes | Yes | test 7 (convoke+assist interaction) |
| Insufficient mana (assisting player) | Yes | Yes | test 9 |
| Non-assist spell rejected | Yes | Yes | test 10 |

## Previous Findings (re-review only)

N/A -- first review.
