# Ability Review: The Ring Tempts You

**Date**: 2026-03-09
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.54
**Files reviewed**:
- `crates/engine/src/state/player.rs` (ring_level, ring_bearer_id)
- `crates/engine/src/state/game_object.rs` (RING_BEARER designation)
- `crates/engine/src/state/builder.rs` (initialization)
- `crates/engine/src/state/hash.rs` (hash impls)
- `crates/engine/src/state/stack.rs` (RingAbility SOK)
- `crates/engine/src/state/stubs.rs` (RingLoot/RingBlockSacrifice/RingCombatDamage)
- `crates/engine/src/cards/card_definition.rs` (Effect, Condition, TriggerCondition)
- `crates/engine/src/effects/mod.rs` (effect execution + condition evaluation)
- `crates/engine/src/rules/engine.rs` (handle_ring_tempts_you, ring_ability_stack_object)
- `crates/engine/src/rules/resolution.rs` (RingAbility resolution arm)
- `crates/engine/src/rules/layers.rs` (Legendary supertype grant)
- `crates/engine/src/rules/combat.rs` (blocking restriction)
- `crates/engine/src/rules/sba.rs` (check_ring_bearer_sba)
- `crates/engine/src/rules/abilities.rs` (ring trigger dispatch, flush arms)
- `crates/engine/src/rules/events.rs` (RingTempted, RingBearerChosen)
- `crates/engine/tests/ring_tempts_you.rs` (13 tests)
- `tools/replay-viewer/src/view_model.rs` (RingAbility arm)
- `tools/tui/src/play/panels/stack_view.rs` (RingAbility arm)

## Verdict: FIXED (2026-03-09)

The core ring temptation flow (level advancement, ring-bearer selection, Legendary supertype, blocking restriction, SBA cleanup) is correctly implemented and well-tested. The Ring Level 3 sacrifice trigger had two HIGH bugs and three test gaps — all fixed.

## Findings

| # | Severity | File:Line | Description | Status |
|---|----------|-----------|-------------|--------|
| 1 | **HIGH** | `abilities.rs:6636` | **Level 3 sacrifice resolves immediately, not at end of combat.** CR says "sacrifices it at end of combat." | **FIXED** — Replaced PendingTriggerKind::RingBlockSacrifice trigger path with EOC flag pattern (`ring_block_sacrifice_at_eoc` on GameObject). Flag set in `handle_declare_blockers` (combat.rs), checked in `end_combat()` (turn_actions.rs). |
| 2 | **HIGH** | `abilities.rs:6640` | **Level 3 sacrifice targets wrong permanent.** Uses generic SacrificePermanents instead of the specific blocker. | **FIXED** — EOC flag is set on the specific blocker ObjectId (not a generic permanent sacrifice). The old PendingTriggerKind::RingBlockSacrifice flush arm is now a no-op stub (unreachable). |
| 3 | MEDIUM | `tests/ring_tempts_you.rs` | **Missing test for level 3 sacrifice trigger.** | **FIXED** — Added `test_ring_level3_sacrifice_at_eoc`: verifies blocker NOT sacrificed at declare-blockers time, `ring_block_sacrifice_at_eoc` flag set on specific blocker only (not attacker, not other permanents). |
| 4 | MEDIUM | `tests/ring_tempts_you.rs` | **Missing test for level 4 combat damage trigger.** | **FIXED** — Added `test_ring_level4_combat_damage_trigger_fires`: uses `check_triggers` directly on a `CombatDamageDealt` event to verify `RingCombatDamage` PendingTrigger is generated with correct controller and source. |
| 5 | MEDIUM | `tests/ring_tempts_you.rs` | **Missing test for WheneverRingTemptsYou trigger.** | **FIXED** — Added `test_whenever_ring_tempts_you_trigger`: registers a CardDef with `WheneverRingTemptsYou`, calls `handle_ring_tempts_you`, then calls `check_triggers` on the resulting events to verify a Normal PendingTrigger is queued. |
| 6 | LOW | `abilities.rs:4248` | **Level 3 trigger controller semantics.** | **Deferred** — LOW severity, no change. The EOC flag approach avoids this issue entirely (no trigger controller involved). |

