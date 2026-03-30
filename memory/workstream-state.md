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
| W6: Primitive + Card Authoring | BF-S3: backfill fixes | ACTIVE | 2026-03-30 | **ALL PBs COMPLETE (PB-0–37)**; BF-S1+S2 done (25 fixes total) |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-30
**Workstream**: W6 (BF-S2)
**Task**: Backfill fix sprint session 2

**Completed**:
- **BF-S2 DONE**: 8 card def fixes in 1 commit (55c43be)
  - 4 target tightening: fell_the_profane, kabira_takedown, torch_the_tower, dragonlord_silumgar — TargetCreature/TargetPermanent → TargetPermanentWithFilter{creature, planeswalker}
  - 1 colorless filter: forerunner_of_slaughter — TargetCreatureWithFilter{exclude_colors: all 5}
  - 1 stale TODO: cryptic_coat — statics already implemented
  - 2 new abilities: goblin_sharpshooter (death trigger + untap), crown_of_skemfar (graveyard return)
- Lower yield than BF-S1 (8 vs 17) — confirms alphabetical ranges don't map to fixable patterns
- BF-S2 range (clavileno→hallowed_spiritkeeper) has ~158 TODO files but most are genuine DSL gaps

**Next**:
1. **BF-S3 through BF-S9**: Continue backfill. Strongly recommend pattern-based approach across ALL defs rather than alphabetical ranges. Key fixable patterns remaining: creature-or-PW targeting (a few more outside BF-S2 range), graveyard activated abilities, simple death triggers.
2. Consider collapsing BF-S3–S9 into fewer pattern-based sessions since per-range yield is low.
3. After backfill: resume A-29+ card authoring.

**Hazards**:
- `activated_ability_cost_reductions` index on channel lands (eiganjo, boseiju etc.) may be off-by-one — index 0 targets mana tap, not channel ability. Needs investigation.
- Most remaining TODOs are genuine DSL gaps: count-based token creation, player choice (M10), replacement effects, ability granting.

**Commit prefix**: `W6-cards:`

## Handoff History

### 2026-03-30 — W6: BF-S1
- BF-S1: 17 card def fixes (e2f07e1, 88f0580). ~30-40% false positive rate confirmed. Key patterns: death triggers, Cost::Sacrifice, UntapPermanent, simple activated, bounce.

### 2026-03-30 — W6: BF-1 + BF-2
- BF-1 re-triage: 1451 defs, 773 clean (53%), 678 with TODOs. BF-2 gap closure committed. 9 backfill sessions planned.

### 2026-03-29 — W6: PB-37
- PB-37: G-26 residual complex activated. Condition::WasCast, EffectDuration::UntilYourNextTurn(PlayerId), once_per_turn, was_cast+abilities_activated_this_turn, temporary_protection_qualities, expire_until_next_turn_effects(). 7 card defs fixed. 9 new tests. 2437 tests.

### 2026-03-29 — W6: PB-36
- PB-36: G-31 evasion/protection extensions. BlockingExceptionFilter, CantBlock/CantBeBlockedExceptBy, GrantPlayerProtection, combat enforcement. 16 card defs fixed. 2428 tests.

### 2026-03-28 — W6: PB-35
- PB-35: G-27/G-29/G-30 modal triggers + graveyard abilities. 14 card defs fixed. 2419 tests.
