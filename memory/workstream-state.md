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
| W6: Primitive + Card Authoring | PB-2: Conditional ETB tapped (56 cards) | ACTIVE | 2026-03-14 | **TOP PRIORITY**. PB-0+PB-1 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-13
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-1 — Pain land mana-with-damage (8 cards)

**Completed**:
- Added `ManaAbility.damage_to_controller` field for pain land self-damage (CR 605)
- Added `TriggerCondition::WhenSelfBecomesTapped` for City of Brass (fires on any tap)
- Extended `try_as_tap_mana_ability` to recognize `Sequence(AddMana+DealDamage)` and `AddManaAnyColor` patterns
- Fixed all 8 pain land card defs: battlefield_forge, caves_of_koilos, city_of_brass, llanowar_wastes, shivan_reef, sulfurous_springs, underground_river, yavimaya_coast
- 5 new tests in pain_lands.rs; commit 6601de0; 1982 tests, 0 clippy warnings

**Next**:
1. **PB-5**: Targeted abilities (32 cards, 2-3 sessions) — highest leverage
2. **PB-2**: Conditional ETB tapped (56 cards, 2-3 sessions) — most cards
3. Follow execution order in `docs/primitive-card-plan.md`

**Hazards**: Pre-existing uncommitted changes in working tree from prior sessions (CLAUDE.md, command.rs, engine.rs, encore.rs, docs, memory). ~106 card defs still produce wrong game state (was 114, fixed 8 pain lands).

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-13 — W6: PB-1 pain land mana-with-damage (8 cards)
- ManaAbility.damage_to_controller + WhenSelfBecomesTapped trigger; 8 card defs fixed; commit 6601de0; 1982 tests

### 2026-03-13 — Cross-cutting: P1 sanity reviews + interaction gap fixes
- 3 P1 abilities reviewed (Trample/Protection/First Strike); 1 HIGH + 6 MEDIUM fixed; 9 LOW deferred; interaction-gaps.md created

### 2026-03-13 — W6: PB-0 quick-win card fixes (20 cards)
- 20 card defs fixed; color_indicator field + MustAttackEachCombat keyword; commit e3ca167; 1972 tests

### 2026-03-13 — W5 → W6: Wave 3 mana-land (78 cards) + strategic pivot
- Wave 3: 78 cards authored+reviewed+fixed, commit 0896563; DSL gap audit; unauthored card scan (1,195 cards); primitive-first plan created; W5 retired → W6

### 2026-03-13 — W5: Wave 2 combat-keyword (187 cards) complete
- 14 sessions (26–39); 38 review batches; 13 HIGH fixes; commits d83ac94+01e3b52; 640 total card defs; 1944 tests
