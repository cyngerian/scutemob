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
| W3: LOW Remediation | T3 + W3-LC layer correctness audit | paused | — | T2 done; T3 ManaPool pending; **W3-LC S1 DONE** (46 bugs classified, 0 ambiguous). S2 next: fix 7 HIGH sites. See `memory/w3-layer-audit.md` |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | PB-20: Additional Combat Phases | ACTIVE | 2026-03-19 | Feature branch: w6-pb20-additional-combat-phases |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-19
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-19: Mass Destroy / Board Wipes (full pipeline: plan → implement → review → fix → close)

**Completed**:
- PB-19 complete: DestroyAll + ExileAll + OwnerOf + LastEffectCount + AddCounterAmount
  - Engine: Effect::DestroyAll (indestructible/regen/umbra), Effect::ExileAll, EffectAmount::LastEffectCount, Effect::AddCounterAmount, PlayerTarget::OwnerOf, AllPermanentsMatching controller filter fix
  - 7 card defs fixed: wrath_of_god, damnation, supreme_verdict, path_of_peril, sublime_exhalation, final_showdown, scavenger_grounds
  - 5 new card defs: vanquish_the_horde, fumigate, bane_of_progress, ruinous_ultimatum, cyclonic_rift
  - Review: 2M 2L — all fixed (commander destroy count CR 903.9a, OwnerOf for bounce, ExileAll owner events)
  - 13 new tests, 2175 total
- Commits: 224131d (implement), f934425 (review fixes)

**Next**:
1. PB-20: Additional Combat Phases (10 cards, 2 sessions)
2. PB-21: Fight & Bite (5 cards, 1 session)
3. Phase 2: Author ~1,020 remaining cards

**Hazards**:
- None known

**Commit prefix used**: `W6-prim:`

## Handoff History

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

### 2026-03-18 — W6: PB-15 review
- PB-15 retroactive review (17/20): Saga & Class, 2 cards — 1H 1M fixed; 2M 1L deferred (DSL gaps)
- Commit: 013fddb

### 2026-03-18 — W6: PB-14 review
- PB-14 retroactive review (16/20): Planeswalker support + emblems, 31 cards — 1H fixed; 1M 2L deferred
- Commit: b776522

### 2026-03-18 — W6: PB-13 review
- PB-13 retroactive review (15/20): Specialized mechanics, 19 cards — 2H 5M fixed; 9M 1L deferred
- Commit: 9001176

### 2026-03-18 — W6: PB-12 review
- PB-12 retroactive review (14/20): Complex replacement effects, 11 cards — 2H 4M fixed; 2M deferred
- Commit: 6ba09f1
