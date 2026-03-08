# Ability Review: Gift

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.174
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KW variant, line 1286-1304)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Gift, GiftType, Condition::GiftWasGiven)
- `crates/engine/src/state/game_object.rs` (gift_was_given, gift_opponent fields)
- `crates/engine/src/state/stack.rs` (gift_was_given, gift_opponent on StackObject; GiftETBTrigger SOK)
- `crates/engine/src/state/hash.rs` (all hash arms)
- `crates/engine/src/state/mod.rs` (zone-change resets)
- `crates/engine/src/state/builder.rs` (init false/None)
- `crates/engine/src/state/stubs.rs` (PendingTriggerKind::GiftETB, gift_opponent field)
- `crates/engine/src/rules/casting.rs` (gift_opponent validation)
- `crates/engine/src/rules/resolution.rs` (inline gift for instants/sorceries, GiftETBTrigger resolution, execute_gift_effect)
- `crates/engine/src/rules/abilities.rs` (flush_pending_triggers GiftETB arm)
- `crates/engine/src/rules/engine.rs` (CastSpell routing)
- `crates/engine/src/effects/mod.rs` (EffectContext fields, Condition::GiftWasGiven)
- `tools/replay-viewer/src/view_model.rs` (GiftETBTrigger + Gift arms)
- `tools/tui/src/play/panels/stack_view.rs` (GiftETBTrigger arm)
- `crates/engine/tests/gift.rs` (8 tests)
- `crates/engine/src/cards/helpers.rs` (missing GiftType export)

## Verdict: needs-fix

One HIGH finding (missing hash coverage for StackObject gift fields) and one MEDIUM
finding (inline gift effect not gated to instants/sorceries). The rest of the implementation
is clean: CR correctness is good, multiplayer handling is correct, trigger wiring follows
the Offspring pattern properly, and tests cover the key scenarios.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `hash.rs:2131` | **Missing StackObject hash for gift fields.** `gift_was_given` and `gift_opponent` on StackObject are not hashed. **Fix:** Add hash lines after `offspring_paid`. |
| 2 | MEDIUM | `resolution.rs:337` | **Inline gift effect not gated to !is_permanent.** Could double-gift if permanent has Spell ability. **Fix:** Add `!is_permanent` guard. |
| 3 | LOW | `helpers.rs:15-19` | **GiftType not exported from helpers.rs.** Card defs must import from `cards::card_definition` directly. **Fix:** Add `GiftType` to helpers re-exports. |
| 4 | LOW | `gift.rs` | **No test for countered spell (CR 702.174j).** Plan listed `test_gift_countered_no_effect` but it was not implemented. |
| 5 | LOW | `gift.rs` | **No test for GiftType::Food.** Food token creation is implemented but not tested. |

### Finding Details

#### Finding 1: Missing StackObject hash for gift fields

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:2131`
**CR Rule**: Architecture invariant (hash coverage)
**Issue**: The `HashInto for StackObject` implementation (lines 2050-2134) hashes all fields
through `offspring_paid` (line 2131) but does NOT hash `gift_was_given` or `gift_opponent`,
which are the two fields added to StackObject at lines 303-311 of `stack.rs`. The
`HashInto for GameObject` implementation at lines 886-888 does hash these fields correctly,
and the `HashInto for PendingTrigger` at line 1375 hashes `gift_opponent`. Only the
StackObject hash is missing.

Missing hash fields break loop detection (CR 104.4b) because two game states that differ
only in gift status on a stack object would produce identical hashes.

**Fix**: In `crates/engine/src/state/hash.rs`, after line 2131 (`self.offspring_paid.hash_into(hasher);`),
add:
```rust
// Gift (CR 702.174a) -- whether gift cost was paid and who was chosen
self.gift_was_given.hash_into(hasher);
self.gift_opponent.hash_into(hasher);
```

#### Finding 2: Inline gift effect not gated to instants/sorceries

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:337`
**CR Rule**: 702.174b -- "On a permanent, the second ability represented by gift is 'When
this permanent enters, if its gift cost was paid, [effect].'" / 702.174j -- "For instant
and sorcery spells, the effect of a gift ability always happens before any other spell
abilities of the card."
**Issue**: The inline gift effect at line 337 fires for ANY spell with `gift_was_given == true`
that has an `AbilityDefinition::Spell` effect. This block is not gated by `!is_permanent`.
Per CR 702.174b, permanents should deliver the gift exclusively through their ETB trigger,
not inline at spell resolution time.

