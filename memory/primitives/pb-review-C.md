# Primitive Batch Review: PB-C -- Extra Turns

**Date**: 2026-04-05
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 500.7 (extra turn creation, LIFO ordering), 702.174g (Gift an extra turn), 608.2n (spell resolution destination), 614.1a (replacement effects)
**Engine files reviewed**: `cards/card_definition.rs`, `effects/mod.rs`, `state/hash.rs`, `rules/resolution.rs`
**Card defs reviewed**: 5 (nexus_of_fate.rs, temporal_trespass.rs, temporal_mastery.rs, teferi_master_of_time.rs, emrakul_the_promised_end.rs)

## Verdict: needs-fix

One MEDIUM finding: Nexus of Fate's "from anywhere" graveyard replacement is only
implemented for the resolution case (`self_shuffle_on_resolution`), not for discard,
mill, or other zone transitions. The card def comment acknowledges this but the
replacement effect from the oracle text is not present at all. One LOW finding for a
minor test coverage gap. The engine changes (Effect::ExtraTurn, hash, GiftType wiring,
self_exile/self_shuffle flags) are all correct per CR rules.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| (none) | -- | -- | Engine changes are clean. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | MEDIUM | `nexus_of_fate.rs` | **Incomplete replacement effect.** Oracle says "from anywhere", only resolution case handled. **Fix:** add a TODO comment noting the gap, or implement `AbilityDefinition::Replacement` for non-resolution zone changes. |
| 2 | LOW | `teferi_master_of_time.rs` | **Missing -3 loyalty ability placeholder.** The -3 is documented in TODO comments but no `LoyaltyAbility` stub exists (only +1 and -10). Not blocking. |

## Test Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| 3 | LOW | `extra_turns.rs` | **No eliminated-player extra turn test.** If an eliminated player has an extra turn queued, `advance_turn()` should skip them. No test covers this edge case. |

### Finding Details

#### Finding 1: Nexus of Fate -- Incomplete "from anywhere" replacement

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/nexus_of_fate.rs:6-10`
**Oracle**: "If Nexus of Fate would be put into a graveyard from anywhere, reveal Nexus of Fate and shuffle it into its owner's library instead."
**CR Rule**: 614.1a -- "Effects that use the word 'instead' are replacement effects."
**Issue**: The oracle text specifies a replacement effect that applies whenever Nexus of Fate
would be put into a graveyard from ANY zone (hand via discard, library via mill, stack via
countering, battlefield, etc.). The current implementation only handles the resolution case
via `self_shuffle_on_resolution: true`. If Nexus of Fate is countered, discarded, or milled,
it will incorrectly go to the graveyard. The file header comment mentions this limitation but
the card def does not include any replacement ability -- neither an `AbilityDefinition::Replacement`
nor a TODO in the abilities vec.
**Fix**: Add a TODO comment inside the `abilities` vec explicitly noting that a self-replacement
`AbilityDefinition::Replacement` for "from anywhere" is needed once the non-permanent replacement
infrastructure supports it. Example:
```rust
// TODO: CR 614.1a — "from anywhere" graveyard replacement (discard, mill, counter)
// needs full replacement infrastructure for non-permanents. Only resolution case
// is handled via self_shuffle_on_resolution flag.
```
This makes the gap discoverable by card audits without requiring immediate engine work.

#### Finding 2: Teferi -3 loyalty ability not present as stub

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/teferi_master_of_time.rs:36-37`
**Oracle**: "-3: Target creature you don't control phases out."
**Issue**: The -3 ability is mentioned in a comment between the +1 and -10 abilities but no
`LoyaltyAbility` stub is defined for it. This means `activate_loyalty_ability` with index 1
activates the -10 instead of failing or doing nothing for -3. The loyalty ability indices are
off from what a player would expect (+1 = index 0, -10 = index 1, but oracle has +1, -3, -10
as indices 0, 1, 2).
**Fix**: No fix required for engine correctness (the card works, just missing -3). But for
index correctness, consider adding a placeholder `LoyaltyAbility` for -3 with `Effect::Nothing`
and a TODO comment noting PhaseOut is needed. This ensures ability index 1 maps to -3 and
index 2 maps to -10, matching oracle order.

#### Finding 3: No eliminated-player extra turn skip test

**Severity**: LOW
**File**: `crates/engine/tests/extra_turns.rs`
**CR Rule**: 500.7 + 800.4a -- eliminated players skip turns
**Issue**: The plan identified "Gift extra turn + eliminated player" as an edge case worth
testing. If a player is eliminated after being granted an extra turn, `advance_turn()` should
skip that player's extra turn. No test verifies this behavior. The existing `advance_turn()`
may or may not handle this correctly -- untested.
**Fix**: Add a test `test_extra_turn_eliminated_player_skipped` that grants an extra turn to
a player, eliminates them, then verifies the extra turn is skipped (or popped and not taken).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 500.7 (extra turn creation) | Yes | Yes | test_effect_extra_turn_basic, test_effect_extra_turn_two_turns |
| 500.7 (LIFO ordering) | Yes (pre-existing) | Yes | test_extra_turns_lifo, test_multiple_extra_turns_stack |
| 500.7 (resumption) | Yes (pre-existing) | Yes | test_extra_turn_normal_order_resumes |
| 702.174g (Gift extra turn) | Yes | Yes | test_gift_extra_turn |
| 608.2n (self-exile on resolution) | Yes | Yes | test_self_exile_on_resolution |
| 614.1a (Nexus "from anywhere") | Partial | Partial | Only resolution case via flag; discard/mill/counter not covered |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| nexus_of_fate.rs | Partial | 0 (should be 1) | Partial | "From anywhere" replacement only covers resolution |
| temporal_trespass.rs | Yes | 0 | Yes | Delve + ExtraTurn + self-exile all correct |
| temporal_mastery.rs | Yes | 0 | Yes | Miracle + ExtraTurn + self-exile all correct |
| teferi_master_of_time.rs | Partial | 3 | Partial | +1 draw-only (no discard), -3 missing, -10 correct |
| emrakul_the_promised_end.rs | Partial | 2 | Partial | Cast trigger blocked on player-control; TODOs correctly documented |

## Mass-Update Spot Check

The 131 card defs mass-updated with `self_exile_on_resolution: false` and
`self_shuffle_on_resolution: false` were spot-checked (flowerfoot_swordmaster.rs,
blitz_automaton.rs, beloved_beggar.rs, slickshot_show_off.rs, zurgo_bellstriker.rs,
fyndhorn_elves.rs, llanowar_elves.rs, young_wolf.rs, and others). All correctly set
both fields to `false`. Only the three PB-C cards (nexus_of_fate, temporal_trespass,
temporal_mastery) have `true` values, which is correct. No issues found in the mass update.
