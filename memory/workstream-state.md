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
| W6: Primitive + Card Authoring | W6-review: retroactive PB-2 review (conditional ETB tapped) | ACTIVE | 2026-03-16 | **PRIMARY OBJECTIVE**: review all 20 PB batches. 2/20 done, PB-2 in progress. Use `/implement-primitive --review-only PB-<N>`. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-16
**Workstream**: W6: Primitive + Card Authoring
**Task**: W6-review — retroactive PB-0 commit + PB-1 review

**Completed**:
- Committed PB-0 review fixes (14b4910): Thousand-Faced Shadow Ninjutsu + MustAttackEachCombat test
- PB-1 retroactive review complete (2/20): 8 pain land card defs reviewed against oracle text
- 1 MEDIUM fixed: all 7 pain lands split from dual-color AddMana (2 mana) into two separate single-color activated abilities (1 mana + 1 damage each)
- 2 new tests: battlefield_forge_second_colored_tap_deals_damage, all_pain_lands_deal_damage_on_second_colored_tap
- 2147 tests passing; engine + workspace builds clean
- Review file: `memory/primitives/pb-review-1.md`

**Next**:
1. `/implement-primitive --review-only PB-2` (Conditional ETB tapped, 56 cards — largest batch)
2. Sequential through PB-3, PB-4, ... PB-18
3. After all 20 reviews complete: resume PB-19 (board wipes)

**Hazards**:
- Pre-existing TUI compile warning in `parser.rs` (missing `progress` field on `DashboardData`)
- Unstaged changes: `tools/tui/src/dashboard/mod.rs` (pre-existing from prior session)

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-16 — W6: Project management + TUI + PB-0 review
- 3 new agents, /implement-primitive skill, docs/project-status.md, TUI Progress tab
- PB-0 review complete (1/20): 1M + 1L fixed; 2145 tests

### 2026-03-16 — W6: PB-18 Stax/restrictions
- GameRestriction enum, 6 restriction types, 10 card defs, 10 tests; 2144 total; commit 9c037c6

### 2026-03-16 — W6: PB-17 Library search filters
- max_cmc, min_cmc, has_card_types on TargetFilter; 9 card fixes; 8 tests; 2134 total; commit 894504e

### 2026-03-15 — W6: PB-16 Meld mechanics
- Full Meld framework per CR 701.42 / CR 712.4 / CR 712.8g; MeldPair, Effect::Meld, zone-change splitting; 3 card defs; 7 tests; 2126 total; commit 9d384a3

### 2026-03-15 — W6: PB-15 Saga & Class mechanics
- Saga + Class frameworks, 11 new tests, 2119 total; commit f5878a8

