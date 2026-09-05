# PB-DX55 — task list (visible progress ledger)

Harness note: this build exposes no `TaskCreate`/`TodoWrite` to workers, so this file plus an
`esm task comment` is the substitute (dispatch hygiene 10).

Legend: `[ ]` pending · `[~]` in progress · `[x]` done

## Stage 0 — measure before touching anything
- [x] 0.1 Pre-edit full-workspace baseline to a file; reproduce the 5,243 / 0 / 5 pin on 69 targets
- [x] 0.2 Grep the registry for OOS-SIM6-3 / OOS-SIM5-3 / OOS-SIM5-5 (dispatch hygiene 5)
- [x] 0.3 Re-run the exact §2.6 invocation at HEAD and capture the by-class refusal table
- [x] 0.4 Reconcile EVERY class to a seed or a new filing; publish the merge-base histogram
- [x] 0.5 FILE the three seeds into the registry (pipes escaped), from their handoff source text
- [x] 0.6 Census: mana-bearing `Command` variants (CEILING, from the enum + payment sites)
- [x] 0.7 Census: blocker refusal predicates (engine) vs offer predicates (simulator)
- [x] 0.8 Census: modal ACTIVATED abilities in `all_cards()` (SR-36, not a source grep)
- [x] 0.9 Wire prediction IN WRITING per half, committed before any production line

## Implement
- [x] 1 OOS-SIM6-3 — one cost arithmetic + one solver for every mana-bearing Command, both paths
- [x] 2 OOS-SIM6-3 — offer/acceptance agreement (SR-38): unsolvable is not offered
- [~] 3 OOS-SIM5-3 — ONE blocker-legality predicate consumed by engine and offer
- [x] 4 OOS-SIM5-5 — per-mode target slice for activated abilities through the SHARED helper
- [ ] 5 Consumers: `targeting.rs::plan_targets`, `params.rs`, play-server, TargetPicker

## Probes and gates
- [ ] 6 Refusal-class instrument: aggregate table printed + three classes pinned at ZERO
- [x] 7 LocalGame/HumanChoice drive: empty pool + untapped lands → activation accepted, resolves
- [x] 8 POST /api/game/action drive: same, no manual TapForMana first
- [ ] 9 Blocker probes: offer ABSENT and engine refusal ABSENT on the same fixture, per predicate
- [x] 10 Modal activated ability: bot announces a per-mode target, engine accepts
- [ ] 11 Roster/mechanism gates (one arithmetic, no second copy), each revert-proven

## Close-out
- [ ] 12 Full-workspace suite; delta by NAME (byte-exact set diff, non-end-anchored regex)
- [ ] 13 clippy / fmt / check-defs-fmt / build against the FINAL tree; npm run build state
- [ ] 14 Wire gates executed; HASH/PROTOCOL unmoved (or the bump read off the gate)
- [ ] 15 Fuzz A/B on the PB-DX32 gate config, movement attributed
- [ ] 16 Benches: measured or 'not measured' with the reason
- [ ] 17 Coverage regenerated; flips stated with reason
- [ ] 18 Revert matrix executed by the coordinator, written to pb-DX55-execution-notes.md
- [ ] 19 Registry: three rows closed, OOS-DX55-N filed
- [ ] 20 v4 memo row 19 struck, banner repointed to rank 20 (PB-DX56)
- [ ] 21 CLAUDE.md Current State + workstream-state handoff + W6 row
- [ ] 22 /review, all findings taken or declined with reasons
- [ ] 23 Headline surfaces re-checked against the registry AFTER the fix cycle (hygiene 8)
- [ ] 24 /tmp bench dirs deleted