Currently, the test creature card definitions do not include `AbilityDefinition::Spell`,
so this code path is not exercised for permanents in practice. But if a future card
definition for a permanent includes both `AbilityDefinition::Gift` and
`AbilityDefinition::Spell` (e.g., a creature whose ETB is modeled as a Spell effect),
the gift would fire twice: once inline and once via GiftETBTrigger.

**Fix**: At line 337, change:
```rust
if stack_obj.gift_was_given {
```
to:
```rust
if stack_obj.gift_was_given && !is_permanent {
```
The `is_permanent` variable is already in scope (defined at line 143).

#### Finding 3: GiftType not exported from helpers.rs

**Severity**: LOW
**File**: `crates/engine/src/cards/helpers.rs:15-19`
**CR Rule**: N/A (coding convention -- helpers.rs prelude pattern)
**Issue**: `GiftType` is not included in the `pub use super::card_definition::{...}` block.
Card definitions that use `use crate::cards::helpers::*;` will not have `GiftType` in scope
and will need a separate import.
**Fix**: Add `GiftType` to the re-export list at line 16-19 of helpers.rs.

#### Finding 4: Missing test for countered spell

**Severity**: LOW
**File**: `crates/engine/tests/gift.rs`
**CR Rule**: 702.174j -- "If the spell is countered or otherwise leaves the stack before
resolving, the gift effect doesn't happen."
**Issue**: The plan listed `test_gift_countered_no_effect` as a test to write, but it was
not implemented. The 8 tests cover positive/negative/multiplayer/validation but not the
countering scenario. This is inherently handled by the implementation (gift effect fires
at resolution time, not cast time), but an explicit test would increase confidence.
**Fix**: Add a test that casts a Gift spell with gift paid, then counters it (e.g., via
another spell), and asserts the opponent did not receive the gift.

#### Finding 5: Missing test for Food token gift

**Severity**: LOW
**File**: `crates/engine/tests/gift.rs`
**CR Rule**: 702.174d -- "The chosen player creates a Food token."
**Issue**: `execute_gift_effect` handles `GiftType::Food` (line 6143-6156 of resolution.rs)
with actual token creation logic, but no test exercises this path. The existing tests cover
`GiftType::Card` and `GiftType::Treasure` only.
**Fix**: Add a test using a card definition with `GiftType::Food` and assert the opponent
receives a Food token on the battlefield.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.174a (choose opponent) | Yes | Yes | Casting validation in casting.rs:2005-2028 |
| 702.174b (permanent ETB) | Yes | Yes | test_gift_permanent_etb_trigger |
| 702.174b (instant/sorcery inline) | Yes | Yes | test_gift_basic_instant_card_draw |
| 702.174c (gives-a-gift trigger) | No (deferred) | No | No cards in scope |
| 702.174d (Food) | Yes | No | Finding 5 |
| 702.174e (Card) | Yes | Yes | test_gift_basic_instant_card_draw |
| 702.174f (TappedFish) | Stub | No | Deferred, no-op |
| 702.174g (ExtraTurn) | Stub | No | Deferred, no-op |
| 702.174h (Treasure) | Yes | Yes | test_gift_permanent_etb_trigger, test_gift_multiplayer |
| 702.174i (Octopus) | Stub | No | Deferred, no-op |
| 702.174j (before other effects) | Yes | Partial | test_gift_instant_both_effects_resolve verifies both effects, but does not verify ordering via event sequence |
| 702.174j (countered = no gift) | Yes (inherent) | No | Finding 4 |
| 702.174k (promised) | N/A | N/A | Terminology rule, no separate enforcement needed |
| 702.174m (conditional targets) | N/A | N/A | Targeting optimization, deferred |

## Previous Findings (re-review only)

N/A -- first review.