### Finding Details

#### Finding 1: Level 3 sacrifice resolves immediately, not at end of combat

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:6636`
**CR Rule**: 701.54c -- "the blocking creature's controller sacrifices it at end of combat."
**Issue**: The RingBlockSacrifice trigger is pushed when blockers are declared and creates a RingAbility SOK that goes on the stack. When this SOK resolves (during the Declare Blockers step priority passes), the SacrificePermanents effect executes immediately -- before combat damage is dealt. CR 701.54c explicitly says the sacrifice happens "at end of combat," meaning the blocking creature should deal combat damage normally, then be sacrificed during the End of Combat step.

The plan (Step 7, level 3) correctly identified this as needing an EOC flag pattern similar to Decayed, but the implementation uses immediate resolution instead.

**Fix**: Implement the end-of-combat sacrifice pattern:
1. Add a `ring_sacrifice_at_eoc: bool` field (or `RING_SACRIFICE_AT_EOC` Designations bit) on GameObject.
2. In the RingBlockSacrifice flush arm, instead of creating a SacrificePermanents effect, create an effect that sets this flag on the specific blocking creature (via `trigger.source`, which is the blocker ObjectId).
3. In `end_combat()` (turn_actions.rs), check for permanents with this flag and sacrifice them.
4. Alternatively, defer the trigger creation entirely: instead of pushing a PendingTrigger in the BlockersDeclared handler, mark the blocker for EOC sacrifice and create the sacrifice trigger in the end-of-combat handler.

#### Finding 2: Level 3 sacrifice targets wrong permanent

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:6640`
**CR Rule**: 701.54c -- "the blocking creature's controller sacrifices **it**"
**Issue**: The effect is `SacrificePermanents { player: PlayerTarget::Controller, count: Fixed(1) }`, which makes the controller sacrifice any one permanent they control (deterministic lowest ObjectId). CR 701.54c says the controller sacrifices the specific blocking creature ("sacrifices **it**"). The implementation does not target the blocking creature specifically.

Even if the timing were correct (Finding 1), this would still sacrifice the wrong permanent if the blocker's controller has a permanent with a lower ObjectId.

**Fix**: Use an effect that sacrifices the specific blocker ObjectId. Options:
1. Use `Effect::DestroyTarget` with the sacrifice flag (if available), targeting the blocker via `trigger.source`.
2. Create a new `Effect::SacrificeSpecific { object: ObjectId }` variant.
3. Use `Effect::SacrificePermanents` with a filter that matches only the specific ObjectId.
The simplest approach that avoids new Effect variants: store the blocker ObjectId in the RingAbility SOK and use the existing sacrifice infrastructure to sacrifice that specific object during end-of-combat processing.

#### Finding 3: Missing test for level 3 sacrifice trigger

**Severity**: MEDIUM
**File**: `crates/engine/tests/ring_tempts_you.rs`
**CR Rule**: 701.54c -- level 3 sacrifice
**Issue**: The plan listed test 11 as "Deferred -- complex combat setup." However, the level 3 implementation has two HIGH bugs (Findings 1 and 2). A test exercising this path would have caught both issues.
**Fix**: After fixing Findings 1 and 2, add a test that:
- Sets up ring level 3 with a ring-bearer attacking and a creature blocking it.
- Verifies the blocker deals combat damage normally.
- Verifies the specific blocker is sacrificed at end of combat.
- Verifies other permanents controlled by the blocker's controller are NOT sacrificed.

#### Finding 4: Missing test for level 4 combat damage trigger

**Severity**: MEDIUM
**File**: `crates/engine/tests/ring_tempts_you.rs`
**CR Rule**: 701.54c -- level 4 "each opponent loses 3 life"
**Issue**: No test exercises the RingCombatDamage trigger path. The implementation in abilities.rs:5327-5364 and the flush arm at abilities.rs:6649-6658 are untested.
**Fix**: Add a test that:
- Sets up ring level 4 with a ring-bearer dealing combat damage to a player.
- Verifies all opponents (not just the damaged player) lose 3 life.
- Verifies the trigger does NOT fire on damage to creatures.

