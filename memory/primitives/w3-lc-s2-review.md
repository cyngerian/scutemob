# W3-LC S2 Review: HIGH Layer Correctness Fixes

**Date**: 2026-03-19
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 613.1, 613.1d, 613.1f, 613.4, 302.6, 702.10
**Files reviewed**:
- `crates/engine/src/effects/mod.rs` (lines 3720-3765)
- `crates/engine/src/rules/abilities.rs` (lines 6035-6098)
- `crates/engine/src/rules/mana.rs` (lines 152-207)
- `crates/engine/tests/layer_correctness.rs` (full, 459 lines)

## Verdict: needs-fix

All three fixes correctly address the identified HIGH bugs. The layer-resolved
characteristic reads are in the right locations and use the correct fallback
pattern. However, the abilities.rs fix introduces an ability_index namespace
mismatch between trigger collection and trigger resolution that can cause
incorrect behavior under copy effects or mutate. One finding at MEDIUM, two
at LOW.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `abilities.rs:6093` + `resolution.rs:1939` | **ability_index namespace mismatch.** Index stored from layer-resolved list, consumed from base list. **Fix:** resolution.rs must also use layer-resolved triggered_abilities. |
| 2 | LOW | `layer_correctness.rs:116,145` | **PowerOf/ToughnessOf tests don't exercise resolve_amount.** Tests verify calculate_characteristics output, not the actual effects/mod.rs code path. **Fix:** Add integration test that triggers a DealDamage with EffectAmount::PowerOf under an anthem and checks resulting life total. |
| 3 | LOW | `layer_correctness.rs:184,231` | **Humility trigger suppression test doesn't exercise collect_triggers_for_event.** Test verifies calculate_characteristics removes triggers, but doesn't verify the engine actually skips them during event processing. **Fix:** Add integration test where a creature with an ETB trigger enters under Humility and verify no trigger is placed on the stack. |

### Finding Details

#### Finding 1: ability_index namespace mismatch between trigger collection and resolution

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:6093` and `crates/engine/src/rules/resolution.rs:1939,2068,2098`
**CR Rule**: 613.1f -- Layer 6 ability-adding/removing effects
**Issue**: The S2 fix at abilities.rs:6039 iterates `resolved_chars.triggered_abilities`
(layer-resolved) and stores the enumeration index at line 6093 as `ability_index: idx`.
When this trigger resolves, resolution.rs lines 1939, 2068, and 2098 look up
`obj.characteristics.triggered_abilities.get(ability_index)` -- base characteristics.

Under normal conditions (no copy effects, no mutate merge), the base and resolved lists
are identical (same contents, same order), so indices match. The mismatch manifests in
two edge cases:

1. **Copy effects (Layer 1)**: A Clone copying a creature with different triggered
   abilities gets a completely new resolved list. The index into the resolved list
   won't correspond to the right trigger in the base list.

2. **Mutate merge**: Additional triggered abilities are appended from merged components.
   An index >= base_len would return None from the base list, falling through to the
   card registry (which may not have the merged trigger either).

Before S2, both collection and resolution used base characteristics -- indices were
consistent (though Humility was broken). S2 correctly fixes collection to use
layer-resolved, but resolution still uses base, creating the mismatch.

**Practical impact**: Low in current gameplay (copy + triggered ability + specific index
ordering is rare), but this is an architectural inconsistency that will cause bugs as
more copy effects are implemented.

**Fix**: In resolution.rs, at lines 1939, 2068, and 2098, replace
`obj.characteristics.triggered_abilities.get(ability_index)` with a layer-resolved
lookup:
```
let resolved = crate::rules::layers::calculate_characteristics(state, source_object)
    .unwrap_or_else(|| obj.characteristics.clone());
