# Primitive batch WIP — PB-DX32 (prior batches are stale history; see their own plan files)

**Batch**: PB-DX32 — make the fuzzer's *output* mean something
**Seeds**: `OOS-SIM3-2` (SR-38 / legal-action soundness is asserted nowhere the fuzzer runs)
+ `OOS-SIM3-3` (every "N violations" figure is checkpoint-weighted, not a defect count)
+ `OOS-SIM3-4` (the orphaned-token class is the noise floor and `--stop-on-error` halts on it)
+ `OOS-CARDS2-3` (nothing pins the fuzz deck pool size, so a completeness flip silently
re-rolls every recorded seed)
**Brief (authoritative)**: `memory/primitives/seed-rerank-2026-08-02.md` §4 row 19
(PB-DX32) **plus** `docs/mtg-engine-feedback-engineering.md` §2.3 (row 3 — the promotion
case and the (a)/(b)/(c) component breakdown; this dispatch IS that promotion,
user-approved 2026-08-03).
**Plan**: `memory/primitives/pb-plan-DX32.md`
**Task**: `scutemob-197` · **Branch**: `feat/pb-dx32-make-the-fuzzers-output-mean-something-oos-sim3-2-oo`
**Phase**: implement (stages 0-7 of the plan §5)

---

## Stage 0 is DONE — every baseline re-measured at HEAD (`45dacc7c`)

PB-DX22 is merged (`95f53b78`): the fuzzer now shuffles every library from the game's own
seeded RNG and registers commanders, so **every SIM-3 / SIM-5 number this batch would
otherwise quote is dead**, and `OOS-DX22-13` records that several of them were read off a
5-game sample in the first place. Nothing pre-2026-08-03 may be cited as evidence here.

Committed evidence:
* `memory/primitives/pb-dx32-measurement-head-fuzzer.txt` — `mtg-fuzzer --games 20 --seed 1
  --max-turns 200 --threads 1` under `--profile fuzz`.
* `memory/primitives/pb-dx32-measurement-head-harness.txt` — 5 fuzz-shaped games with the
  journal ON, for the numbers the binary cannot print at HEAD.

Headline figures (full tables in the plan §0):

| measurement | value at HEAD |
|---|---|
| workspace tests, this branch, before any edit | **4,358 / 0 / 5**, residual list empty |
| fuzz violations, 20 games | **426** = 301 `no_orphaned_tokens` + 114 `player_consistency` + 11 `attachment_validity` |
| bot rejections | **542 / 23,613 commands = 22.953‰** |
| wasted taps (`RandomBot`) | **1,986 / 2,641 = 75.2%**, in 968 wasted runs |
| `ManaPoolsEmptied` | 885 |
| violations deduped by `(check, description)` | **94 → 20** (4.7×) |
| leaked tokens in the FINAL state | **0 on all five seeds** |
| deck pool | `all_cards()` **1,803** / `Complete` **1,133** / commander pool **90** |

`OOS-SIM3-4`'s "929 of the 938 remaining violations" is **both stale and a sample**: at HEAD
the orphaned-token class is 70.7% of the run, and `player_consistency` is a second class at
26.8% that no seed row records at that size.

---

## Plan divergences agreed before implementation (do not read these as missed requirements)

1. **The counters do NOT go in `invariants.rs`.** The feedback doc §2.3(b) says
   "`invariants.rs`/`GameResult`", but `check_all(&GameState, Option<u32>)` is a pure
   function of one state and every one of the nine live checks is too. A rejection count, a
   tap run and a `ManaPoolsEmptied` event are properties of the **command stream**. The fold
   therefore lives beside `MechanicsTally` in `local_game.rs` (same mechanism: constant-size,
   always-on, no journal) and the thresholds live in `report.rs`. `invariants.rs` gains
   exactly one new function, and it *is* a pure state function: the end-state leaked-token
   check. Plan §3.0.
2. **`tools/play-server` is touched by exactly one line** (`main.rs:3326`, a `#[cfg(test)]`
   construction site, `..Default::default()`). Criterion (a) mandates the `GameResult`
   field, which closes the escape hatch `driver.rs:76-78` took on purpose. Acceptance:
   `git diff main..HEAD --numstat -- tools/` is one file, `+1 -0`. Plan §3.1.
3. **Criterion (c) is satisfied in its literal wording only.** After the split
   `--stop-on-error` still halts at HEAD — on `player_consistency` and `attachment_validity`,
   neither of which is a *known-transient* class. Widening the split to cover them is
   refused: `player_consistency` is 26.8% of the run and undiagnosed, and suppressing an
   undiagnosed quarter of the signal is exactly what SIM-3's own withdrawal was about.
   Plan §7 R1/R2.
4. **Decision-point runtime coverage is honestly 5 of 22 rows.** The five `Served` rows are
   observable and the mapping to `BlockingDecision`/`EffectChoiceQuestion` is total and
   1:1; the other seventeen are unobservable **by definition** — an `AutoChosen` row is one
   where the engine takes the choice inline and leaves no artefact, and the absence of the
   artefact is the same property that makes it a defect. Three alternatives were considered
   and all three rejected with reasons. Plan §3.5.

---

## Stage 0 step 4 — the two deferred thresholds, measured (this invocation, stages 0-3)

