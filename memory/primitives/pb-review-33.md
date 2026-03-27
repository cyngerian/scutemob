# Primitive Batch Review: PB-33 -- Copy/Clone + Exile/Flicker Timing

**Date**: 2026-03-27
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 707.2, 707.5, 707.9a, 707.9b, 603.7, 603.7b, 610.3, 610.3c
**Engine files reviewed**: `state/stubs.rs`, `state/hash.rs`, `state/game_object.rs`, `state/mod.rs`, `state/builder.rs`, `state/stack.rs`, `state/continuous_effect.rs`, `cards/card_definition.rs`, `effects/mod.rs`, `rules/turn_actions.rs`, `rules/abilities.rs`, `rules/resolution.rs`, `rules/engine.rs`, `rules/layers.rs`, `tools/tui/src/play/panels/stack_view.rs`, `tools/replay-viewer/src/view_model.rs`
**Card defs reviewed**: 15 (kiki_jiki_mirror_breaker, the_fire_crystal, helm_of_the_host, miirym_sentinel_wyrm, the_eternal_wanderer, brutal_cathar, nezahal_primal_tide, chandra_flamecaller, voice_of_victory, zurgo_stormrender, the_locust_god, puppeteer_clique, mirage_phalanx, thousand_faced_shadow, mist_syndicate_naga)

## Verdict: needs-fix

Two HIGH findings (missing hash fields), one MEDIUM (hash incomplete for delayed_action tuple contents). All three are in `state/hash.rs`. Engine logic and card defs are otherwise correct. Tests cover the key patterns well.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `state/hash.rs:~972` | **Missing hash for 3 new GameObject bool fields.** `sacrifice_at_end_step`, `exile_at_end_step`, `return_to_hand_at_end_step` on `GameObject` are not hashed in the `HashInto for GameObject` impl. Two different game states differing only in these flags would hash identically. **Fix:** Add `self.sacrifice_at_end_step.hash_into(hasher); self.exile_at_end_step.hash_into(hasher); self.return_to_hand_at_end_step.hash_into(hasher);` after `self.meld_component.hash_into(hasher);` (line ~1022) in the `HashInto for GameObject` impl. |
| 2 | **HIGH** | `state/hash.rs:4904` | **Incomplete hash for CreateTokenCopy.delayed_action.** Only `delayed_action.is_some()` is hashed; the actual `(DelayedTriggerTiming, DelayedTriggerAction)` tuple contents are not. Two `CreateTokenCopy` effects differing only in delayed_action kind (e.g., `SacrificeObject` vs `ExileObject`) would hash identically. **Fix:** Replace `delayed_action.is_some().hash_into(hasher);` with a full match: `match delayed_action { None => 0u8.hash_into(hasher), Some((timing, action)) => { 1u8.hash_into(hasher); /* hash timing and action using same match patterns as DelayedTrigger hash */ } }`. |
| 3 | MEDIUM | `rules/turn_actions.rs:904-959` | **Flag-based end-step actions re-fire if countered (CR 603.7b deviation).** The `sacrifice_at_end_step` and `exile_at_end_step` flags on tokens are not cleared when the corresponding `PendingTrigger` is queued. If the delayed trigger is countered (e.g., by Stifle), the flag remains and fires again at the next end step. CR 603.7b says "A delayed triggered ability will trigger only once." The `return_to_hand_at_end_step` flag IS cleared at queue time (line 977-978), so it's correct. **Fix:** Clear `sacrifice_at_end_step = false` and `exile_at_end_step = false` on the token when queuing the trigger (same pattern as line 977-978 for return_to_hand). |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | LOW | `kiki_jiki_mirror_breaker.rs:24` | **TODO: target filter lacks "nonlegendary" restriction.** Oracle says "target nonlegendary creature you control" but `TargetFilter` has no `nonlegendary` flag. Pre-existing DSL gap, correctly documented. No fix needed now. |
| 5 | LOW | `miirym_sentinel_wyrm.rs:25,41` | **Two TODOs: nontoken restriction and exclude-self.** Oracle says "another nontoken Dragon" but filter lacks nontoken restriction and no SourceIsNotSelf condition. Pre-existing DSL gaps, correctly documented. No fix needed now. |

### Finding Details

