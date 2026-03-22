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
| W6: Primitive + Card Authoring | — | available | — | **A-11/A-12/A-13 DONE** (Tier 2 removal groups). Next: A-14 removal-damage-each. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 2 authoring — A-11 through A-13 (Tier 2 removal groups)

**Completed**:
- A-11 removal-destroy: 52 new cards across 5 sessions + DSL extension (`DestroyPermanent.cant_be_regenerated: bool` — CR 701.19c). 11 review batches, HIGH findings fixed (Pyroblast targeting, Nature's Claim controller, Staff untap, Elspeth Storm Slayer replacement trigger). 66 existing files updated for new field.
- A-12 removal-exile: 13 new cards across 2 sessions (1 blocked: Touch the Spirit Realm). 3 review batches, 1 HIGH fixed (Shiko exile-only ETB → TODO).
- A-13 removal-damage-target: 23 new cards across 3 sessions. Damage patterns (DealDamage, modal, planeswalker, equipment, sacrifice-as-cost, tribal triggers).
- Total: 88 new card defs, 1 DSL extension, ~968 total card files
- Commits: 52be340 (A-11), 18ca67e (A-12), 68e2b9f + 064ccbb (A-13)
- All tests passing, 0 clippy, workspace builds clean

**Next**:
1. **A-14 removal-damage-each** (17 cards, 2 sessions) — board damage patterns
2. **A-15 removal-bounce** (10 cards, 2 sessions) — return-to-hand patterns
3. **A-16 removal-minus** (4 cards, 1 session) — -X/-X effects
4. **A-17 counter** (16 cards, 3 sessions) — counterspells
5. Then A-18 draw (161 cards) — largest group, will need many sessions

**Hazards**:
- bulk-card-author agents stall ~40% of the time on complex cards (planeswalkers, multi-ability creatures). Write these directly — faster and more reliable.
- Session 141 (A-13) agent stalled completely; had to write all 12 cards manually.
- `CreateToken` still lacks player field — systemic DSL gap for "its controller creates a token" pattern (Pongify, Beast Within, Stroke of Midnight, Resculpt, Rapid Hybridization)
- `WheneverYouDrawCard` trigger event not in DSL — blocks Niv-Mizzet, Teferi emblem
- `WheneverACreatureDies` with subtype filter not in DSL — blocks tribal death triggers
- `Cost::TapCreatureYouControl` not in DSL — blocks Kyren Negotiations, Convoke-style cards
- Modal activated abilities not supported — blocks Goblin Cratermaker
- `ProtectionQuality` must be imported separately (`use crate::state::types::ProtectionQuality;`) — not in helpers.rs

**Commit prefix used**: `W6-cards:`

## Handoff History

### 2026-03-22 — W6: Phase 2 authoring A-05 through A-10 (Tier 1 completion)
- 36 new cards (A-05 through A-10). All reviewed, 8H+9M+10L findings — all HIGH fixed.
- Tier 1 COMPLETE: A-01 through A-10 + A-31/A-37 pre-existing = 12 groups done.
- Commits: d6634d6, 2097db8, 4d0999e, 034385a. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-01 through A-04
- 52 new card defs authored (16 mana-creature + 33 mana-artifact + 3 mana-other).
- 5 groups verified pre-existing (body-only, land-etb-tapped, combat-keyword, mana-land).
- Commits: 10f81c0, 0903f6b. 2281 tests.

### 2026-03-22 — W6: Phase 1 close (F-4 sweep + F-5/F-6/F-7)
- 3 stale TODOs cleaned, 8 cards verified (7/8 PASS), build clean, committed 3bfe888.
- Phase 1 Fix COMPLETE. Next: Phase 2 authoring A-01.

### 2026-03-22 — W6: F-4 session 6 (11 now-expressible cards)
- 3 lands (ETB tapped + mana), 1 conditional mana, 1 conditional ETB, 1 equipment bounce, 1 creature ETB+mana, 1 creature ETB, 3 keyword additions. Review: 1M+1L fixed.
- 2281 tests.

### 2026-03-22 — W6: F-4 session 5 (12 land mana abilities)
- 8 pathway lands (mana tap) + 4 verge lands (conditional mana). Review: 2H fixed (stale oracle text).
- Commit: 8c2aded. 2281 tests.
