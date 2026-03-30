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
| W6: Primitive + Card Authoring | A-29 S131: cant-restriction authoring | ACTIVE | 2026-03-30 | **ALL PBs COMPLETE (PB-0–37)**; BF complete (32 fixes); A-29 S1 done (21 cards) |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-30
**Workstream**: W6 (BF-S3/S4 + A-29 S1)
**Task**: Backfill fix completion + A-29 card authoring session 1

**Completed**:
- **BF-S3**: 6 card def fixes (93b95de) — artifact-or-enchantment targeting (krosan_grip, gemrazer, freyalise), +0/+3 sac ability (mardu_ascendancy), 2 stale TODOs removed (hermes, untimely_malfunction)
- **BF-S4**: 1 fix (7a7b6a6) — putrefy targeting tightened. Cross-range pattern sweep confirmed no more fixable TODOs.
- **BF-S5–S9 marked exhausted** — all remaining ~670 TODO files are genuine DSL gaps. 32 total BF fixes across S1-S4.
- **A-29 Session 130 (S1)**: 21 card defs authored (1c63f89). 10 complete (unblockable creatures, Dovin's Veto, Elvish Champion), 11 with TODOs (stax restrictions, cant-be-countered on creatures, choose-type, power-conditional evasion).

**Next**:
1. **A-29 Session 131**: 8 cards (Autumn's Veil, Slither Blade, Triton Shorestalker, Mist-Cloaked Herald, Blighted Agent, Tormented Soul, Soulless Jailer, Delney). Note: Slither Blade through Tormented Soul already authored in S1 — session 131 may only need Autumn's Veil (already authored with TODO), Soulless Jailer (authored with TODO), Delney (authored with TODO). Re-check authoring plan to confirm which remain.
2. Then sessions 132-133 to complete A-29.
3. After A-29: proceed to A-30 (untap-phase, 12 cards).

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward from prior handoff).
- Most A-29 TODOs require new GameRestriction variants or cant-be-countered on creature spells — these are genuine DSL gaps that need engine work.

**Commit prefix**: `W6-cards:`

## Handoff History

### 2026-03-30 — W6: BF-S2
- BF-S2: 8 card def fixes (55c43be). Target tightening, stale TODOs, new abilities. Lower yield confirms alphabetical ranges don't map to fixable patterns.

### 2026-03-30 — W6: BF-S1
- BF-S1: 17 card def fixes (e2f07e1, 88f0580). ~30-40% false positive rate confirmed. Key patterns: death triggers, Cost::Sacrifice, UntapPermanent, simple activated, bounce.

### 2026-03-30 — W6: BF-1 + BF-2
- BF-1 re-triage: 1451 defs, 773 clean (53%), 678 with TODOs. BF-2 gap closure committed. 9 backfill sessions planned.

### 2026-03-29 — W6: PB-37
- PB-37: G-26 residual complex activated. 7 card defs fixed. 9 new tests. 2437 tests.

### 2026-03-29 — W6: PB-36
- PB-36: G-31 evasion/protection extensions. 16 card defs fixed. 2428 tests.
