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
| W6: Primitive + Card Authoring | A-18: draw (161 cards, 14 sessions) | ACTIVE | 2026-03-22 | Phase 2 authoring — Tier 2 cont'd |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 2 authoring — A-14 through A-17 (Tier 2 cont'd: damage-each, bounce, minus, counter)

**Completed**:
- A-14 removal-damage-each: 16 new cards. Goblin Chainwhirler full ETB (EachPermanentMatching for creatures+planeswalkers). 5H 4M 3L, all HIGH fixed.
- A-15 removal-bounce: 9 new cards + 1 existed (Cyclonic Rift). Snap, Ninjutsu creatures, Auras. 1H 2M 2L, HIGH fixed.
- A-16 removal-minus: 4 new cards (Dismember w/ Phyrexian mana, Drown in Ichor w/ Proliferate). 2H 1M 1L, HIGH fixed.
- A-17 counter: 16 new cards. Mental Misstep uses TargetSpellWithFilter(max_cmc=1, min_cmc=1). 10 wrong-game-state partials stripped (counter-unless, missing costs). 8H 3M 1L, all HIGH fixed.
- Total: 45 new card defs, 963 total card files
- Commit: 6ddc832
- All 2281 tests passing, 0 clippy, workspace builds clean

**Next**:
1. **A-18 draw** (161 cards, 14 sessions) — largest group, will need many sessions
2. **A-19 token-create** (146 cards, 13 sessions) — second largest
3. Then A-20 through remaining groups

**Hazards**:
- Writing cards directly is faster than bulk-card-author agents for complex cards
- CounterUnlessPays not in DSL — all "counter unless pays" cards stripped
- `WheneverYouDiscard` trigger not in DSL — blocks Glint-Horn Buccaneer, Brallin
- `EachPermanentMatching` works for damage-to-all-opponents-creatures (Goblin Chainwhirler pattern)
- `TargetSpellWithFilter` with `max_cmc`/`min_cmc` works for MV-restricted counterspells
- Same DSL gaps as prior session still apply (CreateToken player, WheneverACreatureDies filter, etc.)

**Commit prefix used**: `W6-cards:`

## Handoff History

### 2026-03-22 — W6: Phase 2 authoring A-11 through A-13 (Tier 2 start)
- 88 new cards (A-11 destroy + A-12 exile + A-13 damage-target). DSL ext: DestroyPermanent.cant_be_regenerated.
- Commits: 52be340, 18ca67e, 68e2b9f, 064ccbb. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-05 through A-10 (Tier 1 completion)
- 36 new cards (A-05 through A-10). All reviewed, 8H+9M+10L findings — all HIGH fixed.
- Tier 1 COMPLETE: A-01 through A-10 + A-31/A-37 pre-existing = 12 groups done.

### 2026-03-22 — W6: Phase 2 authoring A-01 through A-04
- 52 new card defs authored (16 mana-creature + 33 mana-artifact + 3 mana-other).
- 5 groups verified pre-existing (body-only, land-etb-tapped, combat-keyword, mana-land).

### 2026-03-22 — W6: Phase 1 close (F-4 sweep + F-5/F-6/F-7)
- Phase 1 Fix COMPLETE. Next: Phase 2 authoring A-01.

### 2026-03-22 — W6: F-4 session 6 (11 now-expressible cards)
- 3 lands, 1 conditional mana, 1 conditional ETB, 1 equipment bounce, 3 keyword additions. 2281 tests.
