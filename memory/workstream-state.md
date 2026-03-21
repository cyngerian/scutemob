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
| W6: Primitive + Card Authoring | — | available | — | PB-22 S1-S5 done. S6-S7 remain. Plan: `memory/primitives/pb-22-session-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-21
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-22 S5 — Copy/clone primitives (BecomeCopyOf, CreateTokenCopy)

**Completed**:
- Added `Effect::BecomeCopyOf { copier, target, duration }` — registers Layer 1 CopyOf CE with configurable duration (CR 707.2)
- Added `Effect::CreateTokenCopy { source, enters_tapped_and_attacking }` — creates blank token with CopyOf CE (CR 707.2 + CR 508.4)
- Added `Condition::CardTypesInGraveyardAtLeast(u32)` for Delirium activation conditions
- Added `GameEvent::BecameCopyOf { copier, source }` (disc 123)
- Hash: Effect disc 64-65, GameEvent disc 123, Condition disc 31
- Fixed Thespian's Stage: full {2},{T} BecomeCopyOf (Indefinite, target land)
- Fixed Shifting Woodland: Delirium {2}{G}{G} BecomeCopyOf (UntilEndOfTurn, 4+ card types)
- Fixed Thousand-Faced Shadow: ETB CreateTokenCopy (tapped+attacking)
- Scion of the Ur-Dragon: TODO improved (needs EffectTarget::LastSearchResult)
- Review: 0 HIGH, 2 MEDIUM (target filter TODOs documented), 3 LOW
- 5 new tests in copy_effects.rs, 2265 total

**Next**:
1. PB-22 S6: Emblem creation (CR 114) — 11 planeswalker cards
2. PB-22 S7: Adventure + dual-zone search
3. After PB-22: Phase 2 card authoring (~1,025 remaining cards)

**Hazards**:
- None — clean commit, all tests passing, no WIP

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-21 — W6: PB-22 S5 (Copy/Clone Primitives)
- Added Effect::BecomeCopyOf + Effect::CreateTokenCopy + Condition::CardTypesInGraveyardAtLeast
- Fixed Thespian's Stage, Shifting Woodland, Thousand-Faced Shadow card defs
- Review: 0H 2M 3L, fixes applied (TODO documentation)
- 5 new tests, 2265 total

### 2026-03-21 — W6: PB-22 S3 (RevealAndRoute + Flicker)
- Added Effect::RevealAndRoute + Effect::Flicker (CR 701.16a, CR 400.7)
- Fixed Goblin Ringleader, Chaos Warp card defs
- 8 new tests, 2254 total

### 2026-03-21 — W6: PB-22 S2 (CoinFlip + RollDice)
- Added Effect::CoinFlip + Effect::RollDice (CR 705/706), deterministic RNG
- Fixed Mana Crypt, Ancient Silver/Brass Dragon card defs
- 10 new tests, 2246 total

### 2026-03-20 — W6: PB-22 S1 (activation_condition)
- Added `activation_condition: Option<Condition>` to activated abilities (CR 602.5b)
- 305 files changed (277 card defs + tests + engine), 3 new tests, 2236 total

### 2026-03-20 — W3: LOW Remediation (sprint complete)
- W3 LOW sprint S1-S6 + bookkeeping: 83→29 LOW open, 119 closed, 2233 tests
- W3 effectively complete — remaining 29 LOWs permanently deferred or DSL-blocked

### 2026-03-19 — W6: PB-21 Fight & Bite
- PB-21 complete: Effect::Fight + Effect::Bite, 4 card defs fixed, 14 new tests, 2206 total
- ALL 22 PRIMITIVE BATCHES COMPLETE (PB-0 through PB-21)
