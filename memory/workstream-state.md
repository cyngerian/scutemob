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
| W6: Primitive + Card Authoring | W6-review: retroactive PB review (PB-6 next) | ACTIVE | 2026-03-16 | **PRIMARY OBJECTIVE**: review all 20 PB batches. 6/20 done. Use `/implement-primitive --review-only PB-<N>`. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-16
**Workstream**: W6: Primitive + Card Authoring
**Task**: W6-review — retroactive PB-3 + PB-4 + PB-5 reviews

**Completed**:
- PB-3 retroactive review (4/20): 10 shocklands — **clean**
- PB-4 retroactive review (5/20): 26 sacrifice-cost cards — **fixed** (1M fixed, 1M+13M+6L deferred)
- PB-5 retroactive review (6/20): 32 targeted ability cards — **fixed**
  - 1H fixed: triggered ability targets now auto-populated from CardDef in flush_pending_triggers
  - 2M fixed: fizzle check for CardDef triggered abilities (CR 608.2b) + target validation at trigger time (CR 603.3d)
  - 1M fixed: Blinkmoth Nexus target narrowed to Blinkmoth subtype
  - 1L fixed: Ghost Quarter simplified to TargetLand
  - 5M + 5L deferred: DSL gaps (colorless filter, non-land filter, attacking/blocking filter, etc.)
- Review files: `memory/primitives/pb-review-3.md`, `memory/primitives/pb-review-4.md`, `memory/primitives/pb-review-5.md`
- 2147 tests passing

**Next**:
1. `/implement-primitive --review-only PB-6` (Static grant with filter, 30 cards)
2. Sequential through PB-7, PB-8, ... PB-18
3. After all 20 reviews complete: resume PB-19 (board wipes)

**Hazards**:
- Pre-existing TUI compile warning in `parser.rs` (missing `progress` field on `DashboardData`)
- Unstaged changes: `tools/tui/src/dashboard/` files (pre-existing from prior session)

**Commit prefix used**: `W6-prim:`

## Handoff History

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

### 2026-03-15 — W6: PB-16 Meld mechanics
- Full Meld framework per CR 701.42 / CR 712.4 / CR 712.8g; MeldPair, Effect::Meld, zone-change splitting; 3 card defs; 7 tests; 2126 total; commit 9d384a3


