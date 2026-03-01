# Ability Review: Embalm

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.128
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 794-801)
- `crates/engine/src/cards/card_definition.rs` (line 279-287)
- `crates/engine/src/state/stack.rs` (line 620-632)
- `crates/engine/src/rules/command.rs` (line 432-442)
- `crates/engine/src/state/hash.rs` (disc 92/27/25)
- `crates/engine/src/rules/abilities.rs` (line 1055-1243)
- `crates/engine/src/rules/resolution.rs` (line 2230-2394)
- `crates/engine/src/rules/engine.rs` (line 349-362)
- `crates/engine/src/testing/replay_harness.rs` (embalm_card action)
- `tools/replay-viewer/src/view_model.rs` (EmbalmAbility match arm)
- `tools/tui/src/play/panels/stack_view.rs` (EmbalmAbility match arm)
- `crates/engine/tests/embalm.rs` (12 tests)

## Verdict: needs-fix

The Embalm implementation is fundamentally correct: the activation handler properly exiles
the card as cost (not at resolution), the token creation uses CardId (not ObjectId) for
registry lookup, and all sorcery-speed validations are in place. However, the resolution
code fails to copy supertypes from the card definition (sets them to empty), which means
a Legendary creature's embalm token would lose the Legendary supertype. This is a CR
violation that produces incorrect game state. One additional finding at MEDIUM level
relates to `702.128b` ("embalmed" status) not being tracked.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `resolution.rs:2286` | **Supertypes not copied from card definition.** Token loses supertypes (e.g., Legendary). **Fix:** copy from `def.types.supertypes`. |
| 2 | **MEDIUM** | `resolution.rs:2290-2294` | **Triggered/activated abilities not populated on token characteristics.** Pre-existing systemic gap (builder conversion not run for runtime tokens), but Embalm makes it visible. **Fix:** note as pre-existing; for Embalm specifically, document the gap. |
| 3 | LOW | `resolution.rs:2242-2394` | **CR 702.128b: "embalmed" status not tracked.** No `is_embalmed` flag on the token. Currently no cards query this status, so no functional impact. **Fix:** add an `is_embalmed: bool` field to `GameObject` when needed (defer). |
| 4 | LOW | `tests/embalm.rs` | **No test for multi-color card embalm.** Sacred Cat is already white, so the color override test does not verify that non-white colors are removed. **Fix:** add a test with a green or multi-color creature definition. |
| 5 | LOW | `tests/embalm.rs` | **No test for Legendary creature embalm (supertypes).** Relates to Finding 1 -- there is no test that would catch the supertypes bug. **Fix:** add a test with a Legendary creature to verify supertypes are copied. |
| 6 | LOW | `resolution.rs:2285` | **`color_indicator` set to None unconditionally.** If the original card had a color indicator (e.g., a DFC back face), the token should not copy it since color is overridden to White (CR 707.9d). Current behavior (None) is correct. No action needed. |

### Finding Details

#### Finding 1: Supertypes not copied from card definition

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:2286`
**CR Rule**: 702.128a -- "Create a token that's a copy of this card, except it's white, it has no mana cost, and it's a Zombie in addition to its other types."
**Related**: CR 707.2 -- copiable values include supertypes.
**Issue**: The resolution code sets `supertypes: im::OrdSet::new()` (empty). Per CR 702.128a,
the token is a copy of the card with specific exceptions (color, mana cost, Zombie subtype
added). Supertypes are NOT listed among the exceptions, so they must be copied from the
card definition. A Legendary creature embalmed should produce a Legendary token (e.g.,
Temmet, Vizier of Naktamun has Embalm and is Legendary -- its token should also be
Legendary). Without supertypes, the Legend Rule (CR 704.5j) would not apply to the
token, producing an incorrect game state.
**Fix**: Change line 2286 from:
```rust
supertypes: im::OrdSet::new(),
```
to:
```rust
supertypes: def.types.supertypes.clone(),
```

#### Finding 2: Triggered/activated abilities not populated on token characteristics

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:2290-2294`
**CR Rule**: 707.2 -- "The copiable values of an object are its name, mana cost, color indicator, card type, subtype, supertype, rules text, abilities, power, and toughness."
**Issue**: The resolution code sets `abilities: im::Vector::new()`, `activated_abilities: Vec::new()`, and `triggered_abilities: Vec::new()`. While keyword abilities are correctly populated from the CardDefinition's `AbilityDefinition::Keyword` variants, non-keyword abilities (triggered, activated) are left empty. The token's `card_id` link means the ETB pipeline handles ETB-related effects via the registry, but post-ETB triggered abilities (e.g., "Whenever this creature attacks, ...") and activated abilities on the characteristics struct would be missing. The engine's trigger checking code (`check_triggers` in abilities.rs) iterates `obj.characteristics.triggered_abilities`, so any ongoing triggers from the card definition would not fire on the embalm token.

This is a **pre-existing systemic gap**: the builder converts `AbilityDefinition` entries into `TriggeredAbilityDef` and `ActivatedAbility` entries on `Characteristics` at state-build time, but this conversion is not available at runtime for tokens created during resolution. The Myriad token avoids this because it copies `characteristics.clone()` from the in-play source object (which already went through the builder).

