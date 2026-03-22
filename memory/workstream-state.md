# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | B16 complete (Dungeon + Ring); all abilities done |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | **W3 LOW sprint DONE** (S1-S6): 83→29 open (119 closed total). TC-21 done. 2233 tests. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | F-1: Apply HIGH fixes from consolidated fix list | ACTIVE | 2026-03-22 | Phase 0 triage DONE. Starting Phase 1 fixes. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 0 triage (T-1 through T-7)

**Completed**:
- T-1: DSL gap audit — 569 TODOs classified (143 now-expressible, 96 partial, 313 blocked, 17 stale)
- T-2: 28 blocked sessions re-evaluated — 17 unblocked to ready
- T-3: 6 deferred sessions re-evaluated — 4 → ready, 2 → blocked
- T-4: 73 review files consolidated — 15 actionable cards remain (most fixed by PB work)
- T-5: 264 pre-existing defs inventoried — 7 silent wrong-state cards identified
- T-6: Triage summary written
- T-7: Committed (9a27d9c)

**Next**:
1. F-1: Apply HIGH fixes from consolidated fix list (5 cards, ~30 min)
2. Then F-2 (MEDIUM fixes), F-3 (LOW fixes)
3. Then fix NOW_EXPRESSIBLE TODOs (~100 cards, ~12 sessions)
4. Then author ready sessions (168 sessions, ~1,467 new cards)

**Hazards**:
- 7 silent wrong-state cards (empty abilities, no TODO): beast_within, call_of_the_nightwing, generous_gift, hanweir_the_writhing_township, overlord_of_the_hauntwoods, swan_song, mana_crypt
- Many pre-existing unstaged changes in working tree from prior sessions (engine code, tests, docs) — not related to this triage work

**Commit prefix used**: `W6-triage:`

## Handoff History

### 2026-03-22 — W6: Card Authoring Infrastructure (I-1 through I-6)
- Created card-fix-applicator agent, 3 new skills (triage-cards, author-wave, audit-cards)
- Updated bulk-card-author (33 groups, 13 KI patterns, MCP 30) and card-batch-reviewer (12 checks, 19 KI patterns, Now-Expressible table)
- Operations plan status: DRAFT → ACTIVE

### 2026-03-21 — W6: PB-22 S7 (Adventure CR 715 + Dual-Zone Search)
- AltCostKind::Adventure + adventure_face on CardDefinition + exile-on-resolution
- also_search_graveyard on Effect::SearchLibrary (dual-zone search)
- 2 new cards (Bonecrusher Giant, Lovestruck Beast), 3 fixed (Monster Manual, Finale, Lozhan)
- Review: 1H 3M, all fixed. 9 new tests, 2281 total. **PB-22 COMPLETE**

### 2026-03-21 — W6: PB-22 S6 (Emblem Creation, CR 114)
- Effect::CreateEmblem + GameEvent::EmblemCreated + emblem trigger scanning
- 6 planeswalker card defs authored/fixed. Review: 5H 6M 2L, all fixed. 2272 total

### 2026-03-21 — W6: PB-22 S5 (Copy/Clone Primitives)
- Effect::BecomeCopyOf + Effect::CreateTokenCopy + Condition::CardTypesInGraveyardAtLeast
- Review: 0H 2M 3L. 5 new tests, 2265 total

### 2026-03-20 — W6: PB-22 S1 (activation_condition)
- activation_condition on activated abilities (CR 602.5b). 305 files, 2236 total
