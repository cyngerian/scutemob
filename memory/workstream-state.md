# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | ALL P3+P4 DONE. Morph complete (KW 157, AbilDef 64, SOK 63). Only Morph variants (Megamorph/Disguise/Manifest/Cloak) need card defs+scripts for "validated". W1 complete. |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; T3 ManaPool pending |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 19 cards total authored; low yield until DSL gaps filled |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-08
**Workstream**: W1: Abilities — Morph Mini-Milestone
**Task**: Full Morph pipeline: plan → implement → review → fix → cards → scripts → close

**Completed**:
- Morph (CR 702.37, P3): FaceDownKind enum, face_down_as on GameObject, AltCostKind::Morph casting, Command::TurnFaceUp + TurnFaceUpMethod, face-down layer override (2/2 colorless no abilities), FaceDownRevealed event (CR 708.9)
- Megamorph (CR 702.37f, P4): engine complete (KW+AbilDef+counter-on-flip), no card def yet
- Disguise (CR 702.162, P4): engine complete (face-down ward {2}), no card def yet
- Manifest (CR 701.40, P4): Effect::Manifest, no card def yet
- Cloak (CR 701.58, P4): Effect::Cloak, no card def yet
- Review: 1 HIGH (Manifest ETB suppression), 3 MEDIUM — all fixed
- Cards: Exalted Angel, Birchlore Rangers, Akroma Angel of Fury (3 Morph card defs)
- Scripts: 197 (cast face-down + flip, PASS), 198 (face-down dies reveal, PASS)
- Coverage: Morph validated; Megamorph/Disguise/Manifest/Cloak = complete (need card+script for validated)
- Corner case 30 (Morph/Manifest face-down): DEFERRED → COVERED
- P3: 40/40 ALL DONE; W1 COMPLETE

**Next**: W1 complete — proceed to Phase 2 (TUI Hardening — W2) or Phase 3 T3 (W3 ManaPool encapsulation) or Phase 4 (M10 — W4).

**Hazards**: Megamorph/Disguise/Manifest/Cloak each need a card def + script to reach "validated". LOW findings F5/F6 still open. Scripts 195/196/197/198 are pending_review in harness.

**Discriminant chain end**: KW 157, AbilDef 64, SOK 63.

**Commit prefix used**: W1-Morph:

## Handoff History

### 2026-03-08 (session end) — W1: Abilities — Transform Mini-Milestone
- Transform (701.28), Disturb (702.145), Daybound/Nightbound (702.146), Craft (702.167); 1911 tests; 183 validated; P3 39/40; P4 88/88 (all done!); scripts 193-196; cards: Delver of Secrets, Beloved Beggar, Brutal Cathar, Braided Net; KW 148-152, AbilDef 60-61, SOK 60-62

### 2026-03-08 (session end) — W1: Abilities — Batch 15 + Mutate + LegalActionProvider
- B15: Friends Forever/ChooseABackground/DoctorsCompanion (CR 702.124i/k/m); KW 144-146; 16 tests; clean review (2 LOW); no scripts (validation-only); commits W1-B15:
- Mutate mini-milestone (CR 702.140): merged_cards/MergedComponent on GameObject; CastWithMutate + MutatingCreatureSpell (SOK 59); AbilDef::MutateCost (AbilDef 59); KW 147; zone-change splitting (CR 729.5); mutate trigger; 9 tests; script 192; cards: Gemrazer, Nethroi, Brokkos; commits W1-Mutate:
- LegalActionProvider: ActivateBloodrush + SaddleMount + CastWithMutate added to legal_actions.rs
- 1889 tests; 179 validated; P4 84/88; P3 38/40; discriminant chain end KW 147/AbilDef 59/SOK 59

### 2026-03-08 (session end) — W1: Abilities — Batch 14
- Cipher (702.99), Haunt (702.55), Reconfigure (702.151), Blood Tokens (111.10g), Treasure Tokens (already done), Decayed Tokens (702.147); 1829 tests; 175 validated; P4 82/88; scripts 187-191; cards: Call of the Nightwing, Blind Hunter, Lizard Blades, Voldaren Epicure, Jadar Ghoulcaller of Nephalia; commits W1-B14:

### 2026-03-07 (session end) — W1: Abilities — Batch 13
- Discover (701.57), Suspect (701.60), Collect Evidence (701.59), Forage (701.61), Squad (702.157), Offspring (702.175), Gift (702.174), Saddle (702.171); 1792 tests; 171 validated; P4 77/88; scripts 179-186; cards: Geological Appraiser, Frantic Scapegoat, Crimestopper Sprite, Camellia the Seedmiser, Ultramarines Honour Guard, Flowerfoot Swordmaster, Nocturnal Hunger, Quilled Charger

### 2026-03-07 (session end) — W1: Abilities — Batch 12
- Enrage, Alliance, Corrupted, Ravenous (702.156), Bloodrush; 1784 tests; 165 validated; P4 71/88; scripts 174-178; cards: Ripjaw Raptor, Prosperous Innkeeper, Vivisection Evangelist, Tyrranax Rex, Ghor-Clan Rampager
