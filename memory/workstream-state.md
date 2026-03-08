# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | KW 147, AbilDef 59, SOK 59 — all implementable batches (0-15) + Mutate done; Morph (5) + Transform (4) deferred; next: W2 TUI hardening or W4 M10 |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; treat stale 2026-03-03 claim as available |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-08
**Workstream**: W1: Abilities — Batch 15 + Mutate + LegalActionProvider

All W1 implementable work complete. Three deliverables this session:

**Batch 15 — Partner Variants (B15)**:
- Friends Forever (CR 702.124i): deck validation in commander.rs; validate_partner_commanders() extended
- Choose a Background (CR 702.124k): Background enchantment pairing; is_legendary_background() helper
- Doctor's Companion (CR 702.124m): Time Lord Doctor pairing; is_time_lord_doctor() helper
- KW 144-146 (no new AbilDef or SOK discriminants)
- 16 tests in partner_variants.rs; clean review (2 LOWs only); no game scripts (validation-only)
- Commit prefix: W1-B15:

**Mutate Mini-Milestone (CR 702.140)**:
- merged_cards: Vec<MergedComponent> + MergedComponent struct on GameObject
- Command::CastWithMutate + StackObjectKind::MutatingCreatureSpell (SOK 59)
- AbilityDefinition::MutateCost (AbilDef 59); KW 147
- Resolution merge: caster chooses over/under; top card's characteristics + ALL abilities from all cards
- Zone-change splitting (CR 729.5): merged permanent leaves, all cards move together; death splits to individual graveyard cards
- Mutate trigger: "whenever this creature mutates" fires on successful merge
- 9 unit tests; game script 192 (approved)
- Card defs: Gemrazer, Nethroi, Brokkos (3 popular Ikoria commanders)
- Discriminant chain end: KW 147, AbilDef 59, SOK 59

**LegalActionProvider update**:
- ActivateBloodrush, SaddleMount, CastWithMutate added to legal_actions.rs
- LAP now covers all implemented abilities including B13-B15 + Mutate

1889 tests. 179 validated. P4 84/88. P3 38/40. Scripts through 192.

**Next for W1**: All implementable batches (0-15) + Mutate mini-milestone complete. Remaining blocked subsystems: Morph tree (5 abilities) + Transform tree (4 abilities) — deferred until dedicated pre-M10 mini-milestones or post-M10. Run `/audit-abilities` to refresh coverage doc, then proceed to W2 (TUI hardening) or W4 (M10 networking).

**Discriminant chain end**: KW 147, AbilDef 59, SOK 59.

**Open gaps**:
- Cipher script 187 pending_review — harness lacks cast_spell_cipher action (LOW, same pattern as Gift's gift_opponent gap)
- Morph (5 blocked), Transform (4 blocked) — deferred subsystems

**Commit prefix used**: W1-B15: (B15), W1-Mutate: (Mutate)

## Handoff History

### 2026-03-08 (session end) — W1: Abilities — Batch 15 + Mutate + LegalActionProvider
- B15: Friends Forever/ChooseABackground/DoctorsCompanion (CR 702.124i/k/m); KW 144-146; 16 tests; clean review (2 LOW); no scripts (validation-only); commits W1-B15:
- Mutate mini-milestone (CR 702.140): merged_cards/MergedComponent on GameObject; CastWithMutate + MutatingCreatureSpell (SOK 59); AbilDef::MutateCost (AbilDef 59); KW 147; zone-change splitting (CR 729.5); mutate trigger; 9 tests; script 192; cards: Gemrazer, Nethroi, Brokkos; commits W1-Mutate:
- LegalActionProvider: ActivateBloodrush + SaddleMount + CastWithMutate added to legal_actions.rs
- 1889 tests; 179 validated; P4 84/88; P3 38/40; discriminant chain end KW 147/AbilDef 59/SOK 59

### 2026-03-08 (session end) — W1: Abilities — Batch 14
- Cipher (702.99), Haunt (702.55), Reconfigure (702.151), Blood Tokens (111.10g), Treasure Tokens (already done), Decayed Tokens (702.147); 1829 tests; 175 validated; P4 82/88; scripts 187-191; cards: Call of the Nightwing, Blind Hunter, Lizard Blades, Voldaren Epicure, Jadar Ghoulcaller of Nephalia; commits W1-B14:

### 2026-03-07 (session end) — W1: Abilities — Batch 13
- Discover (701.57), Suspect (701.60), Collect Evidence (701.59), Forage (701.61), Squad (702.157), Offspring (702.175), Gift (702.174), Saddle (702.171); 1792 tests; 171 validated; P4 77/88; scripts 179-186; cards: Geological Appraiser, Frantic Scapegoat, Crimestopper Sprite, Camellia the Seedmiser, Ultramarines Honour Guard, Flowerfoot Swordmaster, Nocturnal Hunger, Quilled Charger; commits c279fb4–43d1f28

### 2026-03-07 (session end) — W1: Abilities — Batch 12
- Enrage, Alliance, Corrupted, Ravenous (702.156), Bloodrush; 1784 tests; 165 validated; P4 71/88; scripts 174-178; cards: Ripjaw Raptor, Prosperous Innkeeper, Vivisection Evangelist, Tyrranax Rex, Ghor-Clan Rampager; commit ba96d67

### 2026-03-07 (session end) — W1: Abilities — Batch 11
- Modal Choice (CR 700.2 P2 gap), Tribute (702.104), Fabricate (702.123), Fuse (702.102), Spree (702.172); 1754 tests; 160 validated; P4 66/88; scripts 169-173; cards: Abzan Charm, Fanatic of Xenagos, Weaponcraft Enthusiast, Wear//Tear, Final Showdown; commit f9df635

### 2026-03-07 (session end) — W1: Abilities — Batch 10
- Devour, Backup, Champion, Totem Armor, Living Metal, Soulbond, Fortify; 1706 tests; 155 validated; P4 64/88; scripts 162-168; cards: Predator Dragon, Backup Agent, Changeling Hero, Hyena Umbra, Steel Guardian, Silverblade Paladin, Darksteel Garrison; commit ae6cde8

### 2026-03-06 (session end) — W1: Abilities — Batch 9
- Graft (702.58), Scavenge (702.97), Outlast (702.107), Amplify (702.38), Bloodthirst (702.54), Amass (701.47); 1641 tests; 148 validated; P4 57/88; scripts 156-161; cards: Simic Initiate, Deadbridge Goliath, Ainok Bond-Kin, Canopy Crawler, Stormblood Berserker, Dreadhorde Invasion; upkeep_actions() CardDef trigger gap first identified here; fixed in B10
