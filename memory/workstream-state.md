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
| W6: Primitive + Card Authoring | Phase 2.5: DSL gap closure (PB-24 close pending) | paused | — | **PB-24 implemented + reviewed + fixed**. Close phase remaining. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-23
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-24 conditional statics ("as long as X")

**Completed**:
- PB-24 full pipeline through fix phase: plan → implement → review → fix
- Engine: `condition: Option<Condition>` on ContinuousEffectDef/ContinuousEffect, 5 new Condition variants, RemoveCardTypes LayerModification, Quest/Slumber counter types, check_static_condition + calculate_devotion_to_colors
- 13 card defs fixed (Serra Ascendant, Dragonlord Ojutai, Bloodghast, 3 Theros gods, Nadaar, Beastmaster Ascension, Quest for Goblin Lord, Arixmethes, Razorkin Needlehead, Mox Opal, Indomitable Archangel)
- ~170 card defs updated with `condition: None` on existing ContinuousEffectDef structs
- 11 new tests (conditional_statics.rs), 2302 total passing
- Review: 1H 2M 5L found, 1H 2M fixed (Nadaar mana cost, CR 700.5a doc, re-entrancy doc)
- Commits: a69d458, aa23d26

**Next**:
1. **Close PB-24**: run `/implement-primitive` to execute close phase (update project-status.md, CLAUDE.md)
2. Continue gap closure: PB-25 (continuous effect grants, ~98 cards) or PB-26 (trigger variants, ~72 cards)
3. ~201 card defs still need backfill sweep — cards with "as long as" TODOs that are now expressible

**Hazards**:
- PB-24 close phase not yet done — `memory/primitive-wip.md` has `phase: close`
- Backfill not yet run — many of the ~201 cards still have TODOs even though the engine now supports conditions
- Some cards blocked on PB-25 (EffectFilter extensions for "creatures you control have X")

**Commit prefix used**: `W6-prim:`

## Handoff History

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

### 2026-03-22 — W6: Phase 2 authoring A-18 draw + A-19 start
- A-18 S20-S25 (43 new), A-19 S40-S43 (48 new). 91 total. 4H fixed.
- Commits: fc27279, 047532f, f43d88b, 0de9b8c. 2281 tests.
