# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | Batch 0-15 + Mutate mini-milestone |
| W2: TUI & Simulator | — | available | — | Phase 1 done; hardening pending |
| W3: LOW Remediation | T2: dead code removal (MR-M1-14, MR-M9.5-08) + Phase 0 commit | ACTIVE | 2026-02-28 | Completes Phase 0 |
| W4: M10 Networking | — | not-started | — | After W1 completes |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-02-28
**Workstream**: W3: LOW Remediation
**Task**: T1: 10 new tests + 4 existing test improvements
**Completed**:
- MR-M1-19 `object_identity.rs`: same-zone move produces new ObjectId (CR 400.7)
- MR-M1-20 `object_identity.rs`: move to invalid zone returns Err
- MR-M2-07 `turn_invariants.rs`: `run_pass_sequence` now adds library cards (no deck-out)
- MR-M2-08 `concede.rs`: active player concedes after all others passed — fixed `last_regular_active` needed alongside `active_player`
- MR-M2-17 `concede.rs`: non-active player concedes during combat phase
- MR-M4-13 `sba.rs`: aura whose target left battlefield triggers SBA 704.5m
- MR-M5-08 `layers.rs`: CDA vs non-CDA sublayer ordering test (CR 613.3)
- MR-M6-08 `test-data/combat/104_*.json`: first-strike + deathtouch script (70 approved scripts)
- MR-M8-15 `replacement_effects.rs`: self-ETB + global ETB replacement both apply (CR 614.15)
- MR-M9-14 `commander.rs`: 3+ mulligans with escalating bottom count
- MR-M9-15 `commander.rs`: BringCompanion rejected with non-empty stack
- MR-M9.4-13 `loop_detection.rs`: fixed tautological assertion
- MR-M9.4-14 `trigger_doubling.rs`: full ETB→register→doubling pipeline test (drain loop fix)
- MR-M9.4-15 `card_def_fixes.rs`: no-Vessel active player discards to hand size
- Checked Phase 0 boxes in `docs/workstream-coordination.md`
- All 1118 tests pass; 70 approved scripts pass; clippy clean
- Commit: `320b77f` W3: implement all 14 T1 tests
**Next**: W3 T2 — dead code removal (MR-M1-14, MR-M9.5-08); also Phase 0 checkbox `[ ] W3 T1: Dead code removed` still unchecked
**Hazards**: Many pre-session uncommitted files remain (TUI, CLAUDE.md, agent files, ability memory deletes) — not staged, safe to ignore for W3
**Commit prefix used**: `W3:`

## Handoff History

### 2026-02-28 — Cross-cutting (chore): Improve /start-session progress checkboxes
- Fixed `/start-session` to read progress checkboxes from `docs/workstream-coordination.md`
- New Step 3 identifies current phase, next unchecked item, and concrete workstream recommendation
- Also checks `memory/ability-wip.md` for in-progress abilities that take priority

### 2026-02-28 — Cross-cutting (chore): Workstream coordination infrastructure
- Created `docs/ability-batch-plan.md`, `docs/workstream-coordination.md`, `memory/workstream-state.md`
- Created 3 project-scoped skills: `/start-session`, `/start-work`, `/end-session`
- Reorganized `memory/abilities/` (109 plan+review files moved)
- Updated CLAUDE.md with commit convention and session protocol
