# Ability Review: Kicker

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.33
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 244-248)
- `crates/engine/src/cards/card_definition.rs` (lines 152-166, 576-581)
- `crates/engine/src/state/stack.rs` (lines 52-58)
- `crates/engine/src/state/game_object.rs` (lines 260-269)
- `crates/engine/src/state/hash.rs` (lines 345-347, 489-490, 1086-1087, 1988-1989, 2291-2299)
- `crates/engine/src/rules/command.rs` (lines 81-88)
- `crates/engine/src/rules/casting.rs` (lines 56, 169-211, 396, 462-502, 552-574)
- `crates/engine/src/rules/engine.rs` (lines 70-98)
- `crates/engine/src/rules/resolution.rs` (lines 149-156, 183-188)
- `crates/engine/src/rules/copy.rs` (lines 166-177, 333-343)
- `crates/engine/src/effects/mod.rs` (lines 48-93, 963, 1697-1746)
- `crates/engine/src/rules/replacement.rs` (lines 871-891)
- `crates/engine/src/testing/script_schema.rs` (lines 228-232)
- `crates/engine/src/testing/replay_harness.rs` (lines 204, 238, 256)
- `crates/engine/src/cards/definitions.rs` (lines 2059-2137)
- `crates/engine/tests/kicker.rs` (lines 1-920)
- `crates/engine/tests/script_replay.rs` (lines 145, 159)

## Verdict: needs-fix

The kicker implementation is solid overall. The core casting-time validation,
cost stacking, stack-to-permanent propagation, spell-copy inheritance, and
`Condition::WasKicked` evaluation are all correct and well-documented. However,
there is one MEDIUM finding: the use of `.expect()` in engine library code
violates the project convention "never `unwrap()` or `expect()` in engine logic."
While logically safe (the value was validated one block earlier), the convention
exists to prevent panics in the library crate under any circumstances. All other
findings are LOW severity.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `casting.rs:197` | **`.expect()` in engine library code.** Violates project convention. **Fix:** replace with `ok_or` + `?`. |
| 2 | LOW | `replacement.rs:879-884` | **`fire_when_enters_triggered_effects` ignores `intervening_if`.** Works today via Effect::Conditional workaround, but fragile for future cards. |
| 3 | LOW | `resolution.rs:391` | **Stack-resolved triggered abilities get `kicker_times_paid: 0`.** If WhenEntersBattlefield ever moves to the stack path, kicker context is lost. |
| 4 | LOW | `kicker.rs` tests | **No test for kicker + flashback interaction.** CR 118.9d guarantees this works, but no test exercises it. |
| 5 | LOW | `kicker.rs` tests | **No test for kicker + convoke cost reduction.** The cost pipeline positions kicker before convoke, but no test verifies the reduced total. |
| 6 | LOW | `definitions.rs:2134` | **`intervening_if: None` on Torch Slinger instead of `Some(Condition::WasKicked)`.** Works because the condition is modeled in the effect body, but doesn't match the card's oracle text pattern ("When ~ enters, IF it was kicked"). |
| 7 | LOW | `view_model.rs` | **Replay viewer not updated for kicker.** StackObjectView has no `is_kicked` field. Noted as LOW priority in plan. |

### Finding Details

#### Finding 1: `.expect()` in engine library code

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs:197`
**CR Rule**: N/A -- project convention
**Issue**: The line `get_kicker_cost(&card_id, &state.card_registry).expect("kicker already validated above")` uses `.expect()` in the engine library crate. The project convention in `memory/conventions.md` explicitly states: "Engine crate uses typed errors -- never `unwrap()` or `expect()` in engine logic. Tests may use `unwrap()`." While the call is logically safe (the same function was called 5 lines earlier and returned `Some`), the convention is absolute to prevent any possibility of panics in the library crate. A defensive refactor eliminates the second `get_kicker_cost` call entirely.
**Fix**: Restructure the kicker validation block to capture the kicker cost in a local variable on the first call, avoiding the need for a second lookup. For example:

```rust
let (kicker_times_paid, kicker_cost_opt) = if kicker_times > 0 {
    let kicker_info = get_kicker_cost(&card_id, &state.card_registry);
    match kicker_info {
        Some((kicker_cost, is_multikicker)) => {
            if !is_multikicker && kicker_times > 1 {
                return Err(GameStateError::InvalidCommand(
                    "standard kicker can only be paid once (CR 702.33d)".into(),
                ));
            }
            (kicker_times, Some(kicker_cost))
        }
        None => {
            return Err(GameStateError::InvalidCommand(
                "spell does not have kicker".into(),
            ));
        }
    }
} else {
    (0, None)
};

let mana_cost = if let Some(kicker_cost) = kicker_cost_opt {
    let mut total = mana_cost.unwrap_or_default();
    for _ in 0..kicker_times_paid {
        total.white += kicker_cost.white;
        // ... etc.
    }
    Some(total)
} else {
    mana_cost
};
```

This eliminates both the `.expect()` and the redundant registry lookup.

#### Finding 2: `fire_when_enters_triggered_effects` ignores `intervening_if`

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/replacement.rs:879-884`
**CR Rule**: 603.4 -- "A triggered ability may read 'When/Whenever/At [trigger event], if [condition], [effect].' ... The ability triggers only if [condition] is true at the time the trigger event occurs."
**Issue**: The `fire_when_enters_triggered_effects` function matches `AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield, effect, .. }` and fires the effect unconditionally, ignoring the `intervening_if` field. Torch Slinger works around this by using `intervening_if: None` and wrapping the kicker check inside `Effect::Conditional { condition: WasKicked, ... }`. This is a valid design workaround but means the `intervening_if` field on `AbilityDefinition::Triggered` is silently ignored for ETB triggers. Future card definitions that use `intervening_if: Some(...)` on ETB triggers will not have that condition checked.
**Fix**: No immediate fix required (Torch Slinger works correctly). Document the pattern in `gotchas-rules.md`: "ETB triggers fired via `fire_when_enters_triggered_effects` do not check `intervening_if`. Use `Effect::Conditional` inside the effect body instead." When ETB triggers are migrated to the stack-based `PendingTrigger` system, this limitation goes away naturally.

#### Finding 3: Stack-resolved triggered abilities lack kicker context

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:391`
**CR Rule**: 702.33e -- linked abilities refer to specific kicker abilities
**Issue**: `EffectContext::new(stack_obj.controller, source_object, stack_obj.targets.clone())` at line 391 creates a context with `kicker_times_paid: 0`. If a WhenEntersBattlefield trigger were to go through the stack (as it should per CR 603.3), any `Condition::WasKicked` check in the effect would evaluate to false regardless of whether the permanent was kicked. Currently, ETB triggers fire inline via `fire_when_enters_triggered_effects` (which correctly reads `GameObject.kicker_times_paid`), so this is a latent issue, not a current bug.
**Fix**: No immediate fix required. When the trigger system is updated to dispatch WhenEntersBattlefield via the stack, the `TriggeredAbility` resolution path must read `kicker_times_paid` from the source `GameObject` and pass it to `EffectContext::new_with_kicker`.

#### Finding 4: No test for kicker + flashback interaction

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/kicker.rs`
**CR Rule**: 118.9d -- "Additional costs can also be paid when a spell is cast for its alternative cost."
**Issue**: The plan identifies kicker + flashback as a notable interaction (flashback cost + kicker cost stacking). No test verifies that a spell cast via flashback can also be kicked and that the total cost is flashback + kicker (not mana + kicker). The implementation handles this naturally (kicker cost is added to whatever `mana_cost` is, which is already set to the flashback cost), but a test would prevent regressions.
**Fix**: Consider adding a test that casts a hypothetical spell with both flashback and kicker, verifying the total cost and that both flags are set on the stack object. This can be deferred to a future card definition that combines both abilities.

