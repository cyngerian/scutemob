# Ability Review: Food Tokens

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 111.10b
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (lines 742-845)
- `crates/engine/src/effects/mod.rs` (lines 1984-2030)
- `crates/engine/src/state/hash.rs` (lines 2049-2063)
- `crates/engine/src/cards/definitions.rs` (lines 696-709, 732-746, 944-959, 2164-2179)
- `crates/engine/src/state/builder.rs` (lines 570-583, 621-640)
- `crates/engine/src/cards/mod.rs` (lines 14-20)
- `crates/engine/src/lib.rs` (lines 1-13)
- `crates/engine/src/rules/abilities.rs` (lines 48-260)
- `crates/engine/src/rules/resolution.rs` (lines 541-572)
- `crates/engine/tests/food_tokens.rs` (all 651 lines)

## Verdict: needs-fix

The core implementation is correct: `TokenSpec.activated_abilities` field, `food_token_spec()` helper, `make_token` propagation, and hash coverage are all properly done. The Food token has the right characteristics (colorless Food artifact, non-mana activated ability with {2}+{T}+sacrifice cost producing 3 life gain). However, there is one MEDIUM finding (test 11 does not actually test the `make_token` propagation path it claims to test) and two LOW findings (incorrect CR citations).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `food_tokens.rs:622` | **Test 11 does not test `make_token` propagation.** Only validates spec struct fields. **Fix:** Rewrite to create a Food token via `Effect::CreateToken` and verify the resulting `GameObject` has the activated ability. |
| 2 | LOW | `food_tokens.rs:296-297` | **Invalid CR citation "602.2c".** CR 602.2c does not exist. **Fix:** Change to "CR 602.2b / CR 601.2h". |
| 3 | LOW | `food_tokens.rs:10` | **Invalid CR citation "602.2c" in module doc.** Same nonexistent rule. **Fix:** Change to "CR 602.2b (via CR 601.2h)". |

### Finding Details

#### Finding 1: Test 11 does not test `make_token` propagation

**Severity**: MEDIUM
**File**: `crates/engine/tests/food_tokens.rs:622-650`
**CR Rule**: 111.10b -- "A Food token is a colorless Food artifact token with '{2}, {T}, Sacrifice this token: You gain 3 life.'"
**Issue**: Test 11 (`test_food_create_via_effect`) is documented as testing "Effect::CreateToken -- Using `food_token_spec` with `Effect::CreateToken` creates a Food token whose activated ability is correctly propagated by `make_token`." However, the test body only calls `food_token_spec(1)` and asserts on the returned `TokenSpec` struct's fields. It never invokes `Effect::CreateToken`, never calls `execute_effect`, and never examines a `GameObject` created by `make_token`. This means the critical `make_token` propagation path (effects/mod.rs lines 2015-2028) that copies `activated_abilities` from `TokenSpec` into `Characteristics` has zero direct test coverage. While test 3 indirectly exercises the activated ability on a token (placed via `ObjectSpec`, bypassing `make_token`), no test verifies that `make_token` correctly populates `activated_abilities` on the resulting `GameObject`.
**Fix**: Rewrite `test_food_create_via_effect` to:
1. Build a `GameState` with at least one player.
2. Call `execute_effect(state, &Effect::CreateToken { spec: food_token_spec(1) }, &mut ctx)`.
3. Find the resulting Food `GameObject` on the battlefield.
4. Assert that `obj.characteristics.activated_abilities.len() == 1`.
5. Assert that the activated ability has the correct cost (`requires_tap`, `generic: 2`, `sacrifice_self`) and effect (`GainLife { Controller, Fixed(3) }`).
This pattern already exists in `crates/engine/tests/effects.rs` at line 930 (Beast token via `CreateToken`).

#### Finding 2: Invalid CR citation "602.2c" in test 5

**Severity**: LOW
**File**: `crates/engine/tests/food_tokens.rs:296-297`
**CR Rule**: CR 602.2b -- "The remainder of the process for activating an ability is identical to the process for casting a spell listed in rules 601.2b-i." CR 601.2h -- "The player pays the total cost."
**Issue**: Test 5 doc comment says "CR 602.2c -- Sacrifice is a cost, not an effect." CR 602.2c does not exist in the Comprehensive Rules. CR 602.2 has only two children: 602.2a and 602.2b. The correct citation for sacrifice-as-cost during ability activation is CR 602.2b (which delegates to CR 601.2h for cost payment). The code in `abilities.rs:231` also cites "CR 602.2c" which is the origin of this propagated error, but that is pre-existing and outside the scope of this review.
**Fix**: Change the doc comment on test 5 from "CR 602.2c" to "CR 602.2b / CR 601.2h" to accurately reference the rules governing cost payment during ability activation.

