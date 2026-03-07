# Ability Review: Fortify

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.67
**Files reviewed**:
- `crates/engine/src/state/types.rs:1196-1201`
- `crates/engine/src/state/hash.rs:643-647, 975, 2882-2888, 3482-3486`
- `crates/engine/src/state/continuous_effect.rs:92-97`
- `crates/engine/src/cards/card_definition.rs:850-864`
- `crates/engine/src/effects/mod.rs:2063-2169`
- `crates/engine/src/rules/abilities.rs:185-231`
- `crates/engine/src/rules/layers.rs:352-370`
- `crates/engine/src/rules/events.rs:286-294`
- `crates/engine/tests/fortify.rs` (entire file, 574 lines)
- `tools/replay-viewer/src/view_model.rs:850`

## Verdict: needs-fix

One MEDIUM finding: the resolution handler does not check whether the Fortification source is also a creature, violating CR 301.6 ("a Fortification that's also a creature (not a land) can't fortify a land"). Two LOW findings for naming/consistency issues. Overall the implementation is well-structured, closely parallels the existing Equip implementation, and the tests are thorough for the common cases.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `effects/mod.rs:2088-2093` | **CR 301.6 creature-Fortification check missing.** Animated Fortification can illegally fortify a land. **Fix:** Add layer-aware creature type check on the source. |
| 2 | LOW | `state/types.rs:1201` | **Fortify(u32) inconsistent with Equip unit variant.** The `u32` payload is never consumed. **Fix:** Change to unit variant `Fortify` or document why the cost is stored. |
| 3 | LOW | `cards/card_definition.rs:861` | **Field named `equipment` in AttachFortification.** Should be `fortification` for semantic clarity. **Fix:** Rename field to `fortification`. |

### Finding Details

#### Finding 1: CR 301.6 creature-Fortification check missing

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:2088-2093`
**CR Rule**: 301.6 -- "a Fortification that's also a creature (not a land) can't fortify a land."
**Issue**: The resolution handler for `Effect::AttachFortification` comments about CR 301.6 but only checks `equip_id == target_id` (self-attach). It does not check whether the Fortification source has the Creature card type (which can happen via animation effects like March of the Machines). If a Fortification becomes a creature, the fortify ability should fail to attach. The same check is also missing from the activation validation in `abilities.rs:185-231`.

Note: The parallel gap exists in `AttachEquipment` (CR 301.5c: "An Equipment that's also a creature can't equip a creature unless that Equipment has reconfigure"), but that is a pre-existing issue and not introduced by this change.

**Fix**: In `effects/mod.rs` after the `equip_id == target_id` check (line 2093), add a layer-aware check:
```
let source_is_creature = calculate_characteristics(state, equip_id)
    .or_else(|| state.objects.get(&equip_id).map(|o| o.characteristics.clone()))
    .map(|chars| chars.card_types.contains(&CardType::Creature))
    .unwrap_or(false);
if source_is_creature {
    continue; // CR 301.6: creature Fortification can't fortify
}
```
Also add a corresponding check in `abilities.rs` before the target validation (around line 191) so activation is rejected pre-cost-payment.

#### Finding 2: Fortify(u32) inconsistent with Equip

**Severity**: LOW
**File**: `crates/engine/src/state/types.rs:1201`
**CR Rule**: 702.67c -- multiple instances are independent abilities, each with its own cost defined in the `ActivatedAbility`, not the keyword marker.
**Issue**: `KeywordAbility::Fortify(u32)` stores a generic mana cost in the keyword variant, but `KeywordAbility::Equip` is a unit variant. The `u32` is never read outside the hash function. The actual cost is stored in `ActivatedAbility.cost`. This creates an inconsistency and means two Fortify abilities with different costs would produce different keyword values, which may cause issues if code ever checks "does this permanent have Fortify" via set membership.
**Fix**: Either change to `Fortify` (unit variant, matching `Equip`) or add a comment explaining why the cost is stored. The former is preferred for consistency.

#### Finding 3: Field named `equipment` in AttachFortification

**Severity**: LOW
**File**: `crates/engine/src/cards/card_definition.rs:861`
**CR Rule**: 702.67a
**Issue**: The `AttachFortification` effect variant has a field named `equipment` that should be named `fortification`. The doc comment correctly says "The fortification to attach" but the field name is misleading, appearing to be a copy-paste artifact from `AttachEquipment`.
**Fix**: Rename the field from `equipment` to `fortification` in the `Effect::AttachFortification` variant and update all references (hash.rs, effects/mod.rs, tests/fortify.rs).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.67a (fortify is activated, sorcery-speed, target land you control) | Yes | Yes | test_fortify_basic_attaches_to_land, test_fortify_sorcery_speed_only, test_fortify_target_must_be_land, test_fortify_requires_controller_ownership |
| 702.67b (see rule 301) | N/A | N/A | Informational cross-reference |
| 702.67c (multiple instances) | Yes (implicit) | No | Each ActivatedAbility is independent; no test for multiple fortify costs on one card |
| 301.6 (Fortification rules parallel Equipment) | Partial | Partial | Attach/detach/SBA all work; creature-Fortification restriction missing (Finding 1) |
| 301.6 via 301.5c (can't fortify more than one land) | Yes | Yes | test_fortify_moves_between_lands |
| 704.5n (SBA unattach from illegal permanent) | Yes (pre-existing) | Yes | test_fortify_sba_unattaches_from_nonland |
| 301.6 via 301.5d (controller separation) | Yes (implicit) | No | No explicit test, but controller model is shared with Equipment |
| 604.2 / 613 (static abilities apply to fortified land) | Yes | Yes | test_fortify_static_ability_grants_to_land |
| 701.3b (already attached = no-op) | Yes | No | Code handles it but no dedicated test |
| 701.3c (new timestamp on reattach) | Yes | No | Code updates timestamp but no test verifies timestamp changes |

## Previous Findings (re-review only)

N/A -- first review.
