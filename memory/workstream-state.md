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
| W6: Primitive + Card Authoring | PB-13 part 2: Channel, Land animation, Equipment auto-attach, Timing restriction, Clone/copy ETB, Adventure | ACTIVE | 2026-03-15 | **TOP PRIORITY**. PB-13 part 1 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-15
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-13 — Specialized mechanics (part 1 of ~3)

**Completed**:
- KeywordAbility::HexproofPlayer (disc 159, CR 702.11d): player targeting check in casting.rs + abilities.rs via layer-resolved chars; crystal_barricade card def fixed; 3 tests
- Monarch (CR 724): monarch: Option<PlayerId> on GameState, Effect::BecomeMonarch, GameEvent::PlayerBecameMonarch (disc 119), EOT draw in end_step_actions(), combat damage steal (check_monarch_steal_from_combat_damage), transfer_monarch_on_player_leave in sba.rs (3 sites); 6 tests
- Verified Ascend (CR 702.131), Dredge (CR 702.52), Buyback (CR 702.27), Living Weapon (CR 702.92) already fully implemented — cleaned stale TODOs
- Verified Flicker expressible with existing Sequence([ExileObject, MoveZone]) + target_remaps
- Coin flip deferred to interactive play (M10) — Mana Crypt has deterministic worst-case fallback
- Commit 5a4530c; 2088 tests, 0 clippy warnings

**Next**:
1. **PB-13 part 2**: Channel (5 NEO lands), Land animation (5+ cards), Equipment auto-attach (2 cards), Timing restriction (3 cards), Clone/copy ETB (1 card), Adventure (1 card)
2. 3 PB-12 leftover cards: Neriv, Lightning Army of One, Mossborn Hydra
3. Then PB-14 through PB-21 per `docs/primitive-card-plan.md`

**Hazards**: CLAUDE.md has uncommitted edits (minor). Clean working tree otherwise.

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-15 — W6: PB-13 part 1 (player hexproof + monarch)
- HexproofPlayer (disc 159) + Monarch (CR 724) + stale TODO cleanup + 9 tests; commit 5a4530c; 2088 tests

### 2026-03-15 — W6: PB-12 complex replacement effects (8 cards)
- 7 triggers + 8 modifications + 1 TriggerDoublerFilter + 6 helpers + 8 card fixes + 14 tests; commit 20d8981; 2079 tests

### 2026-03-15 — W6: PB-11 mana restrictions + ETB choice (10 cards)
- ManaRestriction enum + restricted mana pool + chosen_creature_type + 10 card fixes + 11 tests; commit 382ae7d; 2065 tests

### 2026-03-14 — W6: PB-10 graveyard targeting (10 cards)
- 2 TargetRequirement variants + has_subtypes filter + 10 card def fixes + 10 tests; commit 0b6b24d; 2054 tests

### 2026-03-14 — W6: PB-9.5 architecture cleanup
- check_and_flush_triggers() helper extracted (26 copies → 1), 5 test CardDefinition defaults fixed; commits e7b13a1 + 2c8f502; 2044 tests
