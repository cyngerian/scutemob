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
| W6: Primitive + Card Authoring | — | available | — | **ALL PBs COMPLETE (PB-0–37)**; BF-1+BF-2 done; BF-S1 done (17 fixes) |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-30
**Workstream**: W6 (BF-S1)
**Task**: Backfill fix sprint session 1

**Completed**:
- **BF-S1 DONE**: 17 card def fixes across 2 commits (e2f07e1, 88f0580)
  - 10 cards with new abilities: ayara, blight_mound, butcher_of_malakir, dictate_of_erebos, grave_pact, blasting_station, awakening_zone, brallin_skyshark_rider, bartolome_del_presidio, chulane_teller_of_tales, haywire_mite
  - 2 cards with stale TODO cleanup: vampiric_rites, elenda_the_dusk_rose, consecrated_sphinx
  - 1 token fix: pawn_of_ulamog (added sac-for-mana ManaAbility to Eldrazi Spawn)
  - 2 partial fixes: cruel_celebrant (creature deaths, not PW), dreadhound (ETB mill + death trigger, not mill-to-GY)
- ~30-40% false positive rate **confirmed** — most BF-S1 candidates have genuine DSL gaps
- Key fixable patterns: death triggers with controller/nontoken filters, Cost::Sacrifice with TargetFilter, UntapPermanent on triggers, simple activated abilities, bounce (MoveZone to Hand)

**Next**:
1. **BF-S2 through BF-S9**: Continue backfill fix sprint. Same approach: verify each TODO before fixing, skip genuine gaps. Most productive pattern: scan ALL defs (not just alphabetical range) for known-fixable patterns.
2. Consider: the session-based alphabetical ranges (BF-S2: clavileno→hallowed_spiritkeeper) don't map well to fixable patterns. Better approach: search by pattern type across all defs.
3. After backfill: resume A-29+ card authoring.

**Hazards**:
- ForEach EachOpponent + SacrificePermanents uses `PlayerTarget::DeclaredTarget { index: 0 }` — verify engine resolves this correctly for each iteration target
- Blight Mound Pest token lacks nested death-trigger lifegain (TokenSpec can't carry triggered abilities)
- "You may" optional effects approximated as mandatory throughout

**Commit prefix**: `W6-cards:`

## Handoff History

### 2026-03-30 — W6: BF-1 + BF-2
- BF-1 re-triage: 1451 defs, 773 clean (53%), 678 with TODOs. BF-2 gap closure committed. 9 backfill sessions planned.

### 2026-03-29 — W6: PB-37
- PB-37: G-26 residual complex activated. Condition::WasCast, EffectDuration::UntilYourNextTurn(PlayerId), once_per_turn on Activated/ActivatedAbility, was_cast+abilities_activated_this_turn on GameObject, temporary_protection_qualities on PlayerState, expire_until_next_turn_effects(). 7 card defs fixed. 9 new tests. 2437 tests, 0 clippy.

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
