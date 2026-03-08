# Ability Review: Cipher

**Date**: 2026-03-08
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.99
**Files reviewed**: `crates/engine/src/rules/resolution.rs` (lines 1620-1700, 4155-4262), `crates/engine/src/rules/abilities.rs` (lines 4885-4955, 5950-5970), `crates/engine/src/state/types.rs` (KeywordAbility::Cipher), `crates/engine/src/state/stack.rs` (CipherTrigger), `crates/engine/src/state/stubs.rs` (CipherCombatDamage + cipher fields), `crates/engine/src/state/game_object.rs` (encoded_cards), `crates/engine/src/state/hash.rs` (all cipher hashes), `crates/engine/src/state/mod.rs` (move_object_to_zone reset), `crates/engine/src/rules/events.rs` (CipherEncoded), `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`, `crates/engine/tests/cipher.rs`

## Verdict: needs-fix

One HIGH finding (missing PendingTrigger hash fields) and one MEDIUM finding (aftermath override not excluded from cipher check). The core encoding and trigger logic is correct. Combat damage dispatch is properly scoped to player damage only. The copy-not-encodable check works correctly via the `is_copy` early exit at resolution.rs:438. Zone-change reset of `encoded_cards` is correct (CR 400.7). Multiplayer controller tracking is correct (trigger uses creature's controller at trigger time, per CR 603.3a). Tests cover the main cases well, though two plan tests are missing. Seven of nine planned tests were implemented.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `hash.rs:1389` | **Missing hash: cipher fields on PendingTrigger.** `cipher_encoded_card_id` and `cipher_encoded_object_id` not in HashInto. **Fix:** add both after `gift_opponent`. |
| 2 | MEDIUM | `resolution.rs:1636` | **Aftermath not excluded from cipher check.** `cast_with_aftermath` has "exile instead" semantics (CR 702.127a) but cipher encoding is not blocked. **Fix:** add `&& !stack_obj.cast_with_aftermath` to the `has_cipher` guard. |
| 3 | LOW | `resolution.rs:4166-4167` | **Unused CipherTrigger fields.** `source_creature` and `encoded_card_id` are bound as `_` and never used at resolution time. Dead data. |
| 4 | LOW | `stubs.rs:398-409` | **Pre-existing hash gap.** `champion_filter`, `champion_exiled_card`, `soulbond_pair_target`, `squad_count` on PendingTrigger are also not hashed (B10+ pattern). Cipher extends this gap. |
| 5 | LOW | `tests/cipher.rs` | **Two planned tests missing.** `test_cipher_flashback_overrides_cipher` (plan item 5) and `test_cipher_controller_change` (plan item 7) were not implemented. |

### Finding Details

#### Finding 1: Missing hash: cipher fields on PendingTrigger

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:1389`
**CR Rule**: Architecture invariant: all new fields must be hashed.
**Issue**: The `PendingTrigger` HashInto impl ends at line 1389 with `self.gift_opponent.hash_into(hasher)`. The two new cipher fields (`cipher_encoded_card_id` and `cipher_encoded_object_id`) are not hashed. Two PendingTriggers with different encoded cards (same source creature, same kind) would produce identical hashes. This matters for the multiple-encoded-cards case (test at line 880).
**Fix**: Add the following after line 1389 in hash.rs, before the closing `}`:
```rust
// CR 702.99a: cipher-specific fields
self.cipher_encoded_card_id.hash_into(hasher);
self.cipher_encoded_object_id.hash_into(hasher);
```

#### Finding 2: Aftermath not excluded from cipher check

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:1636`
**CR Rule**: 702.127a -- "If this spell was cast from a graveyard, exile it instead of putting it anywhere else any time it would leave the stack."
**Issue**: The `has_cipher` guard at line 1635-1650 checks `!stack_obj.is_copy && !stack_obj.cast_with_flashback && !stack_obj.cast_with_jump_start` but does NOT check `!stack_obj.cast_with_aftermath`. CR 702.127a uses the same "exile instead of putting it anywhere else" language as flashback (CR 702.34a) and jump-start (CR 702.133a). If a hypothetical spell had both cipher and aftermath, casting the aftermath half from graveyard should exile it via the aftermath clause, not encode it via cipher. No current cards combine both keywords, so impact is theoretical. Note: the destination logic at line 1668 also lacks `cast_with_aftermath` -- this is a pre-existing aftermath resolution bug (separate from cipher).
**Fix**: Add `&& !stack_obj.cast_with_aftermath` to the `has_cipher` guard at line 1636.

