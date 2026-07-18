# BF-1: Post-Gap-Closure Re-Triage Report

> **Date**: 2026-03-30
> **Scope**: All 1,451 card def files in `crates/engine/src/cards/defs/`
> **Context**: PB-0 through PB-37 ALL COMPLETE. DSL fully expanded.
> **Previous audit**: 2026-03-13 (718 files, 418 with TODOs, 793 TODO lines)

---

## Executive Summary

| Metric | Previous (2026-03-13) | Current (2026-03-30) | Delta |
|--------|----------------------|---------------------|-------|
| Total card def files | 718 | 1,451 | +733 |
| Clean (no TODOs) | 300 (42%) | 773 (53%) | +473 |
| With TODOs | 418 (58%) | 678 (47%) | +260 |
| Total TODO lines | 793 | 1,070 | +277 |
| Empty abilities (vec![]) | 80 | 120 | +40 |

**Net**: Card count doubled, TODO percentage improved from 58% to 47%, but absolute
count grew. Most new TODOs are in Phase 2 bulk-authored cards (A-18 through A-28).

---

## TODO Classification by Feature Need

| Category | Count | % | Fixable Now? | Notes |
|----------|-------|---|-------------|-------|
| COUNT_SCALING | 170 | 16% | Partial | Many need EffectAmount variants we don't have (PowerOfSacrificed, AttackingCreatureCount) |
| OTHER (misc) | 112 | 10% | Varies | Per-mode targets, color filters, misc |
| ZONE_MOVEMENT | 84 | 8% | Partial | Multi-card reanimation, reveal-route, zone-play |
| GENERIC_DSL_GAP | 83 | 8% | Varies | Catch-all "DSL gap" annotations |
| TAP_UNTAP | 67 | 6% | Partial | Tap-other-creatures cost, untap-all-lands, per-turn count |
| COMBAT | 62 | 6% | Partial | Attack restrictions, equipped/enchanted triggers, blocking |
| ACTIVATED_COMPLEX | 60 | 6% | Partial | Unique activated abilities needing individual wiring |
| TOKEN_COMPLEX | 46 | 4% | Partial | Per-opponent tokens, token replacement, count-based |
| DAMAGE_COMPLEX | 46 | 4% | Partial | DamagedPlayer resolution, damage redirection |
| OPTIONAL_CHOICE | 45 | 4% | No (M10) | "You may" requires player interaction |
| SACRIFICE | 37 | 3% | Partial | PowerOfSacrificedCreature, forced opponent sacrifice |
| SUBTYPE_FILTER | 32 | 3% | Mostly yes | Many just need has_subtype on trigger filter |
| PLAYER_CHOICE (M10+) | 29 | 3% | No | Requires networked player interaction |
| AURA_EQUIP | 27 | 3% | Partial | Variant equip costs, enchant-graveyard |
| LIFE_RELATED | 26 | 2% | Partial | Opponent-life-loss triggers, life-total tracking |
| DRAW_DISCARD | 23 | 2% | Partial | Opponent-draw triggers, discard-as-cost |
| PHASE_TRIGGER | 17 | 2% | Partial | First main phase, raid end-step, upkeep variants |
| REPLACEMENT_EFFECT | 16 | 1% | No | Power doubling, token doubling, complex replacements |
| MULTI_TARGET | 16 | 1% | No | Multiple separate targets on one effect |
| MANA_COMPLEX | 16 | 1% | Partial | Mana-spend triggers, tap-for-mana doubling |
| UP_TO_N_TARGETS | 14 | 1% | No | "Up to N" optional targeting |
| PROTECTION | 12 | 1% | Partial | Mass grants, dynamic color protection |
| LANDFALL | 8 | 1% | Yes | LandEntersBattlefield trigger missing |
| TARGET_VARIANT | 8 | 1% | Easy | TargetCreatureOrPlaneswalker, TargetOpponent |
| GRANT_ABILITY | 7 | 1% | Mostly yes | "Creatures you control have {T}: Add G" |
| APPROXIMATION | 3 | 0% | Verify | PB-37 approximations to check |
| RAID_CONDITION | 2 | 0% | No | Condition::YouAttackedThisTurn missing |
| CAST_FROM_ZONE | 2 | 0% | No | New AltCostKind variants needed |

