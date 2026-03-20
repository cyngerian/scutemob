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
| W3: LOW Remediation | — | available | — | **W3 T3 DONE** — ManaPool::spend() encapsulated. W3-LC S1-S6 done. All W3 items complete. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available | — | **ALL PBs DONE (PB-0 through PB-21)**. Phase 2 card authoring next |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-20
**Workstream**: W3: LOW Remediation
**Task**: T3: ManaPool::spend() encapsulation (MR-M1-15 + MR-M1-07)

**Completed**:
- Added `ManaPool::can_spend()`, `ManaPool::spend()`, `ManaPool::get()` methods to `state/player.rs`
- Reduced `casting.rs` free functions (`can_pay_cost`, `pay_cost`, etc.) to thin 1-line wrappers
- Migrated 2 call sites in `engine.rs` and `abilities.rs` to use methods directly
- 17 new unit tests in `tests/mana_pool.rs` (colored, generic, colorless-specific, restricted mana CR 106.12, spend order)
- No raw field manipulation of mana pool remains in engine source
- 2223 tests, 0 clippy warnings
- Commit: 0b5af81
- Also committed housekeeping: stale session plan cleanup (54ddab1)

**Next**:
1. W3 is effectively complete — T1, T2, T3, W3-LC all done
2. Update `docs/workstream-coordination.md` Phase 3 checkboxes (T3 + commit)
3. Top priority: W6 Phase 2 card authoring (~1,025 remaining cards)

**Hazards**:
- ~30 remaining call sites still use `casting::can_pay_cost`/`casting::pay_cost` wrappers — functional but could be migrated to direct method calls opportunistically
- `docs/workstream-coordination.md` Phase 3 checkbox for T3 not yet checked off

**Commit prefix used**: `W3:`

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


