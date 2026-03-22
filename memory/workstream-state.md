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
| W6: Primitive + Card Authoring | PB-22 S6: Emblem creation (CR 114) | ACTIVE | 2026-03-21 | PB-22 S1-S5 done. S6 in progress. Plan: `memory/primitives/pb-22-session-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-21
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-22 S6 — Emblem creation (CR 114)

**Completed**:
- Added `Effect::CreateEmblem { triggered_abilities, static_effects }` (disc 66) — creates emblem in command zone (CR 114.1-114.5)
- Added `GameEvent::EmblemCreated { player, object_id }` (disc 124)
- Added `is_emblem: bool` to `GameObject` (CR 114.5)
- Added `collect_emblem_triggers_for_event` in abilities.rs — scans command zone emblems for matching triggers (CR 113.6p, CR 114.4)
- Wired emblem trigger scanning to SpellCast, begin_combat(), upkeep_actions(), end_step_actions()
- New TriggerEvent variants: AtBeginningOfCombat, AtBeginningOfYourUpkeep, AtBeginningOfEachUpkeep, AtBeginningOfYourEndStep (hash disc 24-27)
- Fixed Ajani Sleeper Agent -6 TODO → CreateEmblem
- Authored 5 new planeswalker defs: Basri Ket, Kaito Bane of Nightmares, Tyvar Kell, Wrenn and Realmbreaker, Wrenn and Seven
- Review: 5 HIGH (oracle mismatches fixed), 6 MEDIUM (trigger scanning + TODOs documented), 2 LOW
- 7 new tests in emblem_tests.rs, 2272 total

**Next**:
1. PB-22 S7: Adventure + dual-zone search
2. After PB-22: Phase 2 card authoring (~1,025 remaining cards)

**Hazards**:
- None — clean commit, all tests passing, no WIP

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-21 — W6: PB-22 S6 (Emblem Creation, CR 114)
- Added Effect::CreateEmblem + GameEvent::EmblemCreated + is_emblem field + emblem trigger scanning
- 4 new TriggerEvent variants (AtBeginningOfCombat, upkeep, end step)
- Fixed Ajani Sleeper Agent, authored Basri Ket, Kaito, Tyvar Kell, Wrenn and Realmbreaker, Wrenn and Seven
- Review: 5H 6M 2L, all HIGH/MEDIUM fixed (oracle corrections + trigger scanning wiring)
- 7 new tests, 2272 total

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
