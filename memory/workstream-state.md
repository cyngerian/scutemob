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
| W5: Card Authoring | — | available | — | 149 cards authored; comprehensive plan at test-data/test-cards/AUTHORING_PLAN.md |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-10
**Workstream**: W1 (B16 closeout) + W5 (card authoring planning)
**Task**: Complete Dungeon/Ring abilities, author 20 cards, build comprehensive card authoring pipeline

**Completed**:
- B16 complete: Dungeon (CR 309, 701.49, 725) + The Ring Tempts You (CR 701.54) — all reviewed + fixed via agents
- Authored 24 card definitions (20 new + 4 Dungeon/Ring) — reviewed by 4 parallel agents, 6 fixes applied
- EDHREC data fetched for all 20 commanders (5,110 unique cards)
- Combined card universe: 1,743 cards (20 decks + EDHREC >= 5k inclusion)
- `generate_authoring_plan.py`: groups cards into 43 categories, variable batch sizes (8-20), 155 ready sessions
- Comprehensive authoring plan at `test-data/test-cards/AUTHORING_PLAN.md`
- Two new agents: `bulk-card-author` (Sonnet, purple), `card-batch-reviewer` (Opus, yellow)
- Explicit per-wave workflow: create tracking plan → author (2 parallel) → build → review (4 parallel) → fix → commit

**Next**:
1. Write `bulk_generate.py` (Phase 1 template script — ~227 cards)
2. Run Phase 1, audit, fix, commit
3. Generate skeletons for remaining ~1,244 cards (Phase 2a)
4. Run bulk-card-author agent sessions by wave (Phase 2b)
5. Start with Wave 1: Lands — ETB Tapped (122 cards, 8 sessions)

**Hazards**: New agents require session restart to appear in registry. ~79 LOW issues still open. `_authoring_worklist.json` only tracks 20-deck universe (1,174 cards), not full 1,743 — TUI shows stale numbers until we update it to read `_authoring_plan.json`.

**Commit prefix used**: W5-cards:, chore:

## Handoff History

### 2026-03-09 — Cross-cutting: Ability Validation Sprint + B16 closeout
- P4 93/105 validated; 6 abilities promoted; harness: gift_opponent, enrich_spec_from_def Gift fix; 4 card defs + 7 scripts; docs updated

### 2026-03-08 (session end) — W1: Abilities — Morph Mini-Milestone
- Morph (CR 702.37, P3); Megamorph/Disguise/Manifest/Cloak engine complete; 3 cards, 2 scripts; P3 40/40 ALL DONE; W1 COMPLETE; KW 157, AbilDef 64, SOK 63

### 2026-03-08 (session end) — W1: Abilities — Transform Mini-Milestone
- Transform (701.28), Disturb (702.145), Daybound/Nightbound (702.146), Craft (702.167); 1911 tests; 183 validated; P3 39/40; P4 88/88 (all done!); scripts 193-196; cards: Delver of Secrets, Beloved Beggar, Brutal Cathar, Braided Net; KW 148-152, AbilDef 60-61, SOK 60-62

### 2026-03-08 (session end) — W1: Abilities — Batch 15 + Mutate + LegalActionProvider
- B15: Friends Forever/ChooseABackground/DoctorsCompanion (CR 702.124i/k/m); KW 144-146; 16 tests; clean review (2 LOW); no scripts (validation-only); commits W1-B15:
- Mutate mini-milestone (CR 702.140): merged_cards/MergedComponent on GameObject; CastWithMutate + MutatingCreatureSpell (SOK 59); AbilDef::MutateCost (AbilDef 59); KW 147; zone-change splitting (CR 729.5); mutate trigger; 9 tests; script 192; cards: Gemrazer, Nethroi, Brokkos; commits W1-Mutate:
- LegalActionProvider: ActivateBloodrush + SaddleMount + CastWithMutate added to legal_actions.rs
- 1889 tests; 179 validated; P4 84/88; P3 38/40; discriminant chain end KW 147/AbilDef 59/SOK 59

### 2026-03-08 (session end) — W1: Abilities — Batch 14
- Cipher (702.99), Haunt (702.55), Reconfigure (702.151), Blood Tokens (111.10g), Treasure Tokens (already done), Decayed Tokens (702.147); 1829 tests; 175 validated; P4 82/88; scripts 187-191; cards: Call of the Nightwing, Blind Hunter, Lizard Blades, Voldaren Epicure, Jadar Ghoulcaller of Nephalia; commits W1-B14:


