# Ability Review: Offspring

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.175
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 1274-1285)
- `crates/engine/src/cards/card_definition.rs` (lines 633-641)
- `crates/engine/src/state/stack.rs` (lines 297-302, 1145-1166)
- `crates/engine/src/state/stubs.rs` (lines 124-129)
- `crates/engine/src/state/game_object.rs` (lines 643-648)
- `crates/engine/src/state/mod.rs` (lines 381-383, 544-546)
- `crates/engine/src/state/builder.rs` (lines 996-998)
- `crates/engine/src/state/hash.rs` (lines 667-668, 882-883, 1997-2005, 2112-2113, 4039-4042)
- `crates/engine/src/rules/command.rs` (lines 305-311)
- `crates/engine/src/rules/engine.rs` (lines 107-144)
- `crates/engine/src/rules/casting.rs` (lines 84, 1966-2001, 3282-3284, 5249-5268)
- `crates/engine/src/rules/resolution.rs` (lines 443-445, 1311-1363, 3631-3842)
- `crates/engine/src/rules/abilities.rs` (lines 5737-5751)
- `crates/engine/src/effects/mod.rs` (line 2900)
- `crates/engine/src/testing/replay_harness.rs` (cast_spell_offspring action)
- `tools/replay-viewer/src/view_model.rs` (lines 586-588, 871)
- `tools/tui/src/play/panels/stack_view.rs` (lines 195-197)
- `crates/engine/tests/offspring.rs` (full file, 572 lines)

## Verdict: needs-fix

The implementation is largely correct and well-structured, following the established Squad pattern closely. All enum variants, hash coverage, exhaustive match arms, and basic CR enforcement are properly implemented. The LKI (last-known information) handling for source-leaves-battlefield is a notable improvement over Squad and correctly uses `source_card_id` captured at trigger-queue time. However, there is one MEDIUM finding regarding the P/T override layer (CR 707.9b copiable values) and one LOW finding.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `resolution.rs:3808-3827` | **P/T override uses Layer 7b instead of copiable values.** The "except 1/1" is not part of the token's copiable values per CR 707.9b. |
| 2 | LOW | `tests/offspring.rs` | **Missing test for CDA interaction with 707.9d.** No test for creature with power/toughness CDA (e.g., Tarmogoyf). |

### Finding Details

#### Finding 1: P/T override uses Layer 7b (PtSet) instead of being part of copiable values

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:3808-3827`
**CR Rule**: 707.9b -- "Some copy effects modify a characteristic as part of the copying process. The final set of values for that characteristic becomes part of the copiable values of the copy."
**Issue**: The "except it's 1/1" clause in CR 702.175a is a copy-with-exception per CR 707.9b. The modified P/T (1/1) should become part of the token's copiable values. The current implementation uses a separate Layer 7b `SetPowerToughness` continuous effect, which is NOT visible to `get_copiable_values()` (in `copy.rs`). This means if another effect copies the Offspring token (e.g., Clone targeting the 1/1 token), the Clone would get the *source creature's* original P/T (e.g., 2/3) instead of 1/1, because `get_copiable_values` only resolves Layer 1 (Copy) effects, not Layer 7b.

Additionally, CR 707.9d states that CDAs defining the overridden characteristic (power/toughness) should NOT be copied. The current CopyOf effect copies all keywords and abilities from the source, including any hypothetical P/T CDAs. With the Layer 7b approach, these CDAs would still be copied and might interfere (a CDA at Layer 7a would be applied before the Layer 7b override, but the CDA would still be present in the token's copiable values -- wrong per 707.9d).

**Impact**: Incorrect behavior when a subsequent copy effect targets the Offspring token. In practice, this scenario is uncommon with current card definitions but is a rules correctness gap.

**Fix**: The cleanest fix within the current copy infrastructure is:
1. When the source IS on the battlefield: instead of `CopyOf(source_object)`, set the token's base `characteristics.power = Some(1)` and `characteristics.toughness = Some(1)` directly on `token_obj` BEFORE calling `state.add_object()`. Then register the `CopyOf` effect at Layer 1. When `get_copiable_values` resolves the CopyOf chain, it will get the source's P/T. BUT, then register a SECOND Layer 1 effect (with a later timestamp) that only overrides P/T to 1/1. This requires a new `LayerModification::SetCopiablePT { power: i32, toughness: i32 }` variant applied at Layer 1.

Alternatively (simpler but less architecturally clean): after creating the CopyOf effect, directly modify the token's `characteristics.power` and `characteristics.toughness` to `Some(1)`. Then, do NOT register a Layer 7b PtSet effect. Instead, ensure the CopyOf effect does NOT override the token's base P/T. This requires modifying `apply_layer_modification` for CopyOf to skip P/T if the copier has a "copy-except-pt" flag.

Given the complexity, the pragmatic fix is: add a `// TODO: CR 707.9b -- P/T override should be part of copiable values` comment documenting the gap, and add a test that verifies the current behavior with a comment noting the known deviation. Full fix requires copy infrastructure changes (new LayerModification variant) that should be done as a separate task.