---

## Actionable Fix Waves

### Wave 1: Backfill Sprint (est. ~100 actual fixes after validation, 131 candidate files)
TODOs where the DSL primitive likely exists from PB-23–37 but the card def
wasn't updated during the PB's "fix existing cards" phase (backfill sweep cards).

**Regex scan identified 155 fixable TODOs across 131 files** (71 fully fixable,
60 partially fixable). Spot-checking revealed ~30-40% false positive rate
(regex matched keywords but the actual gap is different). Estimate ~100 truly
fixable TODOs after per-card validation.

**False positive patterns found**:
- "draw trigger" matched but card needs "Nth card each turn" tracking (Alandra)
- "X-cost" matched but card needs Exert or mana-spend trigger (Arena of Glory)
- "flicker" matched in text but actual gap is Rebound keyword (Ephemerate)
- "graveyard ability" matched but actual gap is mutate-from-graveyard (Brokkos)

**Approach**: Fix sessions should validate each card by reading the full TODO
and checking if the DSL construct actually exists before modifying.

Top contributing PBs:
- PB-27 X-cost (21 TODOs) — many are genuine EffectAmount::XValue fixes
- PB-30 combat damage triggers (17) — most genuine
- PB-32 damage prevention (10) — some genuine
- PB-23 controller creature triggers (14) — mostly genuine
- PB-26 trigger variants (33) — mixed, ~50% genuine
- PB-36 evasion/protection (7) — mostly genuine

9 fix sessions planned (15 files each). See candidate list in session output.

### Wave 2: Simple Extensions (~70 TODOs)
Small DSL additions that unblock many cards:
- `TargetCreatureOrPlaneswalker` variant (8 cards)
- `TargetOpponent` variant (8 cards)
- `LandEntersBattlefield` trigger condition (8 cards — Landfall)
- `has_subtype` on trigger filters (32 cards)
- `Condition::YouAttackedThisTurn` (2 cards — Raid)
- "Up to N targets" optional targeting (14 cards)

### Wave 3: Medium Extensions (~200 TODOs)
Larger primitives that each unblock 10-30 cards:
- Count scaling: `EffectAmount::PowerOfSacrificedCreature`, `AttackingCreatureCount`, etc.
- Optional choice effects (needs M10 player interaction for full fix)
- Complex activated abilities (individual card wiring)
- Per-opponent token creation
- Multi-target effects

### Wave 4: M10-Blocked (~74 TODOs)
Requires networked player interaction (M10):
- "You may" optional effects (45)
- Player choice / interactive (29)

### Wave 5: Complex / Deferred (~200+ TODOs)
Complex replacement effects, token doubling, dynamic color protection, etc.
These are long-tail items that will shrink as more primitives are added.

---

## Card Health Updated

| Category | Count | % |
|----------|-------|---|
| Clean (no TODOs) | 773 | 53% |
| With TODOs (fixable now) | 71 | 5% |
| With TODOs (partially fixable) | 60 | 4% |
| With TODOs (still blocked) | 547 | 38% |
| **Total authored** | **1,451** | |
| Not yet authored | ~291 | |
| **Total universe** | **1,743** | |

---

## Recommended Next Steps

1. **BF-1 Fix Sprint**: Fix the 71 fully-fixable files + fixable TODOs in 60 partially-fixable files (~155 TODO removals). Organize into sessions of 15-20 files each.
2. **BF-2**: Commit gap closure complete marker.
3. **Wave 2 extensions**: Add the 6 simple DSL variants (~70 more TODOs cleared).
4. **Resume A-29+** card authoring with the expanded DSL.
5. **Phase 3 audit**: After all authoring complete, sweep remaining TODOs.
