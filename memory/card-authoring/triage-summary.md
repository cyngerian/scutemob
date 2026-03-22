---
name: Triage Summary
description: Phase 0 triage results — ground truth for card authoring operations (2026-03-22)
type: reference
---

# Triage Summary (Phase 0 Complete)

Generated: 2026-03-22
Source: T-1 through T-5 triage steps

## Card Universe

| Category | Count |
|----------|-------|
| Total card defs on disk | 742 |
| In authoring plan | 477 |
| Pre-existing (not in plan) | 264 |
| Not yet authored (plan only) | ~1,001 |
| **Total card universe** | **~1,743** |

## TODO Classification (569 TODOs across 304 files)

| Classification | TODOs | Cards |
|---------------|-------|-------|
| NOW_EXPRESSIBLE | ~143 | ~100 |
| PARTIALLY_EXPRESSIBLE | ~96 | ~70 |
| STILL_BLOCKED | ~313 | ~180 |
| STALE | ~17 | ~17 |

**Key insight**: ~160 TODOs (28%) are fixable today without engine work.

## Authoring Plan Status

| Status | Sessions | Cards |
|--------|----------|-------|
| ready | 168 | 1,467 |
| complete | 7 | 92 |
| blocked | 15 | 77 |
| **Total** | **190** | **1,636** |

### T-2/T-3: Session Reclassification
- 19 sessions unblocked (blocked/deferred → ready)
- 2 sessions reclassified (deferred → blocked — real DSL gaps)
- 15 sessions remain blocked (true DSL gaps: G-01 CDA, G-04 GainControl, G-18 count-threshold, G-31 reanimate, etc.)

## Review Consolidation (T-4)

Source: 73 review files (20 Phase 1 + 38 Wave 002 + 15 Wave 003)

| Severity | Original | Still Valid | Already Fixed | DSL Gap |
|----------|----------|-------------|---------------|---------|
| HIGH | 29 | 3 | 22 | 2 |
| MEDIUM | ~60 | 7 | ~45 | 3 |

**Actionable fixes**: 15 cards across 4 priority batches (see `consolidated-fix-list.md`)

## Pre-existing Defs (T-5)

264 card defs not in authoring plan:
- 197 clean (no TODOs, no wrong state)
- 21 fixable now (NOW_EXPRESSIBLE TODOs)
- 7 partially expressible
- 32 still blocked
- **7 silent wrong-state** (empty `abilities: vec![]` with no TODO — dangerous):
  - beast_within, call_of_the_nightwing, generous_gift, hanweir_the_writhing_township,
    overlord_of_the_hauntwoods, swan_song, mana_crypt

## Top DSL Gap Buckets (STILL_BLOCKED)

| Gap ID | Description | Card Count | Effort |
|--------|-------------|------------|--------|
| G-03 | Conditional static keyword grants | 18 | L |
| G-01 | CDA / dynamic P/T | 14 | L |
| G-09 | Return-land-to-hand ETB (bounce lands) | 14 | S |
| G-12 | Triggering-object targeting | 12 | M |
| G-04 | GainControl effect | 11 | M |
| G-18 | Count-threshold conditional statics | 10 | M |
| G-02 | Can't block/attack restrictions | 9 | M |
| G-11 | Activated ability from graveyard zone | 8 | M |
| G-10 | Triple-choice mana (filter lands) | 7 | M |
| G-13 | Dynamic conditional hexproof/protection | 7 | M |

## Effort Estimates

### Phase 1: Fix Existing Defs
- **Consolidated fix list**: 15 cards, ~1 hour
- **NOW_EXPRESSIBLE TODOs**: ~100 cards, ~8-12 sessions (8-10 cards/session)
- **Silent wrong-state**: 7 cards, ~1 session
- **STALE TODOs**: 17 cards, ~1 session
- **Subtotal**: ~140 cards, ~12-15 sessions

### Phase 2: Author New Cards
- **Ready sessions**: 168 sessions, ~1,467 cards
- **Estimated effort**: 168 sessions × ~30 min = ~84 hours of agent time
- Sessions run in batches of 8-20 cards each

### Phase 3: Audit
- Final scan for TODOs, empty abilities, known-issue patterns
- Estimated: 2-3 sessions

### Blocked Cards (require engine work)
- 15 blocked sessions = 77 cards
- ~180 individual cards with STILL_BLOCKED TODOs
- Top gaps (G-09 bounce lands, G-12 triggering-object targeting) are small engine additions
- Full resolution: ~10-15 small engine PRs covering the gap buckets

## Recommended Priority Order

1. **Fix silent wrong-state cards** (7 cards — these produce incorrect game state)
2. **Apply consolidated fix list** (15 cards from reviews)
3. **Fix stale TODOs** (17 cards — remove or update wrong TODO text)
4. **Fix NOW_EXPRESSIBLE TODOs** (100 cards — biggest bang-for-buck)
5. **Author ready sessions** (168 sessions / 1,467 new cards)
6. **Implement quick-win gap buckets** (G-09 bounce lands: 14 cards, S effort)
7. **Phase 3 audit**

## Files Created/Updated

| File | Purpose |
|------|---------|
| `memory/card-authoring/dsl-gap-audit-v2.md` | T-1: Full TODO classification |
| `test-data/test-cards/_authoring_plan.json` | T-2/T-3: Session status updates |
| `memory/card-authoring/consolidated-fix-list.md` | T-4: Actionable fix list from reviews |
| `memory/card-authoring/triage-summary.md` | T-6: This file |