#### Finding 3: Invalid CR citation "602.2c" in module doc header

**Severity**: LOW
**File**: `crates/engine/tests/food_tokens.rs:10`
**CR Rule**: Same as Finding 2.
**Issue**: Line 10 of the module doc says "Sacrifice is a cost paid before the ability goes on the stack (CR 602.2c)." This references the same nonexistent rule.
**Fix**: Change "(CR 602.2c)" to "(CR 602.2b, CR 601.2h)" in the module-level doc comment.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 111.10b (Food token definition) | Yes | Yes | test_food_token_spec_characteristics, test_food_token_has_activated_ability |
| 111.10b (colorless) | Yes | Yes | test_food_token_spec_characteristics asserts colors empty |
| 111.10b (Food artifact subtype) | Yes | Yes | test_food_token_has_activated_ability checks subtype |
| 111.10b ({2},{T},sacrifice cost) | Yes | Yes | test_food_token_spec_characteristics checks all three cost components |
| 111.10b (gain 3 life effect) | Yes | Yes | test_food_activate_gain_3_life verifies life gain |
| 602.2 (ability uses stack) | Yes | Yes | test_food_uses_stack_not_mana_ability |
| 602.2 (only controller activates) | Yes | Yes | test_food_opponent_cannot_activate |
| 602.2b (cost payment: tap) | Yes | Yes | test_food_already_tapped_cannot_activate |
| 602.2b (cost payment: mana) | Yes | Yes | test_food_insufficient_mana_cannot_activate |
| 602.2b/601.2h (sacrifice as cost) | Yes | Yes | test_food_sacrifice_is_cost_not_effect |
| 602.5a (summoning sickness N/A for artifacts) | Yes | Yes | test_food_not_affected_by_summoning_sickness |
| 605 (NOT a mana ability) | Yes | Yes | test_food_uses_stack_not_mana_ability, mana_abilities.len() == 0 |
| 704.5d (token ceases to exist) | Yes | Yes | test_food_token_ceases_to_exist_after_sba |
| make_token propagation | Yes | **No** | MEDIUM finding -- test 11 does not actually exercise make_token |

## Additional Notes

### Correctness Observations (no finding required)

1. **Token is correctly NOT a creature**: `food_token_spec` sets `card_types: [CardType::Artifact]` with no `CardType::Creature`. This means SBA 704.5f (zero toughness creature dies) does not apply, which is correct for a 0/0 artifact. The `power: 0` and `toughness: 0` values are set but meaningless for non-creatures.

2. **Activated ability resolution path is correct**: The `handle_activate_ability` function (abilities.rs:48-260) properly: (a) checks priority, (b) validates controller, (c) checks sorcery-speed restriction, (d) clones the effect before mutation, (e) pays tap cost, (f) pays mana cost, (g) pays sacrifice cost (moving to graveyard), (h) pushes to stack with embedded effect. Resolution (resolution.rs:541-572) correctly uses the embedded effect (since the source was sacrificed and may no longer exist).

3. **Hash coverage is complete**: `TokenSpec::hash_into` includes `activated_abilities` (hash.rs:2062). `ActivatedAbility::hash_into` includes all 4 fields (cost, description, effect, sorcery_speed). `ActivationCost::hash_into` includes all 3 fields. The blanket `impl<T: HashInto> HashInto for Vec<T>` covers `Vec<ActivatedAbility>`.

4. **All 6 existing TokenSpec literal sites updated**: Beast (definitions.rs:708), Elephant (definitions.rs:745), Bird (definitions.rs:958), Faerie Rogue (definitions.rs:2178), Spirit (builder.rs:582), Phyrexian Germ (builder.rs:639) -- all have `activated_abilities: vec![]`.

5. **Serde compatibility**: The `#[serde(default)]` attribute on `activated_abilities` means existing serialized `TokenSpec` values (in JSON game scripts) that lack this field will deserialize with `vec![]`, avoiding breaking changes.

6. **Multiplayer correctness**: No multiplayer-specific concerns. Each player controls their own Food tokens. The `NotController` check (abilities.rs:78-83) prevents cross-player activation. Tests use 2-player setup which is adequate since Food has no opponent-interaction mechanics.

7. **`sorcery_speed: false`** on the Food ability is correct -- Food can be activated at instant speed (any time the controller has priority), matching how players commonly use Food tokens.
