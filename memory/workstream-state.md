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
| W6: Primitive + Card Authoring | W6-review: retroactive PB review | available | — | **PRIMARY OBJECTIVE**: review all 20 PB batches (PB-0 to PB-18) before forward progress. Use `/implement-primitive --review-only PB-<N>`. Tracker: `docs/project-status.md` Review Backlog. PB-19+ blocked until reviews complete. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-16
**Workstream**: W6: Primitive + Card Authoring (project management session)
**Task**: Project management restructuring — skills, agents, tracking, review pipeline

**Completed**:
- Deep project review: assessed all 6 workstreams, TUI dashboard gaps, dependency chains
- Created 3 new agents: `primitive-impl-planner` (Opus), `primitive-impl-runner` (Sonnet), `primitive-impl-reviewer` (Opus)
- Created `/implement-primitive` skill with full pipeline + `--review-only` mode for retroactive reviews
- Created `docs/project-status.md` — single source of truth (PB batches, card health, workstreams, review backlog)
- Updated `/start-work` — W6-PB<N> subunit support, reads project-status.md
- Updated `/implement-ability` and `/implement-primitive` — TaskCreate/TaskUpdate at each phase for TUI progress
- All fix phases now address HIGH + MEDIUM + LOW findings (no more deferred LOWs)
- Reviewer verdict changed: "clean" = zero findings at any severity
- Set W6-review as PRIMARY OBJECTIVE: sequential review of PB-0 through PB-18 (20 batches)
- PB-0 review started (in-review in project-status.md)
- Commit 5d9b87a

**Next**:
1. `/start-work W6-review` then `/implement-primitive --review-only PB-0` (first of 20 retroactive reviews)
2. Sequential through PB-1, PB-2, ... PB-18
3. After all 20 reviews complete: resume PB-19 (board wipes)

**Hazards**:
- New agents require session restart to appear in agent registry
- 2 unstaged files: `thousand_faced_shadow.rs`, `keywords.rs` (appear to be PB-18 leftovers)

**Commit prefix used**: `chore:`

## Handoff History

### 2026-03-16 — W6: PB-18 Stax/restrictions + project management
- PB-18 complete (9c037c6): GameRestriction enum, 6 restriction types, 10 card defs, 10 tests; 2144 total
- Project management: 3 new agents, /implement-primitive skill, docs/project-status.md, W6-review primary objective (5d9b87a)

### 2026-03-16 — W6: PB-17 Library search filters
- max_cmc, min_cmc, has_card_types on TargetFilter; 9 card fixes; 8 tests; 2134 total; commit 894504e

### 2026-03-15 — W6: PB-16 Meld mechanics
- Full Meld framework per CR 701.42 / CR 712.4 / CR 712.8g; MeldPair, Effect::Meld, zone-change splitting; 3 card defs; 7 tests; 2126 total; commit 9d384a3

### 2026-03-15 — W6: PB-15 Saga & Class mechanics
- Full Saga framework per CR 714: SagaChapter AbilityDefinition (disc 67), ETB lore counter, precombat main lore counter TBA, chapter triggers, sacrifice SBA (CR 714.4), SBA deferred while chapter on stack
- Full Class framework per CR 716: ClassLevel AbilityDefinition (disc 68), class_level on GameObject, Command::LevelUpClass with sorcery-speed + level-N-1 validation, level-up registers static continuous effects
- 11 new tests in saga_class.rs, 2119 total passing; commit f5878a8

### 2026-03-15 — W6: PB-14 Planeswalker support
- Full loyalty framework: LoyaltyCost, LoyaltyAbility (disc 66), ETB loyalty counters, ActivateLoyaltyAbility command, 0-loyalty SBA, 12 tests; commit d7faeff; 2108 tests

### 2026-03-15 — W6: PB-13 part 3 (Ascend condition + audit)
- Condition::HasCitysBlessing + Arch of Orazca/Twilight Prophet fixes + 1 test; Dredge/Buyback/LivingWeapon confirmed done; coin flip/flicker deferred; 2096 tests

