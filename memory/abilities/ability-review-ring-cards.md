# Ability Review: The Ring Tempts You

**Date**: 2026-03-11
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.54
**Files reviewed**:
- `crates/engine/src/state/player.rs` (ring_level, ring_bearer_id fields)
- `crates/engine/src/state/game_object.rs` (Designations::RING_BEARER, ring_block_sacrifice_at_eoc)
- `crates/engine/src/state/hash.rs` (all ring-related hash discriminants)
- `crates/engine/src/state/stack.rs` (StackObjectKind::RingAbility)
- `crates/engine/src/state/stubs.rs` (PendingTriggerKind::RingLoot/RingBlockSacrifice/RingCombatDamage)
- `crates/engine/src/cards/card_definition.rs` (Effect::TheRingTemptsYou, Condition::RingHasTemptedYou, TriggerCondition::WheneverRingTemptsYou)
- `crates/engine/src/rules/engine.rs` (handle_ring_tempts_you, Command handler)
- `crates/engine/src/rules/command.rs` (Command::TheRingTemptsYou)
- `crates/engine/src/effects/mod.rs` (Effect execution, Condition evaluation)
- `crates/engine/src/rules/layers.rs` (ring-bearer Legendary supertype)
- `crates/engine/src/rules/combat.rs` (blocking restriction, ring_block_sacrifice_at_eoc tagging)
- `crates/engine/src/rules/sba.rs` (check_ring_bearer_sba)
- `crates/engine/src/rules/abilities.rs` (check_triggers ring dispatches, flush_pending_triggers ring arms)
- `crates/engine/src/rules/turn_actions.rs` (end_combat ring sacrifice)
- `crates/engine/src/rules/resolution.rs` (RingAbility resolution + counter arm)
- `crates/engine/src/testing/replay_harness.rs` (ring_tempts_you action)
- `tools/tui/src/play/panels/stack_view.rs` (RingAbility SOK arm)
- `tools/replay-viewer/src/view_model.rs` (RingAbility SOK arm)
- `crates/engine/tests/ring_tempts_you.rs` (13 tests)
- `crates/engine/src/cards/defs/call_of_the_ring.rs`

## Verdict: clean

The Ring Tempts You implementation is thorough and correct across all CR 701.54 subrules. The data model (ring_level on PlayerState, ring_bearer_id, Designations::RING_BEARER), effect execution (handle_ring_tempts_you), layer integration (Legendary supertype), combat restrictions (blocking by greater power), trigger dispatch (levels 2-4), SBA enforcement (control change / zone change clearing), and EOC sacrifice (level 3 flag pattern) are all faithfully implemented. Hash coverage is complete. All TUI/replay-viewer match arms are present. Tests cover positive, negative, multiplayer, and edge cases. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `game_object.rs:507` | **ring_block_sacrifice_at_eoc is a bool field, not a Designations bit.** Follows the `decayed_sacrifice_at_eoc` pattern so this is consistent, but conventions.md says new boolean fields should use Designations. Since this is a transient combat flag (not a CR designation), the bool pattern is defensible. **Fix:** No action needed; document in conventions.md that transient combat flags (EOC sacrifice, etc.) are exempt from the Designations bitfield requirement. |
| 2 | LOW | `ring_tempts_you.rs` | **No test for multiple blockers on ring-bearer at level 3.** CR 701.54c says "Whenever your Ring-bearer becomes blocked by a creature, the blocking creature's controller sacrifices it at end of combat." If two creatures block the ring-bearer, both should be tagged. Only one blocker is tested. **Fix:** Add a test with two blockers on the ring-bearer verifying both get `ring_block_sacrifice_at_eoc = true`. |
| 3 | LOW | `ring_tempts_you.rs` | **No test for ring-bearer re-assignment when a second creature enters.** When a player already has a ring-bearer and the ring tempts them again with two creatures, the lowest-ObjectId creature is chosen (which may differ from the current bearer). No test covers the old bearer losing RING_BEARER and the new one gaining it. **Fix:** Add a test where player controls two creatures, ring tempts twice, verify the bearer designation moves correctly if ObjectId ordering changes. |
| 4 | LOW | `combat.rs:1030-1063` | **ring_block_sacrifice_at_eoc tagging is in handle_declare_blockers, not in check_triggers.** The comment at abilities.rs:4217 correctly explains why, but the pattern is split across two files (combat.rs for tagging, turn_actions.rs for execution). A note in abilities.rs could be more prominent. **Fix:** No code change needed; the split is intentional and well-documented. |
| 5 | LOW | `engine.rs:2303` | **handle_ring_tempts_you doc comment says "ring_bearer_id is unchanged" when no creatures, but stale ring_bearer_id from a previous temptation could point to a now-dead creature.** The SBA in sba.rs handles cleanup, so this is not a bug. **Fix:** Clarify the doc comment: "ring_bearer_id may still reference a stale ObjectId if the previous ring-bearer left the battlefield; the SBA in check_ring_bearer_sba clears it." |

