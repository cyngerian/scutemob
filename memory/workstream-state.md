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
| W6: Primitive + Card Authoring | — | available | — | **ALL PBs COMPLETE (PB-0–37)**; BF complete; **Wave A+B partial COMPLETE** (91+70 = 161 new card defs); Wave A engine review CLEAN |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-31
**Workstream**: W6: Primitive + Card Authoring
**Task**: Wave A engine review + Wave B (A-38 + A-42 partial)

**Completed**:
- Wave A engine review checkpoint: CLEAN (0 findings across 4 engine changes: cant_be_countered, MaxNoncreature/NonartifactSpellsPerTurn, ETBTriggerFilter::color_filter)
- A-38 triage: 105 cards → 53 authorable (31 SAFE + 22 PARTIAL), 48 BLOCKED, 2 existing
- A-38 authoring: 53 new card defs (modal spells, counterspells, tutors, Ninjutsu, Flashback, etc.)
- A-42 partial triage + authoring: 17 new card defs (high-priority tutors, rituals, removal)
- 70 total new card defs this session, 2437 tests, 0 clippy

**Next**:
1. **Continue A-42 authoring** — ~112 cards remain (many safe to author). Start with the sorted priority list; Panharmonicon, Living Death, Deepglow Skate, Heritage Druid, etc.
2. **Wave B engine review checkpoint** after A-42 authoring completes (no engine changes this session — pure card authoring, so this should be quick)
3. **Wave C**: A-30, A-36, A-40, A-41 — blocked on significant engine work

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward)
- A-38 BLOCKED cards (48): need play-from-top, pitch-alt-cost, copy-target-spell, grant-flash, mana-doubling, extra-turn, gain-control
- A-42 BLOCKED cards (~24): need type-override-all-lands (Urborg/Yavimaya/Blood Moon), chosen-type (Roaming Throne), mass-reanimate (Living Death), extra-turn, landfall-mana, tribal-anthem

**Commit prefix**: `W6-cards:`

## Handoff History

### 2026-03-31 — W6: Wave A complete
- Wave A: 91 new card defs (A-29, A-32–A-35, A-39). 2437 tests.

### 2026-03-30 — W6: A-29 S131
- 3 engine primitives + 10 card defs fixed.

### 2026-03-30 — W6: BF-S3/S4 + A-29 S1
- BF-S3/S4: 7 card def fixes. A-29 S1: 21 card defs authored.

### 2026-03-30 — W6: BF-1 + BF-2
- BF-1 re-triage: 1451 defs, 773 clean (53%), 678 with TODOs. BF-2 gap closure.

### 2026-03-29 — W6: PB-37
- PB-37: G-26 residual complex activated. 7 card defs fixed. 9 new tests.
