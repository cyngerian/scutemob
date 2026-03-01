# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | Batch 1: Evasion & Simple Keywords | ACTIVE | 2026-02-28 | Horsemanship, Skulk, Devoid, Decayed, Ingest (Shadow done in B0) |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | Phase 0 complete; T2 done; Phase 1 (abilities) next |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-02-28 (session end)
**Workstream**: W1: Abilities — Batch 0
**Task**: Implement Batch 0: P3 stragglers + Shadow (P4)
**Completed**:
- Bolster (CR 701.39): Effect::Bolster, 8 tests, Cached Defenses card, script 104 — commits a84e6fe
- Adapt (CR 701.46): KeywordAbility::Adapt(u32), Condition::SourceHasNoCountersOfType, 6 tests, Sharktocrab, script 105 — commit fb01b08
- Shadow (CR 702.28): bidirectional block restriction, 7 tests, Dauthi Slayer, script 106 — commit aacef15
- Partner With (CR 702.124j): PartnerWithTrigger, ETB search, has_name TargetFilter, commander validation, 10 tests, Pir+Toothy, script 107 — commit da29f16
- Overload (CR 702.96): alt cost, WasOverloaded condition, ForEach dispatch, 11 tests, Vandalblast, script 108 — commit 2729c3d
- Bonus: CounterType/LibraryPosition/TargetController exported from helpers.rs; TargetController::Opponent enforced in collect_for_each; TUI PartnerWithTrigger arm in stack_view.rs
- Batch 0 checkbox in workstream-coordination.md — ready to check off
**Next**: Batch 1: Evasion & Simple Keywords (Shadow done; remaining: Horsemanship, Skulk, Devoid, Decayed, Ingest). Claim W1-B1.
**Hazards**: Some W5 card defs (Demonic Tutor, Worldly Tutor, Vampiric Tutor, Mana Confluence, Phyrexian Altar, Impact Tremors, Skullclamp) exist as untracked files — committed by W5 session, may need W5 claim to manage
**Commit prefix used**: `W1-B0:`

## Handoff History

### 2026-02-28 (late night) — W5: Card Authoring (second batch)
- 7 cards (Demonic Tutor, Worldly Tutor, Vampiric Tutor, Mana Confluence, Phyrexian Altar, Impact Tremors, Skullclamp); 3 new DSL gaps identified; commit `c3e80e0`

### 2026-02-28 (night) — W5: Card Authoring (first batch)
- Batches A/B/C: authored 30, removed 22 with simplifications; 8 accurate cards remain
- Fixed generate_worklist.py: DSL_GAP_PATTERNS, blocked classification; commit prefix `W5-cards:`

### 2026-02-28 (night) — W2: TUI & Simulator (UX fixes)
- Fix 1-6: Hand scrolling, discard events, zone counters, zone browser overlay, CardDetail return-to, action hints; Esc bug fix

### 2026-02-28 (late) — W2: Card Pipeline Phases 5-9
- Split definitions.rs → 112 files in defs/; build.rs auto-discovery; skeleton generator; agent rewrite

### 2026-02-28 — W3: T2 dead code + T1 tests (Phase 0 complete)
- MR-M1-14/19/20, MR-M2-07/08/17, MR-M4-13, MR-M5-08, MR-M6-08, MR-M8-15, MR-M9-14/15, MR-M9.4-11/13/14/15 closed
