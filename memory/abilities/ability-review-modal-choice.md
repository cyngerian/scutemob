# Ability Review: Modal Choice

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 700.2
**Files reviewed**:
- `crates/engine/src/rules/command.rs` (modes_chosen field on CastSpell)
- `crates/engine/src/rules/casting.rs` (validation logic at ~2797-2867)
- `crates/engine/src/rules/resolution.rs` (mode dispatch at ~211-244)
- `crates/engine/src/rules/copy.rs` (modes_chosen propagation at line 235)
- `crates/engine/src/state/stack.rs` (modes_chosen field at line 270)
- `crates/engine/src/state/hash.rs` (StackObject hash at 2006-2009, ModeSelection hash at 3239-3247)
- `crates/engine/src/cards/card_definition.rs` (allow_duplicate_modes field at 1179)
- `crates/engine/src/rules/abilities.rs` (12 StackObject construction sites)
- `crates/engine/src/rules/engine.rs` (modes_chosen dispatch at 105-134)
- `crates/engine/src/testing/replay_harness.rs` (cast_spell_modal action, modes parameter)
- `crates/engine/src/testing/script_schema.rs` (modes field on PlayerAction)
- `crates/engine/tests/modal.rs` (12 tests)

## Verdict: needs-fix

The core implementation is solid -- casting validation, resolution dispatch, and copy propagation all follow the CR correctly. The if-chain priority order (entwine > explicit modes > escalate backward-compat > auto-mode[0]) is correct. Backward compatibility with existing Entwine/Escalate scripts is preserved. However, there are two MEDIUM findings: a missing test for copy-of-modal-spell (CR 700.2g) that was in the plan but not implemented, and a missing TODO comment in casting.rs for the deferred per-mode targeting (CR 700.2c). One additional MEDIUM for untested allow_duplicate_modes positive path.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tests/modal.rs` | **Missing copy-inherits-modes test.** Plan specified test 11 as copy test (CR 700.2g/707.10) but it was not implemented. **Fix:** add test. |
| 2 | MEDIUM | `tests/modal.rs` | **Missing allow_duplicate_modes positive test.** CR 700.2d's "may choose same mode more than once" path is wired but never exercised. **Fix:** add test. |
| 3 | MEDIUM | `casting.rs:2867` | **No TODO/comment for deferred CR 700.2c.** Per-mode targeting is silently deferred with no code comment. **Fix:** add TODO comment. |
| 4 | LOW | `hash.rs:2006-2009` | **Vec hash without length prefix.** Manual loop skips blanket `Vec<T>::hash_into` which prefixes length. Pre-existing pattern (same for spliced_effects, devour_sacrifices). **Fix:** use `self.modes_chosen.hash_into(hasher)` instead of manual loop. |
| 5 | LOW | `tests/modal.rs` | **Missing escalate-with-explicit-modes test.** Plan specified test 12 as escalate integration but it was replaced with a simpler test. **Fix:** add test for `cast_spell_modal` with `escalate_modes > 0`. |

### Finding Details

#### Finding 1: Missing copy-inherits-modes test

**Severity**: MEDIUM
**File**: `crates/engine/tests/modal.rs`
**CR Rule**: 700.2g -- "A copy of a modal spell or ability copies the mode(s) chosen for it. The controller of the copy can't choose a different mode." / 707.10 -- "A copy of a spell or ability copies both the characteristics of the spell or ability and all decisions made for it, including modes, targets, the value of X..."
**Issue**: The plan document (ability-plan-modal-choice.md, Step 8, test 11) specified a test where a modal spell is cast with storm choosing mode[1], and the storm copy should also execute mode[1]. The `copy.rs` line 235 does `modes_chosen: original.modes_chosen.clone()` which looks correct, but this code path has zero test coverage. The actual test 11 (`test_modal_non_modal_spell_with_modes_chosen_rejected`) tests a different scenario.
**Fix**: Add a test `test_modal_copy_inherits_modes` that casts a modal storm spell choosing a non-default mode, verifies the storm copy on the stack has the same `modes_chosen`, and verifies the copy's effects match the chosen mode.

#### Finding 2: Missing allow_duplicate_modes positive test

**Severity**: MEDIUM
**File**: `crates/engine/tests/modal.rs`
**CR Rule**: 700.2d -- "However, some modal spells include the instruction 'You may choose the same mode more than once.' If a particular mode is chosen multiple times, the spell is treated as if that mode appeared that many times in sequence."
**Issue**: The `allow_duplicate_modes: bool` field is correctly wired into the duplicate check in `casting.rs:2835`, and the negative test (test 7: duplicate rejected on default spell) passes. However, there is no test that sets `allow_duplicate_modes: true` and verifies that choosing the same mode twice is accepted and the effect executes twice. The entire positive branch of CR 700.2d is untested.
**Fix**: Add a test `test_modal_allow_duplicate_modes` with a synthetic spell that has `allow_duplicate_modes: true`, cast it choosing `[0, 0]`, and verify the mode 0 effect executes twice (e.g., GainLife 3 twice = +6 life).

#### Finding 3: No TODO comment for deferred CR 700.2c

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:2867`
**CR Rule**: 700.2c -- "If a spell or ability targets one or more targets only if a particular mode is chosen for it, its controller will need to choose those targets only if they chose that mode."
**Issue**: Per-mode targeting is documented as deferred in the plan file, but there is no TODO comment in the actual casting.rs code near the mode validation block. A future developer reading casting.rs will not know that the target validation is mode-agnostic and that this is a known limitation. The plan file mentions this is documented in `blessed_alliance.rs:30-31` but the core casting validation code should also note it.
**Fix**: Add a `// TODO(CR 700.2c): Per-mode targeting not yet implemented. All targets are validated` / `// regardless of which modes are chosen. See ability-plan-modal-choice.md.` comment after the `validated_modes_chosen` block (around line 2867).

