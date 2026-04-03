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
| W6: Primitive + Card Authoring | Wave B.5: PB-K (additional land drops) | ACTIVE | 2026-04-02 | **ALL PBs COMPLETE (PB-0–37+G)**; BF complete; **Wave A+B COMPLETE** + 23 new defs; Wave B engine review CLEAN |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-02
**Workstream**: W6: Primitive + Card Authoring
**Task**: Wave B.5 — PB-K additional land drops + Case mechanic

**Completed**:
- PB-K engine batch: WheneverOpponentPlaysLand trigger, PutLandFromHandOntoBattlefield effect, LandsYouControl filter, Case solve mechanic (Designations::SOLVED, Condition::SourceIsSolved, Effect::SolveCase, Condition::And)
- 3 new card defs: Burgeoning, Dryad of the Ilysian Grove, Case of the Locked Hothouse
- 5 existing card def fixes: Growth Spiral, Broken Bond, Spelunking, Contaminant Grafter, Chulane
- Review: 1H fixed (Dryad mana cost {2}{G}{G} → {2}{G}), 2 LOW (convention)
- 17 new tests, 2462 total tests, 0 clippy warnings, ~1719 total card defs

**Next** (agreed priority order):
1. **PB-D** (Chosen creature type, 12 cards, MEDIUM — highest unblock count)
2. **PB-C** (Extra turns, 4 cards, MEDIUM)
3. **PB-F** (Damage multiplier, 3 cards, MEDIUM)
4. **PB-I** (Grant flash, 4 cards, MEDIUM)
5. **PB-H** (Mass reanimate, 5 cards, MEDIUM)
6. **PB-L** (Reveal/X effects, 7 cards, MEDIUM)
7. HIGH batches last (PB-A/B/E/J/M)

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward)
- A-38 remaining BLOCKED (~48): play-from-top, pitch-alt-cost, copy-target-spell, grant-flash, mana-doubling, extra-turn
- A-42 remaining BLOCKED (~32 after PB-N+PB-G+PB-K): extra-turn, mana-doubling, chosen-type-anthem, mass-reanimate, damage-tripling, wheel, domain-count

**Commit prefix**: `W6-prim:` (engine), `W6-cards:` (card defs)

## Handoff History

### 2026-04-01 — W6: A-42 batch 2
- A-42 batch 2: 60 new card defs. Total A-42: 77/131. 1693 total defs. 2437 tests.

### 2026-03-31 — W6: Wave B A-38+A-42 partial
- Wave A engine review CLEAN. A-38: 53 new defs. A-42 batch 1: 17 new defs. 70 total.

### 2026-03-31 — W6: Wave A complete
- Wave A: 91 new card defs (A-29, A-32–A-35, A-39). 2437 tests.

### 2026-03-30 — W6: A-29 S131
- 3 engine primitives + 10 card defs fixed.

### 2026-03-30 — W6: BF-S3/S4 + A-29 S1
- BF-S3/S4: 7 card def fixes. A-29 S1: 21 card defs authored.
