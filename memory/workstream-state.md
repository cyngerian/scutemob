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
| W6: Primitive + Card Authoring | PB-22 S2: Coin flip / d20 | ACTIVE | 2026-03-21 | S1 done (activation_condition). S2-S7 remain. Plan: `memory/primitives/pb-22-session-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-20
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-22 S1 — deferred cleanup (activation condition, mana cost filter, sorcery-speed)

**Completed**:
- Created PB-22 session plan (`memory/primitives/pb-22-session-plan.md`) — 13 deferred items, 7 sessions, ~67 cards unblocked
- Assessed all 13 deferred items: 12 implementable now, 1 genuinely M10-blocked (Tiamat)
- PB-22 S1: Added `activation_condition: Option<Condition>` to `AbilityDefinition::Activated` + runtime `ActivatedAbility`
- Enforced in `abilities.rs` via `check_condition()`, propagated in `enrich_spec_from_def`, hashed
- 305 files changed (277 card defs + tests + engine), 3 new tests, 2236 total
- Discovered sorcery-speed timing and mana cost filter were already done — S1 only needed activation_condition

**Next**:
1. PB-22 S2: Coin flip / d20 (Effect::CoinFlip, Effect::RollDice — 3 card defs, ~5 cards)
2. PB-22 S3-S7: reveal-route, flicker, tapped-and-attacking tokens, copy/clone, emblems, adventure
3. After PB-22: Phase 2 card authoring (~1,025 remaining cards)

**Hazards**:
- None — clean commit, all tests passing, no WIP

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-20 — W3: LOW Remediation (sprint complete)
- W3 LOW sprint S1-S6 + bookkeeping: 83→29 LOW open, 119 closed, 2233 tests
- All reviews clean except S6 HIGH (hash gap, fixed immediately)
- W3 effectively complete — remaining 29 LOWs permanently deferred or DSL-blocked

### 2026-03-19 — W6: PB-21 Fight & Bite
- PB-21 complete: Effect::Fight + Effect::Bite, 4 card defs fixed, 14 new tests, 2206 total
- ALL 22 PRIMITIVE BATCHES COMPLETE (PB-0 through PB-21)
- Next: Phase 2 card authoring (~1,025 remaining cards)

### 2026-03-19 — Exploratory: Rules audit + W3-LC plan
- Audited 4 abilities (Devoid, Flanking, Fabricate, Suspend) against CR rules
- Found 1 MEDIUM bug: Flanking reads base characteristics instead of layer-resolved (Humility breaks it)
- Identified 69 base-characteristic reads across 12 engine files that may have same issue
- Created `memory/w3-layer-audit.md` — 4-session audit plan (W3-LC S1-S4)
- Updated `docs/workstream-coordination.md` Phase 3 with W3-LC checkboxes
- **W3 and W6 are independent** — can run in parallel with no conflicts
- No code changes, no commit needed

### 2026-03-19 — W6: PB-18 review (FINAL)
- PB-18 retroactive review (20/20): Stax / restrictions, 10 cards — 2H 4M 1L fixed; 2M deferred (DSL gap)
- Engine: attack tax (combat.rs), mana ability restrictions (mana.rs), zone scope (abilities.rs), simulator filter
- Commit: ca13879
- **ALL RETROACTIVE REVIEWS COMPLETE**



