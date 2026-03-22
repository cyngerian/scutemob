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
| W6: Primitive + Card Authoring | F-4: Re-author now-expressible TODOs | ACTIVE | 2026-03-22 | F-1/F-2/F-3 DONE. Starting F-4 |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 1 fixes (F-1 through F-3)

**Completed**:
- F-1: Applied all actionable HIGH+MEDIUM fixes (3 card fixes: H1 Rograkh color_indicator, H2 Skrelv comment, M1 Thousand-Year Elixir targets; 4 TODO refinements: M20, M42, M48, M3; 2 verified already fixed: M5 Ajani, M12 Crown of Skemfar; 3 verified no-fix: M13 Emrakul DSL gap, M59 Dryad Arbor correct, M5 already fixed). Commit 00c38a9.
- F-2: All MEDIUM findings resolved — 11 "still valid" handled in F-1, 24+ already fixed by PB, 3 file-not-found deferred. Commit a354532.
- F-3: LOW findings verified — planeswalker loyalty correct (7/7), remaining LOWs cosmetic. Stale TODOs overlap with F-4.

**Next**:
1. F-4: Re-author ~143 cards whose TODOs are now expressible (from T-1 "now expressible" list)
2. F-5: Review all fixed/re-authored cards
3. F-6: Build verification
4. F-7: Phase 1 complete commit
5. Then Phase 2 authoring (A-01 through A-33)

**Hazards**:
- 7 silent wrong-state cards still present: beast_within, call_of_the_nightwing, generous_gift, hanweir_the_writhing_township, overlord_of_the_hauntwoods, swan_song, mana_crypt
- Many pre-existing unstaged changes in working tree from prior sessions (engine code, tests, docs)

**Commit prefix used**: `W6-fix:`

## Handoff History

### 2026-03-22 — W6: Phase 1 Fixes (F-1 through F-3)
- F-1: 3 card fixes + 4 TODO refinements + 5 verified (already fixed / DSL gap / correct)
- F-2: All MEDIUM resolved (most by prior PB work). F-3: LOWs verified cosmetic-only.
- 0 actionable HIGH/MEDIUM remaining in consolidated fix list
- Commits: 00c38a9, a354532

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
