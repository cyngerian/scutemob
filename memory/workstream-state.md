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
| W6: Primitive + Card Authoring | PB-15: Saga & Class enchantments | ACTIVE | 2026-03-15 | **TOP PRIORITY**. PB-14 complete. Now: PB-15 (Saga/Class). Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-15
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-14 — Planeswalker support

**Completed**:
- Full planeswalker loyalty framework per CR 306/606
- `CardDefinition.starting_loyalty: Option<u32>` (CR 306.5a) — 129 card defs + 16 test files patched
- `LoyaltyCost` enum: Plus/Minus/Zero/MinusX (CR 606.4) + hash
- `AbilityDefinition::LoyaltyAbility { cost, effect, targets }` (disc 66)
- ETB loyalty counter placement in `replacement.rs` (CR 306.5b)
- `Command::ActivateLoyaltyAbility` with full validation (CR 606.3/606.6)
- `StackObjectKind::LoyaltyAbility` (disc 67) + resolution in `resolution.rs`
- `loyalty_ability_activated_this_turn` on `GameObject` (CR 606.3), reset in `turn_actions.rs`
- `LegalAction::ActivateLoyaltyAbility` in simulator (legal_actions, heuristic_bot, random_bot)
- `activate_loyalty_ability` replay harness action
- Ajani + Tyvar card defs updated with `starting_loyalty` + `LoyaltyAbility` stubs
- 12 new tests in `planeswalker.rs`, 2108 total passing, 0 clippy warnings
- Commit d7faeff

**Deferred from PB-13 (carried forward)**:
- Equipment auto-attach (13d), Timing restriction (13i) → PB-18, Clone/copy ETB (13j), Adventure (13m), Coin flip/d20 (13h), Flicker (13l), PB-12 leftovers

**Next**:
1. **PB-15 (Saga & Class)**: lore counters, chapter abilities, sacrifice after final chapter SBA
2. Continue through PB-16 to PB-21 per `docs/primitive-card-plan.md`

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-15 — W6: PB-13 part 3 (Ascend condition + audit)
- Condition::HasCitysBlessing + Arch of Orazca/Twilight Prophet fixes + 1 test; Dredge/Buyback/LivingWeapon confirmed done; coin flip/flicker deferred; 2096 tests

### 2026-03-15 — W6: PB-13 part 2 (Channel + land animation)
- Cost::DiscardSelf + hand-zone activation + 5 NEO lands + Blinkmoth/Inkmoth animate + 7 tests; commit 50758e5; 2095 tests

### 2026-03-15 — W6: PB-13 part 1 (player hexproof + monarch)
- HexproofPlayer (disc 159) + Monarch (CR 724) + stale TODO cleanup + 9 tests; commit 5a4530c; 2088 tests

### 2026-03-15 — W6: PB-12 complex replacement effects (8 cards)
- 7 triggers + 8 modifications + 1 TriggerDoublerFilter + 6 helpers + 8 card fixes + 14 tests; commit 20d8981; 2079 tests

### 2026-03-15 — W6: PB-11 mana restrictions + ETB choice (10 cards)
- ManaRestriction enum + restricted mana pool + chosen_creature_type + 10 card fixes + 11 tests; commit 382ae7d; 2065 tests
