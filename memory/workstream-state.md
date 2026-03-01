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
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-02-28 (late night)
**Workstream**: W5: Card Authoring
**Task**: Author top ready cards from worklist (second batch)
**Completed**:
- Authored 7 more cards (commit `c3e80e0`): Demonic Tutor, Worldly Tutor, Vampiric Tutor, Mana Confluence, Phyrexian Altar, Impact Tremors, Skullclamp (partial — death trigger omitted, clear TODO)
- Exported `LibraryPosition` + `TargetController` from helpers.rs prelude
- Triaged ~50 top ready cards; found most blocked by DSL gaps not yet in generate_worklist.py
- 3 newly-identified DSL gaps blocking many cards: (1) multi-type OR filter, (2) Activated has no TargetRequirement field, (3) WheneverCreatureDies has no controller filter
**Next**: W5 is low-yield until more ability/DSL gaps are resolved. Recommended: finish W1 abilities first, then add DSL_GAP_PATTERNS for the 3 new gaps to generate_worklist.py, then resume W5. Simple 1-deck feasible cards still available (Fyndhorn Elves, Dark Ritual, Ornithopter, Avacyn's Pilgrim, Terminate, etc.)
**Hazards**: W1 Shadow changes (combat.rs, types.rs, hash.rs, view_model.rs, tests/shadow.rs) are staged but uncommitted — leave for W1 session
**Commit prefix used**: `W5-cards:`

## Handoff History

### 2026-02-28 (night) — W5: Card Authoring (first batch)
- Batches A/B/C: authored 30, removed 22 with simplifications; 8 accurate cards remain
- Fixed generate_worklist.py: DSL_GAP_PATTERNS, blocked classification; commit prefix `W5-cards:`

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
