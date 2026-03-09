# Ability Review: B15 Partner Variants

**Date**: 2026-03-08
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.124 (subrules i, k, m, f)
**Files reviewed**:
- `crates/engine/src/state/types.rs:274-287`
- `crates/engine/src/state/hash.rs:681-686`
- `tools/replay-viewer/src/view_model.rs:892-897`
- `crates/engine/src/rules/commander.rs:498-759` (validate_partner_commanders + helpers)
- `crates/engine/tests/partner_variants.rs` (all 559 lines)

## Verdict: clean

The implementation correctly handles all three partner variants (Friends Forever, Choose a Background, Doctor's Companion) per CR 702.124i/k/m. Cross-variant rejection per CR 702.124f is properly enforced. The Background enchantment exemption from the commander creature-type check works correctly. Hash discriminants are present. All 16 tests pass and cover the key acceptance and rejection cases. Two LOW findings identified: a fragile subtype-count check for Doctor's Companion and a missing test from the plan. Neither affects correctness for any existing or foreseeable card.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `commander.rs:757` | **"No other creature types" check uses total subtype count.** Fragile for hypothetical multi-typed Doctor cards. |
| 2 | LOW | `partner_variants.rs` | **Missing planned test: non-legendary Background rejected.** Plan specified it; not implemented. |

### Finding Details

#### Finding 1: "No other creature types" subtype-count check

**Severity**: LOW
**File**: `crates/engine/src/rules/commander.rs:757`
**CR Rule**: 702.124m -- "a legendary Time Lord Doctor creature card that has no other creature types"
**Issue**: The `is_time_lord_doctor` helper checks `def.types.subtypes.len() == 2` to enforce "no other creature types." This conflates all subtypes (creature, enchantment, artifact, land, planeswalker) into a single count. A hypothetical creature enchantment that is a Time Lord Doctor with an enchantment subtype like "Aura" would be incorrectly rejected. The CR says "no other creature types," not "no other subtypes." However, no such Doctor Who card exists in practice, and the engine lacks a creature-subtype registry to distinguish creature subtypes from other subtype categories. The code comment correctly documents this limitation.
**Fix**: No immediate fix required. When/if a subtype-category registry is added (tracked as a LOW issue elsewhere), update this check to filter to creature subtypes only. Add a TODO comment referencing this.

#### Finding 2: Missing non-legendary Background test

**Severity**: LOW
**File**: `crates/engine/tests/partner_variants.rs`
**CR Rule**: 702.124k -- "a legendary Background enchantment card"
**Issue**: The plan specified `test_choose_a_background_non_legendary_background_rejected` to verify that a non-legendary Background enchantment is rejected as a commander pair. This test was not implemented. The code is correct -- `is_legendary_background` checks `SuperType::Legendary` -- but the test gap means this behavior is not regression-protected.
**Fix**: Add a test that creates a non-legendary Background enchantment (remove `SuperType::Legendary` from the helper) and verifies it is rejected when paired with a ChooseABackground creature.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.124a | Yes (pre-existing) | Yes | Partner infra |
| 702.124b | Yes (pre-existing) | Yes | 100-card deck check |
| 702.124c | Yes (pre-existing) | Yes | Combined color identity |
| 702.124d | Yes (pre-existing) | Yes | Independent tax/damage |
| 702.124f | Yes | Yes | Cross-variant rejection: 4 tests cover FF+Partner, CAB+Partner, FF+CAB, FF+DC |
| 702.124g | N/A | N/A | Multiple partner abilities on one card -- no card defs use this yet |
| 702.124h | Yes (pre-existing) | Yes | Plain Partner |
| 702.124i | Yes | Yes | FriendsForever: 3 tests (valid pair, one-sided, mixed with Partner) |
| 702.124j | Yes (pre-existing) | Yes | PartnerWith |
| 702.124k | Yes | Yes | ChooseABackground: 6 tests (valid pair, missing subtype, no-choose, both-choose, mixed-Partner, type-check exemption). Missing: non-legendary Background test (LOW). |
| 702.124m | Yes | Yes | DoctorsCompanion: 4 tests (valid pair, missing Time Lord, extra types, no Doctor) |
| 702.124n | N/A | N/A | Effect reference distinction -- no effects reference partner by name yet |
