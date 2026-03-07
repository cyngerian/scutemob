# Ability Review: Ravenous

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.156
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KW variant, line 1241-1246)
- `crates/engine/src/state/hash.rs` (KW hash disc 135 line 661; GO hash line 869; SO hash line 2054; SOK hash disc 50 lines 1944-1952)
- `crates/engine/src/state/stack.rs` (x_value field line 280-283; RavenousDrawTrigger SOK disc 50 lines 1068-1082)
- `crates/engine/src/state/game_object.rs` (x_value field lines 613-616)
- `crates/engine/src/state/stubs.rs` (PendingTriggerKind::RavenousDraw lines 112-117)
- `crates/engine/src/state/mod.rs` (x_value: 0 on zone-change, lines 374-375 and 529-530)
- `crates/engine/src/state/builder.rs` (x_value: 0 on test objects, line 990-991)
- `crates/engine/src/rules/command.rs` (CastSpell x_value field, line 287)
- `crates/engine/src/rules/engine.rs` (x_value passthrough, lines 107-138)
- `crates/engine/src/rules/casting.rs` (x_value on mana cost, lines 2675-2686; SO construction, line 3086)
- `crates/engine/src/rules/resolution.rs` (counter placement lines 1145-1181; trigger queuing lines 1183-1220; draw resolution lines 3330-3359; fizzle arm line 5380)
- `crates/engine/src/rules/abilities.rs` (flush pending trigger, lines 5277-5290)
- `crates/engine/src/rules/copy.rs` (x_value propagation, line 240)
- `crates/engine/src/rules/lands.rs` (comment placeholder, line 369)
- `crates/engine/src/effects/mod.rs` (EffectContext x_value field lines 78-81; XValue resolution line 2603; token x_value: 0 line 2818)
- `crates/engine/src/testing/replay_harness.rs` (x_value field line 288; propagation line 351)
- `tools/replay-viewer/src/view_model.rs` (SOK arm line 576-578; KW display line 858)
- `tools/tui/src/play/panels/stack_view.rs` (SOK arm lines 186-188)
- `crates/engine/tests/ravenous.rs` (5 tests)

## Verdict: needs-fix

One MEDIUM finding: the draw trigger resolution incorrectly requires the Ravenous permanent to still be on the battlefield. Per CR 603.4 and CR 702.156a, the intervening-if condition is only "if X is 5 or more" -- once the trigger is on the stack, it resolves and draws a card regardless of whether the source permanent is still around. Triggered abilities do not fizzle for lack of source (only for lack of legal targets, which this ability has none). This blocks correct interaction with removal in response to the draw trigger.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `resolution.rs:3340-3346` | **Incorrect battlefield check on draw trigger resolution.** The trigger should draw even if the Ravenous creature left the battlefield. **Fix:** remove the `permanent_on_battlefield` check. |
| 2 | LOW | `resolution.rs:3350` | **draw_card error silently swallowed.** The `if let Ok(...)` pattern silently drops errors from `draw_card`. **Fix:** match on result and handle Err (or use `unwrap_or_default` with a comment). |
| 3 | LOW | `ravenous.rs` | **Missing test: creature removed before draw trigger resolves.** No test covers the case where the Ravenous creature is destroyed (e.g., by another effect) while the draw trigger is on the stack. **Fix:** add a test that removes the creature after ETB but before draw trigger resolution, verifying the draw still happens. |

### Finding Details

#### Finding 1: Incorrect battlefield check on draw trigger resolution

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:3340-3346`
**CR Rule**: 603.4 -- "If the ability triggers, it checks the stated condition again as it resolves. If the condition isn't true at that time, the ability is removed from the stack and does nothing."
**CR Rule**: 702.156a -- "When this permanent enters, if X is 5 or more, draw a card."
**Issue**: The resolution arm checks `permanent_on_battlefield` and skips the draw if the Ravenous creature has left the battlefield. However, the intervening-if condition in CR 702.156a is solely "if X is 5 or more." The draw trigger is not targeted and does not require the source to exist. Triggered abilities only fizzle for lack of legal targets (CR 608.2b), and this ability has no targets. If a player casts a Ravenous creature with X=6 and the opponent destroys it in response to the draw trigger, the controller should still draw a card. The current code incorrectly prevents this.

The `x_value >= 5` check is the correct intervening-if re-check. The `permanent_on_battlefield` check is an extra condition that contradicts the CR.

**Fix**: Remove lines 3340-3344 (the `permanent_on_battlefield` variable) and change line 3346 from `if x_value >= 5 && permanent_on_battlefield {` to `if x_value >= 5 {`.

#### Finding 2: draw_card error silently swallowed

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:3350`
**CR Rule**: N/A (code quality)
**Issue**: `if let Ok(drawn_events) = crate::rules::turn_actions::draw_card(state, controller)` silently drops any `Err` from `draw_card`. While this may be acceptable (draw_card errors are rare), other resolution arms in this file tend to propagate or at least log errors.
**Fix**: Add a comment explaining why Err is dropped, or match both arms. Low priority since draw_card failing is a degenerate case (player already lost).

#### Finding 3: Missing test for creature removed before draw resolves

**Severity**: LOW
**File**: `crates/engine/tests/ravenous.rs`
**CR Rule**: 603.4 -- intervening-if only checks "if X is 5 or more", not creature presence
**Issue**: The 5 tests cover X=0/3/4/5/10 (counters and draw boundary) well, but none test the interaction where the creature leaves the battlefield while the draw trigger is on the stack. This is the most common real-game scenario (opponent removes the creature in response). This test would also catch Finding 1.
**Fix**: Add `test_ravenous_draw_still_fires_if_creature_removed()`: cast with X=5, resolve spell (creature enters, draw trigger goes on stack), then destroy/exile the creature, then resolve the draw trigger -- verify CardDrawn event still fires.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.156a (replacement: enters with X counters) | Yes | Yes | test_x3, test_x5, test_x10 |
| 702.156a (trigger: draw if X >= 5) | Yes | Yes | test_x5, test_x10 |
| 702.156a (no draw if X < 5) | Yes | Yes | test_x0, test_x3, test_x4 |
| 107.3m (X = cast-time value) | Yes | Yes | x_value propagated from CastSpell -> StackObject -> GameObject -> EffectContext |
| 107.3m (permanent's X = 0) | Yes (via zone-change reset) | No | x_value reset to 0 in mod.rs zone-change; no explicit test |
| 603.4 (intervening-if re-check) | Partial | No | x_value re-check correct; battlefield check is wrong (Finding 1) |
| 107.3m (copies inherit X) | Yes | No | copy.rs line 240 propagates x_value; no test |
| 704.5f (0/0 dies to SBA) | Implicit | Partial | test_x0 mentions it but doesn't assert creature went to graveyard |

## Previous Findings (re-review only)

N/A -- first review.
