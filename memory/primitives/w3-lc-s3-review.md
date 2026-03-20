# W3-LC S3 Review: MEDIUM Layer Correctness Fixes

**Date**: 2026-03-19
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 613.1d (Layer 4 type changes), 613.1f (Layer 6 ability add/remove), 702.11 (Hexproof), 702.18 (Shroud), 702.16 (Protection), 702.25 (Flanking), 702.43 (Modular), 702.95 (Soulbond)
**Files reviewed**:
- `crates/engine/src/rules/abilities.rs` (lines 210-265, 710-740, 3335-3490, 4305-4350, 6385-6420, 6555-6585)
- `crates/engine/src/rules/resolution.rs` (lines 1785-1810, 1925-1980, 2065-2125, 3670-3695, 5170-5195)
- `crates/engine/src/rules/casting.rs` (lines 5170-5200)
- `crates/engine/src/rules/engine.rs` (lines 2250-2275)

## Verdict: needs-fix

The 17 classified sites are all fixed correctly. The layer-resolved pattern is applied
consistently, the fallback is defensive and safe, and CR citations are present at every
changed site. The resolution.rs namespace alignment (S2 Finding 1) is properly addressed.
However, the S3 Soulbond fix only covers Trigger 2 (OtherETB) while Trigger 1 (SelfETB)
retains two base-characteristic reads that were missed during the S1 audit classification.
One MEDIUM finding, two LOW observations.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `abilities.rs:3357-3371` | **Soulbond Trigger 1 entering_has_soulbond uses base OR layer.** Over-permissive: fires under Humility. **Fix:** Replace with layer-resolved only. |
| 2 | MEDIUM | `abilities.rs:3385-3388` | **Soulbond Trigger 1 pair_target uses base card_types.** Animated non-creature won't be found. **Fix:** Use layer-resolved types. |
| 3 | LOW | `engine.rs:2260` | **Ring-bearer selection missing is_phased_in() check.** Pre-existing, not introduced by S3. |

### Finding Details

#### Finding 1: Soulbond entering_has_soulbond uses base OR layer-resolved

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:3356-3371`
**CR Rule**: 613.1f -- Layer 6 ability-adding/removing effects; 702.95a -- Soulbond
**Issue**: The `entering_has_soulbond` variable computes `base || layer`, which is
overly permissive. If a creature has printed Soulbond but Humility (or Dress Down)
removes it via Layer 6, the `base` check returns true, short-circuiting the correct
`layer` check. The trigger fires when it should not.

The S1 audit classified only lines 3444-3446 (Trigger 2 / OtherETB path) as
needs-layer-calc, missing the Trigger 1 / SelfETB path at 3356-3371.

**Fix**: Replace lines 3356-3371 with a single layer-resolved check:
```rust
let entering_has_soulbond =
    crate::rules::layers::calculate_characteristics(state, *object_id)
        .or_else(|| state.objects.get(object_id).map(|o| o.characteristics.clone()))
        .map(|c| c.keywords.contains(&KeywordAbility::Soulbond))
        .unwrap_or(false);
```

#### Finding 2: Soulbond Trigger 1 pair_target uses base card_types

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:3385-3388`
**CR Rule**: 613.1d -- Layer 4 type-changing effects
**Issue**: The `pair_target` scan for Trigger 1 (SoulbondSelfETB) checks
`obj.characteristics.card_types.contains(&CardType::Creature)` -- base types.
An animated permanent (e.g., a Mutavault that is currently a creature via its
activated ability) would not be found as a valid pairing target. Similarly, a
creature that has lost its creature type through a type-changing effect would
still be found.

This site was also missed during S1 audit classification.

**Fix**: Replace lines 3385-3388 with:
```rust
&& {
    let chars = crate::rules::layers::calculate_characteristics(state, obj.id)
        .unwrap_or_else(|| obj.characteristics.clone());
    chars.card_types.contains(&CardType::Creature)
}
```

#### Finding 3: Ring-bearer selection missing is_phased_in()

**Severity**: LOW
**File**: `crates/engine/src/rules/engine.rs:2260`
**CR Rule**: 702.26d -- Phased-out permanents are treated as though they don't exist
**Issue**: The ring-bearer creature selection at line 2259-2267 checks
`obj.zone == ZoneId::Battlefield` and `obj.controller == player` but does not
filter out phased-out permanents with `obj.is_phased_in()`. A phased-out creature
could be selected as ring-bearer. This is a pre-existing bug, not introduced by S3.

**Fix**: Add `&& obj.is_phased_in()` to the filter at line 2260 (after the
`obj.zone == ZoneId::Battlefield` check). Can be addressed in S4 or later since
phasing + ring temptation is extremely uncommon.

## CR Coverage Check

All S3 sites address the same core issue: base-characteristic reads on battlefield
permanents bypass the layer system. The CR rules are:

| CR Rule | Sites Fixed | Correctly Applied? |
|---------|------------|-------------------|
| 613.1d (Layer 4 type changes) | Cipher, Modular (x2), Ninjutsu, Ring-bearer, Soulbond Trigger 2 | Yes |
| 613.1f (Layer 6 ability add/remove) | Activated abilities (x2), Hexproof/shroud/protection (x3), Flanking (x2), Triggered ability resolution (x3) | Yes |
| 702.95 (Soulbond) Trigger 2 | OtherETB creature + keyword check | Yes |
| 702.95 (Soulbond) Trigger 1 | SelfETB entering_has_soulbond + pair_target | **No** (missed, Findings 1-2) |

## S2 Finding Resolution

| # | S2 Finding | S3 Status | Notes |
|---|-----------|-----------|-------|
| 1 | MEDIUM: ability_index namespace mismatch (resolution.rs:1939,2068,2098) | **RESOLVED** | All 3 sites now use layer-resolved triggered_abilities with proper fallback to card registry |
| 2 | LOW: PowerOf/ToughnessOf tests don't exercise resolve_amount | DEFERRED | Not in S3 scope (effects/mod.rs is S4) |
| 3 | LOW: Humility trigger suppression test doesn't exercise collect_triggers_for_event | DEFERRED | Acceptable -- layer_correctness.rs tests cover the layer system; integration gap is low risk |

## Additional Observations (not findings)

1. **Activated ability error handling is correct.** Returning `GameStateError::InvalidCommand`
   when layer-resolved ability index is out of bounds is the right behavior -- it prevents
   activating abilities that have been removed by Humility. However, the simulator's
   `legal_actions.rs:332` still uses base `obj.characteristics.activated_abilities` to
   enumerate legal actions, which would present removed abilities as legal. This is a
   simulator-level consistency issue, not an engine rules bug.

2. **Double calculate_characteristics at abilities.rs:222+246.** The same object gets
   layer-resolved twice (sorcery-speed check and cost/effect clone). This is because
   the first block drops `obj` before the second block. Functionally correct but slightly
   wasteful. Not a finding -- correctness over performance per the audit plan.

3. **Fallback pattern is consistently applied.** All 17 sites use
   `calculate_characteristics(state, id).unwrap_or_else(|| obj.characteristics.clone())`.
   Since the object exists on the battlefield (confirmed by prior zone check), the
   `calculate_characteristics` call should always return `Some(...)`. The fallback is
   defensive dead code, which is appropriate.

4. **No new tests in S3.** The S3 fixes are mechanical (same pattern applied to 17 sites).
   The S2 review's layer_correctness.rs tests cover the core layer resolution behavior.
   Specific interaction tests (e.g., "Humility prevents Flanking trigger") would be
   valuable but are not strictly required for these one-line pattern fixes.