#### Finding 5: No test for kicker + convoke cost reduction

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/kicker.rs`
**CR Rule**: 601.2f -- kicker cost is announced before convoke reduction
**Issue**: The plan identifies kicker + convoke as a notable interaction. The casting pipeline positions kicker addition before convoke reduction, which is correct. No test verifies that a kicked convoke spell can have its total (base + kicker) reduced by tapping creatures.
**Fix**: Consider adding a test with a hypothetical convoke+kicker spell. Can be deferred.

#### Finding 6: Torch Slinger uses effect-body conditional instead of `intervening_if`

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs:2134`
**CR Rule**: 702.33e / 603.4 -- "When ~ enters, if it was kicked" is an intervening-if pattern
**Issue**: The card oracle text reads "When Torch Slinger enters, **if it was kicked**, it deals 2 damage to target creature." The "if it was kicked" is textually an intervening-if condition (CR 603.4), meaning the trigger should not go on the stack at all if the condition is false. The implementation uses `intervening_if: None` and wraps the check inside `Effect::Conditional`. The behavioral difference: with a true intervening-if, the trigger never fires (no stack object). With Effect::Conditional, the trigger fires but does nothing. Since the inline fire path (`fire_when_enters_triggered_effects`) doesn't use the stack anyway, the distinction is moot for now.
**Fix**: No immediate fix. When ETB triggers migrate to the stack, model "if it was kicked" as a proper `intervening_if`. This may require extending the `InterveningIf` enum (currently only `ControllerLifeAtLeast`) or unifying it with `Condition`.

#### Finding 7: Replay viewer not updated

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**CR Rule**: N/A -- feature gap
**Issue**: The `StackObjectView` struct in the replay viewer does not include an `is_kicked` or `kicker_times_paid` field, so kicked spells are not visually distinguished in the stepper UI.
**Fix**: Add `pub is_kicked: bool` to `StackObjectView` and populate it from `so.kicker_times_paid > 0`. LOW priority -- the viewer works without it.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.33a (kicker is additional cost) | Yes | Yes | test_kicker_basic_cast_with_kicker, test_kicker_basic_cast_without_kicker |
| 702.33b (dual kicker "and/or") | No | No | Not implemented -- single kicker cost only. Noted in plan as future work. |
| 702.33c (multikicker) | Partial | Partial | Data model supports it (kicker_times_paid: u32, is_multikicker: bool). No multikicker card def or test. Validation correctly rejects kicker_times > 1 for standard kicker. |
| 702.33d (spell is "kicked") | Yes | Yes | test_kicker_basic_cast_with_kicker, test_kicker_standard_kicker_rejects_multiple |
| 702.33e (linked abilities) | Yes | Yes | test_kicker_permanent_etb_kicked, test_kicker_permanent_etb_not_kicked |
| 702.33f (specific kicker costs) | No | No | Not implemented -- requires dual kicker (702.33b) first. |
| 702.33g (conditional kicker targets) | No | No | Noted in plan as "advanced feature that can be deferred." |
| 702.33h (sticker kicker) | No | No | Not relevant for this engine (Un-set mechanic). |
| 118.8d (doesn't change mana value) | Yes | Yes | test_kicker_does_not_change_mana_value |
| 118.8a (multiple additional costs) | Yes | Yes | test_kicker_with_commander_tax |
| 601.2b (announce kicker intent) | Yes | Yes | All cast tests announce via kicker_times field |
| Spell copy inherits kicked | Yes | No | copy.rs line 176 propagates kicker_times_paid. No test for storm/cascade + kicker. |
| Permanent copy does NOT inherit kicked | Not needed yet | No | Clone/copy-as-permanent not implemented. Documented in plan. |
| Kicker only works when CAST | Yes | No | Permanents entering without casting have kicker_times_paid=0 by default. No explicit test. |

## Previous Findings (re-review only)

N/A -- this is the initial review.

---

verdict: needs-fix
