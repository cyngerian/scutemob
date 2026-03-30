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
| W6: Primitive + Card Authoring | PB-37: Residual complex activated | available | — | PB-36 complete |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-29
**Workstream**: W6 (PB-36)
**Task**: PB-36 Evasion/protection extensions (G-31)

**Completed**:
- **PB-36 DONE**: G-31 Evasion/protection extensions. Engine: BlockingExceptionFilter enum (HasKeyword(Box<KW>)/HasAnyKeyword), CantBlock(160)/CantBeBlockedExceptBy(161) in KeywordAbility, Effect::GrantPlayerProtection(73), combat.rs per-blocker CantBlock check + per-attacker CantBeBlockedExceptBy check + provoke impossibility for CantBlock+CantBeBlockedExceptBy. All exhaustive matches updated (hash.rs, view_model.rs, lib.rs, mod.rs, helpers.rs). 16 card defs fixed. 9 new tests. Review: 3M fixed (hash vec length prefixes, provoke+evasion guard). 6L documented. 2428 tests, 0 clippy. Commits: 2eeccd5 (implement), 02499fa (fixes), 186ee22 (close).
- Condition::WasCast deferred (The One Ring fires unconditionally, documented TODO).
- Duration cleanup ("until your next turn" for GrantPlayerProtection) deferred to PB-37/future.

**Next**:
1. PB-37: Residual complex activated (G-30 PW abilities + remaining complex effects)
2. Backfill sweeps accumulating (~500+ cards across PB-23–36)

**Hazards**:
- Backfill sweeps accumulating (~500+ cards across PB-23–36)

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-29 — W6: PB-36
- PB-36: G-31 evasion/protection extensions. BlockingExceptionFilter, CantBlock(160)/CantBeBlockedExceptBy(161), GrantPlayerProtection(73), combat enforcement. 16 card defs fixed. 9 new tests. Review: 3M fixed, 6L documented. 2428 tests. Commits: 2eeccd5, 02499fa, 186ee22.

### 2026-03-28 — W6: PB-35
- PB-35: G-27/G-29/G-30 modal triggers + graveyard abilities. 14 card defs fixed, 1H 3M fixed. 2419 tests. Commits: 727a0f5, ed895e7.

### 2026-03-27 — W6: PB-34
- PB-34: G-23/G-24/G-25 mana production. 7 filter lands fixed, AddManaScaled orphan fix. Review clean (2L). 2408 tests. Commit: 71ad3ce.


### 2026-03-27 — W6: PB-33
- PB-33: G-22/G-28 copy/clone + exile/flicker timing. 15 card defs fixed, 2H 1M fixed. 2403 tests. Commits: 3bf6d25, f08c0fc.

### 2026-03-26 — W6: PB-32
- PB-32: G-18/G-19/G-20/G-21 static/effect primitives, 22 card defs fixed, 2M fixed. 2396 tests. Commits: 8401dca, 1502d1c.

### 2026-03-26 — W6: PB-31
- PB-31: G-16 RemoveCounter + G-17 SpellAdditionalCost, 18 card defs fixed, 2M fixed. 2383 tests. Commits: b9f8efa, aeb87d5.

### 2026-03-25 — W6: PB-30
- PB-30: G-8 Combat damage triggers, 27 card defs fixed, 5H 4M fixed. 2371 tests. Commits: b5577c7, b8c8dc6.

### 2026-03-25 — W6: PB-28 + PB-29
- PB-28: G-6 CDA, 9 card defs fixed, 1M fixed. 2353 tests. Commits: ee56134, 3882c1b.
- PB-29: G-7 Cost reduction statics, 13 card defs fixed, 1H fixed. 2363 tests. Commits: e562ec0, bf6e992.
- A-19 token-create S44-S52: 96 new cards. Reviewed, 3H fixed.
- Commits: 5d967ca, 83c1302. 2281 tests.
