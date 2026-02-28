# Ability Review: Bestow

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.103
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Bestow)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Bestow)
- `crates/engine/src/state/stack.rs` (was_bestowed field)
- `crates/engine/src/state/game_object.rs` (is_bestowed field)
- `crates/engine/src/state/hash.rs` (hash coverage for both fields)
- `crates/engine/src/state/mod.rs` (move_object_to_zone: is_bestowed reset)
- `crates/engine/src/rules/command.rs` (cast_with_bestow field)
- `crates/engine/src/rules/engine.rs` (dispatch)
- `crates/engine/src/rules/casting.rs` (validation, cost, type transform, get_bestow_cost)
- `crates/engine/src/rules/resolution.rs` (bestow fallback + is_bestowed transfer + type re-apply)
- `crates/engine/src/rules/sba.rs` (CR 702.103f bestowed Aura revert)
- `crates/engine/src/rules/events.rs` (BestowReverted)
- `crates/engine/src/rules/copy.rs` (CR 702.103c: copies inherit bestowed status)
- `crates/engine/src/rules/abilities.rs` (all StackObject literals updated)
- `crates/engine/src/testing/replay_harness.rs` (cast_spell_bestow action)
- `tools/replay-viewer/src/view_model.rs` (format_keyword)
- `crates/engine/tests/bestow.rs` (9 tests)

## Verdict: needs-fix

The bestow implementation is thorough and demonstrates strong understanding of the CR
rules. All core rules (702.103a-f) are implemented, the SBA exception for bestowed Auras
is correctly placed within `check_aura_sbas`, the resolution fallback for illegal targets
works, and the type transformation is applied at both cast time (for validation) and on the
stack source object. Hash coverage is complete. The copy path correctly inherits bestowed
status per CR 702.103c. However, there is one MEDIUM issue (contradictory doc comment in
stack.rs that directly contradicts the correct implementation and CR 702.103c) and one
MEDIUM issue (missing 303.4d "Aura that's also a creature" SBA check in the aura SBA path
that is relevant to bestow edge cases).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `crates/engine/src/state/stack.rs:71` | **Doc comment contradicts implementation and CR 702.103c.** Comment says "Must always be false for copies" but copy.rs correctly sets `was_bestowed: original.was_bestowed` per CR 702.103c. **Fix:** Update comment. |
| 2 | MEDIUM | `crates/engine/src/rules/sba.rs:625-698` | **Missing CR 303.4d "Aura that's also a creature" SBA.** The check_aura_sbas function does not check that an Aura that is also a creature can't enchant anything. If a bestowed Aura somehow gains the Creature type (e.g., via a type-changing effect), the SBA should make it fall off. **Fix:** Add check. |
| 3 | LOW | `crates/engine/src/rules/resolution.rs:225-240` | **Redundant bestow type re-application.** move_object_to_zone preserves characteristics from the stack, so the bestow transformation is already present. The re-application is defensive and harmless but the comment is misleading. **Fix:** Correct the comment. |
| 4 | LOW | `crates/engine/tests/bestow.rs` | **Missing test for CR 702.103c (copy inherits bestowed).** The implementation correctly handles copies in copy.rs but no test verifies this. **Fix:** Add a test. |
| 5 | LOW | `crates/engine/tests/bestow.rs` | **Missing test for bestow + commander tax interaction.** CR 118.9d says additional costs (including commander tax) apply on top of alternative costs. No test verifies bestow cost + commander tax. **Fix:** Add a test. |
| 6 | LOW | `crates/engine/tests/bestow.rs:987-988` | **Dead code: unused `bear` variable.** The variable `bear` on line 988 is created but never used; `bear_spec` is the one passed to the builder. The `let _ = bear;` on line 1016 suppresses the warning but the variable should just be removed. **Fix:** Remove the unused `bear` variable. |

### Finding Details

#### Finding 1: Doc comment contradicts implementation and CR 702.103c

**Severity**: MEDIUM
**File**: `crates/engine/src/state/stack.rs:71`
**CR Rule**: 702.103c -- "If a bestowed Aura spell is copied, the copy is also a bestowed Aura spell. Any rule that refers to a spell cast bestowed applies to the copy as well."
**Issue**: The doc comment on `StackObject::was_bestowed` states: "Must always be false for copies (`is_copy: true`) -- copies are not cast." This directly contradicts CR 702.103c, which explicitly states copies of bestowed spells ARE also bestowed. The actual code in `copy.rs:180` correctly sets `was_bestowed: original.was_bestowed`, so the implementation is right but the documentation is wrong. A future developer reading the doc comment could "fix" the copy.rs code to match the comment and break correctness.
**Fix**: Replace the comment "Must always be false for copies (`is_copy: true`) -- copies are not cast." with "CR 702.103c: If the original spell was bestowed, copies are also bestowed Aura spells."

