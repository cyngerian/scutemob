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
| W6: Primitive + Card Authoring | Phase 2.5: DSL gap closure (PB-24 next) | ACTIVE | 2026-03-23 | **PB-23 DONE**. Starting PB-24 (conditional statics, ~201 cards). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-23
**Workstream**: Executive/oversight (no workstream claimed)
**Task**: DSL gap analysis, gap closure plan, TUI improvements

**Completed**:
- Full DSL gap audit: 1,348 TODOs across 814/1,452 card defs (56%), categorized into 31 gap types
- Created `docs/dsl-gap-closure-plan.md` — 15 new primitive batches (PB-23 through PB-37)
- Updated `docs/card-authoring-operations.md` — Phase 2.5 inserted before Tier 3 authoring
- Updated `docs/primitive-card-plan.md` — Phase 1.5 section with PB-23+ batch table
- Updated `docs/project-status.md` — PB-23+ rows, live card health numbers
- Updated agents: `primitive-impl-planner` (PB-23+ file reference), `primitive-impl-runner` (backfill protocol)
- TUI improvements: live card health scanner (replaces hardcoded values), dynamic ability/corner stats, new card health breakdown (OK/partial/stripped/vanilla with TODO%), pipeline funnel shows done/gap/total
- Reviewed PB-23 output after worker session completed it (34 cards fixed, 2H found and fixed)

**Next**:
1. Continue gap closure: PB-24 (conditional statics, ~201 cards) or PB-26 (trigger variants, ~72 cards)
2. TUI: Cards tab still reads stale `_authoring_worklist.json`, needs rework; test count re-run on refresh; workstream panel should read `workstream-state.md`
3. Oversight: spot-check PB-24+ output quality as worker progresses

**Hazards**:
- TUI changes compile but were verified alongside PB-23 engine changes — if engine regresses, TUI may need rebuild
- 800 cards still have TODOs (was 814 before PB-23)
- Worker session should continue `/implement-primitive PB-24` or PB-26 next

**Commit prefix used**: `chore:` (oversight), `W2:` (TUI)

## Handoff History

### 2026-03-23 — W6: PB-23 controller-filtered creature triggers
- PB-23: 34 cards fixed, 2H 11M fixed. New: controller filter on WheneverCreatureDies, WheneverCreatureYouControlAttacks, WheneverCreatureYouControlDealsCombatDamageToPlayer.
- Commits: b5066d4, 5104b27. 2291 tests.

### 2026-03-23 — W6: Phase 2 authoring A-20 through A-23
- A-20 pump-buff (27), A-21 counters-plus (49), A-22 equipment (11), A-23 death-trigger (34). 121 total. 4H fixed.
- Commits: e5b0436, ec08405. 2281 tests.

### 2026-03-23 — W6: Phase 2 authoring A-19 token-create S44-S52
- A-19 token-create S44-S52: 96 new cards. Reviewed, 3H fixed.
- Commits: 5d967ca, 83c1302. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-18 draw + A-19 start
- A-18 S20-S25 (43 new), A-19 S40-S43 (48 new). 91 total. 4H fixed.
- Commits: fc27279, 047532f, f43d88b, 0de9b8c. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-18 draw S10-S19
- 119 new cards (10 sessions of 16 complete)
- Commits: 4a15b5e, 9bf3870, 6ecdb68. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-14 through A-17 (Tier 2 cont'd)
- 45 new cards (damage-each, bounce, minus, counter). All HIGH fixed.
- Commit: 6ddc832. 2281 tests.
