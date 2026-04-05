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
| W6: Primitive + Card Authoring | PB-C: Extra turns | ACTIVE | 2026-04-05 | **ALL PBs COMPLETE (PB-0–37+G+K+D)**; BF complete; **Wave A+B COMPLETE**; Wave B engine review CLEAN |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-05
**Workstream**: W6: Primitive + Card Authoring
**Task**: Wave B.5 — PB-C extra turns

**Completed**:
- PB-C: Effect::ExtraTurn { player, count } + self_exile/self_shuffle_on_resolution flags + GiftType::ExtraTurn wiring. 3 new cards (Nexus of Fate, Temporal Trespass, Temporal Mastery) + 1 fix (Teferi -10 loyalty). 1M 2L fixed. 7 tests.
- 2481 tests, 0 clippy warnings, ~1730 total card defs

**Next** (agreed priority order):
1. **PB-F** (Damage multiplier, 3 cards, MEDIUM)
2. **PB-I** (Grant flash, 4 cards, MEDIUM)
3. **PB-H** (Mass reanimate, 5 cards, MEDIUM)
4. **PB-L** (Reveal/X effects, 7 cards, MEDIUM)
5. HIGH batches last (PB-A/B/E/J/M)

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands may be off-by-one (carried forward)
- Cavern of Souls "can't be countered" deferred (needs CounterRestriction primitive)
- Nexus of Fate "from anywhere" graveyard replacement only covers resolution case (needs full non-permanent replacement infrastructure)
- A-42 BLOCKED categories remaining: mana-doubling, mass-reanimate, damage-tripling, wheel, domain-count

**Commit prefix**: `W6-prim:` (engine), `W6-cards:` (card defs)

## Handoff History

### 2026-04-02 — W6: PB-N + PB-G
- PB-N: 19 misc card defs. PB-G: BounceAll + TargetFilter extensions + 4 cards. 2445 tests.

### 2026-04-01 — W6: A-42 batch 2
- A-42 batch 2: 60 new card defs. Total A-42: 77/131. 1693 total defs. 2437 tests.

### 2026-03-31 — W6: Wave B A-38+A-42 partial
- Wave A engine review CLEAN. A-38: 53 new defs. A-42 batch 1: 17 new defs. 70 total.

### 2026-03-31 — W6: Wave A complete
- Wave A: 91 new card defs (A-29, A-32–A-35, A-39). 2437 tests.

### 2026-03-30 — W6: BF-S3/S4 + A-29 S1
- BF-S3/S4: 7 card def fixes. A-29 S1: 21 card defs authored.
