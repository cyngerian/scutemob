# Ability Review: Eternalize

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.129
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 802-809)
- `crates/engine/src/cards/card_definition.rs` (line 288-296)
- `crates/engine/src/state/stack.rs` (line 633-648)
- `crates/engine/src/rules/command.rs` (line 444-454)
- `crates/engine/src/state/hash.rs` (disc 93/28/26)
- `crates/engine/src/rules/abilities.rs` (line 1249-1443)
- `crates/engine/src/rules/resolution.rs` (line 2408-2585)
- `crates/engine/src/rules/engine.rs` (line 364-377)
- `crates/engine/src/testing/replay_harness.rs` (eternalize_card action)
- `tools/replay-viewer/src/view_model.rs` (EternalizeAbility match arm + keyword)
- `tools/tui/src/play/panels/stack_view.rs` (EternalizeAbility match arm)
- `crates/engine/tests/eternalize.rs` (12 tests)

## Verdict: needs-fix

The Eternalize implementation is structurally correct and closely mirrors the Embalm
implementation (which was reviewed and fixed in its own cycle). The activation handler
properly exiles the card as cost, the resolution creates a token with Black color, 4/4
P/T override, no mana cost, and Zombie subtype added. The sorcery-speed restriction,
split-second blocking, and priority management are all correct. However, there is one
MEDIUM finding related to the pre-existing systemic gap around non-keyword abilities
not being populated on runtime-created tokens (inherited from Embalm and already tracked
as a known issue). The Embalm reviewer's Finding 1 (missing supertypes) was already
fixed before Eternalize was implemented -- the Eternalize resolution correctly copies
`def.types.supertypes`. There are also several LOW findings around test naming, missing
multiplayer test, and the `mana_value() > 0` guard pattern.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `resolution.rs:2476-2485` | **Triggered/activated abilities not populated on token.** Pre-existing systemic gap (same as Embalm Finding 2). **Fix:** document with TODO; defer to infrastructure fix. |
| 2 | LOW | `tests/eternalize.rs:629` | **Test 7 function name misleading.** Named `test_eternalize_token_has_etb_abilities` but tests keyword (Haste) retention, not ETB triggers. **Fix:** rename to `test_eternalize_token_retains_printed_keywords`. |
| 3 | LOW | `tests/eternalize.rs:657` | **Test 7 adds Blue mana for generic cost (comment says Red).** Comment says "Give p1 {4}{R}{R} mana" but adds 4 Blue + 2 Red. Functionally correct (blue pays generic) but the comment and code are inconsistent. **Fix:** change `ManaColor::Blue, 4` to `ManaColor::Colorless, 4` or update the comment. |
| 4 | LOW | `tests/eternalize.rs` | **No multiplayer test (4-player).** Embalm has `test_embalm_multiplayer_only_active_player` with 4 players. Eternalize only tests 2-player scenarios. **Fix:** add a 4-player test verifying that non-active players cannot activate eternalize. |
| 5 | LOW | `tests/eternalize.rs` | **No test for eternalize with a card that already has Zombie subtype.** CR 702.129a says "Zombie in addition to its other types" -- if the card is already a Zombie, the token should still be a Zombie (not duplicate). **Fix:** add a test with a Zombie creature to verify no subtype duplication. |
| 6 | LOW | `resolution.rs:2479-2483` | **TODO comment already documents the abilities gap.** The code has `TODO(eternalize-review-finding-1)` referencing Embalm's gap. This is good self-documentation. No additional fix needed. |
| 7 | LOW | `abilities.rs:1346` | **`mana_value() > 0` guard skips payment for zero-cost eternalize.** If a card had eternalize cost `{0}`, the entire mana payment block (including the `can_pay_cost` check) would be skipped. No current cards have zero-cost eternalize, but the pattern is fragile. **Fix:** defer; same pattern used by all activated ability handlers in abilities.rs. Fix holistically when addressing the pattern. |

### Finding Details