### Finding Details

#### Finding 1: ring_block_sacrifice_at_eoc bool vs Designations

**Severity**: LOW
**File**: `crates/engine/src/state/game_object.rs:507`
**CR Rule**: N/A (coding convention)
**Issue**: The conventions.md says "When adding a new boolean designation to `GameObject`, add a new flag to `Designations`." However, `ring_block_sacrifice_at_eoc` is a transient combat flag, not a CR designation. It follows the identical `decayed_sacrifice_at_eoc` pattern. This is correct behavior -- CR designations (renowned, suspected, etc.) belong in Designations; transient combat state flags do not.
**Fix**: No code change. Optionally clarify in conventions.md that transient combat flags are exempt from the Designations requirement.

#### Finding 2: Multiple blockers test gap

**Severity**: LOW
**File**: `crates/engine/tests/ring_tempts_you.rs`
**CR Rule**: 701.54c -- "Whenever your Ring-bearer becomes blocked by a creature, the blocking creature's controller sacrifices it at end of combat."
**Issue**: The "whenever" and "a creature" wording means each blocker independently triggers this. If two creatures block the ring-bearer, both should be tagged for EOC sacrifice. The current test only uses one blocker. The implementation in combat.rs iterates all blockers and correctly handles this case, but there's no test verifying multiple blockers are all tagged.
**Fix**: Add `test_ring_level3_multiple_blockers_all_tagged` with two blockers on the ring-bearer, asserting both have `ring_block_sacrifice_at_eoc = true`.

#### Finding 3: Ring-bearer re-assignment test gap

**Severity**: LOW
**File**: `crates/engine/tests/ring_tempts_you.rs`
**CR Rule**: 701.54a -- "That creature becomes your Ring-bearer until another creature becomes your Ring-bearer"
**Issue**: The test `test_ring_tempts_you_rechoose_same_creature_emits_event` only tests one creature. There is no test where the deterministic lowest-ObjectId selection changes the ring-bearer from one creature to another (e.g., if a new creature with a lower ObjectId enters), verifying the old bearer loses RING_BEARER and the new one gains it.
**Fix**: Add a test with two creatures where the ring-bearer changes on the second temptation (or manually construct the scenario with two creatures with known ObjectId ordering).

