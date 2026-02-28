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
| W5: Card Authoring | Batch A: Tier 2 ready cards (Skullclamp, Ancient Tomb, Blood Artist, etc.) | ACTIVE | 2026-02-28 | Phase 9 ready; top Tier 2 by deck count |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-02-28 (evening)
**Workstream**: W5: Card Authoring (setup only — no cards authored)
**Task**: Create W5 workstream infrastructure; plan Phase 9 batch authoring
**Completed**:
- Added W5 as a proper workstream (separate from W1 abilities — different files, parallelizable)
- Updated workstream-state.md, CLAUDE.md, docs/workstream-coordination.md, start-work skill
- Commit prefix established: `W5-cards:`
- Worklist confirmed: 1,061 ready cards, top Tier 2 = Skullclamp (7 decks), Ancient Tomb, Wooded Foothills (6), Blood Artist, Viscera Seer, Exotic Orchard, Godless Shrine, etc. (5)
- Phase 5 progress checkboxes added to workstream-coordination.md
**Next**: Claim W5, run `python3 tools/generate_skeleton.py` for Batch A (~10 Tier 2 cards), author abilities, `cargo check` per card, `cargo test --all`, commit `W5-cards: author ...`
**Hazards**: Working tree has uncommitted changes to CLAUDE.md, workstream-state.md, workstream-coordination.md, start-work skill — commit as `chore:` before starting card authoring
**Commit prefix used**: `chore:` (infrastructure only this session)

## Handoff History

### 2026-02-28 (late) — W2: Card Pipeline Phases 5-9
- Split definitions.rs → 112 files in defs/; build.rs auto-discovery; skeleton generator; agent rewrite; commit `f9f7c45`

### 2026-02-28 — W2: Overview layout + Card DSL detail pane
- Overview bottom row: 3-column layout; Card DSL parser + detail pane; commit `8d78f7b`

### 2026-02-28 — W3: T2 dead code removal + Phase 0 complete
- MR-M1-14, MR-M9.5-08, MR-M9.4-11 closed; commit `7d535ec`; Phase 0 complete

### 2026-02-28 — W3: T1 tests (14 total)
- MR-M1-19/20, MR-M2-07/08/17, MR-M4-13, MR-M5-08, MR-M6-08, MR-M8-15, MR-M9-14/15, MR-M9.4-13/14/15; commit `320b77f`

### 2026-02-28 — Cross-cutting (chore): Workstream coordination infrastructure
- Created ability-batch-plan.md, workstream-coordination.md, workstream-state.md, 3 skills
