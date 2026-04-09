# Primitive Batch Review: PB-E -- Mana Doubling

**Date**: 2026-04-07
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 605.1b, 605.4, 605.4a, 605.5a, 106.12, 106.12a, 106.12b, 106.6a
**Engine files reviewed**: `crates/engine/src/rules/mana.rs`, `crates/engine/src/cards/card_definition.rs`, `crates/engine/src/state/replacement_effect.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/effects/mod.rs`, `crates/engine/src/rules/replacement.rs`, `crates/engine/src/cards/helpers.rs`
**Card defs reviewed**: 9 (6 fixes + 3 new)

## Verdict: needs-fix

One HIGH finding (Forbidden Orchard PendingTrigger wrong ability_index), two MEDIUM findings (stale TODOs in card defs, missing Forbidden Orchard test), and two LOW findings (TargetRequirement::TargetPlayer instead of TargetOpponent, Zendikar Resurgent draw trigger not integration-tested). The engine core for mana triggers and mana multiplication is correctly implemented per CR 605.1b/605.4a/106.12/106.12b. The Nyxbloom ordering (multiplier before triggers) is correct. Card defs for Mirari's Wake, Crypt Ghast, Wild Growth, Leyline of Abundance, Badgermole Cub, Nyxbloom Ancient, Mana Reflection, and Zendikar Resurgent are all correct against oracle text. Hash support is complete.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `mana.rs:351-352` | **Forbidden Orchard PendingTrigger uses wrong ability_index.** PendingTrigger::blank sets ability_index=0, but the Spirit token trigger is at index 1 in Forbidden Orchard's abilities vec. **Fix:** track the ability index during iteration and pass it to the PendingTrigger. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | MEDIUM | `crypt_ghast.rs` | **Stale TODO comment.** Lines 5-6 say mana trigger is "not expressible" but it IS now implemented. **Fix:** remove the TODO comment block at lines 4-6. |
| 3 | MEDIUM | `wild_growth.rs` | **Stale TODO comment.** Lines 5-6 say mana trigger on enchanted land is "not expressible" but it IS now implemented. **Fix:** remove the TODO comment block at lines 5-6. |
| 4 | LOW | `forbidden_orchard.rs:55` | **TargetRequirement::TargetPlayer should be TargetOpponent.** Oracle says "target opponent" not "target player". There is no TargetOpponent variant in the enum currently. **Fix:** deferred -- add TargetOpponent variant when target validation is implemented; document as known DSL gap in the TODO comment. |

### Finding Details

#### Finding 1: Forbidden Orchard PendingTrigger uses wrong ability_index

**Severity**: HIGH
**File**: `crates/engine/src/rules/mana.rs:351-352`
**CR Rule**: 605.5a -- "An ability with a target is not a mana ability... These follow the normal rules for... triggered abilities, as appropriate."
**Oracle**: Forbidden Orchard: "Whenever you tap this land for mana, target opponent creates a 1/1 colorless Spirit creature token."
**Issue**: When `fire_mana_triggered_abilities` detects a triggered ability with targets (the Forbidden Orchard case), it pushes a `PendingTrigger::blank(trigger_source_id, player, PendingTriggerKind::Normal)`. The `blank()` constructor sets `ability_index: 0`. However, Forbidden Orchard's abilities are: index 0 = `AbilityDefinition::Activated` (mana ability), index 1 = `AbilityDefinition::Triggered` (Spirit token trigger). When this PendingTrigger resolves from the stack, `resolution.rs` looks up `ability_index: 0` in the card definition, which is the mana ability, NOT the Spirit token trigger. The wrong effect will execute (or fail silently).
**Fix**: In the iteration loop at line 321 (`for ability in &def.abilities`), track the index with `.enumerate()`. When pushing the PendingTrigger for a non-mana trigger, set `ability_index` to the correct value:
```rust
for (ability_idx, ability) in def.abilities.iter().enumerate() {
    // ... existing match logic ...
    } else {
        let mut trigger = PendingTrigger::blank(trigger_source_id, player, PendingTriggerKind::Normal);
        trigger.ability_index = ability_idx;
        state.pending_triggers.push_back(trigger);
    }
}
```

#### Finding 2: Stale TODO in crypt_ghast.rs

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/crypt_ghast.rs:4-6`
**Issue**: The comment says "mana doubling for a specific land subtype needs a trigger on mana ability resolution. Not expressible." This is now expressible and implemented on lines 22-34. The stale TODO gives the false impression the feature is missing.
**Fix**: Remove lines 4-6 (the TODO block).

#### Finding 3: Stale TODO in wild_growth.rs

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/wild_growth.rs:5-6`
**Issue**: The comment says "mana trigger on enchanted land is not expressible. Needs a trigger that fires when the attached land produces mana." This is now expressible via `ManaSourceFilter::EnchantedLand` and implemented on lines 21-31.
**Fix**: Remove lines 5-6 (the TODO block).

#### Finding 4: Forbidden Orchard target validation

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/forbidden_orchard.rs:55`
**Oracle**: "target opponent creates a 1/1 colorless Spirit creature token"
**Issue**: Uses `TargetRequirement::TargetPlayer` but oracle text says "target opponent". No `TargetOpponent` variant exists in the enum.
**Fix**: Deferred -- add `TargetOpponent` variant when target validation is fully implemented. Add a note to the existing TODO comment that the target should be "opponent" not "any player".

## Test Findings

| # | Severity | Description |
|---|----------|-------------|
| 5 | MEDIUM | **Missing Forbidden Orchard test.** Plan specified `test_mana_trigger_this_permanent` to verify the Spirit token trigger goes on the stack (not immediate). Not implemented. This would have caught Finding 1. **Fix:** add test after Finding 1 is fixed. |
| 6 | LOW | **Zendikar Resurgent draw trigger not integration-tested.** Test 9 only checks triggered_abilities count + land mana trigger, not that casting a creature actually draws a card. Plan specified `test_zendikar_resurgent_creature_cast_draw`. **Fix:** add a test that casts a creature with Zendikar Resurgent on battlefield and verifies a card is drawn. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 605.1b  | Yes | Yes | Tests 1-4, 10 (triggered mana ability criteria) |
| 605.4a  | Yes | Yes | Tests 1-4 (immediate resolution, no stack) |
| 605.5a  | Yes | **No** | Forbidden Orchard (targets = not mana ability) -- no test; Finding 5 |
| 106.12  | Yes | Yes | Test 10 (requires_tap guard) |
| 106.12a | Yes | Yes | Tests 1-4 (trigger fires on mana production) |
| 106.12b | Yes | Yes | Tests 5-8 (replacement effect, multiplication) |
| 106.6a  | Yes | Yes | Test 7 (multiplicative stacking) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Mirari's Wake | Yes | 0 | Yes | |
| Crypt Ghast | Yes | 1 (stale) | Yes | Finding 2: stale TODO |
| Wild Growth | Yes | 1 (stale) | Yes | Finding 3: stale TODO |
| Leyline of Abundance | Yes | 1 (opening hand) | Yes | Opening hand TODO is a separate gap |
| Badgermole Cub | Yes | 1 (earthbend) | Yes | Earthbend TODO is a separate gap |
| Forbidden Orchard | Yes (token beneficiary approx) | 1 (token-for-opponent) | No* | Finding 1: wrong ability_index; Finding 4: TargetPlayer vs TargetOpponent; *token goes to controller not opponent (documented DSL gap) |
| Nyxbloom Ancient | Yes | 0 | Yes | |
| Mana Reflection | Yes | 0 | Yes | |
| Zendikar Resurgent | Yes | 0 | Yes | Both abilities present |
