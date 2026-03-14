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
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; T3 ManaPool pending |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | PB-0: Quick-win card fixes (23 cards) | ACTIVE | 2026-03-13 | Fixing wrong-game-state card defs — ETB-tapped, painlands, etc. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-13
**Workstream**: W5 → W6 (strategic pivot)
**Task**: Wave 3 mana-land authoring + DSL gap audit + primitive-first card plan

**Completed**:
- Authored Wave 3 mana-land: 78 new card defs (sessions 71–77), committed `0896563`
- 15 review batches, fix pass (8 HIGH, 5 MEDIUM), 718 total defs, 1972 tests
- **DSL gap audit**: scanned all 718 existing card defs — 418 have TODOs, 122 produce wrong game state. Audit at `memory/card-authoring/dsl-gap-audit.md`
- **Unauthored card scan**: analyzed 1,195 remaining cards from authoring universe. Found new gaps: library_search_filters (74 cards), planeswalkers (31, was 4), stax restrictions (13), mass destroy (12), additional combat (10), fight/bite (5)
- **Created primitive-first card plan**: `docs/primitive-card-plan.md` — 21 primitive batches (PB-0 to PB-21), then bulk authoring, then final audit. Zero deferrals. All 1,743 cards complete pre-alpha.
- W5 retired, replaced by W6

**Next**:
1. **TOP PRIORITY**: Start W6 with PB-0 (23 quick-win card fixes, 1 session)
2. Then PB-1 (pain land mana-with-damage, 8 cards, 1 session)
3. Then PB-5 (targeted abilities, 32 cards, highest leverage)
4. Follow execution order in `docs/primitive-card-plan.md`

**Hazards**: 122 existing card defs produce wrong game state (ETB-tapped missing, painlands give free colored mana, etc.). PB-0 through PB-3 fix the most dangerous ones. The audit doc at `memory/card-authoring/dsl-gap-audit.md` has the full list.

**Commit prefix**: `W6-prim:` (primitive batches), `W6-cards:` (bulk authoring)

## Handoff History

### 2026-03-13 — W5: Wave 3 mana-land (78 cards) + strategic pivot to W6
- Wave 3: 78 cards authored+reviewed+fixed, commit 0896563; DSL gap audit; unauthored card scan (1,195 cards); primitive-first plan created; W5 retired → W6

### 2026-03-13 — W5: Wave 2 combat-keyword (187 cards) complete
- 14 sessions (26–39); 38 review batches; 13 HIGH fixes; commits d83ac94+01e3b52; 640 total card defs; 1944 tests

### 2026-03-12 — W5 recovery: Wave 1 recovered, reviewed, fixed, committed
- Recovered lost session (82 cards on disk); 17 review batches; fix pass (39 files); commit e04ce0d; 453 total card defs

### 2026-03-10 — W5: Card Authoring (Phase 1)
- bulk_generate.py: 114 template card defs (371 total); 20 review batches; all HIGH/MEDIUM fixed; 1972 tests

### 2026-03-10 — W1 (B16 closeout) + W5 (card authoring planning)
- B16 complete: Dungeon + Ring; 24 card defs; EDHREC data; 1,743 card universe; authoring plan + 2 new agents
