# PB-DX56 task list (`scutemob-235`)

Status legend: `[ ]` pending · `[~]` in progress · `[x]` done

## Stage 0 — measure and predict BEFORE any production line
- [~] 0.1 Pre-edit full-workspace baseline to a file; reconcile against PB-DX55's **5,287 / 0 / 5** on **72** targets (or report the discrepancy)
- [ ] 0.2 Re-measure BOTH classes at HEAD on the exact filed invocation (`--games 20 --seed 1 --max-turns 200`), by-check histogram PRINTED
- [ ] 0.3 Re-observe the PB-DX32 gate config (seeds 1,2,3 × 25 turns) at HEAD
- [ ] 0.4 Wire prediction PER HALF in writing, committed, before any production line (HASH / PROTOCOL)
- [ ] 0.5 Coverage prediction (0 flips) with the reason, before regeneration

## Stage 1 — OOS-FB1-1: make a crash diagnosable
- [ ] 1.1 `InvariantViolation` gains structured `evidence`, deliberately NOT part of the `(check, description)` dedupe key
- [ ] 1.2 `check_player_consistency` emits its own evidence (which player, lost vs conceded, active vs priority, turn/phase/step)
- [ ] 1.3 `check_attachment_validity` emits its own evidence (attacher name + card types + subtypes, target id, where the target went)
- [ ] 1.4 Bounded command-history ring in `LocalGame`; `GameResult` carries it; retained only for games that violated
- [ ] 1.5 `CrashReport.command_history` filled from it (the `Vec::new()` at `bin/fuzzer.rs` deleted)
- [ ] 1.6 Write-before / delete-after in-flight tombstone — `crash-reports/inflight_<seed>.json` — the only mechanism that survives `abort()`
- [ ] 1.7 `--replay <seed>` reproduces to the violating turn and DUMPS the violating state with the check's own evidence
- [ ] 1.8 PROOF: a deliberately planted panic in an isolated build produces an artefact that `--replay` reproduces to the same turn
- [ ] 1.9 PROOF: a planted SIGABRT leaves the tombstone on disk

## Stage 2 — OOS-DX32-1 diagnosed BY EXECUTION and dispositioned
- [ ] 2.1 Run the new tooling; answer "is it ever true AT REST?" with executed evidence, per ARM (active-player arm vs priority-holder arm)
- [ ] 2.2 Disposition: transient split + strictly stronger end-state check, or engine fix — whichever the measurement says
- [ ] 2.3 Probe that plants the condition and reddens the disposition

## Stage 3 — OOS-DX22-8 diagnosed to its MECHANISM and fixed
- [ ] 3.1 Identify the zone-move path that leaves the dangling attachment (executed, not inferred)
- [ ] 3.2 Engine fix
- [ ] 3.3 Engine-level probe built from the MECHANISM, not from a fuzz seed

## Stage 4 — no undiagnosed HARD class survives
- [ ] 4.1 Every remaining HARD class on the standard invocation fixed, classified-transient-with-end-state-check, or filed with its mechanism NAMED
- [ ] 4.2 `--stop-on-error` no longer halts on an undiagnosed class

## Stage 5 — gates, evidence, close-out
- [ ] 5.1 Revert matrix, coordinator-executed, in `memory/primitives/pb-DX56-execution-notes.md`; UNDISCRIMINATED rows disclosed in the test itself
- [ ] 5.2 >= 3 adversarial bypass attempts per new source gate by a SECOND agent, recorded
- [ ] 5.3 Fuzz A/B vs the merge base in an isolated worktree with its own `CARGO_TARGET_DIR` under the scratchpad (deleted after); movement attributed by class
- [ ] 5.4 Gate-config ratchets ANSWERED, never loosened
- [ ] 5.5 Both wire gates executed; test delta by byte-exact NAME set difference + count reconciliation + non-end-anchored duplicate scan
- [ ] 5.6 `clippy -D warnings` + `cargo fmt --check` + `tools/check-defs-fmt.sh` + `cargo build --workspace` against the FINAL tree; `npm run build` N/A with the reason; benches "not measured" with the reason
- [ ] 5.7 Coverage regenerated, 0 flips stated with the reason
- [ ] 5.8 Registry rows closed (pipes escaped), `OOS-DX56-N` filed (grep FIRST)
- [ ] 5.9 v4 memo §4 row 20 struck, banner repointed to rank 21 (PB-DX57)
- [ ] 5.10 CLAUDE.md Current State + `workstream-state.md` handoff AND its W6 row
- [ ] 5.11 `/review`, all findings taken or declined with reasons
- [ ] 5.12 Headline surfaces re-checked against the registry AFTER the fix cycle (dispatch hygiene 8)
- [ ] 5.13 `/tmp` bench/fuzz dirs deleted before finishing
