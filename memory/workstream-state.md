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
| W6: Primitive + Card Authoring | — | paused | — | F-4 in progress (24/~100 cards done). Next: F-4 session 2 |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: F-4 session 1 — implement now-expressible TODOs

**Completed**:
- F-4 session 1: 24 card defs fixed (TODOs replaced with real abilities). Commit bd63333.
  - Batch 1 (10): Castle cycle (Scry, token, mass pump, draw+life), Karn's Bastion (Proliferate),
    Kher Keep (Kobold token), Strip Mine + Wasteland (destroy land), Deserted Temple (untap land),
    Halimar Depths (ETB Scry 3)
  - Batch 2 (11): Mortuary Mire (ETB GY→top), Torch Courier (sac haste grant),
    Tainted Field/Isle/Wood (conditional mana via activation_condition), Glistening Sphere
    (ETB tapped + proliferate + any-color), Minamo + Wirewood Lodge (untap targets),
    Skemfar Elderhall (sac + -2/-2 + tokens), Gnarlroot Trapper + The Seedcore (stale TODO fixes)
  - Batch 3 (3): Voldaren Estate (PayLife fix + Blood token), Oboro (self-bounce),
    Geier Reach Sanitarium (ForEach EachPlayer draw+discard)

**Next**:
1. F-4 session 2-N: ~76-100 more NOW_EXPRESSIBLE cards remain (see `memory/card-authoring/dsl-gap-audit-v2.md`)
2. F-5: Review all fixed/re-authored cards
3. F-6: Build verification
4. F-7: Phase 1 complete commit
5. Then Phase 2 authoring (A-01 through A-42)

**Hazards**:
- 7 silent wrong-state cards still present: beast_within, call_of_the_nightwing, generous_gift, hanweir_the_writhing_township, overlord_of_the_hauntwoods, swan_song, mana_crypt
- Pre-existing unstaged changes in CLAUDE.md from prior sessions
- Some NOW_EXPRESSIBLE cards reference G-XX gap IDs — those need careful evaluation (some are truly still blocked)

**Commit prefix used**: `W6-fix:`

## Handoff History

### 2026-03-22 — W6: F-4 session 1 (24 now-expressible card defs)
- 24 card defs fixed across 3 batches: activated abilities, ETB triggers, stale TODOs
- Patterns: Scry, Proliferate, CreateToken, DestroyPermanent, UntapPermanent, MoveZone,
  ApplyContinuousEffect, ForEach EachPlayer, activation_condition, Cost::PayLife
- Commit: bd63333. 2281 tests passing.

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
