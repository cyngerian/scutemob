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
| W6: Primitive + Card Authoring | — | available | — | **ALL PBs COMPLETE (PB-0–37)**; BF complete; **Wave A+B COMPLETE** (91+130 = 221 new card defs); Wave A engine review CLEAN |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-01
**Workstream**: W6: Primitive + Card Authoring
**Task**: A-42 authoring batch 2 (Wave B continuation)

**Completed**:
- A-42 full triage: 131 cards → 109 need authoring, 18 already exist, 4 blocked (S69)
- A-42 classification: ~50 SAFE, ~12 PARTIAL, ~47 BLOCKED
- A-42 batch 2 authoring: 60 new card defs authored (38 SAFE + 22 PARTIAL with TODOs)
- Key cards: Blood Moon, Magus of the Moon, Urborg, Yavimaya, Ashaya, Lotus Cobra, Evolution Sage, Angrath's Marauders, Dig Through Time, Food Chain, Warstorm Surge, Natural Order
- Total A-42: 77/131 authored (17 prior + 60 this session)
- 2437 tests, 0 clippy warnings, 1693 total card defs

**Next**:
1. **A-42 remaining ~54 cards are mostly BLOCKED** on engine primitives — minimal additional authoring possible without new PBs
2. **Wave B engine review checkpoint** — pure card authoring this session, should be quick
3. **Wave C**: A-30, A-36, A-40, A-41 — blocked on significant engine work
4. **Consider new PBs** for highest-impact gaps: BounceAll/MoveZoneAll (4 cards), chosen-type (6+ cards), mana-doubling (3 cards), ExcludeSubtype filter (3 cards), extra-turn (3 cards)

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward)
- A-38 BLOCKED cards (48): need play-from-top, pitch-alt-cost, copy-target-spell, grant-flash, mana-doubling, extra-turn, gain-control
- A-42 BLOCKED cards (~54): need extra-turn, mana-doubling, chosen-type-anthem, mass-reanimate, BounceAll, damage-tripling, wheel, domain-count, etc.

**Commit prefix**: `W6-cards:`

## Handoff History

### 2026-03-31 — W6: Wave B A-38+A-42 partial
- Wave A engine review CLEAN. A-38: 53 new defs. A-42 batch 1: 17 new defs. 70 total.

### 2026-03-31 — W6: Wave A complete
- Wave A: 91 new card defs (A-29, A-32–A-35, A-39). 2437 tests.

### 2026-03-30 — W6: A-29 S131
- 3 engine primitives + 10 card defs fixed.

### 2026-03-30 — W6: BF-S3/S4 + A-29 S1
- BF-S3/S4: 7 card def fixes. A-29 S1: 21 card defs authored.

### 2026-03-30 — W6: BF-1 + BF-2
- BF-1 re-triage: 1451 defs, 773 clean (53%), 678 with TODOs. BF-2 gap closure.

### 2026-03-29 — W6: PB-37
- PB-37: G-26 residual complex activated. 7 card defs fixed. 9 new tests.
