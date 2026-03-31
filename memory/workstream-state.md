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
| W6: Primitive + Card Authoring | Wave A: A-32+ | ACTIVE | 2026-03-30 | **ALL PBs COMPLETE (PB-0–37)**; BF complete; A-29 DONE (19/24, 5 deferred); starting A-32 land-fetch |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-30
**Workstream**: W6 (A-29 S131)
**Task**: A-29 session 131 — engine primitives + card def fixes for cant-restriction group

**Completed**:
- 3 engine primitives: `CardDefinition.cant_be_countered`, `ETBTriggerFilter.color_filter`, `GameRestriction::MaxNoncreatureSpellsPerTurn` + `MaxNonartifactSpellsPerTurn` (with per-player counters, casting enforcement, turn reset, hash)
- 10 card defs fixed (fed6064): Deafening Silence, Ethersworn Canonist, Shadow Alley Denizen fully complete; Hullbreaker Horror, Dragonlord Dromoka, Niv-Mizzet Parun, Allosaurus Shepherd, Vexing Shusher, Nezahal, Toski cant_be_countered set
- 131 card defs + 11 test files updated for new `cant_be_countered` field on CardDefinition
- A-29 S131 plan cards (Slither Blade through Tormented Soul, Phantom Ninja) were already complete from S1
- S132 cards: Ghostly Prison, Eidolon of Rhetoric, Elvish Champion already complete from S1

**Next**:
1. **A-29 S132**: Remaining TODO cards — Vexing Shusher (activated MakeSpellUncounterable), Tetsuko Umezawa (power-or-toughness filter). Already authored with TODOs, need engine work.
2. **A-29 S133**: Shadow Alley Denizen now complete. Only 1 card (already done).
3. **A-29 remaining TODOs** (genuine DSL gaps): Autumn's Veil (color-scoped counter protection), Soulless Jailer (graveyard/exile restrictions), Delney (power-conditional blocking + trigger doubling), Tetsuko (power-or-toughness filter), Vexing Shusher activated ability.
4. After A-29: proceed to A-30 (untap-phase, 12 cards) per Wave C ordering.

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward).
- Remaining A-29 TODOs are complex DSL gaps — may warrant deferral or new PB-style batches.
- CLAUDE.md has new "Card Authoring Wave Process" section with engine review checkpoints between waves.

**Commit prefix**: `W6-cards:`

## Handoff History

### 2026-03-30 — W6: BF-S3/S4 + A-29 S1
- BF-S3/S4: 7 card def fixes. BF-S5–S9 exhausted. 32 total BF fixes.
- A-29 S1: 21 card defs authored. 10 complete, 11 with TODOs.

### 2026-03-30 — W6: BF-S2
- BF-S2: 8 card def fixes (55c43be). Target tightening, stale TODOs, new abilities.

### 2026-03-30 — W6: BF-S1
- BF-S1: 17 card def fixes (e2f07e1, 88f0580). ~30-40% false positive rate confirmed.

### 2026-03-30 — W6: BF-1 + BF-2
- BF-1 re-triage: 1451 defs, 773 clean (53%), 678 with TODOs. BF-2 gap closure committed.

### 2026-03-29 — W6: PB-37
- PB-37: G-26 residual complex activated. 7 card defs fixed. 9 new tests. 2437 tests.
