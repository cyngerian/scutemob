# Ability Review: Prototype

**Date**: 2026-03-02
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.160 + 718 (Prototype Cards)
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 871-882)
- `crates/engine/src/cards/card_definition.rs` (lines 359-382)
- `crates/engine/src/state/game_object.rs` (lines 503-517)
- `crates/engine/src/state/stack.rs` (lines 173-184)
- `crates/engine/src/state/hash.rs` (lines 531-532, 707-708, 1643-1644, 3196-3206)
- `crates/engine/src/rules/command.rs` (lines 150-163)
- `crates/engine/src/rules/casting.rs` (lines 68, 972-997, 1651-1698, 1783, 1832, 3004-3050)
- `crates/engine/src/rules/resolution.rs` (lines 335-355, 1466, 2544, 2728, 2931)
- `crates/engine/src/rules/engine.rs` (lines 94, 112)
- `crates/engine/src/rules/commander.rs` (lines 210-218)
- `crates/engine/src/rules/copy.rs` (lines 160-205, 360-392)
- `crates/engine/src/state/mod.rs` (lines 255-308, 370-407)
- `crates/engine/src/state/builder.rs` (line 924)
- `crates/engine/src/effects/mod.rs` (line 2470)
- `crates/engine/src/testing/replay_harness.rs` (all CastSpell sites)
- `crates/engine/tests/prototype.rs` (all 10 tests)

## Verdict: needs-fix

The core casting and resolution implementation is solid. The `prototype` field is correctly
kept orthogonal to `alt_cost` (matching the CR 118.9 ruling), the cost selection chain is
sound, and prototype characteristics are properly applied to the spell on the stack and to
the permanent at resolution. Hash coverage is complete for all new fields.

However, there are two HIGH findings: (1) when a prototyped permanent leaves the battlefield,
`move_object_to_zone` clones the prototype-modified characteristics (P/T, mana_cost, colors)
without reverting them to the card's printed values, violating CR 718.4; and (2) `copy.rs`
hardcodes `was_prototyped: false` for spell copies, contradicting CR 718.3c which says copies
of prototyped spells are also prototyped. There is also one MEDIUM finding for an incomplete
test that only checks the flag, not the characteristics, after a zone change.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `state/mod.rs:259,307` | **Characteristics not reverted on zone change.** `move_object_to_zone` clones prototype-modified characteristics to the new object without reverting to printed values. **Fix:** After `move_object_to_zone`, if the old object's `is_prototyped` was true and the destination zone is NOT Battlefield/Stack, re-apply printed characteristics from CardDefinition. |
| 2 | **HIGH** | `rules/copy.rs:204` | **Copy of prototyped spell not prototyped.** `was_prototyped: false` contradicts CR 718.3c. **Fix:** Set `was_prototyped: original.was_prototyped`. |
| 3 | MEDIUM | `tests/prototype.rs:622-628` | **Test 5 only checks flag, not characteristics.** Does not verify P/T, mana_cost, colors revert after bounce. **Fix:** Add assertions for power==6, toughness==4, mana_value==7, colors.is_empty(). |
| 4 | LOW | `tests/prototype.rs` | **No copy test.** Missing test for CR 718.3c/718.3d (copy of prototyped spell/permanent inherits prototype characteristics). **Fix:** Add `test_prototype_copy_inherits_characteristics`. |
| 5 | LOW | `testing/replay_harness.rs` | **No `cast_spell_prototype` action.** Steps 5-7 are marked incomplete in WIP, so this is expected. Noted for completeness. |

### Finding Details

#### Finding 1: Characteristics not reverted on zone change (CR 718.4)