#### Finding 3: Unused CipherTrigger fields

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:4166-4167`
**Issue**: `source_creature` and `encoded_card_id` on `CipherTrigger` are bound as `_` in the resolution match arm. The `encoded_card_id` is redundant because the copy spell uses `source_object: encoded_object_id` and the card definition is looked up from the source object's `card_id`. The `source_creature` could be useful for future "creature still on battlefield" checks but is currently unused. Not a correctness issue.
**Fix**: No immediate fix needed. Document as intentional (future use) or remove if unneeded.

#### Finding 4: Pre-existing PendingTrigger hash gap

**Severity**: LOW
**File**: `crates/engine/src/state/stubs.rs:365-390`
**Issue**: Fields added to PendingTrigger in Batch 10 (`champion_filter`, `champion_exiled_card`, `soulbond_pair_target`, `squad_count`) are also not individually hashed. Cipher extends this pre-existing gap. All trigger-kind-specific fields from B10 onward should be added to the hash, but this is a broader remediation item.
**Fix**: Deferred to W3 LOW remediation. Track as a single issue covering all un-hashed PendingTrigger fields from B10+.

#### Finding 5: Missing planned tests

**Severity**: LOW
**File**: `crates/engine/tests/cipher.rs`
**Issue**: The plan specified 9 tests but only 7 were implemented. Missing: (a) `test_cipher_flashback_overrides_cipher` -- verifies flashback exile takes priority over cipher encoding. (b) `test_cipher_controller_change` -- verifies that if creature changes controller, new controller controls the trigger. Both are edge cases covered by the code logic but untested.
**Fix**: Add both tests in a future pass. The flashback test is especially valuable since it validates the `cast_with_flashback` guard that is critical for correctness.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.99a (spell ability: encoding at resolution) | Yes | Yes | test_cipher_basic_encode_on_creature |
| 702.99a (static ability: trigger on combat damage) | Yes | Yes | test_cipher_combat_damage_triggers_copy |
| 702.99a ("represented by a card" -- no copies) | Yes | Yes | test_cipher_copy_is_not_encodable |
| 702.99a ("you may" -- optional, no creature) | Yes | Yes | test_cipher_no_creatures_goes_to_graveyard |
| 702.99a (copy is cast, not just created) | Yes | Yes | SpellCast event checked in combat trigger test |
| 702.99b (term "encoded") | Yes | Yes | encoded_cards field verified in basic test |
| 702.99c (stays encoded while in exile + on battlefield) | Yes | Yes | test_cipher_creature_leaves_encoding_broken |
| 702.99c (encoding broken if card leaves exile) | Yes | Partial | Checked at resolution time; no dedicated test for card-removed-from-exile scenario |
| Ruling: flashback overrides cipher | Yes | No | Missing test_cipher_flashback_overrides_cipher |
| Ruling: controller change | Partial | No | Controller at trigger time is correct (CR 603.3a); missing test_cipher_controller_change |
| Ruling: multiple encoded cards | Yes | Yes | test_cipher_multiple_encoded_cards_fire_separate_triggers |
| Ruling: blocked creature, no trigger | Yes | Yes | test_cipher_no_combat_damage_no_trigger |
| CR 702.127a: aftermath overrides cipher | No | No | Finding 2 |

## Previous Findings (re-review only)

N/A -- first review.
