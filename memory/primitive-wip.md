# Primitive WIP — PB-RS1 (OOS-RS-1) · FIX CYCLE COMPLETE + TEST-GAP CLOSED

<!-- last_updated: 2026-07-19 -->

- **PB**: PB-RS1 — library ordering reconciliation (reveal/scry family reads the opposite end from draw)
- **Task**: `scutemob-143`
- **Branch**: `feat/pb-rs1-reconcile-library-topbottom-revealscry-family-reads-t`
- **Class**: CORRECTNESS (Invariant #9 — live-wrong on shipped `Complete` defs)
- **Phase**: fix (complete — see "Fix cycle" section below)
- **Binding spec**: `memory/primitives/rider-seed-triage-2026-07-19.md` §5 (full spec), §2.1 (chain notes)
- **Plan file**: `memory/primitives/pb-plan-RS1.md`
- **Review file**: `memory/primitives/pb-review-RS1.md`
- **Wire expectation**: NONE — no PROTOCOL/HASH bump. If a schema fingerprint must be
  re-pinned, STOP and re-scope. **Confirmed unchanged after fix cycle: PROTOCOL 26 / HASH 63.**

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
- [x] 10. Review by `primitive-impl-reviewer`; disposition findings. Verdict:
      needs-fix (1 MEDIUM correctness, 1 MEDIUM citation, 4 LOW). Fix cycle
      applied 2026-07-19 -- see "Fix cycle" section below. All gates re-green
      after fixes; PROTOCOL 26 / HASH 63 unchanged.

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

## Fix cycle (2026-07-19)

Applied every finding from `memory/primitives/pb-review-RS1.md`. Every finding was
fixed (none declined).

### MEDIUM-1 (correctness) — a fifth top-N read, still inverted

`RestrictSearchTopN` (`crates/engine/src/effects/mod.rs` -- `SearchLibrary`, the
Aven Mindcensor search-restriction arm) computed "the top N" as the N *lowest*
ObjectIds (`all_lib.sort(); .take(n)`), which decouples from library position
after any shuffle/cascade-to-bottom/scry-to-bottom and, under this engine's
declaration convention, is the BOTTOM N. Live-wrong on `aven_mindcensor.rs`,
which is `Completeness::Complete`. **Fixed**: routed through
`Zone::top_n(top_n as usize)`, deleted the false "ObjectId ascending" comment,
cited CR 701.23/121.1. **Added a discriminating regression test**:
`test_restrict_search_top_n_reads_true_top_not_object_id_order` in
`crates/engine/tests/mechanics_m_z/library_ordering.rs` (Test 5) -- builds a
real `Aven Mindcensor` from `all_cards()`, a 5-card opponent library with the
only matching land (Swamp) at the TRUE bottom and 4 non-land fillers above it,
executes the real `Effect::SearchLibrary` with the replacement ability
registered, and asserts the search finds nothing (the Swamp stays in the
library). **Verified it fails on revert**: temporarily restored the old
ObjectId-proxy block, re-ran the test — it FAILED with exactly the expected
message; restored the fix, re-ran — PASSED. All 6 tests in `library_ordering.rs`
pass with the fix in place.

Fixing this required bumping the `src/effects/mod.rs` SR-25 bare-lookup-ratchet
ceiling in `crates/engine/tests/core/bare_lookup_ratchet.rs` from 110 to 111
(one new `state.zones.get(&lib_id)` NONSWALLOW predicate read, same shape as
the pre-existing PB-OS8 entry immediately above it in that file) — documented
inline with a new dated comment block, per that file's established convention.

### MEDIUM-2 (citation) — `GameEvent::Scried` still cites CR 701.18 (Play)

Fixed at `crates/engine/src/rules/events.rs:766`: `701.18` → `701.22`.

Per the fix-cycle brief, also grepped the whole workspace for remaining
`701.18`/`701.19` cites attached to Scry and `701.16`/`701.19a` cites attached
to Reveal/Search, and closed every one found in engine source, engine tests,
and test-data scripts (kept the "zero card-def diffs" invariant intact --
`crates/card-defs/` was NOT touched, even though several card-def files carry
the same stale `701.18`/`701.16a` comments; those are pre-existing and out of
this PB's scope per plan §7). Fixed, beyond the review's two explicit MEDIUM
findings:
- `crates/engine/src/effects/mod.rs:4969` (RevealAndRoute's own inline comment,
  missed by the original implement pass): `701.16a` → `701.20a`.
- `crates/engine/src/state/hash.rs:6396` (Scry hash-arm comment): `701.18` → `701.22`.
- `crates/engine/src/state/hash.rs:6598` (RevealAndRoute hash-arm comment): `701.16a` → `701.20a`.
- `crates/engine/tests/core/card_def_fixes.rs` (4 occurrences, module doc +
  section header + test doc + inline comment): `701.18` → `701.22`.
- `docs/mtg-engine-corner-case-audit.md` (living correctness ledger, 2 entries):
  Read the Bones `701.18` → `701.22`; Path to Exile `701.19` → `701.23`.
- `docs/mtg-engine-ability-coverage.md` (live coverage doc, 1 entry): Search
  library's `701.19a` → `701.23a`.
- Golden scripts (citation-only edits, verified no assertion touched — see
  "Script diff verification" below): `baseline/009_read_the_bones_scry_draw.json`
  (3 occurrences, `701.18`→`701.22`), `etb-triggers/205_nadaar_ventures_on_etb.json`
  (`701.18`→`701.22`), `etb-triggers/002_solemn_simulacrum_fetches_land.json`
  (`701.19`→`701.23`), `stack/199_sakura_tribe_elder_search.json` (3 occurrences:
  search `701.19`→`701.23` x2, `701.19a`→`701.23a`, plus a same-script Shuffle
  citation `701.19`→`701.24` found adjacent), `stack/034_brainstorm_then_fetch.json`
  (Shuffle citation `701.16`→`701.24`, found adjacent to the search citation).

**Explicitly left alone** (out of scope, disposition: found, not fixed):
- `crates/card-defs/*.rs` stale `701.18`/`701.16a` comments (temple_of_*,
  read_the_bones.rs, viscera_seer.rs, zhalfirin_void.rs) — fixing these would
  violate the plan's "zero card-def diffs" hard constraint (§7) and the
  fix-cycle's explicit gate #2 requiring that diff stay empty.
- `test-data/generated-scripts/stack/004_swords_to_plowshares_exiles_creature.json`'s
  `701.18` citation — attached to `exile_object`, not Scry/Reveal/Search; a
  different pre-existing miscitation (should be 701.9, Exile), out of this
  PB's citation family.
- Widespread `CR 701.16: Sacrifice` miscitations (`altar_of_dementia.rs`,
  `greater_good.rs`, `pbp_power_of_sacrificed_creature.rs`, `pb-plan-P.md`,
  `card_definition.rs:1493`'s `TapPermanent` citation) — CR 701.16 is
  Investigate, not Sacrifice (current Sacrifice is CR 701.21); this is a
  large, unrelated pre-existing citation bug (already partially flagged in
  `ability-review-investigate.md`'s own LOW finding) well outside PB-RS1's
  Scry/Surveil/Reveal/Search scope. Not touched.
- `docs/mtg-engine-milestone-reviews.md:991`'s `701.19` for SearchLibrary --
  that whole historical table uses a uniformly stale CR numbering scheme
  (`701.5`=Destroy, `701.6`=CreateToken, `701.7`=Discard, `701.13`=Mill,
  `701.20`=Shuffle, all differ from the current CR text), consistent with it
  being a point-in-time milestone snapshot rather than a live tracked doc;
  patching one entry among many stale ones would create false precision
  without fixing the pattern. Not touched.
- `memory/primitives/rider-seed-triage-2026-07-19.md`'s own `701.19 (Scry)`
  citations (the original triage brief's error, already fully diagnosed,
  quoted verbatim, and superseded by the plan's §0.2/§2 correction) --
  retroactively editing the source document would break the quote the plan's
  Finding A is built on. Not touched.
- `memory/primitives/pb-plan-RS1.md` / `pb-review-RS1.md` themselves and other
  point-in-time plan/review artifacts (`ability-plan-*.md`, `pb-review-12.md`,
  `pb-review-17.md`, `pb-plan-19.md`, `w-pb2-review-batch1.md`, archive files)
  — historical records of what was cited/found at each point in time, not
  live docs to retroactively rewrite.

### LOW findings 3–6 — batched into the same fix pass

- **LOW-3** (`crates/engine/src/rules/replacement.rs:340`): `701.19` → `701.23`
  (Search, not Regenerate).
- **LOW-4** (`crates/engine/tests/mechanics_m_z/reveal_and_route.rs:1`):
  module doc `701.16a` → `701.20a` (Reveal, not Investigate).
- **LOW-5** (bottom-dispatch asymmetry, `effects/mod.rs` `matched_dest` in
  RevealAndRoute and `destination` in LookAtTopThenPlace): added explanatory
  comments at both sites (no behavior change, per the review's "add a comment
  recording the asymmetry" option) — both are currently latent (no card def
  routes matched/placed cards to a library), and adding a live `matches!`
  branch would be an unrequested behavior change outside a narrow fix cycle.
- **LOW-6** (`crates/engine/tests/mechanics_m_z/library_ordering.rs:533-535`,
  inverted comment rationale in `test_scry_two_to_bottom_lands_below_everything`):
  rewrote the comment to correctly state Top1/Top2 are declared LAST (highest
  ObjectIds) and explain the push_front processing order that lands them at
  indices 0/1. The assertion itself was always correct; only the reasoning
  in the comment was inverted.

### Reviewer coverage gap — closed

The reviewer had no Bash tool, so `git diff main...HEAD` and the test suite
were never executed during review; this fix cycle closed that gap:

1. `git diff main...HEAD --stat` and `git diff main...HEAD -- test-data/`
   (plus `git diff` / `git status --porcelain` for the fix cycle's own
   uncommitted script edits): confirmed the 5 disclosed scripts
   (`etb-triggers/002`, `stack/012`, `stack/018`, `stack/199`, `stack/034`)
   plus 2 more touched in THIS fix cycle for pure citation corrections
   (`baseline/009`, `etb-triggers/205`) — every single diff line in every
   touched script is a `cr_ref`/`cr_sections_tested`/`generation_notes`/`note`
   string swap or (for the original 5) a setup/library-declaration reorder.
   Read every diff hunk directly: zero assertion fields (`assertions`,
   `expected`, counts, positions) were touched anywhere. No undisclosed
   *assertion* edits found.
2. `git diff main...HEAD -- crates/card-defs/` (and the fix cycle's own
   `git diff -- crates/card-defs/`): both empty. Confirmed.
3. `crates/card-types/src/cards/card_definition.rs`'s diff (implement +
   fix cycle combined): two hunks, both pure `///` doc-comment edits (Scry's
   CR cite, RevealAndRoute's CR cite + drift explanation). Confirmed
   comment-only.

### Gates (fix cycle)

`cargo test --all`: 0 failures (confirmed twice, before and after `cargo fmt`
auto-fixed one line in the new Test 5). `cargo clippy --all-targets -D
warnings`: clean. `cargo fmt --check`: clean (after one `cargo fmt` pass).
`tools/check-defs-fmt.sh`: clean (1804 defs). `cargo build --workspace`:
clean (TUI + replay-viewer included). `PROTOCOL_VERSION` = 26,
`HASH_SCHEMA_VERSION` = 63 — both confirmed unchanged by direct grep of the
`pub const` declarations.

## Test-coverage gap closure (2026-07-19, post-`/review`)

`/review` re-pass found all 5 ACs PASS and golden scripts clean, but flagged
one residual gap: the two new bottom-write dispatches
(`effects/mod.rs` RevealAndRoute `unmatched_dest`, LookAtTopThenPlace
`rest_to`) had no test that discriminates `expect_move_object_to_bottom_of_zone`
(correct, `push_front`) from `expect_move_object_to_zone` (wrong, `push_back`/
append) — every existing assertion on those paths checked membership/count,
not position, so reverting either dispatch would leave the whole suite green.

**Fix**: added positional (`object_ids()` index) coverage for both, citing
CR 121.1 (top = `Zone::top()` = `v.last()`; the last index of `object_ids()`
is the top of the library).

1. `crates/engine/tests/mechanics_m_z/reveal_and_route.rs` —
   `test_reveal_and_route_none_match` strengthened: replaced the old
   `lib_ids.contains(&decoy_id)` membership check with `lib_ids[4] ==
   decoy_id` (the decoy — the library's sole card pre-effect, hence its
   bottom — must land at the top/last index once the 4 unmatched Elves are
   bottomed beneath it) plus a set-equality check that the 4 Elves (fetched
   post-move by name, since CR 400.7 gives them new ObjectIds) occupy indices
   `0..4`.
2. `crates/engine/tests/primitives/pb_os8_look_at_top_then_place.rs` — new
   test `test_look_place_rest_to_bottom_positional_order`: 5-card library
   (1 bottom decoy land + a 4-card examined window: 3 lands + 1 creature),
   `count: 4`, creature placed to hand, 3 lands sent to `rest_to:
   Library{Bottom}`. Asserts `lib_ids[3] == decoy_id` (decoy shifted from
   index 0 to the top/last index once 3 rest-cards are bottomed beneath it)
   plus set-equality that the 3 bottomed lands occupy indices `0..3`.

**Verified both tests actually discriminate** — reverted each dispatch's
`if`/`else` arms in `effects/mod.rs` (swapped which branch calls
`expect_move_object_to_bottom_of_zone` vs `expect_move_object_to_zone`,
keeping the `matches!` condition itself untouched so no unused-variable
warning), re-ran, confirmed FAILURE, then restored and confirmed PASS:

- RevealAndRoute (`unmatched_dest`, `effects/mod.rs:5046-5051`): reverted →
  `test_reveal_and_route_none_match` FAILED with `left: ObjectId(9) right:
  ObjectId(1)` at the `lib_ids[4] == decoy_id` assertion. Restored → all 5
  `reveal_and_route::*` tests pass.
- LookAtTopThenPlace (`rest_to`, `effects/mod.rs:5199-5204`): reverted →
  `test_look_place_rest_to_bottom_positional_order` FAILED with `left:
  ObjectId(9) right: ObjectId(1)` at the `lib_ids[3] == decoy_id` assertion.
  Restored → all 10 `pb_os8_look_at_top_then_place::*` tests pass.

**Roster-floor note (reviewer's minor, non-blocking item)** — left
`pb_rs1_roster_sweep.rs`'s `>= 30` floor unchanged (not tightened to an exact
41). Judgment: unlike `pb_os1_gain_control_reversion_roster`'s exact-count
pin (2 cards, one narrow historically-fixed combination), this roster covers
4 of the engine's most common library-read primitives during an ACTIVE
card-authoring campaign — an exact pin would need routine unrelated updates
as new defs are authored and would erode into "just bump the number."
Documented the reasoning + the reviewer's original finding inline in the
test file's comment (measured count 41 as of 2026-07-19, cited).

### Gates (test-gap closure)

`cargo test --all`: 0 failures. `cargo clippy --all-targets -D warnings`:
clean. `cargo fmt --check`: clean. `tools/check-defs-fmt.sh`: clean (1804
defs). `cargo build --workspace`: clean. `PROTOCOL_VERSION` = 26,
`HASH_SCHEMA_VERSION` = 63 — unchanged (grep-confirmed). `git diff
main..HEAD -- crates/card-defs/` — empty (0 lines). Only 3 files touched:
`crates/engine/tests/core/pb_rs1_roster_sweep.rs`,
`crates/engine/tests/mechanics_m_z/reveal_and_route.rs`,
`crates/engine/tests/primitives/pb_os8_look_at_top_then_place.rs`.
