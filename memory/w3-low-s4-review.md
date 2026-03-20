# W3-LOW Session 4 Review: Behavioral Fixes

**Review Status**: REVIEWED (2026-03-20)
**Branch**: `w3-low-s4-behavioral`
**Commit**: `9bfe46b W3: S4 LOW cleanup -- StackObject boilerplate, AdditionalCost single-pass, Corrupted condition fix`

## Changes Summary

| File | Lines Changed | Purpose |
|------|---------------|---------|
| `crates/engine/src/state/stack.rs` | +61 | New `StackObject::trigger_default()` constructor (MR-TC-25) |
| `crates/engine/src/rules/abilities.rs` | +45/-720 | Replace 14 verbose StackObject constructors with `trigger_default()` |
| `crates/engine/src/rules/casting.rs` | +88/-332 | Replace 5 verbose constructors + single-pass AdditionalCost extraction (MR-TC-24) |
| `crates/engine/src/rules/resolution.rs` | +30/-123 | Replace 2 verbose constructors (Cipher copy + Suspend free-cast) |
| `crates/engine/src/rules/replacement.rs` | +6/-17 | Delegate Corrupted check to shared `check_condition()` (MR-B12-07/08) |
| `crates/engine/src/effects/mod.rs` | +16/-32 | Formatting only (rustfmt re-wrapping) |
| `crates/engine/tests/card_def_fixes.rs` | +4/-11 | Formatting only |
| `crates/engine/tests/combat.rs` | +2/-4 | Formatting only |
| `crates/engine/tests/mana_pool.rs` | +4/-1 | Formatting only |
| `crates/engine/tests/protection.rs` | +4/-10 | Formatting only |

**Net**: -788 lines (361 added, 1149 removed)

## Issue Closures

| Issue | Title | Status |
|-------|-------|--------|
| MR-TC-25 | StackObject boilerplate triggers | CLOSED by this change |
| MR-TC-24 | Multiple AdditionalCost iterator scans | CLOSED by this change |
| MR-B12-07 | Duplicate Corrupted condition in replacement.rs | CLOSED by this change |
| MR-B12-08 | `_ => true` catch-all for conditions in replacement.rs | CLOSED by this change |

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| W3-S4-01 | **LOW** | `casting.rs:1081` | **Unused `_mutate_on_top` variable.** The single-pass extraction sets `_mutate_on_top` from `AdditionalCost::Mutate { on_top }` but the variable is never read (prefixed with `_` to suppress the warning). The old code had the same behavior (`_mutate_on_top` from `unwrap_or((None, false))`), so this is not a regression -- it is pre-existing dead code. Not a correctness issue since mutate resolution reads directly from `additional_costs`. **Fix:** Remove the variable entirely or add a comment explaining why it exists. | OPEN |
| W3-S4-02 | **INFO** | `stack.rs:413-471` | **Constructor is compile-time safe against field omissions.** `trigger_default()` uses exhaustive struct literal syntax (`Self { ... }`) so any new field added to `StackObject` will produce a compile error if not added to `trigger_default()`. This is the correct pattern -- no architectural risk from future field additions. | NOTED |

## Verification Checklist

1. **trigger_default() field completeness**: All 35 fields of `StackObject` are present in the constructor. Verified by comparing struct definition (lines 146-411) against constructor body (lines 434-470). The exhaustive struct literal ensures compile-time enforcement.

2. **Per-trigger mutations**: Sites that had non-default field values now correctly set them after calling `trigger_default()`:
   - `handle_activate_ability`: sets `targets` (spell_targets)
   - `handle_cycle_card`: no overrides needed (targets empty, all defaults)
   - `handle_activate_forecast`: sets `targets` (spell_targets)
   - `handle_activate_bloodrush`: sets `targets` (SpellTarget with target creature)
   - `handle_unearth_card`: no overrides needed
   - `handle_ninjutsu`: no overrides needed
   - `handle_embalm_card`: no overrides needed
   - `handle_eternalize_card`: no overrides needed
   - `handle_encore_card`: no overrides needed
   - `handle_crew_vehicle`: no overrides needed
   - `handle_saddle_mount`: no overrides needed
   - `handle_scavenge_card`: sets `targets` (SpellTarget with target creature)
   - `flush_pending_triggers` (Modular): sets `targets` (modular_targets)
   - `flush_pending_triggers` (Myriad/generic): sets `targets` (trigger_targets)
   - Storm/Gravestorm/Cascade/Casualty/Replicate (casting.rs): no overrides needed
   - Cipher copy (resolution.rs): sets `is_copy = true`
   - Suspend free-cast (resolution.rs): sets `was_suspended = true`

3. **AdditionalCost single-pass match**: All 14 variants of `AdditionalCost` are handled in the match. The match is exhaustive with no `_ =>` catch-all. `Sacrifice` is explicitly handled as a no-op (extracted later by context-specific code).

4. **check_condition() delegation**: `check_condition` was already `pub(crate)` in effects/mod.rs. The replacement.rs call site constructs an `EffectContext::new(controller, new_id, vec![])` which matches the function signature `(state: &GameState, condition: &Condition, ctx: &EffectContext)`. This correctly replaces the inline partial match that only handled `OpponentHasPoisonCounters` and silently defaulted all other `Condition` variants to `true`.

5. **No behavioral changes**: All 2229 tests pass. Clippy clean.

## Assessment

This is a clean, low-risk refactoring session. The three changes are:

- **MR-TC-25 (trigger_default)**: High-value mechanical refactoring. The exhaustive struct literal pattern means Rust's compiler enforces correctness -- any future field addition to `StackObject` that is not added to `trigger_default()` will fail to compile. No behavioral risk.

- **MR-TC-24 (single-pass AdditionalCost)**: Marginal performance improvement (eliminates ~12 redundant iterator scans) with identical semantics. The match is exhaustive against all `AdditionalCost` variants.

- **MR-B12-07/08 (check_condition delegation)**: Genuine correctness improvement. The old code had a `_ => true` catch-all that silently treated all non-Corrupted conditions as satisfied at trigger time. The new code delegates to the shared `check_condition()` which handles all `Condition` variants. This fixes a latent bug where any CardDef with a non-Corrupted `intervening_if` condition would incorrectly fire its ETB trigger.

**Verdict**: No HIGH or MEDIUM findings. One pre-existing LOW (dead variable). Ready to merge.
