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
| W3: LOW Remediation | — | available | — | Phase 0 complete; T2 done; Phase 1 (abilities) next |
| W4: M10 Networking | — | not-started | — | After W1 completes |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-02-28
**Workstream**: W2: TUI & Simulator
**Task**: Overview layout restructure + Card DSL in detail pane
**Completed**:
- Overview bottom row: 3-column layout (Reviews/Engine stacked left, Scripts middle, Cards right)
- Engine Size restored as its own bordered widget, stacked below Code Reviews
- Card DSL parser: extracts CardDefinition blocks from definitions.rs, dedents, keyed by name
- Cards tab detail pane: shows syntax-highlighted DSL (cyan keys, white values) for authored cards
- Detail pane scrollable via Shift+J/K; scroll resets on row/filter change
- Clippy clean; commit `8d78f7b`
**Next**: Other uncommitted TUI files (play/, render.rs) from prior sessions still unstaged. Phase 2 TUI work (file watching, Session tab, stepper subcommand) remains.
**Hazards**: Uncommitted changes in engine files (turn_structure.rs, builder.rs, replay_harness.rs, tests) and other TUI files from prior sessions
**Commit prefix used**: `W2:`

## Handoff History

### 2026-02-28 — W3: T2 dead code removal + Phase 0 complete
- MR-M1-14, MR-M9.5-08, MR-M9.4-11 closed; commit `7d535ec`
- Phase 0 complete; Phase 1 (abilities) next

### 2026-02-28 — W3: T1 tests (14 total)
- MR-M1-19/20, MR-M2-07/08/17, MR-M4-13, MR-M5-08, MR-M6-08, MR-M8-15, MR-M9-14/15, MR-M9.4-13/14/15
- 1118 tests pass; 70 approved scripts; commit `320b77f`

### 2026-02-28 — Cross-cutting (chore): Improve /start-session progress checkboxes
- Fixed `/start-session` to read progress checkboxes from `docs/workstream-coordination.md`
- New Step 3 identifies current phase, next unchecked item, and concrete workstream recommendation

### 2026-02-28 — Cross-cutting (chore): Workstream coordination infrastructure
- Created `docs/ability-batch-plan.md`, `docs/workstream-coordination.md`, `memory/workstream-state.md`
- Created 3 project-scoped skills: `/start-session`, `/start-work`, `/end-session`
- Reorganized `memory/abilities/` (109 plan+review files moved)
