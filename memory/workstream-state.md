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
| W6: Primitive + Card Authoring | W6-review: retroactive PB review | ACTIVE | 2026-03-16 | **PRIMARY OBJECTIVE**: review all 19 PB batches before forward progress. Use `/implement-primitive --review-only PB-<N>`. Tracker: `docs/project-status.md` Review Backlog. PB-19+ blocked until reviews complete. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-16
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-18 — Stax/restriction framework

**Completed**:
- New `GameRestriction` enum (6 variants) + `ActiveRestriction` struct on GameState
- `AbilityDefinition::StaticRestriction` variant (hash disc 69) for card defs
- Registration via `register_static_continuous_effects` (replacement.rs)
- `check_cast_restrictions()` in casting.rs — MaxSpellsPerTurn, OpponentsCantCast*, OpponentsCantCastFromNonHand
- `check_activate_restrictions()` in abilities.rs — ArtifactAbilitiesCantBeActivated, OpponentsCantCastOrActivateDuringYourTurn
- `is_cast_restricted_by_stax()` in simulator legal_actions.rs
- 7 new card defs: Rule of Law, Drannith Magistrate, Propaganda, Ghostly Prison, Eidolon of Rhetoric, Collector Ouphe, Stony Silence
- 3 card defs fixed: Archon of Emeria, Grand Abolisher, Dragonlord Dromoka
- 10 new tests in restrictions.rs; 2144 total passing, 0 clippy warnings
- Commit 9c037c6

**Deferred from PB-18**:
- CantAttackYouUnlessPay combat enforcement (Propaganda/Ghostly Prison) — data registered but interactive mana payment during attack declaration needs new Command variant
- Silence (spell effect, not static), Myrel (needs attack trigger), Hope of Ghirapur (targeted sacrifice) — separate DSL gaps
- Prior carried-forward deferrals from PB-13/14/17 still apply

**Next**:
1. **PB-19 (Mass destroy / board wipes)**: 12 cards — Effect::DestroyAll + Effect::ExileAll
2. Continue through PB-20 to PB-21 per `docs/primitive-card-plan.md`

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-16 — W6: PB-17 Library search filters
- max_cmc, min_cmc, has_card_types on TargetFilter; 9 card fixes; 8 tests; 2134 total; commit 894504e

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

