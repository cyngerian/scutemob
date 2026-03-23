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
| W6: Primitive + Card Authoring | — | available | — | **A-24 through A-28 DONE** (61 new cards). Next: A-29 cant-restriction. ~1452 card defs. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-23
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 2 authoring — A-24 through A-28

**Completed**:
- A-24 attack-trigger: 24 new cards (Shared Animosity, Aurelia, Derevi, etc.)
- A-25 activated-tap: 19 new cards (Maze of Ith, Birthing Pod, Arcanis, etc.)
- A-26 activated-sacrifice: 7 new cards (Altar of Dementia, Dreamstone Hedron, etc.)
- A-27 sacrifice-outlet: 4 new cards (Miren, Diamond Valley, Claws of Gix, Altar of Bone)
- A-28 discard-effect: 7 new cards (Waste Not, Megrim, Burglar Rat, etc.)
- 4 review batches completed, 8 HIGH findings fixed
- Commit: 1afbf25
- Total: ~1452 card def files (+61 this session)
- All 2281 tests passing, workspace builds clean

**Next**:
1. **A-29 cant-restriction** (3 sessions, 24 cards) — next authoring group
2. Continue through A-30 untap-phase, A-32 land-fetch, etc.
3. A-19 S40-S43 review still pending (48 cards from 2026-03-22 session)

**Hazards**:
- `WheneverCreatureDies` has no controller filter — fires on ALL creatures dying
- `WheneverCreatureYouControlAttacks` trigger does not exist (blocks most A-24 cards)
- `WheneverOpponentDiscards` trigger does not exist (blocks all A-28 cards)
- `Cost::RemoveCounter` does not exist — counter removal as cost uses TODO
- `EffectAmount::PowerOfSacrificedCreature` does not exist (blocks Altar of Dementia, Greater Good, Life's Legacy)
- `AdditionalCost::SacrificeCreature` for spells does not exist (blocks Altar of Bone, Life's Legacy)
- `PlayerTarget::Owner` does not exist — zone returns use Controller as proxy (wrong under steal effects)

**Commit prefix used**: `W6-cards:`

## Handoff History

### 2026-03-23 — W6: Phase 2 authoring A-20 through A-23
- A-20 pump-buff (27), A-21 counters-plus (49), A-22 equipment (11), A-23 death-trigger (34). 121 total. 4H fixed.
- Commits: e5b0436, ec08405. 2281 tests.

### 2026-03-23 — W6: Phase 2 authoring A-19 token-create S44-S52
- A-19 token-create S44-S52: 96 new cards. Reviewed, 3H fixed.
- Commits: 5d967ca, 83c1302. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-18 draw + A-19 start
- A-18 S20-S25 (43 new), A-19 S40-S43 (48 new). 91 total. 4H fixed.
- Commits: fc27279, 047532f, f43d88b, 0de9b8c. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-18 draw S10-S19
- 119 new cards (10 sessions of 16 complete)
- Commits: 4a15b5e, 9bf3870, 6ecdb68. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-14 through A-17 (Tier 2 cont'd)
- 45 new cards (damage-each, bounce, minus, counter). All HIGH fixed.
- Commit: 6ddc832. 2281 tests.
