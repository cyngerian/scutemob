---
pb: PB-AC4
title: Modal & optional targeting
phase: backfill
plan_file: memory/primitives/pb-plan-AC4.md
review_file: memory/primitives/pb-review-AC4.md
---

# PB-AC4 — Modal & optional targeting

## Scope (from task brief, campaign-plan §2 PB-AC4 row)
1. **Per-mode `TargetRequirement` on `ModeSelection`** — each mode carries its own
   target requirements; targets are chosen only for the *chosen* modes (CR 601.2c).
   Wire through casting (target selection at cast time) and resolution
   (per-mode target lookup, illegal-target handling per CR 608.2b).
2. **Optional / `UpToN` target slots** — `TargetRequirement::UpToN` ALREADY EXISTS
   (shipped by PB-T, `card_definition.rs:2599`). **Do not re-implement.** Verify the
   full chain (cast-time min/max enforcement → resolution → fizzle rules) and extend
   only real residual gaps found by tracing that chain.

## Existing modal infra — STUDY BEFORE ADDING FIELDS
- `ModeSelection { min_modes, max_modes, modes: Vec<Effect>, allow_duplicate_modes,
  mode_costs: Option<Vec<ManaCost>> }` — `card_definition.rs:3397`
- `modes_chosen` on `CastSpell` + `StackObject` (Batch 11, Modal Choice CR 700.2)
- Spree `mode_costs`; Fuse; Escalate `escalate_modes`

## CR refs (VERIFY via mtg-rules MCP — advisory only)
- 601.2c (announce targets; "up to N"), 700.2 (modal spells), 700.2d (duplicate modes),
  608.2b (illegal targets on resolution), 115.x (targeting)

## Roster (planner derives the REAL list)
Grep card defs for BOTH `// TODO` and `// ENGINE-BLOCKED` markers citing modal /
mode-target / up-to-N / optional-target patterns. Discounted yield ~20 cards.
Card rosters in the plan doc are advisory — oracle text via MCP is authoritative.

## Hazards (from task brief)
1. Verify KW/AbilDef/SOK discriminant chain from *current code* before adding variants.
2. New struct fields / mutable runtime fields MUST be added to `state/hash.rs` HashInto
   impls (PB-AC1 review HIGH was exactly this — `ModeSelection` changes are hash-relevant;
   bump `HASH_SCHEMA_VERSION` + update the sentinel in all test files).
3. Exhaustive matches in `tools/tui/src/play/panels/stack_view.rs` AND
   `tools/replay-viewer/src/view_model.rs` (StackObjectKind + KeywordAbility) —
   verify with `cargo build --workspace` after every impl phase.
4. Do NOT commit phantom `.claude/skills/*` deletions that appear in fresh worktrees.
5. Harness: new cast/target actions may need `script_schema.rs` + `translate_player_action`
   wiring.
6. Load `memory/gotchas-rules.md` before planning — this batch touches casting/targeting.

