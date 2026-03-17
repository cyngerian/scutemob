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
| W6: Primitive + Card Authoring | W6-review: PB-7 retroactive review | ACTIVE | 2026-03-16 | **PRIMARY OBJECTIVE**: review all 20 PB batches. 7/20 done. Use `/implement-primitive --review-only PB-<N>`. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-16
**Workstream**: W6: Primitive + Card Authoring
**Task**: W6-review — retroactive PB-6 review

**Completed**:
- PB-6 retroactive review (7/20): 30 static grant with filter cards — **fixed**
  - 1H fixed: Goblin Warchief self-haste (added intrinsic Haste keyword)
  - 5M fixed: Archetype of Endurance (Hexproof), Archetype of Imagination (Flying), Iroas God of Victory (Menace) static grants; Vito (activated Lifelink), Vault of the Archangel (activated Deathtouch+Lifelink)
  - 6L fixed: stale TODO comments updated
- Review file: `memory/primitives/pb-review-6.md`
- 2147 tests passing
- Commit: 6b13b50

**Next**:
1. `/implement-primitive --review-only PB-7` (Count-based scaling, 29 cards)
2. Sequential through PB-8, PB-9, ... PB-18
3. After all 20 reviews complete: resume PB-19 (board wipes)

**Hazards**:
- Pre-existing TUI compile warning in `parser.rs` (missing `progress` field on `DashboardData`)
- Unstaged changes: `tools/tui/src/dashboard/` files, `.claude/skills/implement-primitive/SKILL.md`, `CLAUDE.md` (all pre-existing from prior sessions)

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-16 — W6: PB-6 review
- PB-6 retroactive review (7/20): 30 static grant cards — 1H 5M 6L fixed
- Commit: 6b13b50



### 2026-03-16 — W6: PB-2 review
- PB-2 retroactive review complete (3/20): 56 conditional ETB tapped land card defs reviewed
- 1H fixed: 10 missed card defs; 1M fixed: Isolated Chapel; 1M+2L deferred
- Commit: 8ecfe08

### 2026-03-16 — W6: PB-0 commit + PB-1 review
- PB-0 fixes committed (14b4910): Ninjutsu fix, MustAttackEachCombat test
- PB-1 review complete (2/20): 1M fixed — pain lands split to single-color abilities; 2147 tests

### 2026-03-16 — W6: Project management + TUI + PB-0 review
- 3 new agents, /implement-primitive skill, docs/project-status.md, TUI Progress tab
- PB-0 review complete (1/20): 1M + 1L fixed; 2145 tests

### 2026-03-16 — W6: PB-18 Stax/restrictions
- GameRestriction enum, 6 restriction types, 10 card defs, 10 tests; 2144 total; commit 9c037c6

### 2026-03-16 — W6: PB-17 Library search filters
- max_cmc, min_cmc, has_card_types on TargetFilter; 9 card fixes; 8 tests; 2134 total; commit 894504e


