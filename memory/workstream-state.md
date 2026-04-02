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
| W6: Primitive + Card Authoring | — | available | — | **ALL PBs COMPLETE (PB-0–37+G)**; BF complete; **Wave A+B COMPLETE** + 23 new defs; Wave B engine review CLEAN |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-02
**Workstream**: W6: Primitive + Card Authoring
**Task**: Wave B.5 — PB-N re-triage + PB-G engine batch

**Completed**:
- Wave B engine review checkpoint: CLEAN (zero engine changes in Wave B, all 130 files were card defs)
- Categorized 101 blocked A-38/A-42 cards into 14 primitive batch categories (PB-A through PB-N)
- PB-N re-triage: pulled 20 authorable cards from 34-card misc bucket (9 clean, 11 partial)
- PB-N authoring: 19 new card defs (Steelshaper's Gift already existed). Commit `189465c`
- PB-G engine batch: `Effect::BounceAll { filter, max_toughness_amount }` + 3 new TargetFilter fields (`max_toughness`, `exclude_subtypes`, `is_attacking`). 4 new cards (Aetherize, Whelming Wave, Scourge of Fleets, Filter Out). Fixed Crux of Fate + Recruiter of the Guard. Box<TargetFilter> for clippy large_enum_variant. 8 new tests. Commit `c0e32b7`
- Updated ops plan with Wave B.5 engine batch priority order
- 2445 tests, 0 clippy warnings, ~1716 total card defs

**Next** (agreed priority order):
1. **PB-K** (Additional lands, 3 cards, LOW) — Burgeoning, Dryad of the Ilysian Grove, Case of the Locked Hothouse
2. **PB-D** (Chosen creature type, 12 cards, MEDIUM — highest unblock count)
3. **PB-C** (Extra turns, 4 cards, MEDIUM)
4. **PB-F** (Damage multiplier, 3 cards, MEDIUM)
5. **PB-I** (Grant flash, 4 cards, MEDIUM)
6. **PB-H** (Mass reanimate, 5 cards, MEDIUM)
7. HIGH batches last (PB-A/B/E/J/M)

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward)
- A-38 remaining BLOCKED (~48): play-from-top, pitch-alt-cost, copy-target-spell, grant-flash, mana-doubling, extra-turn
- A-42 remaining BLOCKED (~35 after PB-N+PB-G): extra-turn, mana-doubling, chosen-type-anthem, mass-reanimate, damage-tripling, wheel, domain-count

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
