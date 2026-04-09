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
| W6: Primitive + Card Authoring | PB-J: Copy/redirect spells | ACTIVE | 2026-04-09 | 2 HIGH remain (PB-J, PB-M) |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-09
**Workstream**: W6: Primitive + Card Authoring
**Task**: Wave B.5 — PB-A + PB-B + PB-E (play-from-top, play-from-GY, mana doubling)

**Completed**:
- PB-A: PlayFromTopPermission, PayLifeForManaValue. 6 new cards + 4 fixes. 2H 2M fixed. 18 tests.
- PB-B: PlayFromGraveyardPermission, CastSelfFromGraveyard. 6 card fixes. 2M 3L fixed. 15 tests.
- PB-E: WhenTappedForMana + ManaSourceFilter, MultiplyMana, AddManaMatchingType. 3 new cards + 6 fixes. 1H 3M fixed. 11 tests.
- 2575 tests, 0 clippy warnings, ~1750 total card defs

**Next** (agreed priority order):
1. HIGH batches: **PB-J** (copy/redirect spells, 4 cards), **PB-M** (Panharmonicon trigger doubling, 1 card)
2. After HIGH batches: Wave C (A-30, A-36, A-40, A-41), then final audit (X-1 through X-3)

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward)
- Cavern of Souls "can't be countered" deferred (needs CounterRestriction primitive)
- Nexus of Fate "from anywhere" graveyard replacement only covers resolution case
- A-42 BLOCKED categories: mana-doubling resolved by PB-E; wheel still blocked
- PB-A remaining card TODOs: Elven Chorus mana grant, Vizier mana restriction, Bolas's Citadel sac-10, Mystic Forge exile-top, Radha +X/+X (separate gaps)
- Pitch-alt-costs (Force of Negation/Vigor) still blocked — separate PB needed
- Forbidden Orchard: TargetPlayer should be TargetOpponent (deferred to M10)

**Commit prefix**: `W6-prim:` (engine), `W6-cards:` (card defs)

## Handoff History

### 2026-04-07 — W6: PB-A + PB-H + PB-L
- PB-A: play from top of library. PB-H: mass reanimate. PB-L: reveal/X effects. 2549 tests.

### 2026-04-06 — W6: PB-C + PB-F + PB-I
- PB-C: ExtraTurn + self_exile/self_shuffle. PB-F: TripleDamage, DamageTargetFilter. PB-I: FlashGrant, OpponentsCanOnlyCastAtSorcerySpeed. 2504 tests.

### 2026-04-04 — W6: PB-K + PB-D
- PB-K: land drops, Case mechanic. PB-D: chosen creature type, 8 fixes. 2474 tests.

### 2026-04-02 — W6: PB-N + PB-G
- PB-N: 19 misc card defs. PB-G: BounceAll + TargetFilter extensions + 4 cards. 2445 tests.

### 2026-04-01 — W6: A-42 batch 2
- A-42 batch 2: 60 new card defs. Total A-42: 77/131. 1693 total defs. 2437 tests.

### 2026-03-31 — W6: Wave B A-38+A-42 partial
- Wave A engine review CLEAN. A-38: 53 new defs. A-42 batch 1: 17 new defs. 70 total.
