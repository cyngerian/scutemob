# Ability Review: Tribute

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.104
**Files reviewed**:
- `crates/engine/src/state/types.rs:1203-1211`
- `crates/engine/src/state/hash.rs:645-649` (KeywordAbility), `:854-855` (GameObject), `:3165-3166` (TriggerCondition)
- `crates/engine/src/state/game_object.rs:579-584`
- `crates/engine/src/state/mod.rs:371-372`, `:524-525`
- `crates/engine/src/state/builder.rs:967-968`
- `crates/engine/src/rules/resolution.rs:1033-1071` (Tribute ETB block), `:3344`, `:4460`, `:4652`, `:4863` (token sites)
- `crates/engine/src/rules/lands.rs:360-363`
- `crates/engine/src/rules/replacement.rs:930-965` (TributeNotPaid arm in fire_when_enters_triggered_effects)
- `crates/engine/src/cards/card_definition.rs:1111-1117` (TriggerCondition::TributeNotPaid)
- `crates/engine/src/effects/mod.rs:2807` (token template)
- `tools/replay-viewer/src/view_model.rs:851`
- `crates/engine/tests/tribute.rs` (8 tests, 650 lines)

## Verdict: clean

The Tribute implementation is correct and complete for the current bot-play model. All CR 702.104 subrules are handled. The `tribute_was_paid` field is properly initialized at all object-creation sites (builder, 2 zone-change sites, 4 token-creation sites, 1 effects/mod.rs template), correctly hashed, and reset on zone changes per CR 400.7. The `TriggerCondition::TributeNotPaid` variant is cleanly integrated into `fire_when_enters_triggered_effects` with the correct intervening-if check. Tests cover the key positive, negative, multiplayer, and zone-change scenarios. Two LOW findings noted below are systemic issues, not Tribute-specific bugs.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `replacement.rs:954` | **Inline trigger fires without stack.** CR 603.4 requires intervening-if triggers to use the stack; current fires inline. Systemic pattern, not Tribute-specific. |
| 2 | LOW | `tests/tribute.rs:315` | **Test 3 does not exercise ETB flow for paid path.** Manual state setup validates the condition check but not the full resolution pipeline. Acceptable since bot never pays. |

### Finding Details

#### Finding 1: Inline trigger fires without stack

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:949-964`
**CR Rule**: 603.4 -- "When the trigger event occurs, the ability checks whether the stated condition is true. The ability triggers only if it is; otherwise it does nothing. If the ability triggers, it checks the stated condition again as it resolves."
**Issue**: The `TributeNotPaid` trigger fires inline via `fire_when_enters_triggered_effects` rather than going on the stack. This means opponents cannot respond to the trigger before it resolves, and the second resolution-time check per CR 603.4 does not occur. However, this is the same pattern used for all `WhenEntersBattlefield` triggers in the engine (line 940-948). Since tribute_was_paid cannot change between fire time and resolution time when both happen in the same instant, the result is correct for bot play. This is a systemic design limitation, not a Tribute-specific defect.
**Fix**: No immediate fix needed. When the engine adds stack-based ETB triggers (likely M11+), TributeNotPaid should be migrated to use a PendingTrigger with a StackObjectKind variant, which would enable the intervening-if re-check at resolution time and allow opponent responses.

#### Finding 2: Test 3 uses manual state rather than ETB flow for paid path

**Severity**: LOW
**File**: `crates/engine/tests/tribute.rs:315-370`
**CR Rule**: 702.104b -- "This condition is true if the opponent chosen as a result of the tribute ability didn't have the creature enter the battlefield with +1/+1 counters."
**Issue**: Test 3 (`test_tribute_paid_trigger_does_not_fire`) validates the "tribute was paid" path by manually constructing a battlefield object with `tribute_was_paid = true` and confirming no trigger fires. It does not exercise the full cast-resolve-ETB pipeline for the paid path. This is acceptable because the bot never pays tribute, so there is no in-engine code path that sets `tribute_was_paid = true` during resolution. The test correctly validates the condition check logic in isolation.
**Fix**: When interactive opponent choices are added (future milestone), add an integration test that exercises the full ETB flow with tribute payment, including counter placement and trigger suppression.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.104a (static ability, "as creature enters") | Yes | Yes | test_tribute_basic_not_paid, resolution.rs:1033 |
| 702.104a ("choose an opponent") | Partial | Yes | Bot auto-selects; multiplayer test confirms non-controller opponent exists |
| 702.104a ("N +1/+1 counters") | Yes (bot declines) | Yes | test_tribute_basic_not_paid (0 counters), test_tribute_paid (manual 2 counters) |
| 702.104b ("if tribute wasn't paid" condition) | Yes | Yes | test_tribute_not_paid_trigger_fires, test_tribute_paid_trigger_does_not_fire |
| CR 400.7 (zone-change reset) | Yes | Yes | test_tribute_paid_resets_on_zone_change |
| CR 603.4 (intervening-if) | Partial (inline) | Yes | Checks at fire time; no stack-based re-check (LOW Finding 1) |
| Different N values | Yes | Yes | test_tribute_n_value_draw_card (Tribute 3) |
| No-trigger card | Yes | Yes | test_tribute_no_trigger_card |
| Multiplayer | Yes | Yes | test_tribute_multiplayer_fires (4 players) |
| Hash coverage (KeywordAbility) | Yes | -- | hash.rs:645-649 |
| Hash coverage (tribute_was_paid) | Yes | -- | hash.rs:855 |
| Hash coverage (TriggerCondition) | Yes | -- | hash.rs:3165-3166 |
| Builder initialization | Yes | -- | builder.rs:968 |
| Zone-change initialization (2 sites) | Yes | -- | mod.rs:372, mod.rs:525 |
| Token-creation initialization (4 sites) | Yes | -- | resolution.rs:3344,4460,4652,4863 |
| Effects token template | Yes | -- | effects/mod.rs:2807 |
| Replay viewer match arm | Yes | -- | view_model.rs:851 |
| Lands.rs stub | Yes | -- | lands.rs:360-363 |

verdict: clean
