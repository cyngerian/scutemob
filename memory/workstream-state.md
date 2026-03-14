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
| W6: Primitive + Card Authoring | PB-4: Sacrifice as activation cost (26 cards) | ACTIVE | 2026-03-14 | **TOP PRIORITY**. PB-0 through PB-3 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-14
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-3 — Shockland pay-life-or-tapped (10 cards)

**Completed**:
- Added `ReplacementModification::EntersTappedUnlessPayLife(u32)` variant to replacement_effect.rs
- Combined match arm with `EntersTapped` in `emit_etb_modification` — deterministic fallback (always enters tapped pre-M10)
- Hash discriminant 8 in hash.rs
- 10 shockland card defs updated: replacement + dual mana abilities (TODOs removed)
- 3 new unit tests (single shockland, all 10 loop, variant distinction)
- Commit 734cfff; 1993 tests, 0 clippy warnings

**Next**:
1. Follow execution order in `docs/primitive-card-plan.md` — PB-4 through PB-21
2. **PB-4**: Sacrifice as activation cost (26 cards) or **PB-5**: Targeted abilities (32 cards)
3. ~40 fewer wrong-game-state cards remaining (was ~50 after PB-2, fixed 10 shocklands)

**Hazards**: Pre-existing uncommitted changes in working tree from prior sessions (CLAUDE.md, command.rs, engine.rs, encore.rs, docs, memory).

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-14 — W6: PB-3 shockland pay-life-or-tapped (10 cards)
- EntersTappedUnlessPayLife(u32) variant + 10 card defs fixed; commit 734cfff; 1993 tests

### 2026-03-14 — W6: PB-2 conditional ETB tapped (56 cards)
- unless_condition on AbilityDefinition::Replacement + 10 Condition variants; 56 card defs fixed; commit 091baa5; 1990 tests

### 2026-03-13 — W6: PB-1 pain land mana-with-damage (8 cards)
- ManaAbility.damage_to_controller + WhenSelfBecomesTapped trigger; 8 card defs fixed; commit 6601de0; 1982 tests

### 2026-03-13 — Cross-cutting: P1 sanity reviews + interaction gap fixes
- 3 P1 abilities reviewed (Trample/Protection/First Strike); 1 HIGH + 6 MEDIUM fixed; 9 LOW deferred; interaction-gaps.md created

### 2026-03-13 — W6: PB-0 quick-win card fixes (20 cards)
- 20 card defs fixed; color_indicator field + MustAttackEachCombat keyword; commit e3ca167; 1972 tests
