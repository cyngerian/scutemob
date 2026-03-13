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
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; T3 ManaPool pending |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | Wave 1 DONE (82 cards, committed e04ce0d); Wave 2 next |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-12
**Workstream**: W5: Card Authoring
**Task**: Phase 2 Wave 1 — land-etb-tapped (82 cards) — full cycle: recover → review → fix → commit

**Completed**:
- Recovered lost prior session: 82 Wave 1 card files confirmed on disk, build+tests clean
- 17 review batches run in parallel (4 at a time) via card-batch-reviewer agents
- Fix pass: 39 files updated — 2 HIGH (missing Legendary supertypes on The World Tree + Emeria), ~46 MEDIUM (bulk-implemented ETB-tapped, mana-tap, ETB-Scry/Surveil/GainLife, Cycling), LOW metadata (subtype ordering, Cult Conscript)
- Committed `e04ce0d`: `W5-cards: Phase 2 Wave 1 — land-etb-tapped (82 cards)` — 1972 tests pass
- Saved feedback memory: bulk-card-author was systematically too conservative; documented patterns + reference files

**Next**:
1. W5 Phase 2 Wave 2: next card group from the authoring plan — claim W5, check `test-data/test-cards/AUTHORING_PLAN.md` for next wave, create wave plan file, run bulk-card-author with reference file hints (see feedback memory)
2. W3 T3: ManaPool::spend() encapsulation (last unchecked Phase 3 item)

**Hazards**: bulk-card-author leaves implementable patterns as TODOs — always point it to lonely_sandbar.rs, jungle_shrine.rs, aven_riftwatcher.rs and tell it to implement ETB-tapped/mana-tap/Scry/Surveil/GainLife/Cycling. See `memory/feedback_bulk_card_author_too_conservative.md`.

**Commit prefix used**: `W5-cards:`

## Handoff History

### 2026-03-12 — W5 recovery: Wave 1 recovered, reviewed, fixed, committed
- Recovered lost session (82 cards on disk); 17 review batches; fix pass (39 files); commit e04ce0d; 453 total card defs

### 2026-03-10 — W5: Card Authoring (Phase 1)
- bulk_generate.py: 114 template card defs (371 total); 20 review batches; all HIGH/MEDIUM fixed; 1972 tests

### 2026-03-10 — W1 (B16 closeout) + W5 (card authoring planning)
- B16 complete: Dungeon + Ring; 24 card defs; EDHREC data; 1,743 card universe; authoring plan + 2 new agents

### 2026-03-09 — Cross-cutting: Ability Validation Sprint + B16 closeout
- P4 93/105 validated; 6 abilities promoted; harness: gift_opponent, enrich_spec_from_def Gift fix; 4 card defs + 7 scripts; docs updated

### 2026-03-08 (session end) — W1: Abilities — Morph Mini-Milestone
- Morph (CR 702.37, P3); Megamorph/Disguise/Manifest/Cloak engine complete; 3 cards, 2 scripts; P3 40/40 ALL DONE; W1 COMPLETE; KW 157, AbilDef 64, SOK 63
