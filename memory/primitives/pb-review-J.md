# Primitive Batch Review: PB-J -- Copy/Redirect Spells

**Date**: 2026-04-09
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 707.10, 707.10c, 115.7, 115.7a, 115.7d
**Engine files reviewed**: `card_definition.rs`, `effects/mod.rs`, `casting.rs`, `abilities.rs`, `events.rs`, `hash.rs`
**Card defs reviewed**: 4 (bolt_bend.rs, deflecting_swat.rs, untimely_malfunction.rs, complete_the_circuit.rs)

## Verdict: needs-fix

Two MEDIUM findings: (1) ChangeTargets player redirect doesn't check `has_lost` on the
preferred controller target, and (2) self-targeting prevention for
TargetSpellOrAbilityWithSingleTarget is documented but not implemented. One LOW finding
for Untimely Malfunction mode 2 missing a TODO about variable target count.
No HIGH findings. Engine changes are broadly correct and well-structured.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `effects/mod.rs:5129` | **ChangeTargets controller-prefer skips has_lost check.** Fix: add has_lost guard. |
| 2 | **MEDIUM** | `casting.rs:5421` | **Self-targeting prevention not implemented.** Doc comment claims it, code doesn't enforce it. |
| 3 | LOW | `effects/mod.rs:5160` | **Object redirect ignores original TargetRequirement.** Simplified approach noted in plan. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | LOW | `untimely_malfunction.rs` | **Mode 2 missing TODO for variable target count.** Oracle: "one or two target creatures". |

### Finding Details

#### Finding 1: ChangeTargets controller-prefer skips has_lost check

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:5129`
**CR Rule**: 115.7a -- "each target can be changed only to another legal target"
**Issue**: When `must_change: true`, the deterministic fallback prefers the effect controller
as the new target (line 5129-5130). But it does not check whether the controller `has_lost`.
If the controller has lost the game, they are not a legal target (CR 115.5 -- a player who
has lost the game is not a legal target). The fallback path (lines 5136-5143) correctly
filters out players with `has_lost`, but the preferred-controller path does not.
**Fix**: Add a `has_lost` check before preferring the controller:
```rust
let controller = ctx.controller;
let controller_alive = !state.players.get(&controller).map(|ps| ps.has_lost).unwrap_or(true);
let new_pid = if controller_alive && controller != *current_pid {
    Some(controller)
} else {
    // existing fallback...
};
```

#### Finding 2: Self-targeting prevention not implemented

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:5421`
**CR Rule**: 115.4 -- a spell or ability on the stack is a legal target for another spell or
ability unless otherwise specified. However, the plan (pb-plan-J.md) and the doc comment on
`TargetSpellOrAbilityWithSingleTarget` (card_definition.rs:2270) both state: "The targeting
spell itself is excluded as a valid target (prevents self-targeting loops)." The actual
validation code at casting.rs:5421-5437 does NOT exclude the casting spell's own stack object.
This means Bolt Bend could target itself on the stack (it has exactly 1 target -- itself
would satisfy `targets.len() == 1`), creating an infinite self-reference loop.
**Fix**: Add self-targeting prevention in the validation. The `validate_target` function
receives the caster's stack object ID implicitly through the call context. Add a parameter
or check: if the target object ID equals the caster's own stack entry, reject it.
The exact approach depends on how `validate_target` receives context about the casting
spell -- check if `caster_stack_id` is available or needs to be threaded through.

#### Finding 3: Object redirect ignores original TargetRequirement

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:5160`
**CR Rule**: 115.7a -- "each target can be changed only to another legal target"
**Issue**: The object redirect for `must_change: true` picks the smallest ObjectId in the
same zone as the original target, without checking whether the new object satisfies the
original spell's `TargetRequirement`. For example, a spell targeting a creature could be
redirected to a non-creature artifact in the same zone. The plan acknowledges this as a
simplification ("simplified approach is safer for M9.4"), and the original spell's
`TargetRequirement` is not readily available from the `StackObject`. This is acceptable
for pre-M10 but should be documented as a known limitation.
**Fix**: Add a comment at the redirect site noting this limitation and referencing the
plan's rationale. No code change needed now; M10 interactive choice will replace the
deterministic fallback entirely.

#### Finding 4: Untimely Malfunction mode 2 missing variable target TODO

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/untimely_malfunction.rs:52`
**Oracle**: "One or two target creatures can't block this turn."
**Issue**: The oracle text specifies 1-2 targets for mode 2, but the card def only has one
`TargetCreature` slot at index 2. The DSL does not currently support variable target counts
(same limitation seen in Abzan Charm's mode 2, which has a TODO). Untimely Malfunction
should have a similar TODO comment noting this limitation.
**Fix**: Add a TODO comment on line 52:
```rust
// Mode 2: One or two target creatures can't block this turn.
// TODO: "one or two target creatures" requires variable target count (1-2 targets),
// which is not expressible in the current DSL. Currently only supports one target.
// Same limitation as Abzan Charm mode 2.
```

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 707.10  | Yes         | Yes     | test_copy_spell_on_stack_basic, test_copy_spell_on_stack_twice |
| 707.10c | Partial     | No      | Copies keep original targets (deferred interactive choice). Correct behavior for deterministic. |
| 115.7   | Yes         | Yes     | Dispatch in effects/mod.rs covers both modes |
| 115.7a  | Yes         | Yes     | test_change_targets_must_change_redirects_to_new_player, test_change_targets_no_alternative_leaves_unchanged |
| 115.7d  | Yes         | Yes     | test_change_targets_may_choose_new_leaves_unchanged |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Bolt Bend | Yes | 0 | Yes | Cost reduction + ChangeTargets correct |
| Deflecting Swat | Yes | 0 | Yes | CommanderFreeCast + TargetSpell correct (targets both spells and abilities on stack) |
| Untimely Malfunction | Partial | 0 (should be 1) | Partial | Mode 2 variable target count not supported (Finding 4) |
| Complete the Circuit | Partial | 1 (expected) | Partial | Delayed copy trigger correctly deferred; TODO is accurate and detailed |

## Test Summary

8 tests in `copy_redirect.rs` covering:
- CopySpellOnStack: basic copy (1 copy), multi-copy (2 copies)
- ChangeTargets must_change: player redirect, no-alternative unchanged, object redirect
- ChangeTargets may_choose_new: deterministic unchanged
- TargetSpellOrAbilityWithSingleTarget: single-target acceptance
- Integration: Bolt Bend 3-player redirect scenario

Tests are well-structured with correct CR citations. No test for the self-targeting
prevention (Finding 2) since the prevention itself is not implemented.
