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
| W6: Primitive + Card Authoring | PB-18: Stax/restrictions | ACTIVE | 2026-03-16 | **TOP PRIORITY**. PB-17 complete. ContinuousRestriction system. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-16
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-17 — Library search filters

**Completed**:
- Added `max_cmc`, `min_cmc` (Option<u32>), `has_card_types` (Vec<CardType>, OR semantics) to TargetFilter
- Updated `matches_filter()` in effects/mod.rs for CMC (CR 202.3) and OR card-type checks
- Updated hash.rs for deterministic hashing of new fields
- 8 new tests in library_search.rs (max_cmc, min_cmc, has_card_types OR, empty filter, combined, MV=0, no-match, top-of-library)
- 9 card defs fixed: urzas_saga (artifact CMC≤1), maelstrom_of_the_spirit_dragon (Dragon search), scion_of_the_ur_dragon (Dragon to GY), inventors_fair (artifact tutor), assassins_trophy + ghost_quarter + boseiju_who_endures (opponent search via ControllerOf), haven_of_the_spirit_dragon (GY Dragon targeting), finale_of_devastation (partial creature search)
- 2134 tests passing, 0 clippy warnings, full workspace builds clean
- Commit 894504e

**Deferred from PB-13/14 (carried forward)**:
- Equipment auto-attach (13d), Timing restriction (13i) → PB-18, Clone/copy ETB (13j), Adventure (13m), Coin flip/d20 (13h), Flicker (13l), PB-12 leftovers
- Hanweir Garrison/Township attack triggers (tapped-and-attacking tokens not in DSL)

**Deferred from PB-17**:
- Tiamat: multi-card search with uniqueness constraint (up to 5 different Dragons)
- Goblin Ringleader: "reveal top N, route by type" pattern (not SearchLibrary)
- Finale of Devastation: graveyard search, X≥10 conditional pump + mass haste
- Scion of the Ur-Dragon: copy-self effect after search
- Inventors' Fair: activation condition "only if you control 3+ artifacts"

**Next**:
1. **PB-18 (Stax/restrictions)**: 13 cards — ContinuousRestriction system for casting/attacking limits
2. Continue through PB-19 to PB-21 per `docs/primitive-card-plan.md`

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-15 — W6: PB-16 Meld mechanics
- Full Meld framework per CR 701.42 / CR 712.4 / CR 712.8g; MeldPair, Effect::Meld, zone-change splitting; 3 card defs; 7 tests; 2126 total; commit 9d384a3

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