#### Finding 1: Missing hash for 3 new GameObject bool fields

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:~1022` (end of `HashInto for GameObject`)
**CR Rule**: Architecture invariant -- all state fields must be hashed for deterministic state identity.
**Issue**: The three new `bool` fields added to `GameObject` in PB-33 (`sacrifice_at_end_step`, `exile_at_end_step`, `return_to_hand_at_end_step`) are correctly added to `TokenSpec` hash (lines 3905-3906) and to the `make_token()` function, but they are missing from the `HashInto for GameObject` implementation. The `encore_sacrifice_at_end_step` field at line 937 is a different, pre-existing field. Two game states where the only difference is a token's `sacrifice_at_end_step = true` vs `false` would produce identical hashes, breaking loop detection and state comparison.
**Fix**: Add three lines after `self.meld_component.hash_into(hasher);` (line 1022) in the `HashInto for GameObject` impl:
```rust
self.sacrifice_at_end_step.hash_into(hasher);
self.exile_at_end_step.hash_into(hasher);
self.return_to_hand_at_end_step.hash_into(hasher);
```

#### Finding 2: Incomplete hash for CreateTokenCopy.delayed_action

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:4904`
**CR Rule**: Architecture invariant -- hash must distinguish all structurally different Effect values.
**Issue**: `delayed_action.is_some().hash_into(hasher)` only hashes `true`/`false`, not the contents. A `CreateTokenCopy` with `delayed_action: Some((AtNextEndStep, SacrificeObject))` hashes identically to one with `delayed_action: Some((AtEndOfCombat, ExileObject))`. This can cause false hash collisions in state comparison and loop detection.
**Fix**: Replace line 4904 with:
```rust
match delayed_action {
    None => 0u8.hash_into(hasher),
    Some((timing, action)) => {
        1u8.hash_into(hasher);
        match timing {
            crate::state::stubs::DelayedTriggerTiming::AtNextEndStep => 0u8.hash_into(hasher),
            crate::state::stubs::DelayedTriggerTiming::AtOwnersNextEndStep => 1u8.hash_into(hasher),
            crate::state::stubs::DelayedTriggerTiming::WhenSourceLeavesBattlefield => 2u8.hash_into(hasher),
            crate::state::stubs::DelayedTriggerTiming::AtEndOfCombat => 3u8.hash_into(hasher),
        }
        match action {
            crate::state::stubs::DelayedTriggerAction::ReturnFromExileToBattlefield { tapped } => {
                0u8.hash_into(hasher);
                tapped.hash_into(hasher);
            }
            crate::state::stubs::DelayedTriggerAction::ReturnFromExileToHand => 1u8.hash_into(hasher),
            crate::state::stubs::DelayedTriggerAction::ReturnFromGraveyardToHand => 2u8.hash_into(hasher),
            crate::state::stubs::DelayedTriggerAction::SacrificeObject => 3u8.hash_into(hasher),
            crate::state::stubs::DelayedTriggerAction::ExileObject => 4u8.hash_into(hasher),
        }
    }
}
```

#### Finding 3: Flag-based end-step actions re-fire if countered

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/turn_actions.rs:904-959`
**CR Rule**: 603.7b -- "A delayed triggered ability will trigger only once -- the next time its trigger event occurs"
**Issue**: When `sacrifice_at_end_step` or `exile_at_end_step` triggers are queued in `end_step_actions()`, the flag on the token is NOT cleared. If the trigger is countered (Stifle), the flag persists and will fire again at the next end step. This violates CR 603.7b which says delayed triggers fire only once. The `return_to_hand_at_end_step` flag IS correctly cleared at line 977-978. This is an inconsistency.
**Fix**: Add flag-clearing code for each token before pushing the PendingTrigger. For sacrifice_at_end_step (after line 916):
```rust
if let Some(obj) = state.objects.get_mut(&token_id) {
    obj.sacrifice_at_end_step = false;
}
```
Same pattern for exile_at_end_step (after line 944).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 707.2 (copying objects) | Yes | Indirect | CreateTokenCopy uses Layer 1 CopyOf |
| 707.9a (copy gains ability) | Yes | Yes | test_create_token_copy_with_haste |
| 707.9b (copy modifies characteristic) | Yes | Yes | test_create_token_copy_not_legendary |
| 603.7 (delayed triggered abilities) | Yes | Yes | All 7 tests |
| 603.7b (fires only once) | Yes | Yes | test_delayed_trigger_fires_only_once; flag-clearing gap (Finding 3) |
| 603.7c (object no longer in zone) | Yes | Partial | Resolution checks zone before acting |
| 610.3 (exile "until" effects) | Yes | Yes | test_exile_until_source_leaves |
| 610.3c (returns under owner's control) | Yes | Implicit | Resolution uses obj.owner |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| kiki_jiki_mirror_breaker | Yes | 1 (nonlegendary filter) | Yes | Pre-existing DSL gap for target filter |
| the_fire_crystal | Yes | 0 | Yes | Clean |
| helm_of_the_host | Yes | 0 | Yes | Clean |
| miirym_sentinel_wyrm | Yes | 2 (nontoken, exclude-self) | Mostly | Tokens Dragons wrongly trigger |
| the_eternal_wanderer | Yes | 2 (attack restriction, -4) | Partial | +1 correct; 0 correct; -4 stubbed |
| brutal_cathar | Yes | 0 | Yes | DFC with WhenSourceLeavesBattlefield |
| nezahal_primal_tide | Yes | 2 (can't be countered, hand size) | Partial | Exile+return correct |
| chandra_flamecaller | Yes | 1 (0 ability) | Partial | +1 correct; 0 stubbed; -X correct |
| voice_of_victory | Yes | 1 (stax restriction) | Partial | Mobilize correct |
| zurgo_stormrender | Yes | 1 (token LTB trigger) | Partial | Mobilize correct |
| the_locust_god | Yes | 1 (draw-then-discard) | Partial | Death trigger correct |
| puppeteer_clique | Partial | 1 (reanimate) | Partial | Correctly noted as partial |
| mirage_phalanx | Yes | 2 (soulbond grant, loses soulbond) | Partial | Copy+exile correct for self |
| thousand_faced_shadow | Yes | 2 (from-hand, attacking filter) | Partial | Copy effect correct |
| mist_syndicate_naga | Yes | 0 | Yes | Clean |

## Test Coverage Summary

7 tests in `delayed_triggers.rs` covering:
- Positive: sacrifice_at_end_step, exile_at_end_step, return_to_hand_at_end_step, fires-only-once, gains_haste, except_not_legendary, exile-until-source-leaves
- Negative: fires-only-once verifies trigger cleanup; exile token SBA removal
- Missing: No test for CreateTokenCopy with delayed_action (Kiki-Jiki pattern), no test for AtEndOfCombat timing (Mirage Phalanx pattern), no test for countered delayed trigger (Stifle)

## Previous Findings (first review)

N/A -- first review.