#### Finding 2: Missing CR 303.4d "Aura that's also a creature" SBA

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/sba.rs:625-698`
**CR Rule**: 303.4d -- "An Aura that's also a creature can't enchant anything. If this occurs somehow, the Aura becomes unattached, then is put into its owner's graveyard."
**Issue**: The `check_aura_sbas` function checks several conditions for illegal Auras (unattached, target gone, enchant restriction mismatch, protection) but does not check CR 303.4d: an Aura that is simultaneously a creature cannot enchant anything. While bestow correctly removes the Creature type when bestowed (so this normally won't trigger), a type-changing continuous effect (e.g., Humility making everything a creature, or an opponent's "all permanents are creatures" effect) could make a bestowed Aura also a creature, which should cause it to fall off. For bestowed permanents, this would interact with 702.103f: if the Aura gains the Creature type, it becomes illegal, which should trigger the bestow revert (not graveyard, since it's bestowed). The current SBA code would catch it through the enchant-restriction check (Enchant Creature on a target that still matches), so this finding applies only to the explicit 303.4d "Aura is also a creature" case which bypasses the enchant-restriction check.
**Fix**: Add a 303.4d check in `check_aura_sbas`: if an Aura on the battlefield also has the Creature card type (per layer-computed characteristics), mark it as illegal. The existing bestowed/normal partition will handle the consequences correctly. This is a pre-existing gap not introduced by the bestow implementation, but bestow makes it more relevant.

#### Finding 3: Redundant bestow type re-application at resolution

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:225-240`
**CR Rule**: 702.103b
**Issue**: The comment at line 225-226 says "re-apply the type transformation after move_object_to_zone (which resets to printed types)." But `move_object_to_zone` (in `state/mod.rs:259`) does `characteristics: old_object.characteristics.clone()`, which preserves the characteristics from the stack object -- including the already-applied bestow transformation. The re-application is harmless (inserting into an OrdSet that already contains the value is a no-op), but the comment is misleading and could cause confusion.
**Fix**: Update the comment to: "CR 702.103b: Defensively re-apply bestow type transformation. move_object_to_zone preserves stack characteristics, but this ensures correctness if the enrichment path changes."

#### Finding 4: Missing test for CR 702.103c (copy inherits bestowed)

**Severity**: LOW
**File**: `crates/engine/tests/bestow.rs`
**CR Rule**: 702.103c -- "If a bestowed Aura spell is copied, the copy is also a bestowed Aura spell."
**Issue**: The implementation in `copy.rs:180` correctly inherits `was_bestowed` from the original, but no test in `bestow.rs` verifies this behavior. A regression could go undetected.
**Fix**: Add a test `test_bestow_copy_inherits_bestowed_status` that creates a bestowed spell on the stack, copies it (e.g., via the `copy_spell_on_stack` function), and verifies the copy has `was_bestowed = true` and retains Aura characteristics.

#### Finding 5: Missing test for bestow + commander tax

**Severity**: LOW
**File**: `crates/engine/tests/bestow.rs`
**CR Rule**: 118.9d -- "If an alternative cost is being paid to cast a spell, any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost."
**Issue**: Commander tax applies on top of alternative costs (CR 118.9d / CR 903.8). The casting code correctly chains commander tax onto the bestow cost (lines 232-243), but no test verifies this interaction.
**Fix**: Add a test `test_bestow_commander_tax_applies` that casts a bestow creature from the command zone with commander tax and verifies the total cost is bestow_cost + tax.

#### Finding 6: Dead code in test file

**Severity**: LOW
**File**: `crates/engine/tests/bestow.rs:988`
**CR Rule**: N/A (code quality)
**Issue**: `let bear = ObjectSpec::creature(p1, "Mock Bear", 2, 2).in_zone(ZoneId::Hand(p1));` creates a variable that is never used (the test uses `bear_spec` instead). The `let _ = bear;` on line 1016 suppresses the compiler warning but the variable serves no purpose.
**Fix**: Remove line 988 (`let bear = ...`) and line 1016 (`let _ = bear;`).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.103a (bestow is alternative cost) | Yes | Yes | test_bestow_alternative_cost_pays_bestow_cost |
| 702.103b (becomes Aura on stack) | Yes | Yes | test_bestow_cast_as_aura_basic |
| 702.103c (copy inherits bestowed) | Yes | No | copy.rs:180 correct, no test (Finding 4) |
| 702.103d (only modified characteristics evaluated) | Yes | Partial | Implicit via type transform before validation |
| 702.103e (illegal target -> creature fallback) | Yes | Yes | test_bestow_target_illegal_at_resolution_becomes_creature |
| 702.103f (unattach -> ceases bestowed) | Yes | Yes | test_bestow_unattach_reverts_to_creature |
| 702.103g (phases in unattached) | No | No | Phasing deferred (CLAUDE.md: 3 DEFERRED) |
| 118.9a (one alternative cost) | Yes | Yes | test_bestow_cannot_combine_with_flashback, test_bestow_cannot_combine_with_evoke |
| 118.9c (mana value unchanged) | Yes | Partial | Implicit (printed cost stored separately), no explicit mana_value() assertion |
| 118.9d (tax applies to alt cost) | Yes | No | casting.rs handles it, no test (Finding 5) |
| 608.3b (bestow fallback at resolution) | Yes | Yes | test_bestow_target_illegal_at_resolution_becomes_creature |
| 303.4d (Aura + creature SBA) | No | No | Pre-existing gap, relevant to bestow (Finding 2) |
| Enters w/o casting = creature | Yes | Yes | test_bestow_enters_without_casting_is_creature |
| Non-bestow spell rejected | Yes | Yes | test_bestow_non_bestow_spell_rejected |
| Normal (non-bestow) cast | Yes | Yes | test_bestow_cast_normally_as_creature |
