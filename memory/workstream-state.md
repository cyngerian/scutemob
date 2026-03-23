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
| W6: Primitive + Card Authoring | — | available | — | **A-18 draw COMPLETE**; A-19 token-create 4/14 sessions done (48 cards). Next: S44+. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 2 authoring — A-18 draw completion + A-19 token-create start

**Completed**:
- A-18 draw sessions S20-S25 COMPLETE: 43 new cards (S24 blocked, 6 cards skipped)
- Reviewed by card-batch-reviewer: 11 HIGH findings fixed (overbroad triggers, wrong costs)
- A-19 token-create sessions S40-S43: 48 new cards
- Total this session: 91 new card defs
- Commits: fc27279, 047532f, f43d88b
- Total: ~1173 card def files
- All 2281 tests passing, 0 clippy, workspace builds clean

**Next**:
1. **A-19 token-create sessions S44-S53** (107 remaining cards, 10 sessions) — continue token group
2. Review A-19 S40-S43 with card-batch-reviewer
3. Then A-20 pump-buff through remaining groups

**Hazards**:
- Direct authoring 5-10x faster than bulk-card-author for complex cards
- `WheneverYouCastSpell` lacks spell-type filter — Lys Alana Huntmaster, Murmuring Mystic have empty abilities to avoid wrong game state
- `WheneverCreatureDies` overbroad (all creatures, not "your creatures") — Bastion of Remembrance, Pawn of Ulamog, etc. use it as approximation
- Per-creature combat damage triggers NOT in DSL — Old Gnawbone, Professional Face-Breaker have TODOs
- Copy-token creation not in DSL — Kiki-Jiki, Miirym have TODOs
- Token doubling replacement not in DSL — Doubling Season, Parallel Lives body-only
- Count-based token amounts not in DSL — Dockside Extortionist, Avenger of Zendikar use fixed approximations
- `Ward(u32)` takes a parameter (not bare `Ward`) — caught in S42 build
- `KeywordAbility::Mentor` does not exist — caught in S43 build
- 3 existing cards in A-19: Hanweir Garrison (S45), Basri Ket (S46), Wrenn and Seven (S53) — skip these

**Commit prefix used**: `W6-cards:`

## Handoff History

### 2026-03-22 — W6: Phase 2 authoring A-18 draw S10-S19
- 119 new cards (10 sessions of 16 complete)
- Commits: 4a15b5e, 9bf3870, 6ecdb68. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-14 through A-17 (Tier 2 cont'd)
- 45 new cards (damage-each, bounce, minus, counter). All HIGH fixed.
- Commit: 6ddc832. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-11 through A-13 (Tier 2 start)
- 88 new cards (A-11 destroy + A-12 exile + A-13 damage-target). DSL ext: DestroyPermanent.cant_be_regenerated.
- Commits: 52be340, 18ca67e, 68e2b9f, 064ccbb. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-05 through A-10 (Tier 1 completion)
- 36 new cards (A-05 through A-10). All reviewed, 8H+9M+10L findings — all HIGH fixed.
- Tier 1 COMPLETE: A-01 through A-10 + A-31/A-37 pre-existing = 12 groups done.

### 2026-03-22 — W6: Phase 2 authoring A-01 through A-04
- 52 new card defs authored (16 mana-creature + 33 mana-artifact + 3 mana-other).
