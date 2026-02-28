# Ability Review: Enchant

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.5, 303.4, 704.5m
**Files reviewed**:
- `crates/engine/src/state/types.rs` (EnchantTarget enum, KeywordAbility::Enchant variant)
- `crates/engine/src/state/hash.rs` (EnchantTarget hash, KeywordAbility hash, AuraAttached event hash)
- `crates/engine/src/state/mod.rs` (EnchantTarget re-export)
- `crates/engine/src/lib.rs` (EnchantTarget public re-export)
- `crates/engine/src/rules/sba.rs` (get_enchant_target, matches_enchant_target, check_aura_sbas)
- `crates/engine/src/rules/casting.rs` (Aura target validation at cast time)
- `crates/engine/src/rules/resolution.rs` (Aura attachment on resolution)
- `crates/engine/src/rules/events.rs` (AuraAttached event variant)
- `crates/engine/src/testing/replay_harness.rs` (enrich_spec_from_def subtype propagation)
- `tools/replay-viewer/src/view_model.rs` (format_keyword match arm)
- `crates/engine/tests/enchant.rs` (8 unit tests)
- `crates/engine/tests/sba.rs` (updated test_cc31 to use EnchantTarget)

## Verdict: needs-fix

The implementation is structurally sound: EnchantTarget enum is well-designed, hash coverage
is complete, the SBA refactor correctly replaces the old `enchants_creatures` boolean with
keyword-based lookup through the layer system, and the resolution attachment ordering is
correct (before register_static_continuous_effects). However, one MEDIUM finding exists:
the Aura cast-time validation does not verify the target is on the battlefield, which means
an Aura with "Enchant creature" could be cast targeting a creature card in the graveyard or
hand. Two additional MEDIUM findings cover missing tests from the plan (fizzle test) and
missing 303.4d self-enchantment SBA check.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `casting.rs:217` | **Aura target not checked for battlefield zone.** Target could be in graveyard/hand. **Fix:** Add zone check. |
| 2 | **MEDIUM** | `enchant.rs` | **Missing fizzle test (CR 608.2b).** Plan specified test_enchant_aura_fizzles_when_target_illegal_at_resolution but it was not implemented. **Fix:** Add fizzle test. |
| 3 | **MEDIUM** | `sba.rs:621` | **CR 303.4d self-enchantment not checked.** An Aura attached to itself is illegal but not detected by check_aura_sbas. **Fix:** Add self-enchant check in SBA. |
| 4 | LOW | `enchant.rs` | **Missing continuous effect test.** Plan specified test_enchant_continuous_effect_on_attached_creature but it was not implemented. |
| 5 | LOW | `casting.rs:207` | **Aura detection uses raw subtypes.** Uses `chars.subtypes.contains(SubType("Aura"))` with a string literal; fragile if Aura subtype representation changes. |
| 6 | LOW | `sba.rs:634` | **Aura detection in SBA uses raw subtypes.** Same fragile string-based check as finding 5; consistent with rest of codebase but worth noting. |
| 7 | LOW | `sba.rs:654` | **Double calculate_characteristics calls for same aura.** Characteristics computed at line 654 (for enchant keyword) and again at line 676 (for protection check). Could share the result. |

### Finding Details

#### Finding 1: Aura target not checked for battlefield zone

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:217-233`
**CR Rule**: 303.4a -- "An Aura spell requires a target, which is defined by its enchant ability." Combined with CR 115.1/115.4, an Aura's target must be a legal permanent on the battlefield (for non-player enchant types).
**Issue**: The Enchant-specific validation at lines 204-235 calls `matches_enchant_target()` which checks the target's card types but not its zone. Meanwhile, `validate_targets()` at line 202 runs with an empty `requirements` list for Auras (since Aura cards don't have `AbilityDefinition::Spell`), so `validate_object_satisfies_requirement()` is never called -- and that function is the one that checks `on_battlefield`. This means an Aura with `Enchant(Creature)` could be cast targeting a creature card in the graveyard or hand. The spell would later fizzle at resolution (since the target zone wouldn't match), but the cast should be rejected outright per CR 303.4a.
**Fix**: In the Enchant-specific validation block (casting.rs ~line 218), after matching `Target::Object(target_id)`, add a zone check: verify the target object is on the battlefield (`state.objects.get(&target_id).map(|o| o.zone == ZoneId::Battlefield).unwrap_or(false)`). For `EnchantTarget::Player`, the target should be `Target::Player` -- object targets are always illegal. Return `InvalidTarget` if the zone check fails.

#### Finding 2: Missing fizzle test (CR 608.2b)

**Severity**: MEDIUM
**File**: `crates/engine/tests/enchant.rs`
**CR Rule**: 608.2b -- "If, when a spell or ability tries to resolve, all its targets are illegal, the spell or ability doesn't resolve."
**Issue**: The ability plan (Step 7, test 3) specified `test_enchant_aura_fizzles_when_target_illegal_at_resolution`: "Cast an Aura targeting a creature, kill the creature before resolution, verify the Aura fizzles (SpellFizzled event) and goes to graveyard (not battlefield)." This test was not implemented. While the general fizzle mechanism is tested elsewhere, the Aura-specific interaction (where the Aura spell targets a creature that dies before resolution) is an important edge case that exercises the cast-then-fizzle path for permanent spells with targets.
**Fix**: Add test `test_702_5_aura_fizzles_when_target_killed`. Setup: cast Aura targeting creature. Before passing priority to resolve, move the creature to graveyard (simulating it being killed). Pass priority for all players. Assert: SpellFizzled event emitted; Aura is in graveyard (not battlefield); no AuraAttached event.

#### Finding 3: CR 303.4d self-enchantment not checked in SBA

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/sba.rs:621`
**CR Rule**: 303.4d -- "An Aura can't enchant itself. If this occurs somehow, the Aura is put into its owner's graveyard."
**Issue**: The `check_aura_sbas` function checks: (1) unattached, (2) target gone, (3) enchant restriction mismatch, (4) protection. It does NOT check whether an Aura is attached to itself (`obj.attached_to == Some(*aura_id)`). While this state is unlikely to arise naturally, the CR explicitly states it's an SBA. An Aura that is also an Enchantment with `Enchant(Enchantment)` attached to itself would pass all current checks.
**Fix**: In `check_aura_sbas`, after the `match obj.attached_to` block (inside the `Some(target_id)` arm), add: `if target_id == **aura_id { return true; }` -- or equivalently, at the start of the `Some(target_id)` arm, check `target_id == **aura_id` and return true (illegal). Add a corresponding test.

