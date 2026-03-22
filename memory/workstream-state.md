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
| W6: Primitive + Card Authoring | F-4/F-5 card fixes | ACTIVE | 2026-03-22 | F-4 S6 done; continuing F-4 or pivoting to F-5 |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: F-4 session 6 — implement 11 now-expressible card abilities

**Completed**:
- F-4 session 6: 11 card defs fixed across 4 patterns:
  - 3 lands: ETB tapped + mana (Valakut, Spinerock Knoll, Creeping Tar Pit)
  - 1 land: conditional mana with activation_condition (Temple of the False God — ControlAtLeastNOtherLands(4))
  - 1 land: conditional ETB tapped (Den of the Bugbear — Not(ControlAtLeastNOtherLands(2)))
  - 1 equipment: bounce-self activated (Cryptic Coat — MoveZone to Hand with OwnerOf)
  - 1 creature: ETB tapped + mana (Arixmethes — enters tapped + {T}: Add {G}{U})
  - 1 creature: ETB tapped (Oathsworn Vampire)
  - 3 creatures: keyword additions (Hydroelectric Specimen: Flash; Bloomvine Regent: Flying + Dragon-enters trigger; Marang River Regent: Flying)
- Review: 0 HIGH, 1 MEDIUM (Arixmethes missing enters-tapped — fixed), 1 LOW (stale comment — fixed)
- 2281 tests, 0 clippy warnings

**Next**:
1. F-4 session 7+: most remaining TODOs are genuine DSL gaps. Diminishing returns — consider pivoting to F-5 verification pass.
2. F-5 through F-7: verification and phase 1 close
3. Phase 2 authoring (A-01 through A-42)

**Hazards**:
- 3 silent wrong-state cards still present: beast_within, generous_gift, swan_song (CreateToken lacks controller_override field — genuine DSL gap)
- Pre-existing unstaged changes in CLAUDE.md and docs/card-authoring-operations.md from prior sessions
- Shizo has over-permissive TargetCreature (should be legendary only — TargetFilter lacks has_supertype)
- Arixmethes still missing slumber counter mechanics (enters with counters, type change, counter removal trigger) — enters tapped + mana are implemented but card is still partially wrong-state

**Commit prefix used**: `W6-fix:`

## Handoff History

### 2026-03-22 — W6: F-4 session 6 (11 now-expressible cards)
- 3 lands (ETB tapped + mana), 1 conditional mana, 1 conditional ETB, 1 equipment bounce, 1 creature ETB+mana, 1 creature ETB, 3 keyword additions. Review: 1M+1L fixed.
- 2281 tests.

### 2026-03-22 — W6: F-4 session 5 (12 land mana abilities)
- 8 pathway lands (mana tap) + 4 verge lands (conditional mana). Review: 2H fixed (stale oracle text).
- Commit: 8c2aded. 2281 tests.

### 2026-03-22 — W6: F-4 session 4 (18 card abilities)
- 10 new implementations + 6 prior unstaged + 2 stale cleanups. Review: 1H+1M fixed.
- Commit: e4cd042. 2281 tests.

### 2026-03-22 — W6: F-4 sessions 2-3 (17 now-expressible cards)
- S2: 11 bounce lands + Shrieking Drake (ETB MoveZone to Hand). S3: 6 targeted/self-grant abilities.
- Commits: c4c0923, 1138ef4. 2281 tests.

### 2026-03-22 — W6: F-4 session 1 review (4 HIGH fixed)
- Reviewed 24 cards in 4 batches. 4 HIGH fixed. 7 MEDIUM documented as valid DSL gaps.
- 2281 tests, 0 clippy.
