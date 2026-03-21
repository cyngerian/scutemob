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
| W6: Primitive + Card Authoring | PB-22 S1: Trivial wiring (activation condition, mana cost filter, sorcery-speed) | ACTIVE | 2026-03-20 | Deferred cleanup batch — 13 items, 7 sessions |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-20
**Workstream**: W3: LOW Remediation
**Task**: Full LOW cleanup sprint S1-S6 + bookkeeping + reviews

**Completed**:
- S1: 20 stale doc comments, dead fields (echo_cost/cumulative_upkeep_cost), EvolveTrigger→ETBEvolve rename, AdditionalCost+Designations exports
- S2: 6 test gaps (protection multicolor/subtype, first-strike, Thought Vessel, combat script)
- S3: 8 perf/schema/viewer (LazyLock SubType, SBA name move, ON DELETE CASCADE, FTS triggers, bind 127.0.0.1)
- S4: 4 behavioral (StackObject::trigger_default -788 lines, single-pass AdditionalCost, check_condition delegation)
- S5: 8 quick+medium (saturating_add, AltCostKind hash discriminants, mutate/alliance edge case tests)
- S6: TC-21 PendingTrigger 19 Option fields → TriggerData variants (-1738 lines), hash fix (W3S6-01 HIGH)
- Bookkeeping: closed 29 items in reviews doc that were done in prior sessions but never marked
- Final stats: 83→29 LOW open, 119 LOW closed, 2233 tests, 0 clippy
- All reviews clean except S6 HIGH (hash gap, fixed immediately)

**Next**:
1. W3 is effectively complete — remaining 29 LOWs are permanently deferred or DSL-blocked
2. Top priority: W6 Phase 2 card authoring (~1,025 remaining cards)
3. `docs/workstream-coordination.md` Phase 3 checkboxes need updating (T3 done, LC done)
4. `docs/mtg-engine-low-issues-remediation.md` summary stats are stale (still says 39 open)

**Hazards**:
- 9 PendingTrigger Option fields remain (flanking, rampage, renown, poisonous, enlist, recover, ingest) — named PTK variants, not KeywordTrigger
- MR-M6-06 (combat refactor) deferred — only do when combat needs new features

**Commit prefix used**: `W3:`, `chore:`

## Handoff History

### 2026-03-19 — W6: PB-21 Fight & Bite
- PB-21 complete: Effect::Fight + Effect::Bite, 4 card defs fixed, 14 new tests, 2206 total
- ALL 22 PRIMITIVE BATCHES COMPLETE (PB-0 through PB-21)
- Next: Phase 2 card authoring (~1,025 remaining cards)

### 2026-03-19 — Exploratory: Rules audit + W3-LC plan
- Audited 4 abilities (Devoid, Flanking, Fabricate, Suspend) against CR rules
- Found 1 MEDIUM bug: Flanking reads base characteristics instead of layer-resolved (Humility breaks it)
- Identified 69 base-characteristic reads across 12 engine files that may have same issue
- Created `memory/w3-layer-audit.md` — 4-session audit plan (W3-LC S1-S4)
- Updated `docs/workstream-coordination.md` Phase 3 with W3-LC checkboxes
- **W3 and W6 are independent** — can run in parallel with no conflicts
- No code changes, no commit needed

### 2026-03-19 — W6: PB-18 review (FINAL)
- PB-18 retroactive review (20/20): Stax / restrictions, 10 cards — 2H 4M 1L fixed; 2M deferred (DSL gap)
- Engine: attack tax (combat.rs), mana ability restrictions (mana.rs), zone scope (abilities.rs), simulator filter
- Commit: ca13879
- **ALL RETROACTIVE REVIEWS COMPLETE**

### 2026-03-19 — W6: PB-17 review
- PB-17 retroactive review (19/20): Library search filters, 27 card defs — 4H 4M fixed; 1M deferred (DSL gap)
- Engine: shuffle_before_placing on SearchLibrary, CR 701.19→701.23 citations
- Commit: b72b0bd

### 2026-03-19 — W6: PB-16 review
- PB-16 retroactive review (18/20): Meld, 1 card — 1H 1M 1L fixed; 2M deferred (DSL gap)
- Commit: 6fce74c


