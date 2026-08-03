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