## Phases
- [x] plan  (primitive-impl-planner → pb-plan-AC4.md, 2026-07-08)
- [x] implement (primitive-impl-runner, 2026-07-08)
  - **Engine field**: `ModeSelection.mode_targets: Option<Vec<Vec<TargetRequirement>>>`
    added to `crates/engine/src/cards/card_definition.rs` (after `mode_costs`, with
    `#[serde(default)]` + author-invariant doc comment: length == `modes.len()`,
    `Spell.targets` must be empty when `Some`, no nested `UpToN`).
  - **Hash**: `crates/engine/src/state/hash.rs` — `impl HashInto for ModeSelection` hashes
    the new field (nested `Option<Vec<Vec<TargetRequirement>>>`, len-prefixed per inner
    vec). `HASH_SCHEMA_VERSION` bumped 30→31 with changelog entry. Updated the
    `HASH_SCHEMA_VERSION, 30u8` sentinel to `31u8` in all 20 test files that assert it
    (mechanical `sed`, verified via grep afterward — 0 remaining `, 30u8` references).
  - **Cast-time validation** (`crates/engine/src/rules/casting.rs`): moved the
    `validated_modes_chosen` computation (previously ~line 4071, well after target
    validation) to BEFORE the target-requirements lookup (~line 3341), since per-mode
    target validation needs the fully-validated ascending-sorted chosen-mode list. Added
    `mode_selection_opt` lookup (fetches `ModeSelection` regardless of whether
    `modes_chosen` is empty, so `mode_targets` is available for the auto-select-mode-0 and
    entwine paths too) and `mode_targets_active: Option<Vec<TargetRequirement>>` (computes
    `concat(mode_targets[m] for m in chosen-mode order)` — covers Entwine [all modes],
    explicit `validated_modes_chosen`, and auto-select-mode-0; Escalate + `mode_targets` is
    an undocumented/unsupported combination, flagged in a code comment per
    "implement-phase default-to-defer" — no AC4-scoped card uses it). When
    `mode_targets_active` is `Some`, validates targets via a NEW positional path instead of
    the flat best-fit path (author-invariant checks: `Spell.targets` must be empty, no
    `UpToN` inside `mode_targets`).
  - **New functions** (`casting.rs`): `validate_targets_positional` (exact-count +
    positional req-for-target mapping — required because slice offsets depend on
    declaration order matching `mode_targets` order, NOT best-fit reassignment) and
    `validate_mapped_targets` (extracted shared tail of `validate_targets_inner`: the
    per-target protection/hexproof/type-check loop, now reused by both the best-fit and
    positional paths).
  - **Resolution per-mode slicing** (`crates/engine/src/rules/resolution.rs`): refactored
    the mode-dispatch block to first compute `chosen_mode_indices: Vec<usize>` (the
    ascending index list itself, not resolved effects) via the same Entwine/explicit
    `modes_chosen`/Escalate/auto-mode-0 priority order as before (byte-identical
    selection semantics — verified via full regression pass). `effects_to_run` is now only
    used when `mode_targets.is_none()` (legacy path, unchanged). When
    `mode_targets.is_some()`, the effect-execution loop instead iterates
    `chosen_mode_indices`, slicing the RAW `stack_obj.targets` (NOT the pre-filtered
    `legal_targets`) per chosen mode using a running `offset`, setting `ctx.targets` to
    each mode's local slice before executing `modes.modes[idx]`. This sidesteps the
    pre-existing `legal_targets` compaction hazard (documented in the plan) — illegal
    targets are instead skipped per-target inside `resolve_effect_target_list_indexed`'s
    existing object/player-existence check, which is CR-equivalent to the CR 608.2b
    zone-match check since any zone change kills the old `ObjectId` (CR 400.7). The
    pre-existing whole-spell "all targets illegal → fizzle" check (earlier in
    `resolve_top_of_stack`, on the flat `stack_obj.targets`) is untouched and still runs
    first — full fizzle behaves identically for `mode_targets` spells.
  - **Card defs (mechanical only, no card-logic changes)**: added `mode_targets: None,` to
    all 36 existing `ModeSelection { ... }` struct literals in `crates/engine/src/cards/defs/`
    (found via `grep -rl "ModeSelection {"`) so they keep compiling under the new required
    field. Also fixed the same pattern in 6 pre-existing test files that construct
    `ModeSelection` literals: `modal.rs` (4 sites), `entwine.rs`, `spree.rs`, `escalate.rs`
    (1 site each). **No card oracle-text/DSL logic was changed** — `cryptic_command.rs`
    (which uses `modes: None`, a stub) was untouched, confirming the mechanical-only scope
    was respected. Card backfill (migrating `izzet_charm`, `casualties_of_war`,
    `cryptic_command`, `abzan_charm`, `blessed_alliance`, etc. to actually USE
    `mode_targets`) is deferred to the `backfill` phase per task scope.
  - **Tests**: new file `crates/engine/tests/pb_ac4_per_mode_targeting.rs` — 10 tests, all
    citing CR rules: `test_601_2c_modal_targets_only_for_chosen_mode` (Casualties-of-War
    wrong-game-state fix — castable choosing 1 mode when other modes' target types don't
    exist on the battlefield), `test_700_2c_unchosen_mode_targets_not_required`,
    `test_601_2c_wrong_type_target_rejected_per_mode` (positional type check),
    `test_700_2f_two_modes_two_targets_sliced_independently` (no cross-mode
    contamination), `test_608_2b_modal_partial_illegal_target_skips_only_that_mode`,
    `test_608_2b_modal_all_targets_illegal_fizzles`,
    `test_700_2d_duplicate_modes_independent_target_slices`,
    `test_ac4_hash_distinguishes_mode_targets` (+ live `HASH_SCHEMA_VERSION == 31`
    sentinel), `test_ac4_backward_compat_mode_targets_none_unaffected`,
    `test_700_2c_multiplayer_choose_subset_across_opponents` (4-player). All use synthetic
    test-only `CardDefinition`s (`Modal Strike`, `Duplicate Destroy`, `Legacy Modal Spell`,
    `Mandatory Destroy Creature`) per task scope (no real card defs modified beyond the
    mechanical `mode_targets: None` backfill).
  - **Deviations from plan**: none substantive. The plan's design (field-only, no new
    discriminants, resolution-time per-mode slicing over raw `stack_obj.targets`) was
    followed exactly, including the two rejected-alternative call-outs (no `Mode` struct
    replacing `modes: Vec<Effect>`; no global-slot-index sparse representation). One
    addition beyond the plan's literal step list: extracted `validate_mapped_targets` as a
    shared helper (not explicitly named in the plan) to avoid duplicating the
    protection/hexproof/type-check loop between the best-fit and positional validators —
    a refactor-only, zero-behavior-change addition, not new primitive surface.
  - **Gates**: `cargo build --workspace` clean (confirmed 0 new match arms needed in
    `tools/tui/src/play/panels/stack_view.rs` / `tools/replay-viewer/src/view_model.rs`,
    per the plan's prediction — no new enum variants were added). `cargo test --all`:
    2950 passed / 0 failed (2940 baseline + 10 new; all pre-existing modal/entwine/
    spree/escalate/UpToN regression suites re-verified green). `cargo clippy --all-targets
    -- -D warnings` clean. `cargo fmt --check` clean.
- [x] review (primitive-impl-reviewer → pb-review-AC4.md, 2026-07-08) — verdict needs-fix:
  1 MEDIUM + 3 LOW findings (no HIGH).
- [x] fix (primitive-impl-runner, 2026-07-08)
  - **Finding 1 (MEDIUM)** — Escalate + `ModeSelection.mode_targets` not fail-safe (cast-time
    `mode_targets_active` had no Escalate branch; resolution's `chosen_mode_indices` did,
    which would silently under-resolve escalated modes past 0 with empty target slices).
    **Fixed**: hard-rejected the combination at cast time
    (`crates/engine/src/rules/casting.rs:3526-3530`) —
    `if mode_targets_active.is_some() && escalate_modes > 0 { return
    Err(GameStateError::InvalidCommand("Escalate combined with ModeSelection.mode_targets is
    not supported (CR 700.2c/702.120a)")) }`, placed after `mode_targets_active` is computed
    and before mana payment. New test
    `test_700_2c_702_120a_escalate_with_mode_targets_rejected_at_cast`
    (`crates/engine/tests/pb_ac4_per_mode_targeting.rs`, new "Escalate Modal Strike" card def)
    asserts the rejection AND that the identical card casts/resolves normally without
    Escalate paid (proves the reject is scoped to the Escalate combination only).
  - **Finding 2 (LOW)** — `.expect()` in engine logic at `resolution.rs:446` (provably
    unreachable but violated the no-`expect()` convention). **Fixed**: restructured to
    nested `if let Some(modes_ref) = spell_modes.as_ref() { if let Some(mode_targets) =
    modes_ref.mode_targets.as_ref() { ... } else { <effects_to_run fallback> } } else {
    <effects_to_run fallback> }` (`resolution.rs:441-486`) — both fallback arms preserve the
    original `effects_to_run` loop behavior.
  - **Finding 3 (LOW)** — `mode_targets.len() == modes.len()` author invariant documented but
    unenforced at `casting.rs:3497` / `resolution.rs:450`. **Fixed**: added
    `debug_assert_eq!(mt.len(), ms.modes.len(), ...)` at both the cast-time
    (`casting.rs:3491-3498`) and resolution-time (`resolution.rs:449-456`) sites, matching
    the engine's existing `debug_assert!`-plus-fail-safe-fallback pattern (e.g.
    `layers.rs:1731`, `abilities.rs:8001`). Release builds keep the pre-existing
    `unwrap_or_default()`/`unwrap_or(0)` fallback — no panic possible outside debug/test
    builds.
  - **Finding 4 (LOW)** — existence-vs-zone-match route-around; reviewer's own verdict was
    "Fix (optional): none required." **Deferred/note-only** — no code change; documentation
    already present in-code per the review.
  - **Concern 5** (other author invariants: no nested `UpToN` in `mode_targets`;
    `Spell.targets` empty when `mode_targets` is `Some`) — verified already hard-enforced at
    cast time (`casting.rs:3513-3525`, pre-existing from the implement phase); no action
    needed.
  - **Gates**: `cargo build --workspace` clean (0 new match-arm gaps — no new enum variants
    were introduced by the fix phase). `cargo test --all` (mtg-engine crate): **2951 passed /
    0 failed** (2950 implement-phase baseline + 1 new Finding-1 regression test).
    `cargo clippy --all-targets -- -D warnings` clean. `cargo fmt --check` clean.
- [ ] backfill (bulk-card-author + card-batch-reviewer)
- [ ] close