#### Finding 4: Vec hash without length prefix

**Severity**: LOW
**File**: `crates/engine/src/state/hash.rs:2006-2009`
**CR Rule**: N/A (architecture invariant: deterministic hashing)
**Issue**: The `modes_chosen` hash uses a manual `for` loop instead of calling `self.modes_chosen.hash_into(hasher)`, which would use the blanket `Vec<T>` impl that prefixes the length. Without a length prefix, `modes_chosen: vec![]` hashes identically to any other zero-element sequence, creating a theoretical collision risk when combined with adjacent fields. This is a pre-existing pattern (lines 1996-2005 do the same for `spliced_effects`, `spliced_card_ids`, `devour_sacrifices`).
**Fix**: Replace `for idx in &self.modes_chosen { idx.hash_into(hasher); }` with `self.modes_chosen.hash_into(hasher);`. Consider fixing the same pattern for the other manual-loop Vec fields in the same block as a separate LOW cleanup.

#### Finding 5: Missing escalate-with-explicit-modes test

**Severity**: LOW
**File**: `crates/engine/tests/modal.rs`
**CR Rule**: 700.2a + 702.120a interaction
**Issue**: The plan specified test 12 as "Cast an escalate spell with `modes_chosen: [0, 2]` and `escalate_modes: 1`. Verify modes 0 and 2 (not 0 and 1) execute." This test would verify that explicit `modes_chosen` indices take precedence over the implicit `0..=escalate_modes_paid` backward-compat path in resolution.rs. The actual test 12 (`test_modal_modes_chosen_stored_on_stack_object`) is useful but doesn't cover this interaction.
**Fix**: Add a test `test_modal_escalate_with_explicit_modes` that uses the `cast_spell_modal` path with a non-contiguous mode selection like `[0, 2]`, combined with `escalate_modes: 1`, and verifies the correct modes execute (not the 0..=N fallback).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 700.2a (choose modes at cast time) | Yes | Yes | tests 1-3, 6, 8-9 |
| 700.2b (modal triggered abilities) | No | No | Correctly deferred; no cards in pool use this |
| 700.2c (per-mode targeting) | No | No | Deferred; documented in plan but not in code (Finding 3) |
| 700.2d (no duplicate modes) | Yes (negative) | Yes (negative only) | test 7 covers rejection; positive path untested (Finding 2) |
| 700.2e (other player chooses mode) | No | No | No cards in pool use this; acceptable deferral |
| 700.2f (different targeting per mode) | No | No | Related to 700.2c; deferred |
| 700.2g (copies copy modes) | Yes | No | copy.rs:235 propagates modes_chosen; no test (Finding 1) |
| 700.2h (mode additional costs) | No | No | No cards in pool use this; acceptable deferral |
| 700.2i (pawprint modes) | No | No | Very new mechanic; acceptable deferral |
| 707.10 (copy decisions) | Yes | No | Same as 700.2g |
| 601.2b (announce modes) | Yes | Yes | Covered by all positive tests |
| 603.3c (modal triggered ability) | No | No | Same as 700.2b |
| Entwine interaction | Yes | Yes | test 10 |
| Escalate interaction | Yes | No (explicit) | Backward-compat path tested via existing scripts; explicit modes untested |
| Backward compat (empty modes) | Yes | Yes | test 5 |

## Summary of Action Items

1. **(MEDIUM)** Add `test_modal_copy_inherits_modes` -- storm spell with mode[1], verify copy has same modes_chosen
2. **(MEDIUM)** Add `test_modal_allow_duplicate_modes` -- spell with allow_duplicate_modes=true, cast with [0,0], verify double execution
3. **(MEDIUM)** Add TODO comment in casting.rs near line 2867 about deferred CR 700.2c
4. **(LOW)** Fix hash to use `self.modes_chosen.hash_into(hasher)` instead of manual loop
5. **(LOW)** Add `test_modal_escalate_with_explicit_modes` for escalate + explicit modes interaction

verdict: needs-fix
