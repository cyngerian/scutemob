# Primitive WIP — PB-RS1 (OOS-RS-1) · IN PROGRESS

<!-- last_updated: 2026-07-19 -->

- **PB**: PB-RS1 — library ordering reconciliation (reveal/scry family reads the opposite end from draw)
- **Task**: `scutemob-143`
- **Branch**: `feat/pb-rs1-reconcile-library-topbottom-revealscry-family-reads-t`
- **Class**: CORRECTNESS (Invariant #9 — live-wrong on shipped `Complete` defs)
- **Phase**: implement
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
- [ ] 6. Tests 2-5: de-vacuous `reveal_and_route.rs`, cascade round-trip, scry-to-bottom
      ordering, golden-script reconciliation.
- [ ] 7. Roster sweep from `all_cards()` (NOT grep) — report full affected set in close-out.
- [ ] 8. File `ZoneTarget::Library { position }` gap as a follow-up seed (NOT fixed here).
- [ ] 9. Gates: `cargo test --all`, `clippy -D warnings`, `cargo fmt --check` +
      `tools/check-defs-fmt.sh`.
- [ ] 10. Review by `primitive-impl-reviewer`; disposition findings.

## Explicitly out of scope

- `ZoneTarget::Library { position }` discard / `LibraryPosition` zero-read sites — follow-up seed.
- Any card-def marker flips — this PB repairs behavior only.
- Authoring muxus (gated on this PB, but not part of it).

## Prior state (PB-OS queue, for reference)

THE PB-OS QUEUE IS COMPLETE (PB-OS1..OS11 + OS4b, `scutemob-116`..`141`). Rider-seed
mini-triage DONE (`scutemob-142`); canonical ranked queue R1..R11 lives in
`memory/primitives/rider-seed-triage-2026-07-19.md` §3.