#### Finding 1: Triggered/activated abilities not populated on token

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:2476-2485`
**CR Rule**: 707.2 -- "The copiable values of an object are its name, mana cost, color indicator, card type, subtype, supertype, rules text, abilities, power, and toughness."
**Issue**: The resolution code sets `abilities: im::Vector::new()`, `activated_abilities: Vec::new()`, and `triggered_abilities: Vec::new()`. While keyword abilities are correctly populated from the CardDefinition's `AbilityDefinition::Keyword` variants, non-keyword abilities (triggered, activated) are empty. This means an eternalized token of a card with "Whenever this creature attacks, draw a card" would not have that triggered ability fire.

This is the same pre-existing systemic gap documented in Embalm's review (Finding 2, ability-review-embalm.md). The builder converts `AbilityDefinition` entries into `TriggeredAbilityDef`/`ActivatedAbility` structs at state-build time, but that conversion is not available at resolution time for runtime-created tokens. The code already has a `TODO(eternalize-review-finding-1)` comment documenting this.

For Proven Combatant (vanilla creature) and Haste Warrior (keyword-only abilities), the impact is zero. It would matter for cards like Earthshaker Khenra (has an ETB trigger).

**Fix**: This is tracked as a broader infrastructure issue. The proper fix is to extract the builder's ability conversion logic into a shared function. For this review cycle, the existing TODO comment is sufficient. Mark as deferred.

#### Finding 2: Test 7 function name misleading

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/eternalize.rs:629`
**CR Rule**: N/A (test quality)
**Issue**: The function is named `test_eternalize_token_has_etb_abilities` but the test body verifies keyword retention (Haste), not ETB trigger firing. The plan (Step 7 test list, item 7) originally intended this test to verify ETB triggers fire on the eternalized token. The implementation pivoted to testing keyword retention instead (which is valid), but the name was not updated.
**Fix**: Rename the function to `test_eternalize_token_retains_printed_keywords` to match what it actually tests.

#### Finding 3: Mana comment/code inconsistency in test 7

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/eternalize.rs:651-663`
**CR Rule**: N/A (test clarity)
**Issue**: The comment at line 651 says "Give p1 {4}{R}{R} mana" but the code adds `ManaColor::Blue, 4` (4 blue mana) and `ManaColor::Red, 2` (2 red mana). The eternalize cost for Haste Warrior is `{4}{R}{R}`, so the 4 blue mana would pay the {4} generic portion. This is functionally correct, but a reader would expect the mana pool to match the comment.
**Fix**: Either change line 657 to `ManaColor::Colorless, 4` to more naturally represent "4 generic mana", or update the comment to "Give p1 4 blue + 2 red mana ({4}{R}{R} eternalize cost)".

#### Finding 4: No multiplayer test

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/eternalize.rs`
**CR Rule**: Architecture invariant 5 ("Multiplayer-first")
**Issue**: All 12 eternalize tests use only 2 players. The Embalm test suite includes `test_embalm_multiplayer_only_active_player` with 4 players, verifying the sorcery-speed restriction in a multiplayer context. Eternalize does not have an equivalent test.
**Fix**: Add a 4-player test similar to Embalm's, verifying that players 2/3/4 cannot activate eternalize when player 1 is the active player, even if the card is in their graveyard with sufficient mana.

#### Finding 5: No duplicate Zombie subtype test

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/eternalize.rs`
**CR Rule**: 702.129a -- "Zombie in addition to its other types"
**Issue**: Test 9 uses Proven Combatant (Human Warrior), which does not already have the Zombie subtype. There is no test verifying behavior when the original card is already a Zombie (e.g., a "Zombie Warrior"). The implementation correctly uses `subtypes.insert(SubType("Zombie"...))` which is idempotent for OrdSet, so no duplication would occur. But without a test, a regression that uses `push` instead of `insert` (on a different data structure) would go undetected.
**Fix**: Add a test with a Zombie creature card definition. Verify the token has exactly one Zombie subtype entry, not two.

#### Finding 7: `mana_value() > 0` guard pattern

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:1346`
**CR Rule**: 602.2b -- "Pay all costs"
**Issue**: The guard `if eternalize_cost.mana_value() > 0` means the entire mana payment block is skipped when the cost has zero mana value. This is the same pattern used by all activated ability handlers (Unearth, Cycling, Ninjutsu, Embalm). While no current eternalize card has a zero mana cost, the pattern is fragile because `can_pay_cost()` would accept a zero-cost payment even with an empty pool, making the guard technically unnecessary.
**Fix**: Defer. This is a systemic pattern across all handlers. Address holistically when doing a LOW remediation pass on abilities.rs.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.129a (activated from graveyard) | Yes | Yes | test_eternalize_basic_flow |
| 702.129a (exile as cost) | Yes | Yes | test_eternalize_card_exiled_as_cost |
| 702.129a (token is black) | Yes | Yes | test_eternalize_token_is_black_4_4 (Proven Combatant is blue, so color replacement verified) |
| 702.129a (token is 4/4) | Yes | Yes | test_eternalize_token_is_black_4_4 (original 1/1 overridden to 4/4) |
| 702.129a (no mana cost) | Yes | Yes | test_eternalize_no_mana_cost |
| 702.129a (Zombie added) | Yes | Yes | test_eternalize_token_zombie_subtype |
| 702.129a (sorcery speed) | Yes | Yes | test_eternalize_sorcery_speed (opponent turn + combat step), test_eternalize_split_second_blocks (non-empty stack) |
| 702.129a (supertypes copied) | Yes | No | Code correctly uses `def.types.supertypes.clone()` at resolution.rs:2472, but no test verifies a Legendary creature's supertype is preserved |
| 707.2 (copiable values: keywords) | Yes | Yes | test_eternalize_keyword_retained, test_eternalize_token_has_etb_abilities (keywords) |
| 707.2 (copiable values: abilities) | Partial | No | Keywords copied; triggered/activated abilities empty (Finding 1, pre-existing gap) |
| 707.9b (color override copiable) | Yes | Yes | test_eternalize_token_is_black_4_4 (blue card becomes black) |
| 707.9b (P/T override copiable) | Yes | Yes | test_eternalize_token_is_black_4_4 (1/1 becomes 4/4) |
| 707.9d (no mana cost, CDA) | Yes | Yes | test_eternalize_no_mana_cost |
| CR 302.6 (summoning sickness) | Yes | Yes | test_eternalize_keyword_retained (asserts has_summoning_sickness) |
| CR 602.2b (mana payment) | Yes | Yes | test_eternalize_insufficient_mana |
| CR 702.61a (split second) | Yes | Yes | test_eternalize_split_second_blocks |
| Not-a-cast (ruling) | Yes | Yes | test_eternalize_not_a_cast |
| Zone check (not in graveyard) | Yes | Yes | test_eternalize_not_in_graveyard |
| Multiplayer | Yes | No | Sorcery-speed check enforces active-player-only, but no 4-player test (Finding 4) |

