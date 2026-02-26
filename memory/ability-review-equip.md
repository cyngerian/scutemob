# Ability Review: Equip

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.6
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (Effect::AttachEquipment)
- `crates/engine/src/effects/mod.rs` (AttachEquipment execution, lines 1020-1107)
- `crates/engine/src/rules/events.rs` (GameEvent::EquipmentAttached)
- `crates/engine/src/state/game_object.rs` (sorcery_speed field on ActivatedAbility)
- `crates/engine/src/rules/abilities.rs` (sorcery-speed enforcement, lines 87-108)
- `crates/engine/src/state/hash.rs` (hashes for ActivatedAbility.sorcery_speed, Effect::AttachEquipment, GameEvent::EquipmentAttached)
- `crates/engine/src/testing/replay_harness.rs` (timing_restriction propagation, lines 386-416)
- `crates/engine/tests/equip.rs` (12 tests)

## Verdict: needs-fix

The implementation is largely correct and well-structured. The sorcery-speed enforcement,
attachment/detachment logic, timestamp update, and hash coverage are all properly done.
However, there is one MEDIUM finding: the `AttachEquipment` effect checks the creature type
using raw `obj.characteristics.card_types` instead of layer-computed characteristics, which
means a permanent that gained or lost the Creature type through a continuous effect (e.g.,
Humility removing it, or an effect animating a noncreature into a creature) would be
incorrectly evaluated. There is also one MEDIUM finding for the missing "target creature you
control" validation at activation time -- while this is a pre-existing architectural gap
(activated abilities don't validate TargetRequirements), the Equip ability's defining
characteristic is that it can ONLY target creatures you control, and the current code allows
targeting any object. Additionally, one MEDIUM for a test gap: no test verifies that the
timestamp is updated on re-equip (CR 701.3c), despite this being a documented edge case in
the plan.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `effects/mod.rs:1058` | **Layer-unaware creature type check.** Effect uses raw characteristics instead of `calculate_characteristics`. **Fix:** use layer-computed types. |
| 2 | MEDIUM | `rules/abilities.rs:196-220` | **No "creature you control" target requirement at activation.** Equip can target any object. **Fix:** add target type validation or document as pre-existing gap. |
| 3 | MEDIUM | `tests/equip.rs` | **Missing timestamp update test for CR 701.3c.** Plan explicitly listed this edge case. **Fix:** add test. |
| 4 | LOW | `effects/mod.rs:1088` | **Timestamp increment ordering.** Counter is incremented before reading, which skips one value vs `next_object_id()` pattern. Not incorrect but diverges from convention. |
| 5 | LOW | `tests/equip.rs:373` | **Test 5 (opponent creature) tests resolution behavior, not activation rejection.** CR 702.6a says equip can only TARGET creatures you control; the test verifies the effect does nothing at resolution instead of blocking at activation. This is consistent with the pre-existing gap in Finding 2. |
| 6 | LOW | `tests/equip.rs` | **No test for Equip {0} (zero-cost equip).** Lightning Greaves is Equip {0}; the equip_ability helper sets mana_cost to None for 0, which works but is untested. |
| 7 | LOW | `tests/equip.rs` | **No test for equipment keyword grant via layer system.** Plan test #2 (test_equip_grants_keywords_to_equipped_creature) was listed but not implemented. |

### Finding Details

#### Finding 1: Layer-unaware creature type check

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:1058`
**CR Rule**: 702.6a -- "[Cost]: Attach this permanent to target creature you control."
**Issue**: The `AttachEquipment` effect validates the target's creature type using
`obj.characteristics.card_types.contains(&CardType::Creature)` -- this reads the raw
(base) characteristics of the object, not the layer-computed characteristics. If a
noncreature permanent has been animated into a creature by a continuous effect (e.g.,
Sydri, Galvanic Genius; March of the Machines; Mycosynth Lattice + Karn), the raw
characteristics would NOT include `CardType::Creature`, causing the equip effect to
incorrectly skip attachment. Conversely, if a creature lost its creature type through
a continuous effect (e.g., Lignify applied then type line overwritten), the raw check
would still see Creature.

This same pattern is used elsewhere in the codebase (e.g., other effects check raw
characteristics), so this is partially a systemic issue. However, equipment attachment
is one of the most common interactions where continuous-effect type changes matter
(Cranial Plating on an animated artifact, etc.).

**Fix**: Replace the raw `obj.characteristics.card_types` check with
`crate::rules::layers::calculate_characteristics(state, target_id)` to get the
layer-computed card types. Fall back to raw characteristics if `calculate_characteristics`
returns None (consistent with the pattern used in `doubler_applies_to_trigger` at
`abilities.rs:749-755`).

#### Finding 2: No "creature you control" target requirement at activation

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:196-220`
**CR Rule**: 702.6a -- "Attach this permanent to target creature you control."
**Issue**: The `handle_activate_ability` function validates targets only for
hexproof/shroud/protection (lines 196-220). It does NOT check whether the target is
a creature, whether it's on the battlefield, or whether it's controlled by the
activating player. This means a player can activate Equip targeting a noncreature
permanent, a card in hand, or an opponent's creature, and the activation succeeds --
the mana is spent and the ability goes on the stack. At resolution, the
`AttachEquipment` effect validates the target and does nothing, but the mana is already
gone. Per CR 601.2c / CR 115.1, targets must be legal at the time the ability is put
on the stack. Illegal targets should be rejected at activation, not resolution.

This is a pre-existing architectural gap (no activated ability validates its target
type at activation), but it's particularly impactful for Equip because:
1. The mana is wasted when targeting an illegal object.
2. CR 702.6a explicitly restricts targets to "creature you control."

**Fix**: Either (a) add a `target_requirements: Vec<TargetRequirement>` field to
`ActivatedAbility` and validate at activation time (matching the spell casting
pattern), or (b) log this as a known gap in the ability-wip.md and address it as
a follow-up task. Option (b) is acceptable for this review cycle since it's
pre-existing and affects all activated abilities with targets.

#### Finding 3: Missing timestamp update test for CR 701.3c

**Severity**: MEDIUM
**File**: `crates/engine/tests/equip.rs`
**CR Rule**: 701.3c -- "Attaching an Aura, Equipment, or Fortification on the
battlefield to a different object or player causes the Aura, Equipment, or
Fortification to receive a new timestamp."
**Issue**: The plan (Step 4, test #2 or the interactions list item #4) explicitly
called out timestamp updates on reattach as a key edge case. Test 6
(`test_equip_reequip_detaches_from_previous`) verifies attachment/detachment state
but does NOT assert that the equipment's timestamp was updated. Test 8
(`test_equip_already_attached_to_same_target_no_op`) captures `_old_timestamp` but
never asserts against it (the variable is unused -- prefixed with underscore). This
means CR 701.3c compliance is untested.

**Fix**: In `test_equip_reequip_detaches_from_previous`, capture the equipment's
timestamp before equipping creature B, then assert that the timestamp is strictly
greater after resolution. Additionally, in `test_equip_already_attached_to_same_target_no_op`,
assert that the timestamp is NOT updated (the no-op case should preserve the old
timestamp).

#### Finding 4: Timestamp increment ordering diverges from convention

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:1088-1089`
**CR Rule**: 701.3c / 613.7e
**Issue**: The code increments `timestamp_counter` first, then reads it:
```rust
state.timestamp_counter += 1;
let new_ts = state.timestamp_counter;
```
The `next_object_id()` method (state/mod.rs:133-136) also increments first then reads,
so this is actually consistent with the existing pattern. No fix needed -- this finding
is informational only.

#### Finding 5: Test 5 tests resolution behavior, not activation rejection

**Severity**: LOW
**File**: `crates/engine/tests/equip.rs:373-434`
**CR Rule**: 702.6a -- "target creature you control"
**Issue**: Test 5 (`test_equip_target_opponent_creature_does_nothing`) verifies that
equipping an opponent's creature produces no attachment at resolution. However, per CR
702.6a, the target should be illegal at activation time -- a player cannot even declare
an opponent's creature as a target for Equip. The test documents the current behavior
(effect-level validation) rather than the correct behavior (activation-level rejection).
This is consistent with the pre-existing gap in Finding 2.

**Fix**: Add a comment to the test noting this is a workaround for the missing
activation-time target validation, and that the test should be updated to expect
an activation-time error once `TargetRequirement` validation is added to
`handle_activate_ability`.

#### Finding 6: No test for Equip {0}

**Severity**: LOW
**File**: `crates/engine/tests/equip.rs`
**CR Rule**: 702.6a
**Issue**: The `equip_ability` helper creates Equip {0} by setting `mana_cost: None`.
This works because `handle_activate_ability` skips the mana check when
`ability_cost.mana_cost` is None. However, there is no test that exercises this path.
Lightning Greaves (the most common Equipment in Commander) has Equip {0}, so this
code path is high-frequency.

**Fix**: Add a test `test_equip_zero_cost_succeeds` that creates equipment with
`equip_ability(0)`, activates it without adding any mana, and verifies attachment
succeeds.

#### Finding 7: No test for equipment keyword grant via layer system

**Severity**: LOW
**File**: `crates/engine/tests/equip.rs`
**CR Rule**: 702.6a + 604 (static abilities)
**Issue**: The plan listed test #2 (`test_equip_grants_keywords_to_equipped_creature`)
to verify that after equipping, the layer system correctly grants keywords from the
equipment's static abilities to the equipped creature. This test was not implemented.
While the layer system's `AttachedCreature` filter is tested elsewhere, the end-to-end
path (equip -> attach -> keywords appear on creature) is not covered by the equip test
suite.

**Fix**: Add a test that creates equipment with a static ability granting a keyword
(e.g., Haste) via `AbilityDefinition::Static` with `EffectFilter::AttachedCreature`,
equips a creature, and asserts that `calculate_characteristics` for the creature
includes the granted keyword.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.6a (Equip is activated, sorcery-speed, target creature you control) | Yes | Yes (partial) | Sorcery-speed: tested. Target validation only at resolution (Finding 2). |
| 702.6b (see rule 301) | N/A | N/A | Reference rule only. |
| 702.6c (Equip [quality] restrictions) | No | No | Not in scope for base Equip. |
| 702.6d (multiple equip abilities) | Yes (inherent) | No | Multiple activated abilities supported by the existing framework. No explicit test. |
| 702.6e (Equip planeswalker) | No | No | Not in scope for base Equip. |
| 701.3a (Attach semantics) | Yes | Yes | test_equip_basic_attaches_to_creature |
| 701.3b (Can't attach to illegal; same target = no-op) | Yes | Yes | test_equip_already_attached_to_same_target_no_op |
| 701.3c (New timestamp on reattach) | Yes | No (Finding 3) | Implemented but not tested. |
| 301.5c (Can't equip self; can't equip more than one) | Yes | Yes | test_equip_cannot_equip_self, test_equip_reequip_detaches_from_previous |
| 301.5d (Equipment controller separate from creature controller) | Yes (inherent) | Partial | test_equip_target_opponent_creature_does_nothing touches this. |
| 602.5d (Sorcery-speed timing) | Yes | Yes | Tests 2, 3, 4 cover main phase, active player, empty stack. |
| 704.5n (Equipment on illegal permanent = unattach SBA) | Pre-existing | Pre-existing | Already implemented in sba.rs. |
