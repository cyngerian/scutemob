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
| W6: Primitive + Card Authoring | — | available | — | **PB-J + PB-M DONE**; all HIGH batches complete |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-10
**Workstream**: W6: Primitive + Card Authoring
**Task**: Wave B.5 HIGH batches — PB-J + PB-M

**Completed**:
- PB-J: CopySpellOnStack, ChangeTargets (CR 115.7a/d), TargetSpellOrAbilityWithSingleTarget, GameEvent::TargetsChanged. 3 card fixes (Bolt Bend, Deflecting Swat, Untimely Malfunction). 2M 2L fixed. 9 tests.
- PB-M: 2 bug fixes (SelfEntersBattlefield matching in doubler_applies_to_trigger, entering_object_id in queue_carddef_etb_triggers). 2 new TriggerDoublerFilter variants (AnyPermanentETB, LandETB). 1 new card (Panharmonicon) + 3 fixes (Drivnod, Elesh Norn, Ancient Greenwarden). 1H 1M fixed. 5 tests.
- **All HIGH batches complete.** 2589 tests, 0 clippy, ~1751 total defs.

**Next** (priority order from ops plan):
1. Wave C: **A-30** (untap-phase, 12 cards), **A-36** (static-enchantment, 6 cards), **A-40** (x-spell, 1 card), **A-41** (exile-play, 1 card)
2. Final audit: X-1 through X-7

**Hazards** (carried forward):
- `activated_ability_cost_reductions` index on channel lands may be off-by-one
- Cavern of Souls "can't be countered" deferred (needs CounterRestriction)
- Pitch-alt-costs (Force of Negation/Vigor) still blocked
- Forbidden Orchard: TargetPlayer should be TargetOpponent (deferred to M10)
- PB-M deferred: Isshin attack trigger doubling, Delney power-filtered doubling, Elesh Norn opponent ETB suppression, Drivnod activated ability (exile-from-GY cost)
- Complete the Circuit: delayed copy trigger still TODO (needs "when you next cast" primitive)

**Commit prefix**: `W6-prim:` (engine), `W6-cards:` (card defs)

## Handoff History

### 2026-04-09 — W6: PB-A + PB-B + PB-E
- PB-A: play from top of library. PB-B: play from graveyard. PB-E: mana doubling. 2575 tests.

### 2026-04-07 — W6: PB-A + PB-H + PB-L
- PB-A: play from top of library. PB-H: mass reanimate. PB-L: reveal/X effects. 2549 tests.

### 2026-04-06 — W6: PB-C + PB-F + PB-I
- PB-C: ExtraTurn + self_exile/self_shuffle. PB-F: TripleDamage, DamageTargetFilter. PB-I: FlashGrant, OpponentsCanOnlyCastAtSorcerySpeed. 2504 tests.

### 2026-04-04 — W6: PB-K + PB-D
- PB-K: land drops, Case mechanic. PB-D: chosen creature type, 8 fixes. 2474 tests.

### 2026-04-02 — W6: PB-N + PB-G
- PB-N: 19 misc card defs. PB-G: BounceAll + TargetFilter extensions + 4 cards. 2445 tests.
