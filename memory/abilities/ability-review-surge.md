# Ability Review: Surge

**Date**: 2026-03-05
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.117
**Files reviewed**: `crates/engine/src/state/types.rs`, `crates/engine/src/state/stack.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/state/game_object.rs`, `crates/engine/src/cards/card_definition.rs`, `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/resolution.rs`, `crates/engine/src/rules/copy.rs`, `crates/engine/tests/surge.rs`, `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`

## Verdict: clean

The Surge implementation correctly follows CR 702.117a. The precondition check (`spells_cast_this_turn >= 1` before the surge spell's own counter increment), mutual exclusion with all other alternative costs (CR 118.9a), cost substitution, `cast_alt_cost` propagation through resolution and copy systems, commander tax stacking, and hash coverage are all correct. No HIGH or MEDIUM findings. Two LOW findings related to documentation consistency.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `stack.rs:206` | **Doc comment contradicts copy behavior.** The `was_surged` doc says "Must always be false for copies" but `copy.rs:216` correctly propagates it from the original per ruling 2016-01-22. **Fix:** Update doc to match `was_bargained` pattern: add "Note: Copies of a surged spell are also considered surged (ruling 2016-01-22 / CR 707.2), so this is propagated to copies in the copy system." |
| 2 | LOW | `tests/surge.rs:407-498` | **Mutual exclusion tests are indirect.** `test_surge_mutual_exclusion_with_flashback` tests that a non-surge card rejects `AltCostKind::Surge` (keyword validation), not the actual mutual exclusion path. The structural guarantee (`Option<AltCostKind>` prevents two alt costs simultaneously) means the mutual exclusion code paths in casting.rs are unreachable from the public API. This is fine -- the defense-in-depth guards are correct -- but the test name is misleading. **Fix:** Rename to `test_surge_rejected_on_non_surge_card_flashback_scenario` or add a comment explaining why the mutual exclusion path is structurally unreachable. |

### Finding Details

#### Finding 1: Doc comment contradicts copy behavior

**Severity**: LOW
**File**: `crates/engine/src/state/stack.rs:206`
**CR Rule**: 702.117a / ruling 2016-01-22 -- "If a spell that's a copy of a spell with surge is on the stack, it's considered to have had its surge cost paid."
**Issue**: The doc comment on `was_surged` states "Must always be false for copies (`is_copy: true`) -- copies are not cast." However, `copy.rs:216` correctly propagates `was_surged: original.was_surged` from the original spell, matching the 2016-01-22 ruling. The adjacent `was_bargained` field (lines 196-199) has a similar contradiction but includes a corrective note. `was_surged` lacks this note.
**Fix**: Update the doc comment on `was_surged` (stack.rs:205-206) to add a note matching the `was_bargained` pattern: "Note: Copies of a surged spell are also considered surged (ruling 2016-01-22 / CR 707.2), so this is propagated to copies in the copy system."

#### Finding 2: Mutual exclusion test names are misleading

**Severity**: LOW
**File**: `crates/engine/tests/surge.rs:407-498`
**CR Rule**: 118.9a -- "If two alternative costs or two alternative sets of costs could apply..."
**Issue**: `test_surge_mutual_exclusion_with_flashback` and `test_surge_mutual_exclusion_with_spectacle` don't actually exercise the mutual exclusion code paths in casting.rs. Since `alt_cost` is `Option<AltCostKind>` (a single field), it's structurally impossible to pass both Surge and Flashback simultaneously. The flashback test actually tests keyword validation (non-surge card rejects Surge), and the spectacle test tests keyword validation (surge card rejects Spectacle). Both tests are valid but their names suggest they're testing something they're not.
**Fix**: Either rename the tests to reflect what they actually test (e.g., `test_surge_alt_cost_rejected_on_card_without_surge_keyword`, `test_spectacle_alt_cost_rejected_on_surge_card`), or add a comment explaining the structural guarantee makes explicit mutual exclusion unreachable.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.117a (surge cost precondition) | Yes | Yes | `test_surge_basic_cast_with_surge_cost`, `test_surge_rejected_no_prior_spell` |
| 702.117a ("another spell" = already cast) | Yes | Yes | `test_surge_after_resolved_spell`, `test_surge_after_countered_spell` |
| 702.117a (optional -- normal cost allowed) | Yes | Yes | `test_surge_optional_normal_cost` |
| CR 118.9a (mutual exclusion) | Yes | Indirect | Structural guarantee via `Option<AltCostKind>`; defense-in-depth guards in casting.rs |
| CR 118.9c (mana cost unchanged) | Yes | No | Cost substitution only changes payment, not characteristics; no explicit test but correct by construction |
| CR 118.9d (commander tax stacks) | Yes | Yes | `test_surge_commander_tax_stacks` |
| Ruling: copies inherit surge status | Yes | No | `copy.rs:216` propagates correctly; no dedicated test |
| Ruling: resolved/countered spells count | Yes | Yes | `test_surge_after_resolved_spell`, `test_surge_after_countered_spell` |
| Turn reset clears precondition | Yes | Yes | `test_surge_reset_at_turn_start` |
| `cast_alt_cost` tracked on permanent | Yes | Yes | `test_surge_cast_alt_cost_tracked` |
| Keyword validation | Yes | Yes | `test_surge_card_without_keyword_rejected` |
| Hash coverage (KeywordAbility) | Yes | -- | Discriminant 103 |
| Hash coverage (AbilityDefinition) | Yes | -- | Discriminant 35 |
| Hash coverage (StackObject.was_surged) | Yes | -- | Hashed at line 1675 |
| Hash coverage (GameObject.cast_alt_cost) | Yes | -- | Hashed via `k as u8` at line 690 |
| View model (replay-viewer) | Yes | -- | `KeywordAbility::Surge` mapped at line 764 |
| Copy propagation | Yes | No | `copy.rs:216` propagates `was_surged` correctly |
| Resolution propagation | Yes | Yes | `resolution.rs:299-301` sets `cast_alt_cost = Some(AltCostKind::Surge)` |
