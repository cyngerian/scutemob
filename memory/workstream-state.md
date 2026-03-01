# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | Batch 1 complete; Batch 2 next (Flanking, Bushido, Rampage, Provoke, Afflict, Renown, Training) |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | Phase 0 complete; T2 done; Phase 1 (abilities) next |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-01 (session end)
**Workstream**: W1: Abilities — Batch 1
**Task**: Implement Batch 1: Evasion & Simple Keywords (5 abilities)
**Completed**:
- Horsemanship (CR 702.31): unidirectional block restriction, discriminant 71, 7 tests, Shu Cavalry card, script 109 — commit 9cc5672
- Skulk (CR 702.118): power-based block restriction (blocker power > attacker power), discriminant 72, 7 tests, Furtive Homunculus, script 110
- Devoid (CR 702.114): CDA in layers.rs ColorChange (Layer 5), discriminant 73, 8 tests, Forerunner of Slaughter, script 111
- Decayed (CR 702.147): can't-block + EOC sacrifice flag, discriminant 74, 8 tests, Shambling Ghast, script 112
- Ingest (CR 702.115): combat damage trigger → exile top library card, StackObjectKind::IngestTrigger, discriminant 75, 6 tests, Mist Intruder, script 113
- CR corrections: Horsemanship=702.31 (not 702.30=Echo), Skulk=702.118 (not 702.120=Escalate), Decayed=702.147 (not 702.145)
- Batch 0 + Batch 1 checkboxes checked in workstream-coordination.md and ability-batch-plan.md
- 1177 tests passing; 98 abilities validated total; P4 6/88
**Next**: Batch 2: Combat Triggers — Blocking (Flanking, Bushido, Rampage, Provoke, Afflict, Renown, Training). Claim W1-B2.
**Hazards**: None — all changes committed in 9cc5672; working tree clean
**Commit prefix used**: `W1-B1:`

## Handoff History

### 2026-02-28 (session end) — W1: Abilities — Batch 0
- Bolster, Adapt, Shadow, Partner With, Overload; 1166 tests; scripts 104-108; P3 36/40, P4 1/88; commit 2729c3d

### 2026-02-28 (late night) — W5: Card Authoring (second batch)
- 7 cards (Demonic Tutor, Worldly Tutor, Vampiric Tutor, Mana Confluence, Phyrexian Altar, Impact Tremors, Skullclamp); 3 new DSL gaps identified; commit `c3e80e0`

### 2026-02-28 (night) — W5: Card Authoring (first batch)
- Batches A/B/C: authored 30, removed 22 with simplifications; 8 accurate cards remain
- Fixed generate_worklist.py: DSL_GAP_PATTERNS, blocked classification; commit prefix `W5-cards:`

### 2026-02-28 (night) — W2: TUI & Simulator (UX fixes)
- Fix 1-6: Hand scrolling, discard events, zone counters, zone browser overlay, CardDetail return-to, action hints; Esc bug fix

### 2026-02-28 (late) — W2: Card Pipeline Phases 5-9
- Split definitions.rs → 112 files in defs/; build.rs auto-discovery; skeleton generator; agent rewrite
