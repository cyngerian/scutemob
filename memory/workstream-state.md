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
| W6: Primitive + Card Authoring | — | available | — | PB-22 S1-S3 done. S4-S7 remain. Plan: `memory/primitives/pb-22-session-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-21
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-22 S3 — Reveal-route + flicker (Effect::RevealAndRoute, Effect::Flicker)

**Completed**:
- Added `Effect::RevealAndRoute` (disc 62) — reveal top N, route by filter to two zones (CR 701.16a)
- Added `Effect::Flicker` (disc 63) — exile + return to battlefield with full ETB hooks (CR 400.7)
- Added `zone_move_event()` helper function in effects/mod.rs
- Hash entries: Effect disc 62-63
- Fixed Goblin Ringleader card def: full ETB trigger (subtype Goblin → hand, rest → bottom)
- Fixed Chaos Warp card def: full implementation (shuffle into library + reveal-and-route)
- Review finding: added explicit `Effect::Shuffle` to Chaos Warp (ShuffledIn position hint is not implemented)
- Grief and Biting-Palm Ninja left as-is (hand-reveal + discard/exile — different DSL gap, not RevealAndRoute)
- 8 new tests (reveal_and_route.rs), 2254 total

**Next**:
1. PB-22 S4: Tapped-and-attacking tokens + equipment auto-attach
2. PB-22 S5-S7: copy/clone, emblems, adventure
3. After PB-22: Phase 2 card authoring (~1,025 remaining cards)

**Hazards**:
- None — clean commit, all tests passing, no WIP

**Commit prefix used**: `W6-prim:`

## Handoff History

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

### 2026-03-19 — Exploratory: Rules audit + W3-LC plan
- Audited 4 abilities (Devoid, Flanking, Fabricate, Suspend) against CR rules
- Found 1 MEDIUM bug: Flanking reads base characteristics instead of layer-resolved
- Created `memory/w3-layer-audit.md` — 4-session audit plan (W3-LC S1-S4)
