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
| W6: Primitive + Card Authoring | PB-25: Continuous effect grants (~98 cards) | ACTIVE | 2026-03-23 | PB-24 done. Starting PB-25 (G-3 gap). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-23
**Workstream**: Executive/oversight (no workstream claimed) + W6 worker in parallel
**Task**: TUI redesign (oversight session) + PB-24 close (worker session)

**Completed**:
- **TUI redesign**: 8 tabs → 4 tabs (Dashboard, Pipeline, Cards, Milestones). Deleted 6 stale tabs (Abilities, Corner Cases, Reviews, Scripts, old Cards, Progress). New Pipeline tab with reverse-sorted PB batches (next at top, done at bottom) + worker status from `primitive-wip.md`. New Cards tab with live filesystem scan replacing stale `_authoring_worklist.json`. Dashboard has compact summary lines for abilities/corner cases/reviews/scripts/engine LOC. Milestones tab puts future (M10/M11/M12) at top, completed dimmed at bottom.
- **Worker completed PB-24**: closed, project-status + ops plan updated. 2302 tests, 0 clippy.
- TUI changes bundled into worker's PB-24 commit (a69d458). Clean build, zero warnings.

**Next**:
1. Continue gap closure: PB-25 (continuous effect grants, ~98 cards) or PB-26 (trigger variants, ~72 cards)
2. TUI: test with `cargo run --bin mtg-tui -- dashboard` in interactive terminal
3. ~201 card defs still need backfill sweep for PB-24 conditions

**Hazards**:
- TUI changes were committed by the worker session (bundled into PB-24 commit) — no separate W2 commit
- Backfill not yet run for PB-24 conditions
- Some cards blocked on PB-25 (EffectFilter extensions for "creatures you control have X")

**Commit prefix used**: `chore:` (oversight), TUI bundled into `W6-prim:`

## Handoff History

### 2026-03-23 — W6: PB-24 conditional statics + TUI redesign
- PB-24: 13 card defs fixed, 1H 2M fixed, ~170 defs updated with `condition: None`. 2302 tests.
- TUI redesign: 8→4 tabs, live card scanner, pipeline tab, worker status display. Zero warnings.
- Commits: a69d458, aa23d26, 12b594d, 4a89567.

### 2026-03-23 — Executive/oversight: DSL gap plan + TUI improvements
- Full DSL gap audit (1,348 TODOs, 31 gap types), created `docs/dsl-gap-closure-plan.md` (PB-23 through PB-37)
- TUI: live card health scanner, dynamic stats, pipeline funnel
- Reviewed PB-23 (34 cards fixed, 2H fixed). Commits: 2665479, 95da16f.

### 2026-03-23 — W6: PB-23 controller-filtered creature triggers
- PB-23: 34 cards fixed, 2H 11M fixed. New: controller filter on WheneverCreatureDies, WheneverCreatureYouControlAttacks, WheneverCreatureYouControlDealsCombatDamageToPlayer.
- Commits: b5066d4, 5104b27. 2291 tests.

### 2026-03-23 — W6: Phase 2 authoring A-20 through A-23
- A-20 pump-buff (27), A-21 counters-plus (49), A-22 equipment (11), A-23 death-trigger (34). 121 total. 4H fixed.
- Commits: e5b0436, ec08405. 2281 tests.

### 2026-03-23 — W6: Phase 2 authoring A-19 token-create S44-S52
- A-19 token-create S44-S52: 96 new cards. Reviewed, 3H fixed.
- Commits: 5d967ca, 83c1302. 2281 tests.
