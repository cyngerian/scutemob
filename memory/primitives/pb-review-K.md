# Primitive Batch Review: PB-K -- Additional Land Drops + Case Mechanic

**Date**: 2026-04-02
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 305.1, 305.2, 305.2a, 305.3, 305.4, 305.6, 305.7, 719.3, 719.3a, 719.3b, 719.3c, 702.169
**Engine files reviewed**: `cards/card_definition.rs`, `state/hash.rs`, `state/game_object.rs`, `state/continuous_effect.rs`, `effects/mod.rs`, `rules/abilities.rs`, `rules/layers.rs`, `testing/replay_harness.rs`
**Card defs reviewed**: 8 (3 new: burgeoning, dryad_of_the_ilysian_grove, case_of_the_locked_hothouse; 5 fixed: growth_spiral, broken_bond, spelunking, contaminant_grafter, chulane_teller_of_tales)

## Verdict: needs-fix

One HIGH finding: Dryad of the Ilysian Grove mana cost is wrong ({2}{G}{G} in def vs {2}{G} oracle). One MEDIUM finding: enrich_spec_from_def drops intervening_if for WheneverOpponentPlaysLand conversions (no current impact but violates the pattern). One LOW finding: Spelunking has 2 remaining TODOs which are correctly deferred.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `replay_harness.rs:2612` | **intervening_if hardcoded to None.** The WheneverOpponentPlaysLand conversion block drops the `intervening_if` field (uses `..` in destructure). Burgeoning has no intervening_if so no current impact, but future cards with this trigger condition and an intervening_if would silently lose it. **Fix:** Extract `intervening_if` from the destructure and forward it as `intervening_if: intervening_if.as_ref().map(|c| InterveningIf { condition: c.clone() })`. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | **HIGH** | `dryad_of_the_ilysian_grove.rs` | **Wrong mana cost.** Oracle says {2}{G}, card def has `green: 2` ({2}{G}{G}). **Fix:** Change line 10 from `green: 2` to `green: 1`. Also fix comment on line 1 from `{2}{G}{G}` to `{2}{G}`. |
| 3 | LOW | `spelunking.rs` | **Two remaining TODOs.** Cave-detection life gain (line 25-28) deferred to PB-A, "Lands you control enter untapped" (line 35-37) deferred to PB-D. Both are correctly documented with deferral targets. No fix needed. |

### Finding Details

#### Finding 1: intervening_if hardcoded to None in WheneverOpponentPlaysLand conversion

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:2612`
**CR Rule**: 603.4 -- intervening-if conditions must be checked at both trigger time and resolution time
**Issue**: The destructuring pattern at line 2604 uses `..` which captures but discards `intervening_if`. The `TriggeredAbilityDef` is constructed with `intervening_if: None` instead of forwarding the original value. Currently safe because Burgeoning has no intervening_if, but this is inconsistent with how other conversions should work and would silently break future cards. Note: this pattern (hardcoded None) is consistent with all other conversion blocks in enrich_spec_from_def, making this a pre-existing convention rather than a PB-K-specific bug. Downgrading from MEDIUM to LOW for that reason.
**Fix:** Extract `intervening_if` in the destructure and forward it: `intervening_if: intervening_if.as_ref().map(|c| InterveningIf { condition: c.clone() })`. Consider fixing all other conversion blocks as a separate LOW remediation task.

#### Finding 2: Dryad of the Ilysian Grove mana cost is {2}{G}, not {2}{G}{G}

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/dryad_of_the_ilysian_grove.rs:10`
**Oracle**: "Dryad of the Ilysian Grove -- {2}{G}" (Scryfall oracle, confirmed via MCP lookup)
**Issue**: The card definition has `ManaCost { generic: 2, green: 2, ..Default::default() }` which encodes {2}{G}{G}. The actual mana cost is {2}{G} (generic 2, green 1). The header comment on line 1 also incorrectly says `{2}{G}{G}`. This causes the card to cost 1 more green mana than it should, affecting both casting legality and mana value calculations. The plan file also contained this error.
**Fix:** Change line 10 to `ManaCost { generic: 2, green: 1, ..Default::default() }`. Change the comment on line 1 from `{2}{G}{G}` to `{2}{G}`.

#### Finding 3: Spelunking has two intentionally deferred TODOs

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/spelunking.rs:25-28,35-37`
**Oracle**: "If you put a Cave onto the battlefield this way, you gain 4 life." + "Lands you control enter untapped."
**Issue**: Two abilities are not implemented: (1) Cave-detection conditional life gain requires effect result tracking (PB-A), (2) lands-enter-untapped requires a global ETB replacement (PB-D). Both are correctly documented with TODO comments citing the blocking primitive batch. The card's other abilities (ETB draw + put land) work correctly.
**Fix:** None needed -- correctly deferred.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 305.1 (land play special action) | Yes | Yes | test_burgeoning_trigger_on_opponent_land_play, test_burgeoning_no_trigger_on_own_land_play |
| 305.2 (additional land plays) | Yes | Yes | test_dryad_additional_land_play |
| 305.4 (put land != play land) | Yes | Yes | test_put_land_does_not_count_as_land_play, test_burgeoning_no_trigger_on_put_land, test_put_land_triggers_landfall |
| 305.7 (basic land types in addition) | Yes | Yes | test_dryad_lands_have_all_basic_types, test_dryad_lands_keep_original_types |
| 719.3a (to solve at end step) | Yes | Yes | test_case_solve_at_end_step, test_case_no_solve_without_condition |
| 719.3b (solved designation persists) | Yes | Yes | test_solve_case_designation_persists_until_ltb |
| 702.169b (solved static ability) | Partial | No | Play-from-top deferred to PB-A (correctly) |
| 400.7 (new object identity on zone change) | Yes | Yes | test_solve_case_designation_persists_until_ltb (zone change clears SOLVED) |
| 603.4 (intervening-if double-check) | Yes | Yes | Case uses And(condition, Not(SourceIsSolved)) as intervening_if; test_case_already_solved_no_retrigger |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| burgeoning | Yes | 0 | Yes | |
| dryad_of_the_ilysian_grove | **No** | 0 | **No** | Mana cost {2}{G}{G} should be {2}{G} (Finding 2) |
| case_of_the_locked_hothouse | Yes | 1 (PB-A play-from-top) | Yes (partial) | Solve mechanic works; solved ability deferred |
| growth_spiral | Yes | 0 | Yes | |
| broken_bond | Yes | 0 | Yes | |
| spelunking | Yes | 2 (PB-A Cave detect, PB-D enter untapped) | Yes (partial) | ETB draw+land works; Cave gain and enter-untapped deferred |
| contaminant_grafter | Yes | 0 | Yes | |
| chulane_teller_of_tales | Yes | 0 | Yes | |

## Test Coverage Summary

17 tests covering all engine primitives. Good positive and negative case coverage. CR citations present in all test doc comments. Test structure follows project conventions (GameStateBuilder, external test file, descriptive names). The `test_put_land_no_op_when_hand_empty` test (no land in hand) is a nice edge case addition beyond the plan.