#### Finding 2: Missing test for CDA interaction (CR 707.9d)

**Severity**: LOW
**File**: `crates/engine/tests/offspring.rs`
**CR Rule**: 707.9d -- CDAs defining the overridden characteristic should not be copied.
**Issue**: No test exists for a creature whose P/T is defined by a CDA (e.g., `*/*` with a CDA that counts something). Per CR 707.9d, the CDA should NOT be copied when the copy instruction overrides that characteristic. This is a test gap rather than an implementation gap (the Layer 7b approach happens to mask the CDA in practice since Layer 7b overrides Layer 7a).
**Fix**: Add a test (can be deferred) with a creature that has a P/T CDA, verifying the Offspring token is still 1/1 regardless of the CDA's value.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.175a (additional cost) | Yes | Yes | `test_offspring_basic_paid`, `test_offspring_not_paid` |
| 702.175a (ETB trigger) | Yes | Yes | `test_offspring_basic_paid` |
| 702.175a (intervening-if at trigger time) | Yes | Yes | `test_offspring_not_paid` (not-paid path) |
| 702.175a (intervening-if at resolution) | Yes | Partial | Checked at resolution (keyword still present); no test for keyword-lost-between-trigger-and-resolution |
| 702.175a (token is copy except 1/1) | Yes | Yes | `test_offspring_token_is_1_1` |
| 702.175a (LKI when source leaves) | Yes | Yes | `test_offspring_source_leaves_still_creates_token` |
| 702.175b (multiple instances) | No | No | Deferred per plan; noted in doc comments |
| 707.9b (P/T part of copiable values) | No | No | Finding 1 -- Layer 7b approach is not copiable |
| 707.9d (CDA not copied for overridden char) | No | No | Finding 2 -- test gap |
| 603.4 (intervening-if) | Yes | Yes | Both trigger-queue and resolution re-checks |
| Ruling: tokens not cast | Yes | Yes | `test_offspring_tokens_not_cast` |
| Ruling: no offspring on non-offspring | Yes | Yes | `test_offspring_rejected_without_keyword` |
| CR 400.7 (zone change resets) | Yes | N/A | `offspring_paid: false` in both `move_object_to_zone` sites |

## Additional Notes

**Well done**:
- `source_card_id` captured at trigger-queue time (abilities.rs:5743-5746) is correct LKI design -- better than Squad's approach
- LKI fallback in resolution (lines 3690-3724) correctly builds characteristics from the card registry when source is gone
- Intervening-if re-check at resolution correctly skips the keyword check when source has left (lines 3659-3679)
- Hash coverage is complete across all 4 sites (GameObject, StackObject, StackObjectKind, AbilityDefinition)
- All exhaustive match arms added (view_model.rs, stack_view.rs)
- Replay harness `cast_spell_offspring` action type added
- Tests are well-structured with CR citations and clear assertion messages
- Binary nature (bool, not u32) correctly differentiates from Squad

**Architecture observations**:
- The `offspring_paid: false` field is correctly added to all existing token/copy creation sites in resolution.rs and effects/mod.rs (verified 8+ sites)
- The replay harness correctly defaults `offspring_paid: false` for all non-offspring cast_spell variants