#### Finding 4: Split tagging pattern documentation

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:1030-1063`
**CR Rule**: 701.54c
**Issue**: The ring level 3 sacrifice logic is split: tagging in combat.rs, execution in turn_actions.rs, and a "no-op" comment in abilities.rs. All three sites are well-documented but someone unfamiliar with the codebase might look in abilities.rs for the ring level 3 handler and be confused by the "retired" RingBlockSacrifice arm.
**Fix**: No code change needed. The comments are adequate.

#### Finding 5: Stale ring_bearer_id doc comment

**Severity**: LOW
**File**: `crates/engine/src/rules/engine.rs:2303`
**CR Rule**: 701.54a, 400.7
**Issue**: The doc comment on line 2299 says "If no creatures: ring_bearer_id is unchanged (if previously None, stays None)" but doesn't mention the case where ring_bearer_id was previously Some(id) and the creature has since left the battlefield. The SBA handles this correctly, but the doc comment could be misleading.
**Fix**: Amend the comment to note that stale ring_bearer_id values from dead creatures are cleaned up by check_ring_bearer_sba.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.54a (ring tempts, choose creature) | Yes | Yes | test_ring_tempts_you_basic_level_1, test_ring_tempts_you_no_creatures, test_ring_tempts_you_rechoose_same_creature |
| 701.54b (designation, not copiable) | Yes | Yes | RING_BEARER designation verified in basic test; not-copiable is enforced by Designations not being in copiable values (zone change resets) |
| 701.54c level 1 (legendary + blocking) | Yes | Yes | test_ring_bearer_is_legendary, test_ring_bearer_blocking_restriction_greater_power, test_ring_bearer_blocking_equal_power_allowed, test_ring_bearer_blocking_lesser_power_allowed, test_non_ring_bearer_no_blocking_restriction |
| 701.54c level 2 (loot on attack) | Yes | Yes | test_ring_level_2_loot_trigger_fires_on_attack |
| 701.54c level 3 (block sacrifice at EOC) | Yes | Yes | test_ring_level3_sacrifice_at_eoc (flag verified; no test for actual EOC sacrifice, shared with Decayed infra) |
| 701.54c level 4 (combat damage, opponents lose 3) | Yes | Yes | test_ring_level4_combat_damage_trigger_fires |
| 701.54d (whenever ring tempts you trigger) | Yes | Yes | test_whenever_ring_tempts_you_trigger |
| 701.54e (is your Ring-bearer condition) | Yes (Designations check) | Implicit | The `is your Ring-bearer` check is a designation check on battlefield creatures under your control; no explicit test but the SBA and combat restriction tests cover the semantics |
| Control change clears ring-bearer | Yes | Yes | test_ring_bearer_control_change_clears_designation |
| Zone change clears ring-bearer (CR 400.7) | Yes | Yes | test_ring_bearer_leaves_battlefield_clears_designation |
| Multiplayer independence | Yes | Yes | test_ring_tempts_you_multiplayer_independence |
| Cap at 4 | Yes | Yes | test_ring_tempts_you_level_progression_capped_at_4 |
| Provoke impossibility check | Yes | No | Implemented in combat.rs:948-961 but no specific test for provoke + ring-bearer interaction |

## Hash Coverage Check

| Field/Variant | Hashed? | Location |
|--------------|---------|----------|
| PlayerState.ring_level | Yes | hash.rs:1003 |
| PlayerState.ring_bearer_id | Yes | hash.rs:1005 |
| GameObject.ring_block_sacrifice_at_eoc | Yes | hash.rs:882 |
| Designations::RING_BEARER | Yes | Covered by u16 bitflags hash |
| Effect::TheRingTemptsYou | Yes | hash.rs:4027 (disc 51) |
| Condition::RingHasTemptedYou | Yes | hash.rs:3684 (disc 18) |
| TriggerCondition::WheneverRingTemptsYou | Yes | hash.rs:3618 (disc 26) |
| GameEvent::RingTempted | Yes | hash.rs:3331 (disc 117) |
| GameEvent::RingBearerChosen | Yes | hash.rs:3337 (disc 118) |
| StackObjectKind::RingAbility | Yes | hash.rs:2123 (disc 66) |
| PendingTriggerKind::RingLoot | Yes | hash.rs:1473 (disc 47) |
| PendingTriggerKind::RingBlockSacrifice | Yes | hash.rs:1474 (disc 48) |
| PendingTriggerKind::RingCombatDamage | Yes | hash.rs:1475 (disc 49) |

All hash discriminants verified. No gaps.

## Match Arm Coverage

| SOK/Enum | TUI stack_view.rs | replay-viewer view_model.rs | resolution.rs | counter arm |
|----------|-------------------|----------------------------|---------------|-------------|
| RingAbility | Yes (line 150) | Yes (line 542) | Yes (line 7169, resolution) | Yes (line 7405, counter) |

## Previous Review

The file `memory/abilities/ability-review-ring-cards.md` previously contained a review of ring-related card definitions (Call of the Ring, Frodo) from 2026-03-09. That review was clean with 3 LOW findings. This review supersedes it and covers the full ability implementation (engine infrastructure + tests), not just card definitions.
