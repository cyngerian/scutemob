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
| W6: Primitive + Card Authoring | — | available | — | **A-18 draw IN PROGRESS** (10/16 sessions done, 119 new cards). Next: S20-S23+S25. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 2 authoring — A-18 draw (sessions 10-19 of 16)

**Completed**:
- A-18 draw sessions 10-19: 119 new card defs (10 sessions of 16 complete)
- Added ProtectionQuality to helpers.rs prelude
- Key cards: Phyrexian Arena, Opt, Preordain, The One Ring, Tatyova, Beast Whisperer, Psychosis Crawler, Teferi Temporal Pilgrim, The Locust God, Liliana Dreadhorde General, Tireless Tracker, Nadir Kraken, Chasm Skulker, and 106 more
- Commits: 4a15b5e, 9bf3870, 6ecdb68
- Total: ~1082 card def files
- All 2281 tests passing, 0 clippy, workspace builds clean

**Next**:
1. **A-18 draw sessions 20-23+25** (43 remaining cards, 5 sessions) — finish the draw group
2. **A-19 token-create** (146 cards, 13 sessions) — second largest group
3. Then A-20 through remaining groups

**Hazards**:
- Direct authoring is 5-10x faster than bulk-card-author agents for complex draw cards
- `WheneverYouDrawACard` trigger EXISTS in DSL — agents were told it doesn't, stale info
- `WheneverCreatureDies` is overbroad (all creatures, no controller/type filter) — used as approx for many death-draw cards
- Per-creature combat damage triggers NOT in DSL — Coastal Piracy, Bident, Ohran Frostfang all have TODOs
- `WheneverYouCastSpell` lacks spell-type filter — Beast Whisperer, Sram use unfiltered approx
- `MayPayOrElse` is opponent-pays-or-else, NOT self-may-pay — Nadir Kraken, Miara simplified
- Horizon lands (Fiery Islet, Silent Clearing, Nurturing Peatland) use `AddManaChoice` for "Add X or Y"
- Session 24 blocked (6 cards including Life from the Loam)
- Kaito, Bane of Nightmares already existed (skipped in S14)

**Commit prefix used**: `W6-cards:`

## Handoff History

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

### 2026-03-22 — W6: Phase 1 close (F-4 sweep + F-5/F-6/F-7)
- Phase 1 Fix COMPLETE. Next: Phase 2 authoring A-01.
