# Ability Review: Fuse

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.102
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KW variant)
- `crates/engine/src/state/hash.rs` (KW, AbilDef, StackObject hashing)
- `crates/engine/src/state/stack.rs` (was_fused field)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Fuse)
- `crates/engine/src/rules/command.rs` (CastSpell fuse field)
- `crates/engine/src/rules/casting.rs` (validation, cost calculation, helpers)
- `crates/engine/src/rules/resolution.rs` (left-then-right execution)
- `crates/engine/src/rules/copy.rs` (was_fused on copies)
- `crates/engine/src/rules/abilities.rs` (was_fused on triggered/activated abilities)
- `crates/engine/src/testing/replay_harness.rs` (cast_spell_fuse action)
- `tools/replay-viewer/src/view_model.rs` (KW::Fuse arm)
- `crates/engine/tests/fuse.rs` (8 tests)

## Verdict: needs-fix

One HIGH finding: copies of fused spells incorrectly set `was_fused: false` instead of propagating the original's fuse state, violating CR 707.2. One MEDIUM finding: target index sharing between halves is undocumented and fragile. Two LOW findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `copy.rs:238` | **Copy of fused spell loses fuse state.** CR 707.2 requires copies to copy casting choices; fuse is dropped. **Fix:** propagate `was_fused: original.was_fused`. |
| 2 | **HIGH** | `stack.rs:276` | **Incorrect doc comment.** States copies must always have `was_fused: false`, contradicting CR 707.2. **Fix:** update comment. |
| 3 | MEDIUM | `resolution.rs:188-212` | **Target sharing between halves is undocumented.** Both halves share one EffectContext; right half must use globally-offset indices. **Fix:** add doc comment explaining the contract. |
| 4 | LOW | `fuse.rs:621` | **Resolution order test doesn't verify ordering.** Asserts both effects occurred but can't distinguish left-before-right from right-before-left. **Fix:** add event-order assertion or use effects where order matters. |

### Finding Details

#### Finding 1: Copy of fused spell loses fuse state

**Severity**: HIGH
**File**: `crates/engine/src/rules/copy.rs:236-238`
**CR Rule**: 707.2 -- "When copying an object, the copy acquires the copiable values of the original object's characteristics and, for an object on the stack, choices made when casting or activating it (mode, targets, the value of X, whether it was kicked, how it will affect multiple targets, and so on)."
**Issue**: The current implementation sets `was_fused: false` on all copies with the comment "Copies are not cast, so was_fused is always false." This is incorrect. CR 707.2 explicitly states that copies acquire "choices made when casting" -- the fuse choice is exactly such a choice. This is analogous to how `was_entwined` is correctly propagated via `was_entwined: original.was_entwined` on line 225 of the same file. A Reverberate targeting a fused Wear // Tear should produce a copy that also resolves both halves.
**Fix**: Change `was_fused: false` to `was_fused: original.was_fused` at `copy.rs:238`. Also update the cascade copy at `copy.rs:447` -- cascade free-casts are cast from exile, so fuse correctly cannot apply there; the `false` is correct for cascade but the comment should clarify this distinction.

#### Finding 2: Incorrect doc comment on StackObject.was_fused

**Severity**: HIGH (misleading invariant documentation that caused Finding 1)
**File**: `crates/engine/src/state/stack.rs:276`
**CR Rule**: 707.2
**Issue**: The doc comment states "Must always be false for copies (`is_copy: true`) -- copies are not cast." This is incorrect per CR 707.2 and directly led to the wrong implementation in copy.rs. Copies do acquire the fuse choice from the original.
**Fix**: Replace line 276 with: `/// Propagated to copies per CR 707.2 (copies copy choices made during casting).`

#### Finding 3: Target sharing between halves is undocumented

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:188-212`
**CR Rule**: 702.102d -- "the controller of the spell follows the instructions of the left half and then follows the instructions of the right half."
**Issue**: Both halves share a single `EffectContext` with a single `legal_targets` list. The right half's `DeclaredTarget { index: N }` must use globally-offset indices (e.g., if left half has 1 target, right half's first target is at index 1). This contract is not documented anywhere -- not in the resolution code, not in AbilityDefinition::Fuse's doc comment, and not in the plan's card definition guidance. A card definition author could easily use `index: 0` for both halves' first targets, causing the right half to re-target the left half's target.
**Fix**: Add a doc comment to `AbilityDefinition::Fuse` explaining the target index contract: "When fused, the combined target list is left-half targets (indices 0..N) followed by right-half targets (indices N..M). The right half's `DeclaredTarget` indices must account for this offset." Also add a comment in resolution.rs at line ~194.

#### Finding 4: Resolution order test doesn't verify ordering

**Severity**: LOW
**File**: `crates/engine/tests/fuse.rs:621`
**CR Rule**: 702.102d
**Issue**: `test_fuse_resolution_order_left_then_right` asserts that P2 lost 3 life and P1 gained 5 life, but these effects target different players and are independent -- executing them in either order produces the same final state. The test name claims to verify order but cannot actually distinguish left-then-right from right-then-left execution.
**Fix**: To truly verify order, use effects where order matters (e.g., left half creates a token, right half destroys all creatures -- if right executes first, the token survives). Alternatively, inspect the event list ordering. Current test is still valuable as a positive test that both halves execute; just the name is slightly misleading.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.102a (static ability, hand only) | Yes | Yes | test_fuse_from_hand_only_rejected_from_graveyard |
| 702.102b (combined characteristics) | Partial | No | Instant-speed timing checked; MV/color not tested |
| 702.102c (combined mana cost) | Yes | Yes | test_fuse_combined_mana_cost_required |
| 702.102d (left then right resolution) | Yes | Partial | Both execute; order not truly verified (Finding 4) |
| 709.3 (single half without fuse) | Yes | Yes | test_fuse_single_half_cast |
| 709.4d (fused spell combined chars on stack) | Partial | No | Timing check uses it; no test for MV queries |
| 707.2 (copy of fused spell) | **No** | No | Finding 1 -- was_fused not propagated to copies |
| Alt cost rejection | Yes | Yes | test_fuse_alt_cost_rejected |
| No-keyword rejection | Yes | Yes | test_fuse_no_keyword_rejected |
