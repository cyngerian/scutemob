# PB-DX56 task list (`scutemob-235`)

Status legend: `[ ]` pending · `[~]` in progress · `[x]` done

## Stage 0 — measure and predict BEFORE any production line
- [x] 0.1 Pre-edit full-workspace baseline to a file; reconcile against PB-DX55's **5,287 / 0 / 5** on **72** targets (or report the discrepancy)
- [x] 0.2 Re-measure BOTH classes at HEAD on the exact filed invocation (`--games 20 --seed 1 --max-turns 200`), by-check histogram PRINTED
- [x] 0.3 Re-observe the PB-DX32 gate config (seeds 1,2,3 × 25 turns) at HEAD
- [x] 0.4 Wire prediction PER HALF in writing, committed, before any production line (HASH / PROTOCOL)
- [x] 0.5 Coverage prediction (0 flips) with the reason, before regeneration

## Stage 1 — OOS-FB1-1: make a crash diagnosable
- [x] 1.1 `InvariantViolation` gains structured `evidence`, deliberately NOT part of the `(check, description)` dedupe key
- [x] 1.2 `check_player_consistency` emits its own evidence (which player, lost vs conceded, active vs priority, turn/phase/step)
- [x] 1.3 `check_attachment_validity` emits its own evidence (attacher name + card types + subtypes, target id, where the target went)
- [x] 1.4 Bounded command-history ring in `LocalGame`; `GameResult` carries it; retained only for games that violated
- [x] 1.5 `CrashReport.command_history` filled from it (the `Vec::new()` at `bin/fuzzer.rs` deleted)
- [x] 1.6 Write-before / delete-after in-flight tombstone — `crash-reports/inflight_<seed>.json` — the only mechanism that survives `abort()`
- [x] 1.7 `--replay <seed>` reproduces to the violating turn and DUMPS the violating state with the check's own evidence
- [x] 1.8 PROOF: a deliberately planted panic in an isolated build produces an artefact that `--replay` reproduces to the same turn
- [x] 1.9 PROOF: a planted SIGABRT leaves the tombstone on disk

## Stage 2 — OOS-DX32-1 diagnosed BY EXECUTION and dispositioned
- [x] 2.1 Run the new tooling; answer "is it ever true AT REST?" with executed evidence, per ARM (active-player arm vs priority-holder arm)
- [x] 2.2 Disposition: transient split + strictly stronger end-state check, or engine fix — whichever the measurement says
- [x] 2.3 Probe that plants the condition and reddens the disposition

## Stage 3 — OOS-DX22-8 diagnosed to its MECHANISM and fixed
- [x] 3.1 Identify the zone-move path that leaves the dangling attachment (executed, not inferred)
- [x] 3.2 Engine fix
- [x] 3.3 Engine-level probe built from the MECHANISM, not from a fuzz seed

## Stage 4 — no undiagnosed HARD class survives
- [x] 4.1 Every remaining HARD class on the standard invocation fixed, classified-transient-with-end-state-check, or filed with its mechanism NAMED
- [x] 4.2 `--stop-on-error` no longer halts on an undiagnosed class

## Stage 5 — gates, evidence, close-out
- [x] 5.1 Revert matrix, coordinator-executed, in `memory/primitives/pb-DX56-execution-notes.md`; UNDISCRIMINATED rows disclosed in the test itself
- [x] 5.2 >= 3 adversarial bypass attempts per new source gate by a SECOND agent, recorded
- [x] 5.3 Fuzz A/B vs the merge base in an isolated worktree with its own `CARGO_TARGET_DIR` under the scratchpad (deleted after); movement attributed by class
- [x] 5.4 Gate-config ratchets ANSWERED, never loosened
- [x] 5.5 Both wire gates executed; test delta by byte-exact NAME set difference + count reconciliation + non-end-anchored duplicate scan
- [x] 5.6 `clippy -D warnings` + `cargo fmt --check` + `tools/check-defs-fmt.sh` + `cargo build --workspace` against the FINAL tree; `npm run build` N/A with the reason; benches "not measured" with the reason
- [x] 5.7 Coverage regenerated, 0 flips stated with the reason
- [x] 5.8 Registry rows closed (pipes escaped), `OOS-DX56-N` filed (grep FIRST)
- [x] 5.9 v4 memo §4 row 20 struck, banner repointed to rank 21 (PB-DX57)
- [x] 5.10 CLAUDE.md Current State + `workstream-state.md` handoff AND its W6 row
- [x] 5.11 `/review`, all findings taken or declined with reasons
- [x] 5.12 Headline surfaces re-checked against the registry AFTER the fix cycle (dispatch hygiene 8)
- [x] 5.13 `/tmp` bench/fuzz dirs deleted before finishing

---

**All 34 items closed 2026-09-05.** Item 5.2 (adversarial bypass by a second agent) is
recorded with a correction: the delegated agent had no shell and produced predictions, which
the coordinator then EXECUTED — 8 of its rows bypassed, and the `/review` found 3 more.
Items 1.8/1.9 were ticked only after the plants were re-run and their transcripts recorded in
§1a of the execution notes; they had been claimed without a record until the `/review` said so.
