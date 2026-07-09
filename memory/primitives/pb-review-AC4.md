# Primitive Batch Review: PB-AC4 — Modal & Optional Targeting

**Date**: 2026-07-08
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 601.2c, 700.2 (700.2a/700.2c/700.2d/700.2f), 608.2b, 400.7
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (`ModeSelection.mode_targets` field)
- `crates/engine/src/rules/casting.rs` (`mode_selection_opt`, `mode_targets_active`, `validate_targets_positional`, `validate_mapped_targets`)
- `crates/engine/src/rules/resolution.rs` (`chosen_mode_indices`, per-mode slicing)
- `crates/engine/src/state/hash.rs` (schema 30→31, `ModeSelection` HashInto)
- `crates/engine/src/effects/mod.rs` (`resolve_effect_target_list_indexed` — dispatch tail, unchanged)
**Card defs reviewed**: 36 modal card defs (mechanical `mode_targets: None,` only) + 4 test-file `ModeSelection` literals
**Tests reviewed**: `crates/engine/tests/pb_ac4_per_mode_targeting.rs` (10 tests)

## Verdict: needs-fix (fix phase COMPLETE — see `## Fix-Phase Resolution` below)

The primitive is well-designed and, for the code paths any current card exercises, CR-correct.
The full dispatch chain (card def → `mode_targets_active` cast-time concat → positional
validation → `stack_obj.targets` per-mode raw slicing at resolution → partial/full fizzle)
was traced end-to-end and the cast-time target order matches the resolution slicing order for
Entwine, explicit `modes_chosen`, duplicate-mode, and auto-mode-0 paths. Hash is correct and
fully sentinel-bumped. Tests are genuine (T1 and T5 in particular truly exercise the
wrong-game-state fix and the partial-fizzle path). **No HIGH findings.** One MEDIUM (an
unenforced, non-fail-safe Escalate + `mode_targets` combination that silently under-resolves
for a future author) and three LOW findings (an `.expect()` in engine logic, an unenforced
length invariant, and a documentation caveat on the existence-vs-zone-match route-around).
Because at least one finding exists, the verdict is needs-fix.

## Engine Change Findings

| # | Severity | File:Line | Description | Status |
|---|----------|-----------|-------------|--------|
| 1 | **MEDIUM** | `casting.rs:3481-3501` vs `resolution.rs:309-343` | **Escalate + `mode_targets` is not fail-safe.** Cast-time `mode_targets_active` has no Escalate branch; resolution's `chosen_mode_indices` does. A future spell combining them (empty `modes_chosen`) validates targets for mode 0 only but resolves modes `0..=escalate_count`, silently under-resolving. **Fix:** reject the combination at cast time (hard error) OR mirror the Escalate branch into `mode_targets_active` so cast and resolution stay in lockstep. | **FIXED** — hard-rejected at cast time. `casting.rs:3526-3530`: `if mode_targets_active.is_some() && escalate_modes > 0 { return Err(GameStateError::InvalidCommand(...)) }`, placed after `mode_targets_active` is computed and before mana payment (so no mana funding is needed to trigger it). Test: `test_700_2c_702_120a_escalate_with_mode_targets_rejected_at_cast` (`pb_ac4_per_mode_targeting.rs`, new "Escalate Modal Strike" card def) — asserts the cast is rejected with the Finding-1 error text, AND a sanity-check second cast of the same card *without* Escalate paid still resolves normally (proving the reject is specific to the Escalate combination, not a regression of the base `mode_targets` path). |
| 2 | LOW | `resolution.rs:446` | **`.expect()` in engine logic.** Provably unreachable (guarded by the enclosing `if let Some(_) = spell_modes.as_ref().and_then(...)`), but violates the "never `expect()` in engine logic" convention. **Fix:** restructure to `if let Some(modes_ref) = spell_modes.as_ref() { if let Some(mode_targets) = modes_ref.mode_targets.as_ref() { ... } }`. | **FIXED** — restructured exactly as suggested (`resolution.rs:441-486`); the inner `else` (mode_targets is None) and outer `else` (spell_modes is None) both fall through to the pre-existing `effects_to_run` loop, preserving original semantics. `cargo build`/`cargo test` confirm no behavior change. |
| 3 | LOW | `casting.rs:3497`, `resolution.rs:450` | **`mode_targets.len() == modes.len()` invariant is documented but unenforced.** Both sites use `.get(idx)...unwrap_or_default()` / `unwrap_or(0)`, so a too-short `mode_targets` silently gives that mode zero targets (fails relatively safe — mode no-ops — but is wrong game state on an authoring error). **Fix:** add a `debug_assert_eq!(mode_targets.len(), modes.len())` at cast time, or reject when a chosen mode index is out of `mode_targets` range. | **FIXED** — `debug_assert_eq!(mt.len(), ms.modes.len(), ...)` added at both sites (`casting.rs:3491-3498`, `resolution.rs:449-456`), matching the existing engine-wide `debug_assert!`-plus-fail-safe-fallback pattern (e.g. `layers.rs:1731`, `abilities.rs:8001`). Release builds keep the existing `unwrap_or_default()`/`unwrap_or(0)` fallback — panics only in debug/test builds, never in release. |
| 4 | LOW | `resolution.rs:432-463` | **Existence-vs-zone-match route-around is correct but relies on an unasserted invariant.** The per-mode path skips illegal targets via `resolve_effect_target_list_indexed`'s object/player *existence* check, while the legacy full-fizzle check (`is_target_legal`, `resolution.rs:7643-7650`) uses *zone-match*. These are equivalent only because CR 400.7 gives every zone-changed object a new `ObjectId`. Verified acceptable (see analysis). **Fix (optional):** none required; the reliance is documented in-code. Note it does NOT newly introduce the pre-existing protection/hexproof/restriction re-check gap — that gap affects the legacy path equally and is out of AC4 scope. | **NOTE-ONLY, no code change** — already documented in-code and CR-correct per the reviewer's own verification; no fix required. |

