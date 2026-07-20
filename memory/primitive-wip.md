# Primitive WIP — PB-RS1 (OOS-RS-1) · IN PROGRESS

<!-- last_updated: 2026-07-19 -->

- **PB**: PB-RS1 — library ordering reconciliation (reveal/scry family reads the opposite end from draw)
- **Task**: `scutemob-143`
- **Branch**: `feat/pb-rs1-reconcile-library-topbottom-revealscry-family-reads-t`
- **Class**: CORRECTNESS (Invariant #9 — live-wrong on shipped `Complete` defs)
- **Phase**: review
- **Binding spec**: `memory/primitives/rider-seed-triage-2026-07-19.md` §5 (full spec), §2.1 (chain notes)
- **Plan file**: `memory/primitives/pb-plan-RS1.md`
- **Review file**: `memory/primitives/pb-review-RS1.md`
- **Wire expectation**: NONE — no PROTOCOL/HASH bump. If a schema fingerprint must be
  re-pinned, STOP and re-scope.

## Steps

- [x] 0. Step-0 probe (write FIRST, before any edit): ≥3-card library via `GameStateBuilder`;
      compare `draw_card` vs `Effect::Scry { count: 1 }` / `RevealAndRoute { count: 1 }`.
      Must FAIL pre-fix, PASS post-fix. Kept as permanent regression.
      DONE: `crates/engine/tests/mechanics_m_z/library_ordering.rs` (5 tests: 3 probes +
      Test 3 cascade round-trip + Test 4 scry-to-bottom, written together). All 5 FAILED
      pre-fix (verbatim output in `pb-plan-RS1.md` §16). Registered in `main.rs`.
- [x] 1. CR decision (103.2 / 121.1 / 701.19 / 702.85a) recorded in the plan.
      Camp A authoritative (top = last element, bottom = index 0), per plan §2.
- [x] 2. `Zone::top_n(n) -> Vec<ObjectId>` in `crates/card-types/src/state/zone.rs` next to `top()`.
      4 unit tests added in a co-located `#[cfg(test)] mod tests`, all pass.
- [x] 3. Rewire all FOUR reads: Scry `:3083-3087`, Surveil `:3117-3121` (DID NOT DROP),
      RevealAndRoute `:4982-4986`, LookAtTopThenPlace `:5076-5080`. Fixed CR 701.18→701.22
      citations at `effects/mod.rs:3075` and `card_definition.rs:1658`.
- [x] 4. Route the bottom-writes through `move_object_to_bottom_of_zone`: Scry (direct,
      `expect_move_object_to_bottom_of_zone`), RevealAndRoute unmatched_dest + LookAtTopThenPlace
      rest_to (§5c Option 1, local dispatch on `LibraryPosition::Bottom`). Surveil has no
      bottom-write (writes to graveyard) -- no action, per Finding B.
- [x] 5. Reconciled `testing/replay_harness.rs:207-212` comment + insert loop (`.iter().rev()`).
- [x] 6. Tests 2-5: de-vacuous `reveal_and_route.rs` (4 tests rewritten with bottom
      decoys + identity assertions), cascade round-trip (`test_cascade_bottomed_card_is_not_seen_by_next_scry`,
      real cascade path via CastSpell), scry-to-bottom ordering (`test_scry_two_to_bottom_lands_below_everything`),
      golden-script reconciliation (5 scripts fixed: 002, 012, 018, 199, 034 — see close-out for
      the ObjectId-tiebreak mechanism; 2 harness_equivalence direct-builder fixtures also fixed:
      delve, modal; 1 stale-convention rust test also fixed: pb_os8 truncation test).
- [x] 7. Roster sweep from `all_cards()` (NOT grep) — 41 distinct cards (Scry 19, Surveil 9,
      RevealAndRoute 12, LookAtTopThenPlace 2). Delta vs 47 grep baseline explained in close-out
      (grep over-counts TODO/comment mentions on 6 blocked cards; enumeration is exact).
- [x] 8. Filed `ZoneTarget::Library { position }` gap as **OOS-RS1-1** in
      `memory/primitives/rider-seed-triage-2026-07-19.md` §1c. NOT fixed here.
- [x] 9. Gates: `cargo test --all` green (0 failures across all suites), `clippy --all-targets -D
      warnings` clean, `cargo fmt --check` clean (after `cargo fmt` auto-fixed 2 new test files),
      `tools/check-defs-fmt.sh` clean (1804 defs), `cargo build --workspace` green (TUI +
      replay-viewer included, no exhaustive-match gaps — expected, no new enum variant added).
- [ ] 10. Review by `primitive-impl-reviewer`; disposition findings.

## Notable fallout discovered and fixed during implementation

- **5 golden scripts** flipped because ObjectId assignment order within libraries moved
  (SR-9b expected fallout, §8 of the plan): `etb-triggers/002_solemn_simulacrum_fetches_land.json`,
  `stack/012_cultivate_ramps_two_lands.json`, `stack/018_kodamas_reach_two_lands.json`,
  `stack/199_sakura_tribe_elder_search.json`, `stack/034_brainstorm_then_fetch.json`. All were
  ObjectId-ascending tie-break flips in `SearchLibrary`/`PutOnLibrary`'s deterministic fallback,
  not top/bottom-read bugs themselves — fixed by reordering each script's declared library array
  (documented per-script in `generation_notes`), never by touching assertions to match a wrong
  outcome.
- **2 `harness_equivalence.rs` direct-builder fixtures** (`delve_direct`, `modal_direct`) had to
  have their library `.object()` push order reversed to match the harness's now-correct
  top-to-bottom-declared convention.
- **1 stale-convention Rust test** (`pb_os8_look_at_top_then_place.rs::test_look_place_truncates_at_top_n_leaves_out_of_window_match_untouched`)
  had silently encoded the OLD (pre-fix) bottom-read convention in its setup — fixed by reversing
  its push order and updating the doc comment, per the "a test that still passes is not evidence
  of correctness" principle from `reveal_and_route.rs`'s own de-vacuous rationale.

## Explicitly out of scope

- `ZoneTarget::Library { position }` discard / `LibraryPosition` zero-read sites — follow-up seed.
- Any card-def marker flips — this PB repairs behavior only.
- Authoring muxus (gated on this PB, but not part of it).

## Prior state (PB-OS queue, for reference)

THE PB-OS QUEUE IS COMPLETE (PB-OS1..OS11 + OS4b, `scutemob-116`..`141`). Rider-seed
mini-triage DONE (`scutemob-142`); canonical ranked queue R1..R11 lives in
`memory/primitives/rider-seed-triage-2026-07-19.md` §3.
