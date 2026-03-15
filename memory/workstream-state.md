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
| W6: Primitive + Card Authoring | PB-14: Planeswalker support | ACTIVE | 2026-03-15 | Loyalty abilities, 0-loyalty SBA, damage redirect. 31+ cards blocked. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-15
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-13 — Specialized mechanics (part 3)

**Completed**:
- 13c: `Condition::HasCitysBlessing` variant added to `card_definition.rs`, evaluated in `effects/mod.rs`, hashed in `hash.rs` (disc 29). Arch of Orazca fixed: Ascend keyword + conditional draw ability (gated on city's blessing). Twilight Prophet: Ascend keyword added (upkeep trigger remains DSL gap — needs reveal-top + mana-value-based DrainLife). 1 new test in `ascend.rs`.
- 13e: Dredge — already fully wired (Golgari Grave-Troll has `KeywordAbility::Dredge(6)`, tests in `dredge.rs`). DSL gap audit was stale.
- 13f: Buyback — already fully wired (Searing Touch has `AbilityDefinition::Buyback`, tests in `buyback.rs`). DSL gap audit was stale.
- 13l: Flicker — no cards in current universe blocked on this primitive. Deferred until cards are authored.
- 13n: Living Weapon — already done (Batterskull uses `KeywordAbility::LivingWeapon`).
- 13h: Coin flip / d20 — deferred to post-alpha (3 cards affected, architectural conflict with deterministic replay).
- 2096 tests, 0 clippy warnings, workspace builds clean.

**Deferred from PB-13 (total)**:
- Equipment auto-attach (13d): needs entering-object variable refs in triggers — 2 cards
- Timing restriction (13i): needs ContinuousRestriction framework → PB-18 (Stax)
- Clone/copy ETB (13j): no actionable cards in universe
- Adventure (13m): needs dedicated casting subsystem — 1-2 cards
- Coin flip / d20 (13h): needs randomness framework — 3 cards (post-alpha)
- Flicker (13l): no blocked cards — build when needed
- PB-12 leftovers (Neriv, Lightning Army of One, Mossborn Hydra): need landfall trigger + scoped damage doubling

**Next**:
1. **PB-14 (Planeswalker support)**: loyalty abilities, 0-loyalty SBA, damage redirect. 31+ cards blocked.
2. Continue through PB-15 to PB-21 per `docs/primitive-card-plan.md`

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

### 2026-03-14 — W6: PB-10 graveyard targeting (10 cards)
- 2 TargetRequirement variants + has_subtypes filter + 10 card def fixes + 10 tests; commit 0b6b24d; 2054 tests
