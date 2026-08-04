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