resolved.triggered_abilities.get(ability_index)
```
This aligns the resolution namespace with the collection namespace. Note: this is
technically a S3+ fix since resolution.rs is not in S2 scope, but it should be
addressed before more copy/mutate interactions are tested.

#### Finding 2: PowerOf/ToughnessOf tests only verify layer system, not resolve_amount

**Severity**: LOW
**File**: `crates/engine/tests/layer_correctness.rs:116,145`
**CR Rule**: 613.4
**Issue**: Tests `test_w3lc_power_of_uses_layer_resolved_pt` and
`test_w3lc_toughness_of_uses_layer_resolved_under_humility` call
`calculate_characteristics()` directly and check the returned P/T values. They
verify that the layer system produces correct output, but they do not exercise the
`resolve_amount()` function in effects/mod.rs that was actually modified in S2.
Since `resolve_amount` is private, this is understandable, but it means the actual
fix (using layer-resolved P/T in the PowerOf/ToughnessOf match arms) is not directly
tested through the engine's command processing pipeline.

The mana.rs tests (lines 261-458) are excellent by contrast -- they go through
`process_command(Command::TapForMana)` and exercise the actual fixed code path.

**Fix**: Add an integration test that processes a command causing an effect with
`EffectAmount::PowerOf` (e.g., "deals damage equal to its power") under an anthem,
and verifies the resulting damage/life change reflects the layer-resolved power.
This would require a card definition or a more complex test setup. Acceptable to
defer if deemed low priority.

#### Finding 3: Trigger suppression test doesn't exercise collect_triggers_for_event

**Severity**: LOW
**File**: `crates/engine/tests/layer_correctness.rs:184,231`
**CR Rule**: 613.1f
**Issue**: `test_w3lc_humility_suppresses_triggered_abilities` verifies that
`calculate_characteristics()` returns an empty `triggered_abilities` list under
Humility. It does not verify that `collect_triggers_for_event()` in abilities.rs
actually skips triggers for Humility-affected creatures. Similarly,
`test_w3lc_etb_filter_uses_layer_resolved_types` checks that an animated land
has Creature in its resolved types but doesn't test the ETB filter path.

The tests prove the layer system works correctly (prerequisite), but don't
prove the engine code paths that were modified in S2 actually consume the
layer-resolved values.

**Fix**: Add an integration test that: (1) creates a creature with an ETB trigger
and a Humility effect, (2) processes commands to have that creature enter the
battlefield, and (3) asserts that no trigger appears on the stack or in
pending_triggers. This requires processing through the full engine pipeline.
Acceptable to defer if deemed low priority.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 613.1 (layer system overview) | Yes | Yes | PowerOf/ToughnessOf use layer-resolved P/T |
| 613.1d (Layer 4 type change) | Yes | Yes | Animated land summoning sickness, ETB filter creature_only |
| 613.1f (Layer 6 ability grant/remove) | Yes | Partial | Humility suppression verified via calculate_characteristics; collect_triggers_for_event not directly tested |
| 613.4 (P/T continuous effects) | Yes | Yes | Anthem and Humility P/T setting verified |
| 302.6 (summoning sickness) | Yes | Yes | 3 mana.rs tests: animated land blocked, Fervor allows, non-creature unaffected |
| 702.10 (Haste) | Yes | Yes | Fervor-granted haste recognized through layer 6 |

## Additional Observations (not findings)

1. **Fallback pattern is safe**: `calculate_characteristics(state, id).unwrap_or_else(|| obj.characteristics.clone())` -- since `obj` was just successfully retrieved from `state.objects`, the object exists, so `calculate_characteristics` will return `Some(...)`. The fallback is defensive dead code, which is fine.

2. **Borrow safety in mana.rs**: At line 119, `obj` is `.clone()`d (owned), so `calculate_characteristics(state, source)` at line 157 doesn't conflict. At line 181, `obj` is `&GameObject` inside a block that ends before `state.move_object_to_zone` at line 193. No borrow issues.

3. **Mana ability fetch at mana.rs:135-138**: Reads `obj.characteristics.mana_abilities` (base) -- under Humility, this should be empty. This is a separate MEDIUM bug from the S1 audit (not in S2 scope) and will be addressed in S3+.

4. **Performance**: Each fix adds one `calculate_characteristics()` call. For `collect_triggers_for_event` (abilities.rs:6037), this is called per battlefield permanent per event, which could be significant in large board states. The audit plan notes this concern and defers optimization. Acceptable for correctness-first approach.

5. **Test quality**: The mana.rs tests (tests 5-8) are excellent integration tests that exercise the actual fixed code paths through `process_command`. The sacrifice test (test 8) even verifies the correct event type (`CreatureDied` vs `PermanentDestroyed`). Well done.
