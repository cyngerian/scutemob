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
| W6: Primitive + Card Authoring | W6-review: PB-8 retroactive review | ACTIVE | 2026-03-16 | Retroactive review PB-8 (cost reduction statics, 10 cards). 8/20 done. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-16
**Workstream**: W6: Primitive + Card Authoring
**Task**: W6-review — retroactive PB-7 review

**Completed**:
- PB-7 retroactive review (8/20): 29 count-based scaling cards — **fixed**
  - 1M fixed: DevotionTo now counts hybrid/phyrexian mana symbols (CR 700.5)
  - 3H fixed: Nykthos wrong Shrine subtype, Faeburrow Elder & Multani P/T None→0/0
  - 1M fixed: Frodo oracle text replaced with actual card text
  - 1L fixed: Toothy migrated to ..Default::default()
  - 2L deferred: raw characteristics reads (systemic)
- Review file: `memory/primitives/pb-review-7.md`
- 2148 tests passing
- Commit: 4ce344f

**Next**:
1. `/implement-primitive --review-only PB-8` (Cost reduction statics, 10 cards)
2. Sequential through PB-9, PB-10, ... PB-18
3. After all 20 reviews complete: resume PB-19 (board wipes)

**Hazards**:
- Pre-existing TUI compile warning in `parser.rs` (missing `progress` field on `DashboardData`)
- Unstaged changes: `tools/tui/src/dashboard/` files, `.claude/skills/implement-primitive/SKILL.md`, `CLAUDE.md` (all pre-existing from prior sessions)

**Commit prefix used**: `W6-prim:`

## Handoff History

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
