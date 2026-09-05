# PB-DX54 task list (`scutemob-232`)

Seed: **OOS-DX25c-6** — a resolving spell cannot be its own redirect victim.
Riders: **OOS-DX25-4**, **OOS-DX25b-4**. v4 rank 17.

No TaskCreate/TodoWrite tool exists in this harness build (dispatch hygiene 10);
this file plus an `esm task comment` is the substitute. Updated as items land.

| # | Item | AC | State |
|---|------|----|-------|
| 0.1 | Pre-edit full-workspace baseline to a file; reproduce PB-DX53's 5,210/0/5 on 67 targets | 7382 | **done** — reproduces EXACTLY, no correction owed |
| 0.2 | CR + ruling research (CR 608.2m, 608.2, 115.7a; Misdirection 2004-10-04 via MCP) | 7379 | **done** — CR 608.2n is the warrant, NOT CR 608.2m; ruling verified via MCP |
| 0.3 | Wire prediction PER OPTION, in writing, committed BEFORE any production line | 7381 | **done** — `54415c25`, committed before any production line |
| 0.4 | Stage-0 blast-radius MEASUREMENT: execute the suite with the pop MOVED (resolve-in-place) | 7379 | **done** — pop moved to the function boundary: **5,207 / 3 / 5**, and all THREE failures are SOURCE gates; ZERO behavioural |
| 0.5 | Design settled between resolve-in-place and shadow entry, with the measurement as the evidence | 7379 | **done** — **resolve-in-place**, on the measurement; option B costs a HASH bump and 6 call-site changes and is less CR-faithful |
| 1.1 | Engine: move the pop to the end of `resolve_top_of_stack_inner` (remove-by-id, all early-return paths) | 7379 | **done** — peek + `depart_resolving_stack_entry` at the two CR-ordered points + a backstop for the four early returns |
| 1.2 | Audit every resolution-time `stack_objects` consumer (copy/cascade/storm push, LKI, counter, top-of-stack queries, view-model, `check_stack_consistency`) | 7379 | **done** — §2 of the execution notes; two AFFECTED SBAs found, everything else command-time |
| 1.3 | Never-double-seen / never-resolved-twice COUNT assertions | 7379 | **done** — `t5`, `t6`, `r4` COUNT-asserted, never presence |
| 2.1 | Probes: single-target redirect onto the resolving Misdirection, asserted BY RESOLUTION EFFECT | 7379 | **done** — `t1`/`t2` through the real Misdirection and the real Bolt Bend, by RESOLUTION EFFECT |
| 2.2 | Self-targeting (CR 601.2c) still refused both ways | 7379 | **done** — `t4`, `t4b` |
| 2.3 | Real `LocalGame`/`HumanChoice` drive + bot-path offer | 7379 | **done** — `c1` human `LocalGame`/`HumanChoice`, `c2` bot offer accepted, `c3` a stated CONTROL |
| 3.1 | Rider **OOS-DX25-4** decided, reason posted as a task comment | 7380 | **done** — rider **OOS-DX25-4 TAKEN**: both counter paths now consume `stack_registry::source_of` |
| 3.2 | Rider **OOS-DX25b-4** decided, reason posted as a task comment | 7380 | **done** — rider **OOS-DX25b-4 DECLINED** and re-filed with the exact variant + wire cost verified BY EXECUTION (both gates) |
| 3.3 | T7/T8 route-around docs rewritten to what is now true; simplified where the workaround was the only reason for the shape | 7380 | **done** — T7, T8 and `target_spell_with_filter_def` docs rewritten; both fixtures KEPT with the coverage reason stated |
| 4.1 | Census by `all_cards()` PRINTED by a test (declared axis + inverse oracle axis) | 7381 | **done** — `r6_census_report`, union 11, printed BY the test |
| 4.2 | Flips predicted and NAMED before regeneration | 7381 | **done** — 0 flips predicted with the reason (card-def diff EMPTY) before regeneration |
| 5.1 | HASH / PROTOCOL gates executed; bump or UNMOVED with the counterfactual stated | 7381 | **done** — HASH 85 / PROTOCOL 44 both UNMOVED, gate-executed, counts 98/132 measured, counterfactual verified by execution |
| 6.1 | Post-edit suite; delta by test NAME, byte-exact set difference; count-vs-name reconciliation; duplicate-name scan | 7382 | **done** — 5,231/0/5 on 68 targets, +21; 21 additions / 0 leavers / 0 removals; reconciliation agrees; duplicate scan EMPTY |
| 6.2 | clippy / fmt / check-defs-fmt / build --workspace against the FINAL tree | 7382 | **done** — clippy / fmt / check-defs-fmt / build --workspace all clean against the FINAL tree |
| 6.3 | `npm run build` or N/A with the reason | 7382 | **done** — N/A, `tools/` diff EMPTY and `node_modules` absent |
| 6.4 | Benches: merge-base A/B, same-code band FIRST, published honestly | 7382 | **done** — six runs, same-code band 5.73% measured FIRST, no regression; the +3% rows refuted by mechanism |
| 6.5 | Coverage regenerated, churn reverted | 7382 | **done** — UNMOVED at 1,140/1,803 = 63.2%, 0 flips, churn reverted |
| 7.1 | Revert matrix executed by the coordinator; UNDISCRIMINATED rows disclosed in the test itself | 7382 | **done** — 7 rows, coordinator-executed, all three files restored byte-exactly; R2/R3 disclosed as coverage measurements |
| 7.2 | Registry rows closed (pipes escaped); `OOS-DX54-N` filed (grep first) | 7382 | **done** — OOS-DX25c-6 + OOS-DX25-4 CLOSED, OOS-DX25b-4 updated, OOS-DX54-1..3 filed, all rows split to 6 cells |
| 7.3 | v4 memo §4 row 17 struck, banner repointed to rank 18 (PB-DX42b) | 7382 | **done** — row 17 struck, banner repointed to rank 18 with its two preconditions split (one discharged on main) |
| 7.4 | CLAUDE.md Current State + workstream-state handoff + its W6 row | 7382 | **done** — CLAUDE.md Current State + narrative, workstream-state handoff + W6 row |
| 8.1 | `/review` run; every finding taken or declined with a reason | 7382 | **done** — 8 findings (2 HIGH / 2 MEDIUM / 3 LOW / 1 NIT), **all 8 taken**; both HIGHs were gates defeated by execution and both defeats re-executed RED |
| 8.2 | Headline surfaces re-checked against the registry AFTER the fix cycle (dispatch hygiene 8) | 7382 | **done** — registry had `OOS-DX54-1..8` while three headlines said `-1..5`; caught AFTER the cycle, all three corrected |
| 8.3 | `/tmp` bench dirs deleted | 7382 | **done** — bench target dirs and the base worktree cleaned at close |
