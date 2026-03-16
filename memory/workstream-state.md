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
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; T3 ManaPool pending |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | PB-17: Library search filters | ACTIVE | 2026-03-15 | **TOP PRIORITY**. 74 cards blocked on SearchFilter enum extension. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-15
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-16 — Meld mechanics

**Completed**:
- Full Meld framework per CR 701.42 / CR 712.4 / CR 712.8g
- MeldPair struct on CardDefinition (pair_card_id, melded_card_id)
- meld_component: Option<CardId> on GameObject — tracks second card in melded permanent
- Effect::Meld variant — exiles source + partner, creates melded permanent on battlefield
- Layer system melded face resolution (CR 712.8g) — uses back_face from melded_card_id definition
- Zone-change splitting — both cards go to destination zone when melded permanent leaves battlefield
- CR 712.4c guard — meld cards cannot be transformed (silently ignored in engine.rs)
- 3 card defs: hanweir_battlements.rs (meld ability wired), hanweir_garrison.rs (new), hanweir_the_writhing_township.rs (new melded back_face holder)
- 7 new tests in meld.rs, 2126 total passing, 0 clippy warnings
- Commit 9d384a3

**Deferred from PB-13/14 (carried forward)**:
- Equipment auto-attach (13d), Timing restriction (13i) → PB-18, Clone/copy ETB (13j), Adventure (13m), Coin flip/d20 (13h), Flicker (13l), PB-12 leftovers
- Hanweir Garrison attack trigger (tapped-and-attacking tokens not in DSL)
- Hanweir Township attack trigger (same DSL gap)

**Next**:
1. **PB-17 (Library search filters)**: 74 cards, SearchFilter enum extension — biggest remaining gap
2. Continue through PB-18 to PB-21 per `docs/primitive-card-plan.md`

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-15 — W6: PB-15 Saga & Class mechanics
- Full Saga framework per CR 714: SagaChapter AbilityDefinition (disc 67), ETB lore counter, precombat main lore counter TBA, chapter triggers, sacrifice SBA (CR 714.4), SBA deferred while chapter on stack
- Full Class framework per CR 716: ClassLevel AbilityDefinition (disc 68), class_level on GameObject, Command::LevelUpClass with sorcery-speed + level-N-1 validation, level-up registers static continuous effects
- 11 new tests in saga_class.rs, 2119 total passing; commit f5878a8

### 2026-03-15 — W6: PB-14 Planeswalker support
- Full loyalty framework: LoyaltyCost, LoyaltyAbility (disc 66), ETB loyalty counters, ActivateLoyaltyAbility command, 0-loyalty SBA, 12 tests; commit d7faeff; 2108 tests

### 2026-03-15 — W6: PB-13 part 3 (Ascend condition + audit)
- Condition::HasCitysBlessing + Arch of Orazca/Twilight Prophet fixes + 1 test; Dredge/Buyback/LivingWeapon confirmed done; coin flip/flicker deferred; 2096 tests

### 2026-03-15 — W6: PB-13 part 2 (Channel + land animation)
- Cost::DiscardSelf + hand-zone activation + 5 NEO lands + Blinkmoth/Inkmoth animate + 7 tests; commit 50758e5; 2095 tests

### 2026-03-15 — W6: PB-13 part 1 (player hexproof + monarch)
- HexproofPlayer (disc 159) + Monarch (CR 724) + stale TODO cleanup + 9 tests; commit 5a4530c; 2088 tests