## Card Definition Findings

No card-def defects. All 36 modal card defs received a mechanical `mode_targets: None,` and
remain on the byte-identical legacy path. Verified `casualties_of_war.rs`, `izzet_charm.rs`
still carry `mode_targets: None` plus their pre-existing union-workaround TODOs (backfill is an
explicitly separate deferred phase). No half-migrations.

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | INFO | (all 36) | Mechanical `None` only; legacy behavior preserved. The migration candidates (`casualties_of_war`, `izzet_charm`, `abzan_charm`, `blessed_alliance`, the charm family, `cryptic_command`) **still produce the pre-AC4 wrong/limited game state** (union-declaration / stub). This is expected — the primitive shipped, real-card backfill is the next phase. Consequence: the primitive currently has **zero real-card coverage**; all validation is synthetic test cards. Acceptable per the phased plan, but the batch is not "done" for gameplay until backfill lands. |

### Finding Details

#### Finding 1: Escalate + mode_targets not fail-safe (MEDIUM)

**Severity**: MEDIUM
**File**: `casting.rs:3481-3501` (cast-time `mode_targets_active`) vs `resolution.rs:309-343` (`chosen_mode_indices`)
**CR Rule**: 700.2c — "its controller will need to choose those targets only if they chose that
mode"; 601.2c (targets announced at cast for the chosen modes). The failure is that cast-time
target validation and resolution-time slicing use *different* chosen-mode derivations.
**Issue**: The two priority ladders diverge on the Escalate case:

- Cast (`casting.rs:3486`): `entwine_paid → validated_modes_chosen (non-empty) → [0] → []`.
  **No Escalate branch.**
- Resolution (`resolution.rs:312`): `entwine → modes_chosen (non-empty) → escalate_count>0 →
  [0] → []`. **Has an Escalate branch.**

A future card with an `EscalateModes` additional cost AND `mode_targets: Some(...)` cast via the
backward-compat Escalate path (empty `modes_chosen`) validates targets for mode 0 only, then at
resolution iterates `0..=escalate_count`. The running `offset` overruns `stack_obj.targets`, so
`stack_obj.targets.get(offset..offset+slice_len)` returns `None → unwrap_or_default()` → empty
slice for every escalated mode beyond 0. Those modes silently do nothing.

**Concrete failure scenario**: hypothetical "Escalate — Choose one or more; destroy target
creature / destroy target artifact" authored with `mode_targets`. Player pays Escalate for a 2nd
mode; only the first mode's target is validated at cast; the escalated 2nd mode resolves with an
empty target and destroys nothing — no error, no fizzle, just a missing effect.

No shipped card triggers this (every Escalate card keeps `mode_targets: None`). The implementer
documented it in a code comment but did **not** enforce it. Per `conventions.md`
"implement-phase default-to-defer," deferring the *feature* is correct — but an unenforced
invariant that yields silent wrong game state should still fail safe.

**Fix**: at cast time, return `GameStateError::InvalidCommand` when a spell has both an
`AdditionalCost::EscalateModes`/Entwine-incompatible Escalate cost and `ModeSelection.mode_targets
== Some`; OR add the Escalate branch to `mode_targets_active` so the two ladders are identical.
A hard reject is the smaller, safer change.

#### Finding 2: `.expect()` in engine logic (LOW)