**Fix**: This is a broader infrastructure issue that affects all runtime-created tokens (Embalm, and potentially future Eternalize/Populate/etc.). The proper fix is to extract the builder's keyword-to-ability conversion logic into a shared function that can be called at both build time and token-creation time. For this review cycle, document the gap in the code with a TODO comment referencing this finding. For Sacred Cat (the only current embalm test card), the impact is zero since Lifelink is a static keyword (not a triggered/activated ability). Severity is MEDIUM because it would matter for cards like Angel of Sanctions (which has an ETB exile ability as an activated ability pattern).

#### Finding 3: CR 702.128b "embalmed" status not tracked

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:2242-2394`
**CR Rule**: 702.128b -- "A token is 'embalmed' if it's created by a resolving embalm ability."
**Issue**: The token created by embalm resolution has no `is_embalmed` flag. CR 702.128b
defines this status for tokens. Currently, no cards in the engine query the "embalmed"
status, so there is no functional impact. However, future cards might reference "embalmed
tokens" (e.g., Anointed Procession has interaction patterns).
**Fix**: Defer. Add `is_embalmed: bool` to `GameObject` when a card or rule needs to query it. Document with a TODO in the resolution code.

#### Finding 4: No test for multi-color card embalm

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/embalm.rs`
**CR Rule**: 702.128a -- "except it's white" (replaces ALL colors)
**Issue**: Test 2 (`test_embalm_token_is_white`) uses Sacred Cat, which is already a
white creature. The test verifies the token is white with exactly one color, but does not
verify that non-white colors are actually removed. A regression where the code copies
original colors AND adds white would pass this test for Sacred Cat but fail for a green
creature.
**Fix**: Add a test card definition for a green creature with Embalm (e.g., Honored Hydra:
{5}{G} 6/6 Snake Hydra Trample Embalm {3}{G}). Verify the token is white-only, not
white+green.

#### Finding 5: No test for Legendary creature embalm

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/embalm.rs`
**CR Rule**: 707.2 -- supertypes are copiable values
**Issue**: No test verifies that supertypes are copied. This would have caught Finding 1.
**Fix**: Add a test with a Legendary creature that has Embalm. Verify the token has the
Legendary supertype.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.128a (activated from graveyard) | Yes | Yes | test_embalm_basic_create_token, test_embalm_sorcery_speed_restriction |
| 702.128a (exile as cost) | Yes | Yes | test_embalm_card_exiled_as_cost, test_embalm_ability_on_stack_card_in_exile |
| 702.128a (token is white) | Yes | Yes | test_embalm_token_is_white (but only tests white card, not multi-color) |
| 702.128a (no mana cost) | Yes | Yes | test_embalm_token_has_no_mana_cost |
| 702.128a (Zombie added) | Yes | Yes | test_embalm_token_is_zombie |
| 702.128a (sorcery speed) | Yes | Yes | test_embalm_sorcery_speed_restriction (3 sub-tests) |
| 702.128a (supertypes copied) | **No** | No | Finding 1 -- supertypes set to empty |
| 702.128b (embalmed status) | No | No | Finding 3 -- no is_embalmed flag (LOW, defer) |
| 707.2 (copiable values: abilities) | Partial | Yes | Keywords copied; triggered/activated empty (Finding 2) |
| 707.9b (color override copiable) | Yes | Partial | Color is White; test only on white card (Finding 4) |
| 707.9d (no mana cost, CDA) | Yes | Yes | Mana cost is None |
| CR 302.6 (summoning sickness) | Yes | Yes | test_embalm_token_has_summoning_sickness |
| CR 602.2b (mana payment) | Yes | Yes | test_embalm_requires_mana_payment |
| CR 702.61a (split second) | Yes | No | Split second check in handler, no test |
| Not-a-cast (ruling) | Yes | Yes | test_embalm_is_not_a_cast |
| Multiplayer (active player only) | Yes | Yes | test_embalm_multiplayer_only_active_player (4-player) |

## Additional Notes

### Positive Observations

1. **Exile-as-cost correctly implemented**: The handler exiles the card during step 9
   (before pushing to stack), which is the correct sequence per CR 702.128a. This is the
   key difference from Unearth and it is handled correctly.

2. **CardId vs ObjectId**: The implementation correctly uses `source_card_id: Option<CardId>`
   in `StackObjectKind::EmbalmAbility` instead of `ObjectId`, recognizing that the original
   ObjectId is dead after the zone change (CR 400.7). This is a well-reasoned design choice.

3. **Hash discriminants**: All three new discriminants (92/27/25) are unique within their
   respective enum types. The `source_card_id` field is correctly hashed.

4. **Counter_stack_object**: The EmbalmAbility variant is correctly included in the
   non-spell counter arm with an appropriate comment about the card already being in exile.

5. **Full ETB pipeline**: The resolution runs the complete ETB pipeline (self_etb,
   replacements, register_permanent, register_static, fire_when_enters) which is correct.

6. **Test quality**: 12 tests with good CR citations, covering positive and negative cases.
   The `test_embalm_ability_on_stack_card_in_exile` test (test 9) is a well-designed
   intermediate-state test that verifies the two-phase nature of embalm.

7. **Replay harness**: The `embalm_card` action correctly uses `find_in_graveyard` since
   the card must be in the graveyard at activation time.

### Comparison with Plan

The implementation closely follows the plan. Discriminant numbers differ from the plan
(plan: 89/27/24; actual: 92/27/25) because JumpStart and Aftermath were added between
planning and implementation. This is correct -- the plan's numbers were estimates.