#### Finding 4: Missing continuous effect test

**Severity**: LOW
**File**: `crates/engine/tests/enchant.rs`
**CR Rule**: 702.5 / 613 -- Aura static abilities apply continuous effects to enchanted creature.
**Issue**: The plan (Step 7, test 8) specified `test_enchant_continuous_effect_on_attached_creature`: "Aura with a static ability (e.g., Pacifism) grants the effect to the enchanted creature. Verify layer-computed characteristics of the attached creature reflect the Aura's effect." This test was not implemented. It would validate the end-to-end flow: cast Aura -> resolve -> attach -> static effect registered -> layer calculation shows modified characteristics.
**Fix**: Add this test after a card definition with a static ability (e.g., an Aura granting +1/+1) is available. Deferred until card definition phase.

#### Finding 5: Aura detection uses raw string subtypes

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:207`
**CR Rule**: N/A (code quality)
**Issue**: The check `chars.subtypes.contains(&SubType("Aura".to_string()))` allocates a new String on every call. This is consistent with the rest of the codebase (Equipment checks do the same), so it's not a divergence, but it's a fragile pattern.
**Fix**: No action required now. If a global constant `AURA_SUBTYPE` is introduced later, update all call sites.

#### Finding 6: Aura detection in SBA uses raw string subtypes

**Severity**: LOW
**File**: `crates/engine/src/rules/sba.rs:634`
**CR Rule**: N/A (code quality)
**Issue**: Same as finding 5. The SBA check at line 634 uses `obj.characteristics.subtypes.contains(&SubType("Aura".to_string()))`. Consistent with codebase conventions.
**Fix**: Same as finding 5 -- address when constants are introduced.

#### Finding 7: Redundant calculate_characteristics calls in SBA

**Severity**: LOW
**File**: `crates/engine/src/rules/sba.rs:654,676`
**CR Rule**: N/A (performance)
**Issue**: `calculate_characteristics(state, **aura_id)` is called at line 654 (to get keywords for enchant restriction) and again at line 676 (to get characteristics for protection check). The result should be computed once and reused.
**Fix**: Hoist the `calculate_characteristics(state, **aura_id)` call before the enchant check, store in a local `aura_chars`, and use `aura_chars.as_ref().map(|c| c.keywords.clone())` for the keyword extraction and `&aura_chars` for the protection check.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.5a (Enchant restricts target/attachment) | Yes | Yes | Tests 1, 2, 3, 7 |
| 702.5b (see rule 303) | N/A | N/A | Reference only |
| 702.5c (multiple Enchant instances) | Partial | No | get_enchant_target returns first only; rare edge case, plan notes "not relevant for MVP" |
| 702.5d (Enchant player) | Partial | No | EnchantTarget::Player exists; matches_enchant_target returns false for objects; no player attachment yet |
| 303.4a (Aura requires target) | Yes | Yes | Test 8 (no-target rejection); Tests 1-3 (valid/invalid targets) |
| 303.4b (attached_to set on resolution) | Yes | Yes | Test 4 (attachment + AuraAttached event) |
| 303.4c (illegal attachment -> graveyard SBA) | Yes | Yes | Tests 5, 6 (unattached + type mismatch) |
| 303.4d (can't enchant itself) | No | No | **Finding 3: SBA missing self-enchant check** |
| 303.4e (Aura controller separate) | N/A | N/A | Implicitly handled by existing controller logic |
| 303.4f (non-cast ETB) | No | No | Documented as deferred in plan |
| 303.4g (no legal target on ETB) | No | No | Documented as deferred in plan |
| 303.4i (illegal ETB attachment) | No | No | Documented as deferred in plan |
| 303.4j (illegal re-attachment) | No | No | Documented as deferred in plan |
| 704.5m (Aura SBA) | Yes | Yes | Tests 5, 6; existing sba.rs test_cc31 updated |
| 608.2b (fizzle) | Yes (general) | No (Aura-specific) | **Finding 2: Missing fizzle test** |
