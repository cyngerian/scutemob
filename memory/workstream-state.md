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
| W6: Primitive + Card Authoring | W6-review: PB-17 retroactive review | ACTIVE | 2026-03-19 | **PRIMARY OBJECTIVE**: review all 20 PB batches. 18/20 done. Use `/implement-primitive --review-only PB-<N>`. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-19
**Workstream**: W6: Primitive + Card Authoring
**Task**: W6-review — retroactive PB-16 review

**Completed**:
- PB-16 retroactive review (18/20): Meld, 1 card — **fixed**
  - 1H fixed: Phantom exiled cards after meld (CR 701.42a). Captured return values from move_object_to_zone, removed phantom exile objects after melded permanent created.
  - 1M fixed: Melded permanent mana value was 0 instead of sum of front faces (CR 712.8g). Synthetic ManaCost with combined MV.
  - 1L fixed: Oracle text mismatch on Hanweir back face.
  - 2M deferred: Attack trigger TODOs on Garrison + Township (DSL gap — tapped-and-attacking tokens)
- Review file: `memory/primitives/pb-review-16.md`
- 2155 tests passing

**Next**:
1. `/implement-primitive --review-only PB-17` (Library search filters, 74 cards)
2. Then PB-18 (Stax / restrictions)
3. After all 20 reviews complete: resume PB-19 (board wipes)

**Hazards**:
- None known

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-19 — W6: PB-16 review
- PB-16 retroactive review (18/20): Meld, 1 card — 1H 1M 1L fixed; 2M deferred (DSL gap)
- Commit: 6fce74c

### 2026-03-18 — W6: PB-15 review
- PB-15 retroactive review (17/20): Saga & Class, 2 cards — 1H 1M fixed; 2M 1L deferred (DSL gaps)
- Commit: 013fddb

### 2026-03-18 — W6: PB-14 review
- PB-14 retroactive review (16/20): Planeswalker support + emblems, 31 cards — 1H fixed; 1M 2L deferred
- Commit: b776522

### 2026-03-18 — W6: PB-13 review
- PB-13 retroactive review (15/20): Specialized mechanics, 19 cards — 2H 5M fixed; 9M 1L deferred
- Commit: 9001176

### 2026-03-18 — W6: PB-12 review
- PB-12 retroactive review (14/20): Complex replacement effects, 11 cards — 2H 4M fixed; 2M deferred; 2M documented
- Commit: 6ba09f1

### 2026-03-18 — W6: PB-11 review
- PB-11 retroactive review (13/20): Mana restrictions + ETB choice, 13 cards — 1H 6M fixed; 1M 7L deferred
- Commit: 8cd6ec2

### 2026-03-18 — W6: PB-10 review
- PB-10 retroactive review (12/20): Return from zone effects, 8 cards — 2H 5M fixed; 3L deferred
- Commit: a5e45c6

### 2026-03-17 — W6: PB-9.5 review
- PB-9.5 retroactive review (11/20): Architecture cleanup, 0 cards — 1M 1L fixed; 1L deferred

### 2026-03-17 — W6: PB-9 review
- PB-9 retroactive review (10/20): 7 hybrid mana & X cost cards — 1H 5M fixed; 2M 7L deferred
- Commit: 4132421

### 2026-03-16 — W6: PB-8 review
- PB-8 retroactive review (9/20): 10 cost reduction statics — 3M fixed; 4L deferred
- Commit: 9a5ab65

### 2026-03-16 — W6: PB-7 review
- PB-7 retroactive review (8/20): 29 count-based scaling cards — 3H 2M 1L fixed; 2L deferred
- Commit: 4ce344f

### 2026-03-16 — W6: PB-6 review
- PB-6 retroactive review (7/20): 30 static grant cards — 1H 5M 6L fixed
- Commit: 6b13b50

### 2026-03-16 — W6: PB-3/4/5 reviews
- PB-3 clean, PB-4 1M fixed + deferred, PB-5 1H 2M 1L fixed + deferred
- Commit: 6d8620e

### 2026-03-16 — W6: PB-2 review
- PB-2 retroactive review (3/20): 1H 1M fixed; 10 missed ETB-tapped cards
- Commit: 8ecfe08

### 2026-03-16 — W6: PB-0/1 reviews + project management
- PB-0: 1M 1L fixed; PB-1: 1M fixed (pain lands); TUI Progress tab; 3 new agents
- Commits: 14b4910, f83367c