**Severity**: HIGH
**File**: `crates/engine/src/state/mod.rs:259` and `:307`
**CR Rule**: 718.4 -- "In every zone except the stack or the battlefield, and while on
the stack or the battlefield when not cast as a prototyped spell, a prototype card has
only its normal characteristics."
**Issue**: `move_object_to_zone` creates the new `GameObject` with
`characteristics: old_object.characteristics.clone()` (line 259) and sets
`is_prototyped: false` (line 307). The problem is that the `characteristics` field still
contains the prototype values (power=3, toughness=2, mana_cost={2}{R}, colors={Red} for
Blitz Automaton) because those were written into the base characteristics at casting time
(casting.rs:1688-1697) and re-applied at resolution (resolution.rs:342-354). When the
permanent moves from battlefield to hand/graveyard/exile, the characteristic values carry
over even though `is_prototyped` is cleared.

This means a Blitz Automaton bounced to hand would appear as a 3/2 red {2}{R} card, when
per CR 718.4 it should be a 6/4 colorless {7} card.

**Fix**: In `move_object_to_zone` (both instances), after creating the new `GameObject`,
if the old object has `is_prototyped == true` AND the destination zone is NOT
`ZoneId::Battlefield` and NOT `ZoneId::Stack`, then look up the card's `CardDefinition`
from the registry and restore the printed characteristics:
```rust
// CR 718.4: When a prototyped permanent leaves the battlefield,
// revert characteristics to printed values.
if old_object.is_prototyped && to != ZoneId::Battlefield {
    if let Some(cid) = &new_object.card_id {
        if let Some(def) = self.card_registry.get(cid.clone()) {
            new_object.characteristics.power = def.power;
            new_object.characteristics.toughness = def.toughness;
            new_object.characteristics.mana_cost = def.mana_cost.clone();
            // Revert colors from printed mana cost (CR 105.2)
            new_object.characteristics.colors = if let Some(ref mc) = def.mana_cost {
                crate::rules::casting::colors_from_mana_cost(mc)
            } else {
                im::OrdSet::new()
            };
        }
    }
}
```
Note: This fix must be applied in BOTH `move_object_to_zone` callsites (lines ~255-308
and ~360-407).

Alternatively, since the resolution.rs already re-applies prototype characteristics after
`move_object_to_zone` for the stack-to-battlefield case, the `move_object_to_zone` function
could be enhanced to accept a reference to `CardRegistry` and handle all characteristic
revert logic in one place. However, this is a larger refactor and the targeted fix above
is sufficient.

#### Finding 2: Copy of prototyped spell not prototyped (CR 718.3c)

**Severity**: HIGH
**File**: `crates/engine/src/rules/copy.rs:204`
**CR Rule**: 718.3c -- "If a prototyped spell is copied, the copy is also a prototyped
spell. It has the alternative power, toughness, and mana cost characteristics of the spell
and not the normal power, toughness, and mana cost characteristics of the card that
represents the prototyped spell."
**Issue**: The `copy_spell_on_stack` function creates the copy StackObject with
`was_prototyped: false` (line 204). The comment is missing entirely -- there is no CR
citation explaining why this is false. Per CR 718.3c, if the original spell was prototyped,
the copy must also be prototyped.

Note that the copy's `kind` is cloned from the original (line 169:
`kind: original.kind.clone()`), which means the source object's characteristics (including
prototype P/T/mana/color) are inherited via the source object reference. However, the
`was_prototyped` flag on the StackObject determines whether the permanent it becomes will
have `is_prototyped = true` (see resolution.rs:342-355). Without this flag, a copied
prototyped spell would resolve into a permanent without `is_prototyped` set, and the
permanent would not be properly handled as prototyped.

**Fix**: Change line 204 from:
```rust
was_prototyped: false,
```
to:
```rust
// CR 718.3c: If a prototyped spell is copied, the copy is also a prototyped spell.
was_prototyped: original.was_prototyped,
```

#### Finding 3: Test 5 only checks flag, not characteristics

**Severity**: MEDIUM
**File**: `crates/engine/tests/prototype.rs:622-628`
**CR Rule**: 718.4 -- "In every zone except the stack or the battlefield ... a prototype
card has only its normal characteristics."
**Issue**: `test_prototype_leaves_battlefield_resumes_normal` only asserts that
`is_prototyped == false` after `move_object_to_zone` to hand. It does NOT assert that
power, toughness, mana_cost, and colors have reverted to their printed values. This means
the test passes even though Finding 1 means the characteristics are still wrong.