Plan §5 Stage 0 step 4. Both measured on this branch, debug build (`cargo test`), before
any Stage 1-3 source edit.

* **`MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED`**: `cargo test -p mtg-simulator --test
  sim5_bot_cast_discipline -- --nocapture` → seeds 0/7/42 (`HeuristicBot`, `AB_MAX_TURNS
  = 25`, `setup::build_initial_state`) printed `mana_pools_emptied: 0`, `1`, `0`.
  Max observed = **1**, matching the plan's cited SIM-5 prior exactly. Pinned at **1**
  (§3.4: not zero, `OOS-SIM2-1` is open).
* **`MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG`**: measured with a throwaway probe
  mirroring the Stage-2 gate's own configuration exactly (3 seeds [1, 2, 3] x 25 turns x
  `RandomBot` x `build_fuzz_state`, `record_journal: false`, debug build — the same
  binary `cargo test` will run for T2.2). Seed 1: 1005 commands / 85 rejections; seed 2:
  886 / 0; seed 3: 876 / 1. **Aggregate: 2,767 commands / 86 rejections = 31.081‰.**
  Runtime was 2.06s for all three seeds — well under plan §7 R3's ~60s concern, so no
  seed-count reduction was needed. Pinned at **40** (~30% headroom over 31.081), with a
  floor `total_commands >= 2200` (2767 x 0.8, rounded down) and `rejections > 0`. This
  is a DIFFERENT number from §0.3's 22.953‰ (200-turn release-profile number) by design
  — plan §5 Stage 0 step 4 explicitly forbids reusing it for the test gate, since the two
  measure different configurations (25 vs 200 turns, debug vs fuzz profile).
  Scratch probe file `crates/simulator/tests/pb_dx32_stage0_measure_scratch.rs` was
  written, run, and deleted — never committed.

---

## Stage 1 DONE — `GameResult` construction collapses to one place (behaviour-NEUTRAL)

Plan §5 Stage 1. Files: `crates/simulator/src/report.rs` (`Default` derive on
`GameResult`), `crates/simulator/src/local_game.rs` (new `LocalGame::result_snapshot`,
rewires the GameOver return), `crates/simulator/src/driver.rs` (rewires the Halted arm
onto `result_snapshot`). New test file `crates/simulator/tests/pb_dx32_fuzz_output.rs`
with **T1.1**
`test_dx32_halted_and_game_over_results_carry_the_same_instrumentation`.

