# Ability Review: Squad

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.157
**Files reviewed**: `crates/engine/src/state/types.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/state/stack.rs`, `crates/engine/src/state/game_object.rs`, `crates/engine/src/state/stubs.rs`, `crates/engine/src/state/builder.rs`, `crates/engine/src/rules/command.rs`, `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/resolution.rs`, `crates/engine/src/rules/abilities.rs`, `crates/engine/src/cards/card_definition.rs`, `crates/engine/src/effects/mod.rs`, `crates/engine/src/testing/replay_harness.rs`, `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`, `crates/engine/tests/squad.rs`

## Verdict: clean

The Squad implementation is well-structured, follows the established Ravenous/Myriad patterns correctly, and matches CR 702.157 faithfully. All discriminants are correct and hash coverage is complete. The intervening-if check uses layer-resolved characteristics at trigger-queue time as required by the 2022-10-07 ruling. Token creation follows the Myriad token-copy pattern with correct Layer 1 CopyOf continuous effects. The source-left-battlefield check inside the token creation loop is a sound defensive measure (CR 608.2b / CR 400.7). No HIGH or MEDIUM findings. Two LOW items noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/squad.rs` | **Missing test for Squad keyword loss (Humility/Dress Down).** Plan test #6 not implemented. **Fix:** Add a test that places a continuous effect removing all abilities (simulating Humility) before spell resolution, then verifies no SquadTrigger fires. This requires a layer-6-removal continuous effect setup, so it may be deferred. |
| 2 | LOW | `resolution.rs:3476-3480` | **Token characteristics cloned from raw object, not copiable values.** The token `characteristics` are cloned from `obj.characteristics` (the raw object), then a Layer 1 CopyOf effect is applied. This works correctly because the CopyOf effect overwrites characteristics at layer resolution. However, the initial clone is slightly wasteful — the characteristics will be overwritten by the copy effect. Not a correctness issue since copy::create_copy_effect is applied immediately after. **Fix:** No fix needed; cosmetic only. |

### Finding Details

#### Finding 1: Missing test for Squad keyword loss

**Severity**: LOW
**File**: `crates/engine/tests/squad.rs`
**CR Rule**: 702.157a — "When this creature enters, if its squad cost was paid..."
**Ruling**: 2022-10-07 — "If, for some reason, the creature doesn't have the squad ability when it's on the battlefield, the ability won't trigger, even if you've paid the squad cost one or more times."
**Issue**: The plan identified 7 tests but only 6 were implemented. Test #6 (`test_squad_trigger_requires_keyword_on_battlefield`) validates the ruling that if Squad is removed from the permanent before the trigger fires (e.g., by Humility), no tokens are created. The implementation correctly checks `has_squad` via layer-resolved characteristics at `resolution.rs:1251-1256`, but this path is untested.
**Fix**: Add a test that applies a continuous effect removing all keywords (Humility-style) before the Squad creature resolves, then verifies no SquadTrigger appears on the stack. If setting up the continuous effect is complex, document as a known test gap and defer.

#### Finding 2: Token characteristics clone is redundant

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:3476-3480`
**CR Rule**: 707.2 — "The 'copiable values' of an object are..."
**Issue**: The token's `characteristics` field is cloned from the source object's raw characteristics, then immediately overwritten by a Layer 1 `CopyOf` continuous effect (line 3539-3544). The initial clone is unused after layer resolution applies. This follows the Myriad pattern exactly, so it is consistent with the codebase — but both are slightly wasteful.
**Fix**: No fix required. This is a cosmetic issue consistent with the existing Myriad implementation.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.157a (static: additional cost) | Yes | Yes | `test_squad_basic_one_payment`, `test_squad_multiple_payments`; casting.rs validates keyword + charges cost |
| 702.157a (triggered: ETB token creation) | Yes | Yes | `test_squad_basic_one_payment`, `test_squad_multiple_payments`, `test_squad_tokens_are_copies` |
| 702.157a (intervening-if: squad_count > 0) | Yes | Yes | `test_squad_zero_payments` |
| 702.157a (intervening-if: keyword present) | Yes | No | Implementation at resolution.rs:1251-1256 checks layer-resolved keywords; no test (Finding 1) |
| 702.157b (multiple instances) | No | No | Documented as V1 limitation (single instance only); all printed Squad cards have one instance |
| Ruling: tokens not cast | Yes | Yes | `test_squad_tokens_not_cast` |
| Ruling: spell countered = no trigger | Yes (implicit) | No | Naturally handled — countered spell never enters battlefield; no explicit test needed |
| Ruling: keyword lost = no trigger | Yes | No | Finding 1 |
| CR 707.2 (copiable values) | Yes | Yes | `test_squad_tokens_are_copies`; Layer 1 CopyOf effect applied |
| CR 400.7 (source leaves battlefield) | Yes | No | Break at resolution.rs:3462-3468; no explicit test |
| CR 601.2b/f-h (additional cost payment) | Yes | Yes | casting.rs:1926-1962; tested via mana deduction in cast tests |

## Previous Findings (re-review only)

N/A — first review.