**Fix**: After the existing assertion on line 626, add:
```rust
// CR 718.4: characteristics should revert to printed values after zone change.
assert_eq!(
    hand_state.objects[&hand_id].characteristics.power,
    Some(6),
    "CR 718.4: power should revert to printed value 6 in hand"
);
assert_eq!(
    hand_state.objects[&hand_id].characteristics.toughness,
    Some(4),
    "CR 718.4: toughness should revert to printed value 4 in hand"
);
let mc = hand_state.objects[&hand_id].characteristics.mana_cost.as_ref().unwrap();
assert_eq!(
    mc.mana_value(),
    7,
    "CR 718.4: mana value should revert to 7 in hand"
);
assert!(
    hand_state.objects[&hand_id].characteristics.colors.is_empty(),
    "CR 718.4: colors should revert to colorless in hand"
);
```

#### Finding 4: Missing copy test (CR 718.3c/718.3d)

**Severity**: LOW
**File**: `crates/engine/tests/prototype.rs`
**CR Rule**: 718.3c -- "If a prototyped spell is copied, the copy is also a prototyped
spell."
718.3d -- "If a permanent that was a prototyped spell is copied, the copy has the
alternative power, toughness, and mana cost characteristics."
**Issue**: No test verifies that copies of prototyped spells/permanents inherit the
prototype characteristics. This is an important interaction documented in the plan but
not tested.
**Fix**: Add a test that copies a prototyped spell on the stack (or a prototyped permanent
on the battlefield) and verifies the copy has `was_prototyped: true` and prototype
characteristics.

#### Finding 5: Replay harness missing `cast_spell_prototype`

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs`
**CR Rule**: N/A (infrastructure)
**Issue**: The replay harness does not have a `"cast_spell_prototype"` action arm.
Steps 5-7 are marked incomplete in `ability-wip.md`, so this is expected and deferred.
**Fix**: Add `"cast_spell_prototype"` arm per the plan when implementing Step 5b.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.160a   | Yes         | Yes     | test_prototype_basic_cast, test_prototype_negative_not_prototype_keyword |
| 718.1      | N/A (frame) | N/A     | Visual frame rule, not engine-relevant |
| 718.2      | Yes         | Yes     | Characteristics applied at casting + resolution |
| 718.2a     | Partial     | No      | Copiable values: base chars set correctly, but copy system has Finding 2 |
| 718.3      | Yes         | Yes     | Choice at cast time via `prototype: bool` on CastSpell |
| 718.3a     | Yes         | Yes     | Prototype cost used as base cost in casting pipeline |
| 718.3b     | Yes         | Yes     | P/T, color, mana_cost set on stack and permanent; test_prototype_basic_cast, test_prototype_color_change, test_prototype_mana_value |
| 718.3c     | **No**      | No      | Finding 2: `copy.rs` sets `was_prototyped: false` for copies |
| 718.3d     | Partial     | No      | Base chars on permanent are correct (copy of permanent would get right chars), but `is_prototyped` not propagated |
| 718.4      | **Partial** | Partial | `is_prototyped` flag reset on zone change, but characteristics NOT reverted (Finding 1). test_prototype_leaves_battlefield_resumes_normal only checks flag (Finding 3). test_prototype_in_graveyard_normal_chars tests graveyard-from-scratch (not post-bounce). |
| 718.5      | Yes         | Yes     | test_prototype_retains_keyword_ability verifies Haste + Prototype keywords retained |
| 105.2      | Yes         | Yes     | `colors_from_mana_cost()` correctly derives colors; test_prototype_color_change |
| 704.5f     | Yes         | Yes     | test_prototype_sba_toughness_check |
| 903.4 (commander identity) | Yes | No | commander.rs:210-218 scans Prototype costs; no unit test for this |
