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
| W6: Primitive + Card Authoring | W6-review: retroactive PB review (PB-1 next) | ACTIVE | 2026-03-16 | **PRIMARY OBJECTIVE**: review all 20 PB batches (PB-0 to PB-18) before forward progress. Use `/implement-primitive --review-only PB-<N>`. Tracker: `docs/project-status.md` Review Backlog. PB-19+ blocked until reviews complete. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-16
**Workstream**: W6: Primitive + Card Authoring
**Task**: W6-review — retroactive PB-0 review

**Completed**:
- PB-0 retroactive review complete (1/20): 18 card defs reviewed against oracle text
- 1 MEDIUM fixed: Thousand-Faced Shadow — added Ninjutsu keyword + cost ability (was TODO despite DSL support since B3)
- 1 LOW fixed: added test_508_1d_must_attack_each_combat_enforced (MustAttackEachCombat keyword)
- 2145 tests passing (+1 from new test); engine builds clean
- Review file: `memory/primitives/pb-review-0.md`

**Next**:
1. `/implement-primitive --review-only PB-1` (Mana with damage, 8 cards)
2. Sequential through PB-2, PB-3, ... PB-18
3. After all 20 reviews complete: resume PB-19 (board wipes)

**Hazards**:
- Pre-existing TUI compile error: `parser.rs` missing `progress` field on `DashboardData` (from project management session)
- Unstaged changes: `thousand_faced_shadow.rs`, `keywords.rs` (PB-0 review fixes)

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-16 — W6: Project management + PB-0 review started
- Project management: 3 new agents, /implement-primitive skill, docs/project-status.md, W6-review primary objective (5d9b87a)
- PB-0 review started (in-review)

### 2026-03-16 — W6: PB-18 Stax/restrictions
- GameRestriction enum, 6 restriction types, 10 card defs, 10 tests; 2144 total; commit 9c037c6

### 2026-03-16 — W6: PB-17 Library search filters
- max_cmc, min_cmc, has_card_types on TargetFilter; 9 card fixes; 8 tests; 2134 total; commit 894504e

### 2026-03-15 — W6: PB-16 Meld mechanics
- Full Meld framework per CR 701.42 / CR 712.4 / CR 712.8g; MeldPair, Effect::Meld, zone-change splitting; 3 card defs; 7 tests; 2126 total; commit 9d384a3

### 2026-03-15 — W6: PB-15 Saga & Class mechanics
- Saga + Class frameworks, 11 new tests, 2119 total; commit f5878a8