**Severity**: LOW
**File**: `resolution.rs:446`
**Invariant**: `conventions.md` — "Engine crate uses typed errors — never `unwrap()` or
`expect()` in engine logic."
**Issue**: `spell_modes.as_ref().expect("mode_targets is Some implies spell_modes is Some")` is
provably unreachable (the enclosing `if let Some(mode_targets) = spell_modes.as_ref().and_then(...)`
guarantees `spell_modes` is `Some`), so it cannot panic on any real state. It still violates the
no-`expect` convention and is trivially avoidable.
**Fix**: restructure to bind the `ModeSelection` in the outer `if let`:
`if let Some(modes_ref) = spell_modes.as_ref() { if let Some(mode_targets) = modes_ref.mode_targets.as_ref() { ... } }`.

#### Finding 3: Unenforced `mode_targets.len() == modes.len()` (LOW)

**Severity**: LOW
**File**: `casting.rs:3497` (`mt.get(idx).cloned().unwrap_or_default()`), `resolution.rs:450`
(`mode_targets.get(idx).map(|v| v.len()).unwrap_or(0)`)
**Issue**: The field doc comment states length must equal `modes.len()`, but nothing asserts it.
A too-short `mode_targets` makes any chosen mode past its end resolve with zero targets — a
silent no-op on that mode. Cast and resolution agree (both treat missing as 0), so it does not
mis-target, but it is wrong game state on an authoring error with no diagnostic.
**Fix**: `debug_assert_eq!(mt.len(), ms.modes.len())` when computing `mode_targets_active`, or a
cast-time validation error if any chosen index is out of `mode_targets` range.

#### Finding 4: Existence-vs-zone-match route-around (LOW / verified acceptable)

**Severity**: LOW (informational — verified correct)
**File**: `resolution.rs:432-463`; compare `is_target_legal` at `resolution.rs:7636-7652` and
`resolve_effect_target_list_indexed` at `effects/mod.rs:5878-5906`
**CR Rule**: 608.2b (illegal-target skip) + 400.7 (new object identity on zone change)
**Issue/verification**: The per-mode path deliberately slices the RAW `stack_obj.targets` (not
the `legal_targets`-compacted list, avoiding the pre-existing compaction index-shift hazard) and
relies on `resolve_effect_target_list_indexed`'s per-target *existence* check to skip illegal
targets. The legacy full-fizzle check uses *zone-match* (`Some(obj.zone) == zone_at_cast`). These
are equivalent because `move_object_to_zone` always mints a new `ObjectId` (CR 400.7), so a
zone-changed target's old id is absent from `state.objects`/`stack_objects` → skipped. Traced T5
(partial illegal) confirms the intended behavior. **Important non-regression note:** neither path
re-checks protection/hexproof/restriction-still-matches at resolution (`is_target_legal` only
checks zone); that is a pre-existing engine simplification affecting both paths equally — AC4 does
NOT worsen it, so it is out of scope here.
**Fix**: none required. Optionally add a `debug_assert` documenting the CR 400.7 reliance.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 601.2c (targets only for chosen modes) | Yes | Yes | `test_601_2c_modal_targets_only_for_chosen_mode`, `test_601_2c_wrong_type_target_rejected_per_mode` |
| 700.2c (per-mode target requirement) | Yes | Yes | T1, `test_700_2c_unchosen_mode_targets_not_required`, T10 multiplayer |
| 700.2d (duplicate modes → independent slices) | Yes | Yes | `test_700_2d_duplicate_modes_independent_target_slices` (`[0,0]` two creatures) |
| 700.2f (per-mode targets, no cross-contamination) | Yes | Yes | `test_700_2f_two_modes_two_targets_sliced_independently` |
| 608.2b partial illegal (skip only that mode) | Yes | Yes | `test_608_2b_modal_partial_illegal_target_skips_only_that_mode` — genuine (response spell kills mode-0 target, land still destroyed, no fizzle) |
| 608.2b full illegal (whole-spell fizzle) | Yes | Yes | `test_608_2b_modal_all_targets_illegal_fizzles` (asserts `SpellFizzled` + life unchanged) |
| 700.2a (illegal-mode gating) | Pre-existing | Indirect | Unchanged by AC4; mode-range/count/dup checks moved earlier (`casting.rs:3363-3416`), logic identical |
| 700.2g (copy carries chosen modes) | Pre-existing | No | `copy.rs` already clones `targets` + `modes_chosen`; per-mode slicing derived at resolution, so copies inherit correct slices for free. No new test — acceptable (no behavior change). |
| Escalate + mode_targets | No (deferred) | No | Finding 1 — not fail-safe; flag. |
| Backward compat (`mode_targets: None`) | Yes | Yes | `test_ac4_backward_compat_mode_targets_none_unaffected` |
| Hash (schema 31, field hashed) | Yes | Yes | `test_ac4_hash_distinguishes_mode_targets` (None vs Some vs different-Some all distinct) + sentinel `== 31` |

