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
| W6: Primitive + Card Authoring | PB-A: Play from top of library | ACTIVE | 2026-04-07 | HIGH batches — PB-A first (6 cards, continuous cast permission) |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-07
**Workstream**: W6: Primitive + Card Authoring
**Task**: Wave B.5 — PB-A (play from top of library)

**Completed**:
- PB-A: PlayFromTopPermission/PlayFromTopFilter on GameState, AbilityDefinition::StaticPlayFromTop (disc 73), AltCostKind::PayLifeForManaValue, casting.rs + lands.rs integration, on_cast_effect bonus (haste grant pattern). 6 new cards (Future Sight, Bolas's Citadel, Mystic Forge, Oracle of Mul Daya, Vizier of the Menagerie, Radha Heart of Keld) + 4 fixes (Courser of Kruphix, Elven Chorus, Thundermane Dragon, Case of the Locked Hothouse). 2H 2M fixed (haste CR 400.7, permission filter scoping, life check CR 119.4, X=0 enforcement). 18 tests.
- 2549 tests, 0 clippy warnings, ~1741 total card defs

**Next** (agreed priority order):
1. HIGH batches: **PB-B** (play from GY/exile, 5 cards), **PB-E** (mana doubling, 9 cards), **PB-J** (copy/redirect spells, 4 cards), **PB-M** (Panharmonicon trigger doubling, 1 card)
2. After HIGH batches: Wave C (A-30, A-36, A-40, A-41), then final audit (X-1 through X-3)

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward)
- Cavern of Souls "can't be countered" deferred (needs CounterRestriction primitive)
- Nexus of Fate "from anywhere" graveyard replacement only covers resolution case
- A-42 BLOCKED categories remaining: mana-doubling, wheel (play-from-top resolved by PB-A)
- PB-A remaining card TODOs: Elven Chorus mana grant, Vizier mana restriction, Bolas's Citadel sac-10, Mystic Forge exile-top, Radha +X/+X (separate gaps)

**Commit prefix**: `W6-prim:` (engine), `W6-cards:` (card defs)

## Handoff History

### 2026-04-07 — W6: PB-H + PB-L
- PB-H: mass reanimate. PB-L: reveal/X effects. 2531 tests.

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
