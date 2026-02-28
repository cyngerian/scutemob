# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | Batch 0: P3 stragglers (Overload, Bolster, Adapt, Partner With) | ACTIVE | 2026-02-28 | Hideaway already closed; working remaining 4 |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | Phase 0 complete; T2 done; Phase 1 (abilities) next |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | Phase 9 ready; top Tier 2 by deck count |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-02-28 (night)
**Workstream**: W5: Card Authoring
**Task**: Author first card batches + fix worklist to block DSL-gap cards
**Completed**:
- Batches A/B/C: authored 30 cards, then audited and removed 22 that used simplifications
- 8 accurate cards remain: Ancient Tomb, Ashnod's Altar, Anguished Unmaking, Viscera Seer, Zulaport Cutthroat, Heroic Intervention, Prismatic Vista, Sakura-Tribe Elder
- Fixed `generate_worklist.py`: added DSL_GAP_PATTERNS (5 patterns), `check_oracle_dsl_gaps()`, blocked classification, reporting; fixed `parse_authored_cards` for one-file-per-card layout; regenerated `_authoring_worklist.json`
- All 22 removed cards now classified as `blocked` with specific DSL gap reason
- Top DSL gaps discovered: targeted_trigger (57 cards), return_from_graveyard (17), nonbasic_land_search (15), count_threshold (14), shock_etb (10)
**Next**: Author more Tier 2 ready cards — run `python3 test-data/test-decks/generate_worklist.py` to get current list; top ready cards by deck count are visible in the JSON; avoid any card with `blocking_dsl_gaps`
**Hazards**: None — all commits clean; worklist JSON regenerated and committed
**Commit prefix used**: `W5-cards:` (cards), `W5:` (worklist fix)

## Handoff History

### 2026-02-28 (night) — W2: TUI & Simulator (UX fixes)
- Fix 1-6: Hand scrolling, discard events, zone counters, zone browser overlay, CardDetail return-to, action hints; Esc bug fix; commit `f9f7c45`

### 2026-02-28 (evening) — W5: Card Authoring (setup)
- Added W5 as workstream; worklist confirmed (1,061 ready); commit prefix `W5-cards:`

### 2026-02-28 (late) — W2: Card Pipeline Phases 5-9
- Split definitions.rs → 112 files in defs/; build.rs auto-discovery; skeleton generator; agent rewrite; commit `f9f7c45`

### 2026-02-28 — W2: Overview layout + Card DSL detail pane
- Overview bottom row: 3-column layout; Card DSL parser + detail pane; commit `8d78f7b`

### 2026-02-28 — W3: T2 dead code + T1 tests (Phase 0 complete)
- MR-M1-14/19/20, MR-M2-07/08/17, MR-M4-13, MR-M5-08, MR-M6-08, MR-M8-15, MR-M9-14/15, MR-M9.4-11/13/14/15 closed; commits `320b77f` `7d535ec`
