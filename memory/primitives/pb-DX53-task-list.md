# PB-DX53 — task list (scutemob-231)

Substitute for TaskCreate (this harness exposes none). Mirrored as an `esm task comment`.
Status legend: `[ ]` pending · `[~]` in progress · `[x]` done.

## Stage 0 — orientation, measurement, prediction (no production line yet)
- [x] 0.1 Pre-edit full-workspace baseline to a file — **5,196 / 0 / 5 on 66 targets**; reproduces CLAUDE.md:273's PB-DX39 close-out exactly, AC 7371's 5,194 does not
- [x] 0.2 MCP rules research: windbrisk ruling 2007-10-01, CR 508.1/508.3d/508.4/508.6, CR 400.7, legions_landing 2017-09-29
- [x] 0.3 Census by `all_cards()`: every `Condition::YouAttackedWithNOrMore` reader (AC 7370)
- [x] 0.4 Inverse oracle axis census ("attacked with", "attacked this turn", …), classified per-declaration vs per-turn
- [x] 0.5 Census: every `Effect::AdditionalCombatPhase` declarer + its `Completeness` marker
- [x] 0.6 Wire probe by EXECUTION — `Condition` in BOTH closures, `TriggerCondition` in NEITHER, counts **98 / 132**: PlayerState / Condition / TriggerCondition / TriggerData / PendingTrigger in the PROTOCOL and HASH closures (extend CLOSURE_MUST_NOT_CONTAIN, run the walk)
- [x] 0.7 `size_of::<PlayerState>()` at the merge base — **376** (GameState 3536, `OrdSet<ObjectId>` 24, `Condition` 304)
- [x] 0.8 Design decision written down with the rejected alternatives costed
- [x] 0.9 Wire prediction COMMITTED in writing before any production line (HASH / PROTOCOL, per half, closure type counts)
- [x] 0.10 Coverage-flip prediction written per def before any code

## Stage 1 — engine implementation
- [x] 1.1 PlayerState gains the per-turn dedup'd attacker set (hashed)
- [x] 1.2 `rules/combat.rs` declare-attackers site accumulates instead of assigning; CR 508.4 entrants excluded
- [x] 1.3 `rules/turn_actions.rs` turn-start clear
- [x] 1.4 `state/builder.rs` initialiser
- [x] 1.5 `state/hash.rs` field hashed + history row appended + HASH bump
- [x] 1.6 Condition evaluation split: per-turn gate for windbrisk vs per-declaration gate for legions_landing
- [x] 1.7 Card-def edits: windbrisk_heights KNOWN RESIDUAL comment rewritten; legions_landing comment reconciled
- [x] 1.8 `rules/combat.rs` OOS-DX21-1 residual comment DELETED (OOS-DX47-6)
- [x] 1.9 Sentinel re-pin by symbol; survivor scan on BOTH axes (shape + suffix-tolerant value); every changed line read for over-replacement

## Stage 2 — tests
- [x] 2.1 Primitive probes: extra-combat accumulation, dedup (vigilant creature in both combats), CR 508.4 entrant exclusion (pinned with cite)
- [x] 2.2 Channel probe: real `LocalGame`/`HumanChoice` drive, 3 attackers combat 1 + 1 combat 2, hideaway activation ACCEPTED and the exiled card RESOLVING
- [x] 2.3 legions_landing byte-identical-across-extra-combat probe (PB-DX21 finding M3)
- [x] 2.4 Roster gates: the two censuses PRINTED by a test (SR-36), ratcheted
- [ ] 2.5 Revert matrix executed by the coordinator; UNDISCRIMINATED rows disclosed in the test itself

## Stage 3 — gates and close-out
- [x] 3.1 Full-workspace post run; delta by test NAME (byte-exact set difference), count-vs-name reconciliation, duplicate-name scan
- [x] 3.2 `clippy --workspace --all-targets -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh`, `cargo build --workspace` — AGAINST THE FINAL TREE
- [x] 3.3 `npm run build` run or N/A with the reason stated
- [x] 3.4 Coverage regenerated, self-dating churn reverted, flips reconciled against the prediction
- [x] 3.5 Benches: same-code band FIRST, then merge-base A/B; `size_of::<PlayerState>()` at both revisions
- [x] 3.6 `memory/primitives/pb-DX53-execution-notes.md` written
- [x] 3.7 Registry: OOS-DX21-1 row closed (pipes escaped); OOS-DX53-N seeds filed after grepping the registry
- [x] 3.8 v4 memo §4 row 16 struck, banner repointed to rank 17 (PB-DX54)
- [x] 3.9 CLAUDE.md Current State + `memory/workstream-state.md` handoff
- [x] 3.10 `/review` run; all findings taken or declined with reasons
- [x] 3.11 Headline surfaces re-checked against the registry AFTER the fix cycle (dispatch hygiene 8)
- [ ] 3.12 Scratchpad bench target dirs deleted (dispatch hygiene 11)
