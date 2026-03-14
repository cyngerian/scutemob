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
| W6: Primitive + Card Authoring | PB-6: Static grant with controller filter (30 cards) | ACTIVE | 2026-03-14 | **TOP PRIORITY**. PB-0 through PB-5 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-14
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-5 — Targeted activated/triggered abilities (32 cards)

**Completed**:
- Added `targets: Vec<TargetRequirement>` to `AbilityDefinition::Activated` and `::Triggered`
- Added matching `targets` field to runtime `ActivatedAbility` and `TriggeredAbilityDef`
- Wired target validation in `handle_activate_ability()` via `validate_targets()` (made pub(crate))
- Added target count validation (mismatched count → InvalidTarget, only when requirements non-empty)
- Updated `HashInto` impls, `enrich_spec_from_def()`, builder.rs
- Mass migration: `#[serde(default)]` + `targets: vec![],` across ~300+ card def and test files
- 9 card defs fully fixed: goblin_motivator, flamekin_village, hanweir_battlements, forerunner_of_slaughter, access_tunnel, blinkmoth_nexus, slayers_stronghold, rogues_passage, briarblade_adept
- 9 new tests in targeted_abilities.rs
- Remaining 23 PB-5 cards have deeper DSL gaps (graveyard targeting, exchange control, color choice, etc.)
- Commit 9ca054a; 2006 tests, 0 clippy warnings

**Next**:
1. Follow execution order in `docs/primitive-card-plan.md` — PB-6 through PB-21
2. **PB-6**: Static grant with controller filter (30 cards) — `EffectFilter::CreaturesYouControl`
3. ~23 PB-5 cards remain blocked on future primitives (PB-10 graveyard targeting, PB-14 planeswalkers, etc.)

**Hazards**: CLAUDE.md has uncommitted changes (test count update). No conflicts.

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-14 — W6: PB-5 targeted activated/triggered abilities (32 cards)
- targets field on Activated/Triggered + validation + 9 card fixes + 9 tests; commit 9ca054a; 2006 tests

### 2026-03-14 — W6: PB-4 sacrifice as activation cost (26 cards)
- SacrificeFilter + Cost::SacrificeSelf + 13 card fixes + 4 tests; commit 0344539; 1997 tests

### 2026-03-14 — W6: PB-3 shockland pay-life-or-tapped (10 cards)
- EntersTappedUnlessPayLife(u32) variant + 10 card defs fixed; commit 734cfff; 1993 tests

### 2026-03-14 — W6: PB-2 conditional ETB tapped (56 cards)
- unless_condition on AbilityDefinition::Replacement + 10 Condition variants; 56 card defs fixed; commit 091baa5; 1990 tests

### 2026-03-13 — W6: PB-1 pain land mana-with-damage (8 cards)
- ManaAbility.damage_to_controller + WhenSelfBecomesTapped trigger; 8 card defs fixed; commit 6601de0; 1982 tests
