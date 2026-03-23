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
| W6: Primitive + Card Authoring | — | available | — | **A-19 token-create 13/14 DONE** (S53 blocked). Next: A-20 pump-buff. ~1269 card defs. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-23
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 2 authoring — A-19 token-create S44-S52

**Completed**:
- A-19 token-create sessions S44-S52: 96 new cards (9 sessions)
- Agents authored bulk, hand-wrote 15 cards where agents ran out of budget
- Reviewed 20 representative cards across 4 batches: 3H 1M findings
- 3 HIGH fixed: Hero of Bladehold (BattleCry keyword), Grave Titan (attack trigger), Bitterblossom (Kindred type)
- 1 MEDIUM documented: Sengir Autocrat LTB gap (WhenLeavesBattlefield not in DSL)
- Commits: 5d967ca, 83c1302
- A-19 total: 144/146 cards (S40-S52), S53 blocked (9 cards)
- Total: ~1269 card def files
- All 2281 tests passing, workspace builds clean

**Next**:
1. **A-20 pump-buff** (3 sessions, 26 cards) — next authoring group
2. Review A-19 S40-S43 still pending (48 cards from prior session not yet reviewed)
3. Continue through A-21 counters-plus, A-22 equipment, etc.

**Hazards**:
- Bulk-card-author frequently misses 3-6 cards per session — always verify file count
- Common DSL misses by agents: `Effect::NoEffect` (use `Nothing`), `ActivationCost` (use `Cost`), missing `modes`/`cant_be_countered` on `Spell`, `loyalty:` (use `starting_loyalty:`), `WheneverYouAttack` (doesn't exist)
- `AbilityDefinition::Spell` needs ALL 4 fields: `effect, targets, modes, cant_be_countered`
- S53 blocked (9 cards including Scute Swarm, Revel in Riches, Titania)

**Commit prefix used**: `W6-cards:`

## Handoff History

### 2026-03-22 — W6: Phase 2 authoring A-18 draw + A-19 start
- A-18 S20-S25 (43 new), A-19 S40-S43 (48 new). 91 total. 4H fixed.
- Commits: fc27279, 047532f, f43d88b, 0de9b8c. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-18 draw S10-S19
- 119 new cards (10 sessions of 16 complete)
- Commits: 4a15b5e, 9bf3870, 6ecdb68. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-14 through A-17 (Tier 2 cont'd)
- 45 new cards (damage-each, bounce, minus, counter). All HIGH fixed.
- Commit: 6ddc832. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-11 through A-13 (Tier 2 start)
- 88 new cards (A-11 destroy + A-12 exile + A-13 damage-target). DSL ext: DestroyPermanent.cant_be_regenerated.
- Commits: 52be340, 18ca67e, 68e2b9f, 064ccbb. 2281 tests.

### 2026-03-22 — W6: Phase 2 authoring A-05 through A-10 (Tier 1 completion)
- 36 new cards (A-05 through A-10). All reviewed, 8H+9M+10L findings — all HIGH fixed.
