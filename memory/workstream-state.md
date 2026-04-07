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
| W6: Primitive + Card Authoring | PB-H: Mass reanimate | ACTIVE | 2026-04-06 | **ALL PBs COMPLETE (PB-0–37+G+K+D+C+F+I+H)**; BF complete; **Wave A+B COMPLETE**; Wave B engine review CLEAN |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-06
**Workstream**: W6: Primitive + Card Authoring
**Task**: Wave B.5 — PB-C + PB-F + PB-I engine batches

**Completed**:
- PB-C: Effect::ExtraTurn + self_exile/self_shuffle flags + GiftType wiring. 3 new + 1 fix. 1M 2L fixed. 7 tests.
- PB-F: TripleDamage, DamageTargetFilter extensions, entered_turn, RegisterReplacementEffect. 1 new + 2 fixes. Clean. 10 tests.
- PB-I: FlashGrant/FlashGrantFilter, Effect::GrantFlash, StaticFlashGrant, OpponentsCanOnlyCastAtSorcerySpeed. 1 new + 3 fixes. 3M fixed. 13 tests.
- 2504 tests, 0 clippy warnings, ~1732 total card defs

**Next** (agreed priority order):
1. **PB-H** (Mass reanimate, 5 cards, MEDIUM)
2. **PB-L** (Reveal/X effects, 7 cards, MEDIUM)
3. HIGH batches last (PB-A/B/E/J/M)

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward)
- Cavern of Souls "can't be countered" deferred (needs CounterRestriction primitive)
- Nexus of Fate "from anywhere" graveyard replacement only covers resolution case (needs full non-permanent replacement infrastructure)
- A-42 BLOCKED categories remaining: mana-doubling, mass-reanimate, wheel, domain-count (damage-tripling resolved by PB-F, grant-flash resolved by PB-I)

**Commit prefix**: `W6-prim:` (engine), `W6-cards:` (card defs)

## Handoff History

### 2026-04-04 — W6: PB-K + PB-D
- PB-K: land drops, Case mechanic. PB-D: chosen creature type, 8 fixes. 2474 tests.

### 2026-04-02 — W6: PB-N + PB-G
- PB-N: 19 misc card defs. PB-G: BounceAll + TargetFilter extensions + 4 cards. 2445 tests.

### 2026-04-01 — W6: A-42 batch 2
- A-42 batch 2: 60 new card defs. Total A-42: 77/131. 1693 total defs. 2437 tests.

### 2026-03-31 — W6: Wave B A-38+A-42 partial
- Wave A engine review CLEAN. A-38: 53 new defs. A-42 batch 1: 17 new defs. 70 total.

### 2026-03-31 — W6: Wave A complete
- Wave A: 91 new card defs (A-29, A-32–A-35, A-39). 2437 tests.
