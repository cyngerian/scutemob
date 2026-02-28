# Ability Review: Living Weapon

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.92
**Files reviewed**:
- `crates/engine/src/state/types.rs:352-360` (KeywordAbility::LivingWeapon)
- `crates/engine/src/state/hash.rs:384-385` (hash discriminant 47)
- `crates/engine/src/state/hash.rs:2325-2329` (Effect::CreateTokenAndAttachSource hash discriminant 34)
- `crates/engine/src/cards/card_definition.rs:416-426` (Effect::CreateTokenAndAttachSource variant)
- `crates/engine/src/effects/mod.rs:348-413` (CreateTokenAndAttachSource execution)
- `crates/engine/src/state/builder.rs:582-616` (trigger wiring)
- `crates/engine/tests/living_weapon.rs` (6 tests, 719 lines)
- `tools/replay-viewer/src/view_model.rs:614` (display name)

## Verdict: clean

The Living Weapon implementation is correct and well-structured. The new
`CreateTokenAndAttachSource` effect variant elegantly solves the atomicity requirement
(create + attach before SBAs). The trigger wiring in builder.rs follows established
patterns (Afterlife, Extort). All six tests cover the key CR rules and known rulings.
Hash coverage is complete for both the keyword enum variant and the new effect variant.
The view_model.rs display arm is present. No HIGH or MEDIUM findings. Three LOW findings
identified, two of which are pre-existing codebase-wide conventions.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `builder.rs:597` | **Token name omits "Token" suffix.** Per CR 111.4, name should be "Phyrexian Germ Token." Pre-existing convention; consistent with Spirit token (Afterlife) and Treasure token. **Fix:** Codebase-wide decision needed; defer. |
| 2 | LOW | `tests/living_weapon.rs:298-367` | **Germ characteristics test is fragile.** The object may be removed by SBA before assertions run; the `else` branch only checks events, not characteristics. **Fix:** Use the buff-equipped test (test 3) pattern to verify characteristics on a surviving Germ, or snapshot state before SBA pass. |
| 3 | LOW | `effects/mod.rs:380-384` | **Source Equipment type not verified.** The effect checks `source_on_bf` but does not verify the source is actually an Equipment. If a non-Equipment somehow had LivingWeapon, the attach would proceed. **Fix:** Add `obj.characteristics.subtypes.contains(&SubType("Equipment"))` check. Defensive; extremely unlikely in practice. |

### Finding Details

#### Finding 1: Token name omits "Token" suffix

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs:597`
**CR Rule**: 111.4 -- "If the spell or ability doesn't specify the name of the token, its name is the same as its subtype(s) plus the word 'Token.'"
**Issue**: The Germ token is named "Phyrexian Germ" but CR 111.4 says the name should be "Phyrexian Germ Token" since the oracle text of Living Weapon doesn't explicitly specify a name (it says "create a 0/0 black Phyrexian Germ creature token"). This is a pre-existing codebase-wide convention: the Afterlife Spirit token is named "Spirit" (not "Spirit Token"), and the Treasure token is named "Treasure" (not "Treasure Token").
**Fix**: This is a codebase-wide naming convention decision. If the team decides to follow CR 111.4 strictly, all token names need the "Token" suffix. Defer to a sweep pass. Not specific to Living Weapon.

#### Finding 2: Germ characteristics test fragility

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/living_weapon.rs:298-367`
**CR Rule**: 702.92a -- token characteristics
**Issue**: Test 2 (`test_living_weapon_germ_has_correct_characteristics`) attempts to verify the Germ token's characteristics (power, toughness, color, subtypes, is_token) by looking up the object after trigger resolution. However, the 0/0 Germ may have already been killed by SBA and removed from `state.objects` (since tokens cease to exist after leaving the battlefield per CR 704.5d). The test has an `else` branch that only verifies `CreatureDied` event presence, which doesn't actually verify characteristics. In practice, Test 3 (`test_living_weapon_germ_survives_with_equipment_buff`) provides stronger characteristic verification since the Germ survives and can be inspected directly. The combined coverage is adequate, but Test 2 alone is fragile.
**Fix**: Consider adding a `+0/+1` buff in Test 2 so the Germ survives long enough for characteristic inspection, or accept that Test 3 provides the definitive characteristic coverage and simplify Test 2 to focus only on event emission.

#### Finding 3: Source Equipment type not verified in effect

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs:380-384`
**CR Rule**: 702.92a -- "When this Equipment enters"
**Issue**: The `CreateTokenAndAttachSource` effect checks whether the source is on the battlefield (`source_on_bf`) before attaching, but does not verify that the source has the Equipment subtype. The AttachEquipment effect (lines 1375-1402) performs a layer-aware creature type check on the target but not an Equipment type check on the source (it relies on the caller having already validated). In the Living Weapon path, the caller is the trigger wiring in builder.rs which only fires for Equipment with the LivingWeapon keyword, so the source is always an Equipment. However, a defensive check would be more robust.
**Fix**: Add an Equipment subtype check to the `source_on_bf` guard: `&& o.characteristics.subtypes.contains(&SubType("Equipment".to_string()))`. Low priority since the trigger system ensures only Equipment can fire this effect.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.92a (triggered ability definition) | Yes | Yes | Test 1: trigger fires on ETB, goes on stack |
| 702.92a (create 0/0 black Phyrexian Germ) | Yes | Yes | Test 2: characteristics verified (fragile), Test 3: Germ survives and is inspected |
| 702.92a (then attach this Equipment to it) | Yes | Yes | Test 1: EquipmentAttached event, Test 3: attached_to verified |
| 702.92a (atomic create+attach before SBAs) | Yes | Yes | Test 3: Germ survives with buff (proves attachment before SBA), Test 4: 0/0 Germ dies after trigger resolves (proves SBA fires after) |
| Batterskull ruling (Germ survives with buff) | Yes | Yes | Test 3: +0/+4 buff, Germ is 0/4, survives |
| Batterskull ruling (Equipment stays after Germ dies) | Yes | Yes | Test 4: Equipment on battlefield, attached_to is None |
| Batterskull ruling (re-equip, Germ dies) | Yes | Yes | Test 5: Equip to Bear, Germ dies to SBA |
| Batterskull ruling (Doubling Season) | Yes (comment) | No | Only count=1 tested; Doubling Season replacement effect not yet in engine. Acceptable. |
| CR 301.5b (Equipment enters unattached) | Yes | Yes | Test 1: Equipment enters first, trigger goes on stack separately |
| CR 704.5f (0 toughness dies to SBA) | Yes | Yes | Test 1, Test 4: Germ with 0 toughness dies |
| CR 704.5n (Equipment unattaches from illegal permanent) | Yes (via SBA system) | Yes | Test 4: Equipment.attached_to is None after Germ dies |
| CR 603.3 (multiplayer: one trigger per ETB) | Yes | Yes | Test 6: 4-player game, exactly 1 trigger |
| Hash coverage (KeywordAbility::LivingWeapon) | Yes | N/A | Discriminant 47 in hash.rs:384 |
| Hash coverage (Effect::CreateTokenAndAttachSource) | Yes | N/A | Discriminant 34 in hash.rs:2326 |
| view_model.rs display | Yes | N/A | "Living Weapon" at line 614 |