## Additional Notes

### Positive Observations

1. **Supertypes fixed before Eternalize**: Unlike the Embalm implementation which initially
   set supertypes to empty (Embalm Finding 1), the Eternalize resolution correctly uses
   `supertypes: def.types.supertypes.clone()` at line 2472. This suggests the Embalm fix
   was applied before Eternalize was implemented, and the implementation learned from it.

2. **CardId vs ObjectId**: Like Embalm, the implementation correctly uses
   `source_card_id: Option<CardId>` in `StackObjectKind::EternalizeAbility`, recognizing
   that the original ObjectId is dead after the zone change (CR 400.7).

3. **source_name for display**: Eternalize adds a `source_name: String` field to its stack
   kind, which Embalm lacks. This allows the TUI to display "Eternalize: Proven Combatant"
   instead of just "Eternalize: ". The hash correctly includes this field.

4. **Hash discriminants**: All three new discriminants (93/28/26) are unique within their
   respective enum types and properly follow the existing sequences.

5. **Counter_stack_object**: The EternalizeAbility variant is correctly included in the
   non-spell counter arm at resolution.rs:2704 with a comment noting the card is already
   in exile. Countering the ability does not return the exiled card.

6. **Full ETB pipeline**: The resolution runs the complete ETB pipeline
   (apply_self_etb_from_definition, apply_etb_replacements, register_permanent_replacement_abilities,
   register_static_continuous_effects, TokenCreated event, PermanentEnteredBattlefield event,
   fire_when_enters_triggered_effects). This matches the Embalm resolution pipeline exactly.

7. **Test coverage quality**: 12 tests with good CR citations covering basic flow, token
   characteristics (color, P/T, mana cost, subtypes, keywords), cost exile timing, sorcery
   speed, zone check, insufficient mana, split second, and not-a-cast verification.

8. **Replay harness**: The `eternalize_card` action correctly uses `find_in_graveyard` since
   the card must be in the graveyard at activation time.

### Comparison with Embalm

The two implementations are nearly identical structurally. Key differences:
- **Color**: Embalm = White, Eternalize = Black (correct per CR)
- **P/T**: Embalm = `def.power` / `def.toughness` (original), Eternalize = `Some(4)` / `Some(4)` (correct per CR)
- **source_name**: Eternalize has it, Embalm does not (cosmetic improvement for TUI)
- **Supertypes**: Embalm had a bug (empty); Eternalize correctly copies from definition

### No "Eternalized" Status Subrule

Unlike Embalm (CR 702.128b: "A token is 'embalmed' if it's created by a resolving embalm
ability"), there is no CR 702.129b for "eternalized" status. The CR stops at 702.129a.
This means there is no need for an `is_eternalized` flag on the token.
