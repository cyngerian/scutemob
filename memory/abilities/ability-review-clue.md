# Ability Review: Clue Tokens

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 111.10f
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs:847-882` (clue_token_spec function)
- `crates/engine/src/cards/mod.rs:15-20` (pub use export)
- `crates/engine/src/lib.rs:8-14` (pub use export)
- `crates/engine/tests/clue_tokens.rs` (721 lines, 11 tests)
- `crates/engine/src/effects/mod.rs:2019-2090` (make_token -- unchanged, verified correctness)
- `crates/engine/src/rules/abilities.rs:184` (requires_tap check -- unchanged, verified correctness)
- `crates/engine/src/state/hash.rs:2062-2077` (TokenSpec HashInto -- unchanged, verified completeness)
- `crates/engine/src/cards/definitions.rs:28-32` (import line -- clue_token_spec NOT imported, verified intentional)

## Verdict: clean

The Clue token implementation is correct, complete, and well-tested. The `clue_token_spec()` function exactly matches the CR 111.10f rule text: a colorless Clue artifact token with "{2}, Sacrifice this token: Draw a card." The critical difference from Food tokens -- no tap cost (`requires_tap: false`) -- is correctly implemented and explicitly tested (test 6). No new enum variants or hash fields were introduced; the implementation reuses existing `TokenSpec`, `ActivatedAbility`, and `Effect::CreateToken` infrastructure. All 11 tests cite CR rules, cover positive cases, negative cases (opponent activation, insufficient mana), and the key edge case (tapped Clue can still activate). No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `definitions.rs:28-32` | **clue_token_spec not imported in definitions.rs.** Plan step 2 said to add it; not yet needed since no card def uses it (step 5 is future). Will be needed when Thraben Inspector is added. **Fix:** Add `clue_token_spec` to the import line when implementing the card definition in step 5. |
| 2 | LOW | `clue_tokens.rs:39` | **Dead code: `find_by_name_in_zone` is unused.** Copied from food_tokens.rs pattern but not called. Has `#[allow(dead_code)]`. **Fix:** Remove the function or use it in a test (e.g., test 8 could use it to check graveyard zone before SBA). |
| 3 | LOW | `clue_tokens.rs:193` | **Comment about empty library draw behavior.** The doc comment says "DrawCards on empty library is silently a no-op per gotchas-infra.md" -- this is not a correctness issue for the test (the test adds a library card), but the claim that draw-from-empty is "silently a no-op" may not align with CR 104.3c (attempting to draw from an empty library causes a player to lose). If the engine silently no-ops instead of recording the failed draw, that would be a separate engine-level issue, not a Clue-specific bug. **Fix:** No fix needed for Clue implementation. Note for future: verify that `DrawCards` on an empty library correctly triggers the loss-condition SBA path (CR 104.3c / CR 121.3). |

### Finding Details

#### Finding 1: clue_token_spec not imported in definitions.rs

**Severity**: LOW
**File**: `crates/engine/src/cards/definitions.rs:28-32`
**CR Rule**: n/a -- organizational, not correctness
**Issue**: The plan (step 2) listed adding `clue_token_spec` to the definitions.rs import alongside `food_token_spec` and `treasure_token_spec`. This was not done. However, since step 5 (card definition for Thraben Inspector) has not been implemented yet, no code in definitions.rs references `clue_token_spec`, so there is no compilation error. The import will be needed when the card definition is added.
**Fix**: Add `clue_token_spec` to the `use super::card_definition::{...}` import at line 28-31 of definitions.rs when implementing step 5 (Thraben Inspector card definition).

#### Finding 2: Dead code find_by_name_in_zone

**Severity**: LOW
**File**: `crates/engine/tests/clue_tokens.rs:39-46`
**CR Rule**: n/a -- code quality
**Issue**: The `find_by_name_in_zone` helper function is defined with `#[allow(dead_code)]` but never used in any test. It was copied from the food_tokens.rs pattern. While harmless, dead code in tests adds noise.
**Fix**: Either remove the function, or use it in test 8 (`test_clue_token_ceases_to_exist_after_sba`) to verify the Clue is in the graveyard before SBA, replacing the inline `.any()` check at line 525-528.

#### Finding 3: Empty library draw comment accuracy

**Severity**: LOW
**File**: `crates/engine/tests/clue_tokens.rs:193`
**CR Rule**: 104.3c -- "If a player is required to draw more cards than are left in their library, they draw the remaining cards and then lose the game the next time a player would receive priority."
**Issue**: The test doc comment states "DrawCards on empty library is silently a no-op per gotchas-infra.md." This characterization may be misleading. Per CR 104.3c and CR 121.3, a player who attempts to draw from an empty library should lose the game (via SBA check, CR 704.5b). The test correctly avoids this by placing a card in the library, so the test itself is correct. But the comment could mislead future developers into thinking empty-library draws are harmless.
**Fix**: No action needed for Clue. The comment is informational and the test is correct. If the engine's DrawCards behavior on empty libraries is verified elsewhere, no concern.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 111.10 (parent) | Yes | Yes | clue_token_spec returns predefined token |
| 111.10f (Clue definition) | Yes | Yes | test_clue_token_spec_characteristics, test_clue_token_has_activated_ability |
| 111.10f -- colorless | Yes | Yes | spec.colors.is_empty() asserted in test 1 |
| 111.10f -- Clue artifact type | Yes | Yes | CardType::Artifact asserted in tests 1 and 2 |
| 111.10f -- Clue subtype | Yes | Yes | SubType("Clue") asserted in tests 1 and 2 |
| 111.10f -- {2} mana cost | Yes | Yes | test 1 asserts generic==2; test 10 negative case |
| 111.10f -- Sacrifice self cost | Yes | Yes | tests 1, 2, 5 assert sacrifice_self |
| 111.10f -- Draw a card effect | Yes | Yes | test 3 verifies draw; test 11 verifies effect type |
| 111.10f -- NO tap cost | Yes | Yes | tests 1, 2, 6 assert requires_tap==false; test 6 specifically activates a tapped Clue |
| 602.2 -- controller only | Yes | Yes | test 9 (opponent cannot activate) |
| 602.2 -- uses the stack | Yes | Yes | test 4 (stack not empty after activation) |
| 602.2b -- sacrifice is cost | Yes | Yes | test 5 (Clue gone before resolution) |
| 605 -- not a mana ability | Yes | Yes | test 4 (uses stack, therefore not mana ability) |
| 704.5d -- token ceases to exist | Yes | Yes | test 8 (TokenCeasedToExist event after SBA) |
| 302.6 -- summoning sickness | Yes | Yes | test 7 (artifact, no creature type, no tap cost) |
| CreateToken path | Yes | Yes | test 11 (make_token propagates activated_abilities) |

## Previous Findings (re-review only)

N/A -- this is the initial review.