**Plan divergence 5 (new, this stage) — the `tools/` one-line insertion moved from
Stage 1 to Stage 2.** The plan's §5 Stage 1 step 3 and §3.1 both name
`tools/play-server/src/main.rs:3326` as a Stage-1 edit (`..Default::default()` appended
to the literal). At Stage 1, `GameResult` has gained the `Default` derive but ZERO new
fields — every field the literal at 3326 (and the two error-path literals in
`driver.rs:120` / `fuzzer.rs:332`) sets is still explicit, so `..Default::default()` is
provably a no-op there and `clippy::needless_update` (`-D warnings`) rejects it (plan §7
R7's exact class, confirmed by executing clippy — see below). Moving the edit to Stage
2, where `rejection_count`/`rejections` are the first new fields, makes the same
`..Default::default()` non-vacuous and clippy-clean, with **zero change to the overall
plan**: still exactly one inserted line in `tools/`, still `..Default::default()`, still
the same three sites, just landing one stage later than the plan's step numbering. Not
a scope change — a sequencing fix. `git diff -- tools/` is confirmed EMPTY at the end
of this stage (was `+1 -0` mid-stage before the revert below).

**T1.1's design deviates from the plan's literal wording in one respect, stated
because the plan asked for it to be**: the plan says "set max_turns low" for the Halted
half, implying a `LocalGame::advance()`-only construction. A truly empty two-player
`GameStateBuilder` fixture reaches `GameOver` (CR 104.3c, draw from an empty library)
within the first turn or two regardless of `max_turns`, so a bare fixture can never
reach `Halted` at all — every attempt produced a `GameOver` instead (confirmed by
running it: the first draft's Halted half returned `error: None` and reddened T1.1's own
assertion). Fixed by stocking each library with 10 unregistered filler objects (no
`card_id`, so Architecture Invariant 9 never sees them) so the game survives to
`max_turns: 3`. The Halted half also routes through `GameDriver::run_game_with_mechanics`
(the actual production caller of `driver.rs`'s Halted arm) rather than calling
`LocalGame::advance()` + `result_snapshot` directly, and checks the resulting
`GameResult` against an INDEPENDENT, identically-parameterised `LocalGame` run for the
"game's own accessors" comparison — the two are deterministic and reach the identical
halt, and this way the revert proof below actually exercises the reverted code.

**Revert proof (EXECUTED)**: replaced `driver.rs`'s
`AdvanceOutcome::Halted(reason) => game.result_snapshot(None, Some(reason.into()))` with
a literal hard-coding `turn_count: 0, total_commands: 0`. Rebuild confirmed (`Compiling
mtg-simulator` present in the captured output). Observed failure:
```
thread 'test_dx32_halted_and_game_over_results_carry_the_same_instrumentation' panicked at crates/simulator/tests/pb_dx32_fuzz_output.rs:173:5:
assertion `left == right` failed: GameResult.turn_count must match the game's own turn accessor
  left: 0
 right: 4
```
Restored immediately after (confirmed via `git diff crates/simulator/src/driver.rs`
showing the clean two-line replacement, no residue).

**NEUTRALITY EVIDENCE**: re-ran Stage 0 step 3 verbatim
(`./target/fuzz/mtg-fuzzer --games 20 --seed 1 --max-turns 200 --threads 1 --verbose`)
and diffed against the committed `pb-dx32-stage0-fuzz-before.txt`. **Exactly one
differing line, the run's wall-clock line — no games/sec change either**, executed
diff:
```
9c9
< Games completed: 20  Time: 11.5s  (2 games/sec)
---
> Games completed: 20  Time: 11.4s  (2 games/sec)
```
Every violation count, histogram row, win/draw/error tally and per-game detail line is
byte-identical between the two files. Output committed as
`memory/primitives/pb-dx32-stage1-fuzz-after.txt`.

**Stage gates, all EXECUTED**: `cargo build --workspace` clean. `cargo test -p
mtg-simulator` **183 / 0 / 0**. `cargo test -p play-server` **78 / 0 / 0** (matches plan's
expected 78/0 exactly). `cargo clippy --workspace --all-targets -- -D warnings` clean
(after the Stage-2-deferral fix above). `cargo fmt --check` clean (ran `cargo fmt` once
to fix one auto-formatting diff in the new test file — a line-wrap choice, not a
substantive change). `tools/check-defs-fmt.sh` — 1803 defs, clean.
`cargo test --workspace --no-fail-fast` — **4,359 / 0 / 5** (+1 over the 4,358
baseline, the one new T1.1 test), residual list empty.
`git diff -- crates/engine/src/ crates/card-defs/ crates/card-types/ crates/view-model/`
EMPTY. `git diff -- tools/` EMPTY (deferred to Stage 2, see divergence above).

---

## Stage 2 DONE — (a) SR-38: the rejection channel becomes a run-level invariant

Plan §5 Stage 2. Files: `crates/simulator/src/local_game.rs` (`MAX_SAMPLED_REJECTIONS`,
`record_rejection`'s cap logic, three doc-comment corrections, `result_snapshot`
extended), `crates/simulator/src/report.rs` (`GameResult::rejection_count` /
`rejections`, `MAX_BOT_REJECTION_PER_MILLE`, `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG`),
`crates/simulator/src/bin/fuzzer.rs` (`print_sr38_summary`, called from both the
`--replay` path and the parallel-run path; `std::process::exit(1)` on breach),
`crates/simulator/src/driver.rs` + `tools/play-server/src/main.rs:3326` (the two
`..Default::default()` insertions deferred from Stage 1 — see that section), `lib.rs`
re-exports (`RejectedCommand`, `MAX_RETAINED_REJECTIONS`, `MAX_SAMPLED_REJECTIONS`,
`MAX_BOT_REJECTION_PER_MILLE`, `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG`). Three new
tests in `pb_dx32_fuzz_output.rs`: **T2.1** `test_dx32_rejections_are_sampled_without_the_journal`,
**T2.2** `test_dx32_sr38_bot_rejection_rate_is_ratcheted`, **T2.3**
`test_dx32_game_result_carries_the_rejection_channel`.

**Both threshold constants pinned exactly at the Stage-0-measured values**, no
deviation: `MAX_BOT_REJECTION_PER_MILLE = 30` (measured 22.953‰ over 5 fuzz-shaped
games) and `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG = 40` (measured 31.081‰ at the
gate's own 3-seed/25-turn/debug-build configuration). **Confirmed live at the binary**:
`./target/fuzz/mtg-fuzzer --games 5 --seed 1 --max-turns 200` printed `542 rejections /
23613 commands = 22.953 per mille`, per-seed band `79/2190, 27/5518, 43/4812, 157/5902,
236/5191` — byte-identical to §0.3's own table.

**A real bug found and fixed while smoke-testing the binary (not by a written test,
by reading the output)**: the first draft of `print_sr38_summary`'s class grouping
truncated the error string at the first `(`, but every recorded rejection's error is
`format!("{:?}", LocalGameError::Rejected(GameStateError))`, so EVERY class collapsed
to the literal string `"Rejected"` — the wrapper, not the actual reason. Fixed by
stripping a `"Rejected("` prefix before the truncation. Confirmed by re-running the
5-seed smoke command: classes now read `InsufficientMana` (16), `InvalidTarget` (15),
`AlreadyDeclaredBlockers` (4), `InvalidCommand` (3), `CrossPlayerBlock` (2) — matching
the five named open-seed shapes in plan §0.4 (`OOS-SIM5-3`, `OOS-SIM5-5`,
`OOS-SIM6-3`, `OOS-CARDS2-4`) almost exactly.

**T2.3's fixture required two new bots not in the plan** (`AlwaysRejectedBot`,
`ConcedeOnFirstCallBot`) because a genuinely discriminating parity check needs a
NON-ZERO `rejection_count` on the GameOver path, and the obvious zero-rejection
GameOver fixture (a player pre-marked `has_lost`, T1.1's own approach) is vacuous for
this specific field — `Default::default()`'s `rejection_count` is also 0, so a
regression that silently drops the field back to its default would pass a
zero-on-both check undetected. `AlwaysRejectedBot` issues a guaranteed-rejected
`PlayLand` every priority window; `ConcedeOnFirstCallBot` concedes CR 104.3a on its
first call, ending the game via `is_game_over` on the very next loop check, after
`AlwaysRejectedBot` has already been rejected at least once.

**Revert proofs (all three EXECUTED, rebuild confirmed each time)**:
* **T2.1**: restored `self.limits.record_journal &&` in `record_rejection`'s cap
  guard. Failure: `"record_journal: false must still sample SOME rejections (SR-38,
  OOS-SIM3-2)"`.
* **T2.2**: set `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG` to 30 (measured − 1,
  rounding down from 31.081). Failure: `"aggregate rejection rate 31.081 per mille
  exceeds the ratchet MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG = 30"` — names the
  exact measured rate, proving the comparison is live.
* **T2.3**: hard-coded `rejection_count: 0` in `result_snapshot`. Failure (GameOver
  half, the first assertion reached): `"p1's AlwaysRejectedBot must have produced >=1
  rejection before p2 conceded: GameResult { ... rejection_count: 0, rejections:
  [RejectedCommand { ... error: "Rejected(NotMainPhase)" }] ... }"` — the debug dump
  shows the sample WAS non-empty while the count field was wrongly 0, which is exactly
  the divergence the test exists to catch.

All three restored immediately after each revert; `git diff` confirmed clean before
moving to the next.

**Stage gates, all EXECUTED**: `cargo build --workspace` clean; `cargo test -p
mtg-simulator` **186 / 0 / 0** (+3); `cargo test -p play-server` **78 / 0 / 0** unmoved.
`cargo clippy --workspace --all-targets -- -D warnings` — one real finding fixed along
the way: the first draft's `loop { match game.advance() { .. => break } }` in the new
`play_fuzz_shaped` helper tripped `clippy::never_loop` / `clippy::while_let_loop`
(since `advance()` with no human seats always resolves in one call — the outer `loop`
was never going to iterate twice). Fixed by dropping the wrapper loop entirely (a
single `match`, mirroring `driver.rs`'s own comment). Clean after. `cargo fmt --check`
clean (ran `cargo fmt` twice — once for the new tests, once after the class-grouping
fix). `tools/check-defs-fmt.sh` — 1803 defs, clean.
`cargo test --workspace --no-fail-fast` — **4,362 / 0 / 5** (+4 over the Stage-1
pin), residual list empty.
`git diff -- crates/engine/src/ crates/card-defs/ crates/card-types/ crates/view-model/`
EMPTY. `git diff --numstat -- tools/` — **exactly one file, `+1 -0`**
(`tools/play-server/src/main.rs`), matching plan §3.1's acceptance criterion (landed
here rather than Stage 1, per the Stage-1 divergence note).

---

## Stage 3 DONE — (b) the waste instrument, promoted and thresholded

Plan §5 Stage 3. Files: `crates/simulator/src/local_game.rs` (new `WasteTally`,
`waste`/`waste_run` fields, `fold_waste`, `waste()` accessor, wired at the two F8 fold
sites, `result_snapshot` extended), `crates/simulator/src/report.rs`
(`GameResult::waste`, `MAX_RANDOM_BOT_WASTED_TAP_PCT`,
`MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG`, `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED`),
`crates/simulator/src/bin/fuzzer.rs` (`print_waste_summary`, called from both paths),
`lib.rs` re-exports. New test **T3.1**
`test_dx32_random_bot_waste_ratio_is_bounded` in `pb_dx32_fuzz_output.rs`; **T3.2**
`test_dx32_streaming_waste_tally_equals_the_sim5_journal_walk` (extended with a
purpose-built controlled sub-case, see below) and **T3.3**
`heuristic_pools_emptied_is_pinned` in `sim5_bot_cast_discipline.rs` — `metrics_of` and
`Metrics` kept exactly as the plan requires (not deleted).

**Both thresholds confirmed live at the binary**: `./target/fuzz/mtg-fuzzer --games 5
--seed 1 --max-turns 200` printed `tap runs: 1258 total, 968 wasted (1986 taps of 2641
total = 75.2%, threshold 85%)` and `CR 500.4 ManaPoolsEmptied: 885` — byte-identical to
§0.3's own numbers.

**A second `_AT_GATE_CONFIG` threshold was needed for T3.1, beyond what the plan wrote
down — the SAME structural reason Stage 2 needed one for SR-38, discovered by running
the test, not anticipated in advance.** The plan's single `MAX_RANDOM_BOT_WASTED_TAP_PCT
= 85` is a 200-turn, `--profile fuzz` measurement. At T3.1's own 3-seed/25-turn debug
configuration the measured waste ratio is **89%** (87/97 taps, seeds 1/2/3) — ABOVE the
85% ceiling, so the test would have been red on arrival using the binary's own constant.
Added `MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG = 95` (a flat +6 percentage points
over 89, not the ~30% multiplicative headroom the per-mille `_AT_GATE_CONFIG` constants
use, since a percentage is bounded at 100 and 89×1.3 would overshoot meaninglessly),
with the reasoning for WHY the two populations genuinely differ (a shorter game's early
taps have proportionally fewer high-value casts to land on) recorded in the constant's
own doc, not just asserted.

**T3.2's revert did NOT discriminate on the plan's own AB_SEEDS fixture — plan §7 R8's
explicitly-anticipated failure mode, hit for real.** Executing the revert (drop the
open-run close in `waste()`) against the `AB_SEEDS`/`HeuristicBot`/25-turn loop left the
test GREEN. Root-caused, not just observed: `HeuristicBot` scores
`LegalAction::TapForMana` at 0, "below passing" (`heuristic_bot.rs:271`), so it never
chooses a standalone tap — every tap it makes is an auto-tap PREFIX bundled with a cast
in one `[taps…, cast]` atomic sequence, and that whole bundle is folded (and its own run
closed) inside the single `apply_sequence` call that commits it, before the call
returns. A run can only survive past one `advance()` iteration if the WHOLE decision
was a standalone tap with nothing queued after — structurally unreachable for
`HeuristicBot`. **Fix, per R8's own instruction ("construct a fixture that ends on a
tap")**: extended T3.2 with a controlled second case — a human seat submits exactly one
`TapForMana` command via `submit()` and nothing follows, which is the only way to force
`waste_run` open at inspection time. Confirmed by an initial two-pass empirical scan
(scratch file, deleted, never committed) that tried to find a naturally-occurring
mid-tap-run halt by truncating `max_commands` at various points in an already-played
journal — every candidate in a window of +1..+6 past a `TapForMana` index still ended on
a non-tap command, which is what led to root-causing the atomic-batch mechanism above
rather than continuing to search blindly.

**Revert proofs (all three EXECUTED, rebuild confirmed each time)**:
* **T3.1**: set `MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG` to 88 (measured − 1,
  truncating 89.69%→89 first). Failure: `"RandomBot wasted-tap ratio 89% exceeds
  MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG = 88"`.
* **T3.2**: dropped the trailing open-run close (`tally.tap_runs += 1`), using the NEW
  controlled human-submit fixture. Failure: `"the still-open run must be closed on the
  snapshot COPY waste() returns ... WasteTally { tap_runs: 0, ... total_taps: 1, ... }
  left: 0 right: 1"` — proves the tap itself is still counted (`total_taps: 1`)
  while only the run-closing logic broke, exactly the divergence the test exists to
  catch.
* **T3.3**: set `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` to 0 (measured − 1). Failure:
  `"seed 7: mana_pools_emptied 1 exceeds MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED (0) — a
  rise past this pin means either a new wasted-tap class or that the greedy-solver
  slack OOS-SIM2-1 leaves on casts that SUCCEED has widened"` — names `OOS-SIM2-1` at
  the pin, satisfying criterion (b)'s literal requirement.

All three restored immediately after each revert; `git diff` confirmed clean before
moving to the next.

**Stage gates, all EXECUTED**: `cargo build --workspace` clean; `cargo test -p
mtg-simulator --test sim5_bot_cast_discipline --test pb_dx32_fuzz_output` **6 + 5 = 11
passed / 0 failed**; `cargo test -p play-server` **78 / 0 / 0** unmoved. `cargo clippy
--workspace --all-targets -- -D warnings` clean. `cargo fmt --check` clean (ran `cargo
fmt` once). `tools/check-defs-fmt.sh` — 1803 defs, clean.
`cargo test --workspace --no-fail-fast` — **4,365 / 0 / 5** (+3 over the Stage-2 pin:
T3.1, T3.2, T3.3), residual list empty.
`git diff main..HEAD --numstat -- crates/engine/src/ crates/card-defs/
crates/card-types/ crates/view-model/` EMPTY. `git diff main..HEAD --numstat --
tools/` — **still exactly one file, `+1 -0`** (unchanged from Stage 2 — Stage 3 touched
no file under `tools/`).

---

## Stage 4 DONE — (c) the noise floor

Plan §5 Stage 4. Files: `crates/simulator/src/invariants.rs` (new
`check_no_leaked_tokens`, `distinct`), `crates/simulator/src/local_game.rs` (new
`transient_violations` field + accessor, `record_violations` split helper wired at
both `check_all` call sites, `result_snapshot` extended to run the leaked-token check
at both real terminal paths), `crates/simulator/src/report.rs`
(`GameResult::transient_violations`), `crates/simulator/src/bin/fuzzer.rs`
(`print_violation_bucket` + rewritten `print_violation_histogram` printing HARD and
TRANSIENT blocks with raw+distinct counts each, `main`'s summary line split into
"Total violations (HARD)" / "Total violations (TRANSIENT, ...)"). Three new tests in
`pb_dx32_fuzz_output.rs`: **T4.1**
`test_dx32_orphaned_tokens_are_transient_and_the_end_state_is_clean`, **T4.2**
`test_dx32_leaked_token_at_game_end_is_a_hard_violation`, **T4.3**
`test_dx32_distinct_collapses_checkpoint_weighting`.

**Fixture seed measured, not guessed**: `play_fuzz_shaped(2, 4, 25)` (RandomBot,
`build_fuzz_state`, `record_journal: false` — the same configuration T2.x/T3.x
already use) was scanned at implementation time (throwaway scratch test, run then
deleted, never committed) and found to produce exactly 4 raw `no_orphaned_tokens`
transient reports (same Treasure token, turn 24, dedup 1), 0 hard violations, 0
leaked tokens at the final state — used for T4.1's non-vacuity and T4.3's real-seeded
half.

**A/B, EXECUTED (criterion (c)'s mandatory before/after)**: re-ran Stage 0 step 3
verbatim (`./target/fuzz/mtg-fuzzer --games 20 --seed 1 --max-turns 200 --threads 1
--verbose`), committed as `memory/primitives/pb-dx32-stage4-fuzz-after.txt`:

| metric | before (§0.2) | after (measured) |
|---|---|---|
| `Total violations` (hard) | 426 | **125** = 114 `player_consistency` + 11 `attachment_validity` — matches the plan's prediction exactly |
| transient (reported, not halting) | — | **301** `no_orphaned_tokens` |
| distinct hard / distinct transient | — | **7** / **67** |
| games with ≥1 **hard** violation | 16 / 20 | **6 / 20** (≤ 8 predicted) |
| crash reports written | 16 files (stale, pre-Stage-4 semantics) | **6 files** (`crash-reports/` cleared first, then re-measured: `crash_{2,5,7,9,15,19}.json`) |
| `--stop-on-error` halts on `no_orphaned_tokens` | yes | **NO** |

**`--stop-on-error` outcome, recorded per §7 R1** (`memory/primitives/pb-dx32-stage4-stoponerror.txt`):
run `--games 20 --seed 1 --max-turns 200 --stop-on-error --verbose` completed only
**2** games (not 20) before halting, on seed 2's `player_consistency` violation
("Active player PlayerId(1) has lost or conceded (turn 123)") — exactly the class
R1 predicted, and it is **not** suppressed here: `player_consistency` stays
undiagnosed and un-widened into the transient split, per plan §7 R1/R2 and plan
divergence 3.

**R10 measured, not predicted**: the 20-game run produced **zero** `leaked_tokens`
violations — `check_no_leaked_tokens` never fired at either terminal path across all
20 games. No new finding here; consistent with §0.3's 0-on-five-seeds measurement,
now confirmed at 20-game scale.

**Revert proofs (all three EXECUTED, rebuild confirmed each time)**:
* **T4.1**: changed `record_violations`'s split predicate from `v.check ==
  "no_orphaned_tokens"` to `v.check == "zone_integrity"`. Failure: `"seed 2 at
  max_turns 25 is known to produce no_orphaned_tokens transient reports (measured at
  implementation time: 4 raw reports)"` — token violations landed in the hard bucket
  instead, so `transient_violations()` came back empty.
* **T4.2**: made `check_no_leaked_tokens` return `Vec::new()` unconditionally (behind
  an early `return`, function-level `#[allow(unreachable_code)]` so `-D warnings`
  doesn't turn the revert into a silent stale-binary pass — plan §7 R7). Failure:
  `"exactly one token, exactly one violation: [] left: 0 right: 1"` — fails on the
  broken-state (leaked-token) half while the clean-state half (asserted first)
  stayed green, which is what proves the probe is paired and not one-sided.
* **T4.3**: made `distinct` return `violations.to_vec()` unconditionally (same
  early-return + `#[allow(unreachable_code)]` shape). Failure: `"left: 3 right: 1"`
  on the hand-built half (three identical `(check, description)` pairs at three
  different turns, expected to collapse to 1, stayed at 3).

All three restored immediately after each revert; `git diff` confirmed clean before
moving to the next.

**Stage gates, all EXECUTED**: `cargo check -p mtg-simulator` clean after each edit;
`cargo build --profile fuzz --bin mtg-fuzzer` clean; `cargo test -p mtg-simulator
--test pb_dx32_fuzz_output` **8 / 0 / 0** (+3 over Stage 3's 5); `cargo clippy
--workspace --all-targets -- -D warnings` clean; `cargo fmt --check` clean (ran
`cargo fmt` once — two import-list rewraps and three string-literal unwraps in the
new tests, no substantive change); `tools/check-defs-fmt.sh` — 1803 defs, clean.
`cargo test --workspace --no-fail-fast` — **4,368 / 0 / 5** (+3 over the Stage-3 pin:
T4.1, T4.2, T4.3), residual list empty.
`cargo test -p mtg-engine --test core hash_schema` / `--test core protocol_schema` —
**HASH 72 / PROTOCOL 35 unmoved**, read off the constants (this stage touches no
engine source at all).
`git diff main..HEAD --numstat -- crates/engine/src/ crates/card-defs/
crates/card-types/ crates/view-model/` EMPTY. `git diff main..HEAD --numstat --
tools/` — **still exactly one file, `+1 -0`** (unchanged — Stage 4 touched no file
under `tools/`).

---

## Stage 5 DONE — (d) the corpus→seed gate (`OOS-CARDS2-3`)

Plan §5 Stage 5. File: `crates/simulator/tests/pb_dx32_fuzz_output.rs` only —
`CORPUS_DEFS`/`CORPUS_COMPLETE`/`COMMANDER_POOL` consts, a `commander_pool()` helper
mirroring `deck.rs:40-47`'s three-clause filter (with a comment naming the line
range, per the plan's anti-drift instruction), **T5.1**
`test_dx32_fuzz_deck_pool_size_is_pinned`, **T5.2**
`test_dx32_commander_pool_filter_mirrors_deck_rs`.

**Pinned exactly as measured at Stage 0 / plan §0.5, both confirmed live by running
the gate**: `CORPUS_DEFS = 1803`, `CORPUS_COMPLETE = 1133`, `COMMANDER_POOL = 90`.
T5.1's shared `MOVED_MSG` constant is appended to all three assertion messages and
states, in these terms: the fuzz deck pool changed, every seeded fixture now deals a
different game (`OOS-CARDS2-3`), update the three constants in the SAME commit as the
card-def change, and expect the seeded pins in `memory/workstream-state.md`'s CARDS-2
handoff item 1 to move.

**Revert proofs (both EXECUTED, rebuild confirmed each time)**:
* **T5.1(a)**: set `CORPUS_COMPLETE` to 1132 (measured − 1). Failure: `"the
  Complete-def count moved from the pinned CORPUS_COMPLETE (1132) to 1133 -- the fuzz
  deck pool changed. Every seeded fixture..."` — names the direction and
  `OOS-CARDS2-3` exactly as required.
* **T5.1(b) / T5.2 discrimination**: dropped the `CardType::Creature` clause from
  `commander_pool()`'s mirrored filter. Pool grew 90 → 128 (Legendary-Complete minus
  the Creature-type requirement). T5.1 reddened: `"the commander pool (Complete +
  Legendary + Creature, deck.rs:40-47) moved from the pinned COMMANDER_POOL (90) to
  128..."`. **T5.2 stayed GREEN** on this same revert — the plan's own instruction —
  because T5.2 asserts membership (`random_deck`'s pick is IN the mirrored pool),
  which still holds when the mirrored pool is a wider superset of the true one; T5.2
  therefore discriminates a DIFFERENT failure mode from T5.1 (a genuinely diverged
  filter, e.g. one that excludes a legal commander), not a mere size change, so no
  restatement was needed.

Both restored immediately after each revert; `git diff` confirmed clean before moving
to the next.

**Stage gates, all EXECUTED**: `cargo test -p mtg-simulator --test pb_dx32_fuzz_output`
**10 / 0 / 0** (+2 over Stage 4's 8); `cargo clippy --workspace --all-targets -- -D
warnings` clean; `cargo fmt --check` clean (ran `cargo fmt` once — an import-list
rewrap and a line-wrap in `commander_pool`'s filter chain). `tools/check-defs-fmt.sh`
— 1803 defs, clean.
`cargo test --workspace --no-fail-fast` — **4,370 / 0 / 5** (+2 over the Stage-4 pin:
T5.1, T5.2), residual list empty.
`git diff main..HEAD --numstat -- crates/engine/src/ crates/card-defs/
crates/card-types/ crates/view-model/` EMPTY. `git diff main..HEAD --numstat --
tools/` — **still exactly one file, `+1 -0`** (unchanged — Stage 5 touched no file
under `tools/`).

---

## Stage 6 DONE — (e) decision-point runtime coverage

Plan §5 Stage 6. New file `crates/simulator/src/decision_coverage.rs`
(`OBSERVABLE_ROW_IDS` = 5 ids, `UNOBSERVABLE_ROW_IDS` = 17 `(id, reason)` pairs,
`ROW_COUNT`, `DecisionCoverage` with a hand-written `Default`, `row_id_for` —
exhaustive on both `BlockingDecision` and `EffectChoiceQuestion`, no wildcard).
Wired: `local_game.rs` (`decisions: DecisionCoverage` field, folded at the
`state.blocking_decision()` branch alongside the pre-existing `DecisionKind` match —
kept as two SEPARATE exhaustive matches on purpose, commented so a future reader
does not merge them; `decision_coverage()` accessor; `result_snapshot` extended;
`AdvanceOutcome` gained `#[allow(clippy::large_enum_variant)]` — `GameResult` crossed
clippy's default threshold once `decision_coverage` (88 bytes) landed, and boxing it
would touch `tools/play-server` match arms outside this batch's footprint, so the
allow follows this crate's own precedent, `crates/engine/src/rules/events.rs:61` /
`card_definition.rs:1238`), `report.rs` (`GameResult::decision_coverage`), `lib.rs`
(new `pub mod decision_coverage;` + re-exports), `bin/fuzzer.rs`
(`print_decision_coverage`, called from both the run path and the `--replay` path).
**New engine test appended to the EXISTING `crates/engine/tests/core/decision_gate.rs`**
(criterion (e)'s "extend, don't rebuild" — `ROWS`, `BASELINE`,
`MAX_AUTO_CHOSEN_COMPLETE_UNION = 80` untouched): `quoted_strings`,
`extract_const_array_block`, and **T6.1**
`runtime_decision_coverage_roster_matches_rows`. New tests in
`pb_dx32_fuzz_output.rs`: **T6.2** `test_dx32_row_id_for_covers_every_observable_row`,
**T6.3** `test_dx32_a_fuzz_run_reaches_at_least_one_served_row`.

**R9 measured, not guessed, and the honest answer is BETTER than the plan's own
worst-case hypothesis.** At T6.3's own gate configuration (10 fuzz-shaped games x
60 turns, `RandomBot`, `build_fuzz_state`, `record_journal: false`, debug build) —
**4 of the 5 served rows are reached**: `triggered_targets`, `search_library`,
`scry`, `discard_cards`. Only `surveil` is never reached at this budget. Deterministic
(re-run twice, identical partition both times), so T6.3 asserts the partition
EXACTLY rather than as a floor, with a message that tells a failing future reader to
report the change as a finding rather than silently re-tuning the seed range.
**A second, independent data point at the release-profile binary's own
configuration** (`--games 20 --seed 1 --max-turns 200`, `--profile fuzz`,
committed as `memory/primitives/pb-dx32-stage6-fuzz-smoke.txt`): **all 5 of 5 served
rows are reached**, including `surveil` (30 observations) — confirming the gap at
T6.3's debug/60-turn budget is a depth artefact, not evidence that `surveil` is hard
to reach in general. Both configurations and both results are recorded rather than
only the more favourable one.

**Revert proofs (all four EXECUTED against the REAL source files, rebuild confirmed
each time — no throwaway fixture stand-ins)**:
* **T6.1(a)**: moved `"surveil"` from `OBSERVABLE_ROW_IDS` to `UNOBSERVABLE_ROW_IDS`
  in `decision_coverage.rs`. Failure: `"OBSERVABLE_ROW_IDS must equal EXACTLY the
  ROWS ids whose class is Served. In OBSERVABLE_ROW_IDS but not Served in ROWS: [].
  Served in ROWS but missing from OBSERVABLE_ROW_IDS: [\"surveil\"]"` — names
  `surveil` and the class mismatch exactly as required.
* **T6.1(b), mandatory**: commented out the `"proliferate"` tuple in
  `UNOBSERVABLE_ROW_IDS` with `//` on every line. Failure: `"...In ROWS but missing
  from the roster: [\"proliferate\"]..."` — proves `strip_line_comments` is being
  applied (an unstripped scan would still have found `"proliferate"` as plain text
  inside the comment and stayed green, the exact comment-satisfiable-gate class
  PB-DX22's review cycle 2 found in this file's own family).
* **T6.2**: changed `row_id_for`'s `EffectChoiceQuestion::Scry` arm to return `None`
  (via a temporary `.and_then` restructuring, since the real code returns a bare
  `&'static str` from inside a `.map`). Failure: `"row_id_for must return \"scry\"
  for this fixture, got None"`.
* **T6.3**: made `DecisionCoverage::observe` a no-op (`let _ = row_id;`). Failure:
  reddened with `reached: {}` printed, naming the empty partition against the
  measured baseline.

All four restored immediately after each revert; `git diff`/re-read confirmed clean
before moving to the next.

**A design deviation from the plan's own reasoning-organization worth stating**:
`row_id_for`'s exhaustive match on `EffectChoiceQuestion` is written as
`.map(|pending| match ... { ... => "id" })` (each arm a bare `&'static str`), not
`.and_then(|pending| match ... { ... => Some("id") })` — `clippy::bind_instead_of_map`
rejects the latter under `-D warnings` because every arm was `Some(_)` in the
non-reverted code (no arm needed `None`). The two are behaviourally identical; the
revert proofs above used the `and_then` shape ONLY as a scratch vehicle to express a
temporary `None` arm, then were restored to the clippy-clean `map` shape.

**Stage gates, all EXECUTED**: `cargo check -p mtg-simulator` clean throughout;
`cargo test -p mtg-simulator --test pb_dx32_fuzz_output` **12 / 0 / 0** (+2 over
Stage 5's 10; T6.3 runs in ~19s, the slowest test in the file, from playing 10 real
fuzz-shaped games to gather the reached/never-reached partition); `cargo test -p
mtg-engine --test core runtime_decision_coverage_roster_matches_rows` **1 / 0 / 0**;
`cargo clippy --workspace --all-targets -- -D warnings` clean (two findings fixed
along the way: `clippy::bind_instead_of_map` in `row_id_for`, and
`clippy::large_enum_variant` on `AdvanceOutcome` once `GameResult` grew past its
default threshold — see the `#[allow]` note above); `cargo fmt --check` clean (ran
`cargo fmt` once). `tools/check-defs-fmt.sh` — 1803 defs, clean.
`cargo test --workspace --no-fail-fast` — **4,373 / 0 / 5** (+3 over the Stage-5 pin:
T6.1, T6.2, T6.3), residual list empty.
`cargo test -p mtg-engine --test core hash_schema` / `--test core protocol_schema` —
**HASH 72 / PROTOCOL 35 unmoved**, read off the constants. `cargo test -p
play-server` — **78 / 0** unmoved.
`git diff main..HEAD --numstat -- crates/engine/src/ crates/card-defs/
crates/card-types/ crates/view-model/` EMPTY. `git diff main..HEAD --numstat --
tools/` — **still exactly one file, `+1 -0`**. `git status --short --
crates/engine/tests/` shows exactly one modified file, `core/decision_gate.rs`
(appended-only, per criterion (e)'s "extend, don't rebuild").
`tools/authoring-report.py` regenerated: body byte-identical except the git-sha/date
stamp lines (`docs/authoring-status.md`, `docs/authoring-status-missing.txt`,
`docs/authoring-status-prev.json`); coverage unmoved **1,133/1,803 = 62.8%**;
regeneration churn reverted (`git checkout --`) before this commit.
