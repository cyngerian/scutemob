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
| W6: Primitive + Card Authoring | — | available | — | **A-20 through A-23 DONE** (121 new cards). Next: A-24 attack-trigger. ~1391 card defs. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-23
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 2 authoring — A-20 through A-23

**Completed**:
- A-20 pump-buff: 27 cards authored, reviewed (4 batches), 4 HIGH fixed (Ezuri self-target, Elesh Norn/Eldrazi Monument/Mikaeus stripped to empty per W5)
- A-21 counters-plus: 49 cards authored (landfall, tribal counters, sacrifice outlets, mutate)
- A-22 equipment: 11 cards authored (Swords cycle, Bone Saw, Kite Shield, etc.)
- A-23 death-trigger: 34 cards authored (Blood Artist, Grave Pact, Kokusho, etc.)
- Commits: e5b0436, ec08405
- Total: ~1391 card def files (+121 this session)
- All 2281 tests passing, workspace builds clean

**Next**:
1. **A-24 attack-trigger** (4 sessions, 33 cards) — next authoring group
2. Continue through A-25 activated-tap, A-26 activated-sacrifice, etc.
3. A-19 S40-S43 review still pending (48 cards from 2026-03-22 session)

**Hazards**:
- `WheneverCreatureDies` has no controller filter — fires on ALL creatures dying. Cards needing "creature you control dies" must use TODO, not the unfiltered trigger (W5 wrong game state)
- `DrainLife { amount }` has no `target` field — it drains all opponents automatically. Don't add `target:` field.
- `AbilityDefinition::Equip` does not exist — use `Keyword(KeywordAbility::Equip)` only
- `AbilityDefinition::Ninjutsu` has no `is_commander` field
- `Cost::RemoveCounter` does not exist — cards needing counter removal as cost use TODO
- `EffectFilter::CreaturesOpponentsControl` does not exist — buff-without-debuff cards must be stripped per W5

**Commit prefix used**: `W6-cards:`

## Handoff History

### 2026-03-23 — W6: Phase 2 authoring A-19 token-create S44-S52
- A-19 token-create S44-S52: 96 new cards. Reviewed, 3H fixed.
- Commits: 5d967ca, 83c1302. 2281 tests.

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
