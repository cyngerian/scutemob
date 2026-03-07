# Ability Review: Bloodrush

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 207.2c (ability word; underlying mechanics CR 602, CR 608.2b, CR 115)
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs:579-600` (AbilityDefinition::Bloodrush disc 52)
- `crates/engine/src/state/stack.rs:1085-1109` (StackObjectKind::BloodrushAbility disc 51)
- `crates/engine/src/rules/command.rs:444-457` (Command::ActivateBloodrush)
- `crates/engine/src/rules/engine.rs:271-290` (dispatch arm)
- `crates/engine/src/rules/abilities.rs:970-1256` (handle_activate_bloodrush)
- `crates/engine/src/rules/resolution.rs:3359-3470` (BloodrushAbility resolution)
- `crates/engine/src/rules/resolution.rs:5492` (countered arm)
- `crates/engine/src/state/hash.rs:1953-1972, 3911-3925` (hash arms)
- `crates/engine/src/testing/replay_harness.rs:626-647` (activate_bloodrush action)
- `tools/replay-viewer/src/view_model.rs:579-581` (SOK arm)
- `tools/tui/src/play/panels/stack_view.rs:189-191` (SOK arm)
- `crates/engine/tests/bloodrush.rs` (8 tests)

## Verdict: needs-fix

One MEDIUM finding: the handler does not emit `PermanentTargeted` for the target creature, so Ward will not trigger when bloodrush targets a creature with Ward. All other aspects of the implementation are correct and well-structured.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `abilities.rs:1248` | **Missing PermanentTargeted event for Ward.** The handler emits `AbilityActivated` and `PriorityGiven` but never emits `PermanentTargeted` for the target creature, so Ward triggers will not fire. **Fix:** Emit `GameEvent::PermanentTargeted { target_id: target, targeting_stack_id: stack_id, targeting_controller: player }` after pushing the stack object (before the `AbilityActivated` event). |
| 2 | LOW | `abilities.rs:1198` | **Dead ObjectId used as source_object.** `source_object: card` stores the pre-discard ObjectId (dead per CR 400.7). `new_grave_id` is the live ID. The comment says "for attribution only" which is acceptable, but using the graveyard ID would be more consistent with CR 400.7. No functional impact since `ContinuousEffect.source` is not used for game state lookups. |
| 3 | LOW | `bloodrush.rs` | **Missing fizzle test.** Plan listed `test_bloodrush_countered_no_pump` (counter with Stifle) and `test_bloodrush_split_second_blocks` but neither was implemented. The 8 tests cover the core scenarios well, but the countered/fizzle case is untested. |

### Finding Details

#### Finding 1: Missing PermanentTargeted event for Ward

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:1248`
**CR Rule**: 702.21a -- "Whenever [this creature] becomes the target of a spell or ability an opponent controls..."
**Issue**: The `handle_activate_bloodrush` function targets a creature on the battlefield but does not emit `GameEvent::PermanentTargeted`. The engine dispatch at `engine.rs:283` calls `check_triggers` on the emitted events, but since `PermanentTargeted` is absent, Ward triggers for the target creature will never fire. The generic `handle_activate_ability` (abilities.rs:460-466) correctly emits this event for all battlefield targets. Bloodrush must do the same.
**Fix**: After pushing the stack object (line 1241) and before the `AbilityActivated` event (line 1248), add:
```rust
events.push(GameEvent::PermanentTargeted {
    target_id: target,
    targeting_stack_id: stack_id,
    targeting_controller: player,
});
```

#### Finding 2: Dead ObjectId used as source_object

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:1198`
**CR Rule**: 400.7 -- "An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence."
**Issue**: `source_object: card` in the `BloodrushAbility` stack kind stores the pre-discard ObjectId. After `move_object_to_zone`, the live ID is `new_grave_id`. The dead ID is used at resolution (resolution.rs:3409) as `source: Some(source_object)` in continuous effects. This is for attribution only and has no functional impact, but is technically inconsistent with CR 400.7.
**Fix**: Change `source_object: card` to `source_object: new_grave_id` (line 1198). Update the comment accordingly.

#### Finding 3: Missing fizzle and split-second tests

**Severity**: LOW
**File**: `crates/engine/tests/bloodrush.rs`
**CR Rule**: CR 608.2b (fizzle), CR 702.61a (split second)
**Issue**: The plan specified 10 tests but only 8 were implemented. Missing: (a) `test_bloodrush_countered_no_pump` -- activate bloodrush, counter it with Stifle, verify card stays in graveyard and creature has no pump; (b) `test_bloodrush_split_second_blocks` -- verify bloodrush fails when split second is on the stack. The split second check IS implemented in the handler (line 1003) but untested. The fizzle path in resolution (lines 3463-3464) is also untested.
**Fix**: Add both tests. For the fizzle test, set up bloodrush, resolve partially (remove target from combat/battlefield before resolution passes), verify no pump. For split second, put a split-second spell on the stack and verify `ActivateBloodrush` returns an error.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 207.2c (ability word) | Yes | Yes | Correctly treated as ability word, not keyword |
| 602.2 (activation) | Yes | Yes | test_bloodrush_basic_pump |
| 602.2a (hand zone, reveal) | Yes | Yes | test_bloodrush_not_in_hand_fails |
| 602.2b (cost payment) | Yes | Yes | test_bloodrush_card_discarded_as_cost, test_bloodrush_insufficient_mana_fails |
| 115 (target attacking) | Yes | Yes | test_bloodrush_target_must_be_attacking, test_bloodrush_no_combat_fails |
| 608.2b (resolution recheck) | Yes | No | Implemented in resolution.rs:3383-3397 but no test for fizzle case |
| 702.61a (split second) | Yes | No | Implemented in abilities.rs:1003 but no test |
| 514.2 (end of turn expiry) | Yes | Yes | test_bloodrush_pump_expires_end_of_turn |
| 702.35a (Madness interaction) | Yes | No | Madness path at abilities.rs:1119-1187; no test (acceptable -- no real bloodrush+madness card) |
| 702.21a (Ward) | No | No | Missing PermanentTargeted event (Finding 1) |

## Previous Findings (re-review only)

N/A -- first review.
