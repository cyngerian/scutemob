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
| W6: Primitive + Card Authoring | PB-22 S7: Adventure + dual-zone search | ACTIVE | 2026-03-21 | S1-S6 done. S7 is the final PB-22 session. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-21
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-22 S7 — Adventure (CR 715) + dual-zone search

**Completed**:
- Added `AltCostKind::Adventure` (disc 27) — cast adventure half from hand, exile on resolution (CR 715.3d)
- Added `adventure_face: Option<CardFace>` to CardDefinition (CR 715.2)
- Added `was_cast_as_adventure` to StackObject, `adventure_exiled_by` to GameObject
- Casting: zone validation (hand for adventure, exile for creature), type/cost override from adventure_face
- Resolution: exile-on-resolution, counter/fizzle go to graveyard (CR 715.3d)
- Copy propagation: `was_cast_as_adventure` propagated to copies (CR 715.3c)
- Added `also_search_graveyard: bool` to Effect::SearchLibrary — dual-zone search
- 2 new card defs: Bonecrusher Giant, Lovestruck Beast; 3 fixed: Monster Manual, Finale of Devastation, Lozhan
- 136 card defs updated with `adventure_face: None`, 26 with `also_search_graveyard: false`
- Review: 1 HIGH (Monster Manual oracle fix), 3 MEDIUM (copy propagation, legal_actions gap, Bonecrusher TODO)
- 9 new tests in adventure_tests.rs, 2281 total
- **PB-22 COMPLETE** — all 7 sessions done

**Next**:
1. Phase 2 card authoring (~1,025 remaining cards) — see `docs/card-authoring-operations.md`

**Hazards**:
- None — clean commit, all tests passing, no WIP

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-21 — W6: PB-22 S7 (Adventure CR 715 + Dual-Zone Search)
- AltCostKind::Adventure + adventure_face on CardDefinition + exile-on-resolution
- also_search_graveyard on Effect::SearchLibrary (dual-zone search)
- 2 new cards (Bonecrusher Giant, Lovestruck Beast), 3 fixed (Monster Manual, Finale, Lozhan)
- Review: 1H 3M, all fixed (oracle mismatch, copy propagation, TODO docs)
- 9 new tests, 2281 total
- **PB-22 COMPLETE — all deferred cleanup done**

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
