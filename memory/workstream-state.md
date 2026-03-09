# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | Batch 16: Venture/Dungeon + The Ring Tempts You | ACTIVE | 2026-03-09 | Dungeon zone infra + 4 dungeon defs + Ring Tempts You; full /implement-ability pipeline per ability |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; T3 ManaPool pending |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 19 cards total authored; low yield until DSL gaps filled |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-09
**Workstream**: Cross-cutting (chore) — Ability Validation Sprint
**Task**: Promote 6 "complete" P4 abilities to "validated" (card defs + scripts + harness fixes)

**Completed**:
- Harness fix: `gift_opponent` field added to `PlayerAction` schema; `AbilityDefinition::Gift → KeywordAbility::Gift` in `enrich_spec_from_def`; `gift_opponent_name` param in `translate_player_action` (all callers updated)
- Card defs created: Lumbering Laundry (Disguise {5}), Den Protector (Megamorph {1}{G}), Write into Being (Manifest), Cryptic Coat (Cloak ETB)
- Scripts approved: 182 (Forage Food), 185 (Gift Nocturnal Hunger), 200 (Forage exile-3), 201 (Disguise), 202 (Megamorph), 203 (Manifest), 204 (Cloak)
- Coverage doc updated: P4 93/105 validated (was 87); 0 complete remaining; all non-N/A P4 abilities validated
- Committed: `chore: validate 6 complete abilities — Disguise, Megamorph, Manifest, Cloak, Forage, Gift`

**Next**: All implementable abilities validated. Proceed to M10 (W4), TUI hardening (W2), LOW remediation W3 T3 (ManaPool hardening), or card authoring (W5). Read `docs/mtg-engine-strategic-review.md` before starting M10. Discuss the 12 N/A abilities (Banding, digital-only, niche) — user flagged for discussion.

**Hazards**: ~79 LOW issues open. W3 T3 (ManaPool hardening) still pending — one unchecked Phase 3 box in workstream-coordination.md.

**Discriminant chain end**: KW 157, AbilDef 55, SOK ~20 (unchanged).

**Commit prefix used**: chore:

## Handoff History

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

### 2026-03-07 (session end) — W1: Abilities — Batch 13
- Discover (701.57), Suspect (701.60), Collect Evidence (701.59), Forage (701.61), Squad (702.157), Offspring (702.175), Gift (702.174), Saddle (702.171); 1792 tests; 171 validated; P4 77/88; scripts 179-186; cards: Geological Appraiser, Frantic Scapegoat, Crimestopper Sprite, Camellia the Seedmiser, Ultramarines Honour Guard, Flowerfoot Swordmaster, Nocturnal Hunger, Quilled Charger

