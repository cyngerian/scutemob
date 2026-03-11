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
| W5: Card Authoring | — | available | — | Phase 1 DONE (114 templated + audited); 371 total card defs |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-10
**Workstream**: W5: Card Authoring
**Task**: Phase 1 — write bulk_generate.py, generate template card defs, audit via reviewers

**Completed**:
- Wrote `test-data/test-cards/bulk_generate.py` — template generator for 5 groups (body-only, ETB tapped lands, mana lands/artifacts/creatures)
- Generated 114 new card definition files (371 total, up from 257)
- Ran 20 card-batch-reviewer agents covering all 114 cards
- Fixed 8 HIGH: missing mana costs (diamonds), missing Artifact type (artifact creatures), missing Basic+Snow supertypes (snow lands), mixed front/back face types (MDFCs)
- Fixed ~25 MEDIUM: 37 conditional ETB TODO comments, MDFC names, Legendary supertypes, Llanowar Tribe 3-green mana fix
- Template script bugs fixed: ETB tapped template now uses card mana_cost, format_types_expr handles multi-type cards
- 1972 tests passing, clean build

**Next**:
1. Follow the per-wave workflow properly next time (create wave plan → author → build → review 4×5 → fix → commit)
2. Phase 2a: generate skeletons for remaining ~1,158 ready cards
3. Wave 1: complex ETB tapped lands (65 skipped cards with cycling, scry, bounce, etc.) via bulk-card-author agent
4. Close out Ring Tempts You ability-wip.md (all steps done, just needs closing)

**Hazards**: ability-wip.md still open for Ring Tempts You (implement phase, all steps checked — needs close). 20 review files in memory/card-authoring/ (can clean up later). bulk_generate.py mana regex is strict — cards with pain-land damage, conditional activation, creature-count mana all correctly skip to Phase 2.

**Commit prefix used**: W5-cards:

## Handoff History

### 2026-03-10 — W1 (B16 closeout) + W5 (card authoring planning)
- B16 complete: Dungeon + Ring; 24 card defs; EDHREC data; 1,743 card universe; authoring plan + 2 new agents

### 2026-03-09 — Cross-cutting: Ability Validation Sprint + B16 closeout
- P4 93/105 validated; 6 abilities promoted; harness: gift_opponent, enrich_spec_from_def Gift fix; 4 card defs + 7 scripts; docs updated

### 2026-03-08 (session end) — W1: Abilities — Morph Mini-Milestone
- Morph (CR 702.37, P3); Megamorph/Disguise/Manifest/Cloak engine complete; 3 cards, 2 scripts; P3 40/40 ALL DONE; W1 COMPLETE; KW 157, AbilDef 64, SOK 63

### 2026-03-08 (session end) — W1: Abilities — Transform Mini-Milestone
- Transform (701.28), Disturb (702.145), Daybound/Nightbound (702.146), Craft (702.167); 1911 tests; 183 validated; P3 39/40; P4 88/88; scripts 193-196; KW 148-152, AbilDef 60-61, SOK 60-62

### 2026-03-08 (session end) — W1: Abilities — Batch 15 + Mutate + LegalActionProvider
- B15+Mutate+LegalActionProvider; 1889 tests; 179 validated; P4 84/88; P3 38/40