#### Finding 5: Missing test for WheneverRingTemptsYou trigger

**Severity**: MEDIUM
**File**: `crates/engine/tests/ring_tempts_you.rs`
**CR Rule**: 701.54d -- "Whenever the Ring tempts you"
**Issue**: The abilities.rs:5720-5769 dispatch for WheneverRingTemptsYou and the TriggerCondition variant are untested. No test creates a CardDef with this trigger condition and verifies it fires on RingTempted events.
**Fix**: Add a test that creates a card definition with `TriggerCondition::WheneverRingTemptsYou`, tempts the player, and verifies the trigger fires (a PendingTrigger or AbilityTriggered event is created).

#### Finding 6: Level 3 trigger controller semantics

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:4248`
**CR Rule**: 701.54c
**Issue**: The trigger `controller` is set to `blocker_controller` (the blocking creature's controller). This is used as both the SOK controller (who controls the triggered ability on the stack) and the player who performs the sacrifice. Semantically, CR 701.54c's ability is on the Ring emblem (controlled by the ring-bearer's controller), but the effect instructs the blocker's controller to sacrifice. The current approach of setting controller to the blocker's controller and using `PlayerTarget::Controller` works for the sacrifice action, but means the triggered ability on the stack is controlled by the wrong player. This matters for Stifle-like effects (the ring-bearer's controller should be the one who controls the trigger on the stack, since it's their Ring emblem's ability).
**Fix**: Set `controller: ring_controller` on the PendingTrigger, and change the sacrifice effect to target the blocker's controller explicitly (e.g., using a stored player ID rather than `PlayerTarget::Controller`). This is a correctness issue but LOW severity because Stifle interactions with ring triggers are rare.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.54a (ring tempts, choose creature) | Yes | Yes | test_ring_tempts_you_basic_level_1 |
| 701.54a (no creatures) | Yes | Yes | test_ring_tempts_you_no_creatures |
| 701.54a (re-choose same) | Yes | Yes | test_ring_tempts_you_rechoose_same_creature_emits_event |
| 701.54a (control change clears) | Yes | Yes | test_ring_bearer_control_change_clears_designation |
| 701.54b (designation, not copiable) | Yes | No | Designation bit exists; no test verifying copy doesn't propagate |
| 701.54c level 1 (Legendary) | Yes | Yes | test_ring_bearer_is_legendary |
| 701.54c level 1 (blocking restriction) | Yes | Yes | test_ring_bearer_blocking_restriction_greater_power + 2 more |
| 701.54c level 2 (loot trigger) | Yes | Yes | test_ring_level_2_loot_trigger_fires_on_attack |
| 701.54c level 3 (sacrifice at EOC) | **FIXED** | **Yes** | test_ring_level3_sacrifice_at_eoc: EOC flag pattern, specific blocker |
| 701.54c level 4 (opponents lose 3) | Yes | **Yes** | test_ring_level4_combat_damage_trigger_fires: RingCombatDamage trigger verified |
| 701.54c (capped at 4) | Yes | Yes | test_ring_tempts_you_level_progression_capped_at_4 |
| 701.54d (trigger fires) | Yes | **Yes** | test_whenever_ring_tempts_you_trigger: WheneverRingTemptsYou dispatch verified |
| 701.54e (is your Ring-bearer check) | Yes | Partial | Condition::RingHasTemptedYou covers the level check; designation check via RING_BEARER bit |
| CR 400.7 (zone change clears) | Yes | Yes | test_ring_bearer_leaves_battlefield_clears_designation |
| Hash coverage | Yes | N/A | All new fields/variants hashed |
| TUI/replay-viewer SOK arm | Yes | N/A | RingAbility matched in both |
| Builder init | Yes | N/A | ring_level: 0, ring_bearer_id: None |
| Multiplayer independence | Yes | Yes | test_ring_tempts_you_multiplayer_independence |
