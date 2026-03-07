# Ability Review: Outlast

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.107
**Files reviewed**: `crates/engine/src/state/types.rs:1113-1117`, `crates/engine/src/cards/card_definition.rs:526-531`, `crates/engine/src/state/hash.rs:612-613,3628-3632`, `crates/engine/src/testing/replay_harness.rs:1711-1732`, `crates/engine/tests/outlast.rs` (full)

## Verdict: clean

The Outlast implementation is correct and complete. CR 702.107a has only one subrule, and
the implementation faithfully translates it: "[Cost], {T}: Put a +1/+1 counter on this
creature. Activate only as a sorcery." The expansion in `enrich_spec_from_def` correctly
sets `requires_tap: true`, `sorcery_speed: true`, and `Effect::AddCounter` targeting
`Source` with `CounterType::PlusOnePlusOne` and `count: 1`. All enforcement (sorcery speed,
summoning sickness, tap cost, mana payment) is delegated to the existing
`handle_activate_ability` infrastructure in `abilities.rs`, which already handles these
checks correctly. Hash coverage is complete for both `KeywordAbility::Outlast` (disc 121)
and `AbilityDefinition::Outlast` (disc 48). Tests are thorough with 7 cases covering
positive, negative, and edge scenarios. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/outlast.rs:85` | **Plan vs implementation mismatch in EffectAmount type.** The plan said `count: EffectAmount::Fixed(1)` but the actual `Effect::AddCounter` uses `count: u32`. The implementation correctly uses `count: 1`. The plan was wrong, the code is right. No action needed. |
| 2 | LOW | `tests/outlast.rs:74-89` | **Tests manually construct the ActivatedAbility instead of using enrich_spec_from_def.** The `outlast_ability()` helper builds the ability by hand rather than going through the enrichment path. This means the enrichment code in `replay_harness.rs:1711-1732` is only indirectly validated (game scripts will exercise it). Not a correctness issue since both paths produce identical structs, but a future enrichment regression would not be caught by unit tests. **Fix:** Consider adding one test that uses `enrich_spec_from_def` with a CardDefinition containing `AbilityDefinition::Outlast` and verifies the resulting ObjectSpec has the correct ActivatedAbility. |

### Finding Details

#### Finding 1: Plan vs implementation mismatch in EffectAmount type

**Severity**: LOW
**File**: `crates/engine/tests/outlast.rs:85`
**CR Rule**: 702.107a -- "Put a +1/+1 counter on this creature"
**Issue**: The ability plan specified `count: EffectAmount::Fixed(1)` but `Effect::AddCounter` uses `count: u32`, not `EffectAmount`. The implementation correctly uses `count: 1`. This is a plan documentation error, not a code error.
**Fix**: No code change needed. Plan was inaccurate; implementation is correct.

#### Finding 2: Tests bypass enrich_spec_from_def

**Severity**: LOW
**File**: `crates/engine/tests/outlast.rs:74-89`
**CR Rule**: 702.107a
**Issue**: All 7 tests use the `outlast_ability()` helper to manually construct the `ActivatedAbility` and attach it via `.with_activated_ability()`. None of them exercise the `enrich_spec_from_def` expansion path that card definitions will use. If the enrichment code had a bug (e.g., wrong counter type, missing sorcery_speed), the unit tests would still pass but real game play would be broken.
**Fix**: Optionally add a test that creates a CardDefinition with `AbilityDefinition::Outlast { cost }`, calls `enrich_spec_from_def`, and asserts the resulting spec has an `ActivatedAbility` with the correct fields. Low priority since game scripts will cover this path.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.107a (activated ability) | Yes | Yes | test_outlast_basic_adds_counter |
| 702.107a (cost includes {T}) | Yes | Yes | test_outlast_already_tapped, test_outlast_summoning_sickness |
| 702.107a (mana cost) | Yes | Yes | test_outlast_requires_mana |
| 702.107a (+1/+1 counter on self) | Yes | Yes | test_outlast_basic_adds_counter, test_outlast_stacks_counters |
| 702.107a (sorcery speed) | Yes | Yes | test_outlast_sorcery_speed_restriction |
| 702.107a (not a spell cast) | Yes | Yes | test_outlast_not_a_cast |
| Ruling: summoning sickness | Yes | Yes | test_outlast_summoning_sickness_prevents_activation |
| Ruling: counters from any source | N/A | N/A | Relates to companion static abilities, not Outlast itself |

## Previous Findings (re-review only)

N/A -- first review.