## Hash Verification

- `mode_targets` **is** hashed (`hash.rs:5269-5280`), inner vecs length-prefixed; it is the
  terminal field of `ModeSelection::hash_into`, so the omitted outer-vec length prefix is safe
  (no intra-field collision). `Some(vec![])` (`true`, 0 iterations) is distinct from `None`
  (`false`). Verified by T8.
- `HASH_SCHEMA_VERSION` bumped 30→31 with changelog (`hash.rs:230-235`). Grep confirms **zero**
  remaining `, 30u8` / `HASH_SCHEMA_VERSION, 30` sentinels; all 24 test-file assertions read `31u8`.
- Minor pre-existing note (not AC4-introduced): `mode_costs` and now `mode_targets` omit the outer
  length prefix; the `mode_costs → mode_targets` field boundary is theoretically ambiguous but
  requires a crafted `ManaCost` byte collision and is a replay-integrity hash, not security. Not a
  finding — flagged only for awareness.

## Card Def Summary

| Card(s) | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|---------|-------------|-----------------|-------------------|-------|
| 36 modal defs (mechanical `None`) | N/A (unchanged) | Unchanged | Same as pre-AC4 | Legacy path byte-identical; verified `casualties_of_war`, `izzet_charm` not half-migrated |
| casualties_of_war, izzet_charm, abzan_charm, blessed_alliance, charm family, cryptic_command | Still legacy union/stub | Yes (pre-existing) | **No** (unchanged wrong/limited state) | Backfill deferred — not an AC4 defect, but primitive has zero real-card coverage until then |

## Backward-Compatibility Assessment

The `mode_targets: None` path is behavior-identical to pre-change: `mode_selection_opt` is looked
up but `mode_targets_active` resolves to `None`, so cast dispatches to the unchanged
`validate_targets_with_source`, and resolution takes the unchanged `effects_to_run` loop. The
mode-choice validation was *moved earlier* (`casting.rs:3363`) and its result moved (consumed at
`stack_obj.modes_chosen`, `casting.rs:4304`); the validation logic itself is unchanged (range/dup/
count), so accept/reject outcomes are identical — only error-message ordering could differ, which
is immaterial. The 2950-test green run (2940 baseline + 10) with all modal/entwine/spree/escalate/
UpToN suites re-verified corroborates no regression. No modal card regressed.

## Fix-Phase Resolution (2026-07-08, primitive-impl-runner)

All 4 findings addressed:

- **Finding 1 (MEDIUM)**: FIXED — hard reject (option (a), the reviewer's preferred choice
  since no shipped card uses the combination). `mode_targets_active.is_some() && escalate_modes
  > 0` returns a typed `GameStateError::InvalidCommand` at `casting.rs:3526-3530`, citing CR
  700.2c/702.120a. New test `test_700_2c_702_120a_escalate_with_mode_targets_rejected_at_cast`
  (`crates/engine/tests/pb_ac4_per_mode_targeting.rs`) proves both the reject AND that the
  same `mode_targets` spell casts/resolves normally without Escalate paid.
- **Finding 2 (LOW)**: FIXED — `.expect()` removed; restructured to nested `if let` binding
  `spell_modes` in the outer arm, `mode_targets` in the inner arm (`resolution.rs:441-486`).
- **Finding 3 (LOW)**: FIXED — `debug_assert_eq!` added at both cast-time (`casting.rs:3491`)
  and resolution-time (`resolution.rs:449`) sites, following the existing
  `debug_assert!`-plus-fail-safe-fallback convention used elsewhere in the engine
  (`layers.rs:1731`, `abilities.rs:8001`). Release builds retain the pre-existing
  `unwrap_or_default()`/`unwrap_or(0)` fallback.
- **Finding 4 (LOW)**: NOTE-ONLY per the reviewer's own "Fix (optional): none required" —
  no code change made.
- **Concern 5 additional author invariants** (no nested `UpToN`; `Spell.targets` empty when
  `mode_targets` is `Some`): already hard-enforced at cast time
  (`casting.rs:3513-3525`, pre-existing from the implement phase) — verified, no further
  action needed.

**Gates**: `cargo build --workspace` clean. `cargo test --all` (engine crate): 2951 passed / 0
failed (2950 baseline + 1 new Finding-1 test). `cargo clippy --all-targets -- -D warnings`
clean. `cargo fmt --check` clean.
