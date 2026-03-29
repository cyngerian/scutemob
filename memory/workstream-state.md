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
| W6: Primitive + Card Authoring | — | available | — | PB-35 DONE |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-28
**Workstream**: W6 (PB-35)
**Task**: PB-35 Modal triggers + graveyard conditions + planeswalker abilities

**Completed**:
- **PB-35 DONE**: G-27 Modal triggers + G-29 Graveyard abilities. Engine: ActivationZone/TriggerZone enums, modes on Triggered abilities, graveyard activation/trigger dispatch, modal trigger resolution. 14 card defs fixed (4 graveyard + 10 modal). 11 new tests. Review: 1H 3M fixed (Shambling Ghast wrong trigger + target, Bloodghast may, Jitte scope). 2419 tests, 0 clippy.
- G-30 Planeswalker: deferred to PB-37 (most PW TODOs are general DSL gaps, not PW-framework).
- Commits: 727a0f5 (implement), ed895e7 (fixes).

**Next**:
1. Continue gap closure: PB-36 (evasion/protection extensions, ~21 cards)
2. PB-37 (residual complex activated) after PB-36
3. Backfill sweeps accumulating (~500+ cards across PB-23–35)

**Hazards**:
- Backfill sweeps accumulating (~500+ cards across PB-23–35)

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-28 — W6: PB-35
- PB-35: G-27/G-29/G-30 modal triggers + graveyard abilities. 14 card defs fixed, 1H 3M fixed. 2419 tests. Commits: 727a0f5, ed895e7.

### 2026-03-27 — W6: PB-34
- PB-34: G-23/G-24/G-25 mana production. 7 filter lands fixed, AddManaScaled orphan fix. Review clean (2L). 2408 tests. Commit: 71ad3ce.


### 2026-03-27 — W6: PB-33
- PB-33: G-22/G-28 copy/clone + exile/flicker timing. 15 card defs fixed, 2H 1M fixed. 2403 tests. Commits: 3bf6d25, f08c0fc.

### 2026-03-26 — W6: PB-32
- PB-32: G-18/G-19/G-20/G-21 static/effect primitives, 22 card defs fixed, 2M fixed. 2396 tests. Commits: 8401dca, 1502d1c.

### 2026-03-26 — W6: PB-31
- PB-31: G-16 RemoveCounter + G-17 SpellAdditionalCost, 18 card defs fixed, 2M fixed. 2383 tests. Commits: b9f8efa, aeb87d5.

### 2026-03-25 — W6: PB-30
- PB-30: G-8 Combat damage triggers, 27 card defs fixed, 5H 4M fixed. 2371 tests. Commits: b5577c7, b8c8dc6.

### 2026-03-25 — W6: PB-28 + PB-29
- PB-28: G-6 CDA, 9 card defs fixed, 1M fixed. 2353 tests. Commits: ee56134, 3882c1b.
- PB-29: G-7 Cost reduction statics, 13 card defs fixed, 1H fixed. 2363 tests. Commits: e562ec0, bf6e992.
- A-19 token-create S44-S52: 96 new cards. Reviewed, 3H fixed.
- Commits: 5d967ca, 83c1302. 2281 tests.
