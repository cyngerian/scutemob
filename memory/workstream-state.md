# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | Batch 14 complete. Batch 15 next (Commander variants: Friends Forever, Choose a Background, Doctor's Companion). | available | — | KW 143, AbilDef 58, SOK 58 — Batch 15 needs no new discriminants (deck validation only) |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; treat stale 2026-03-03 claim as available |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-08
**Workstream**: W1: Abilities — Batch 14

Batch 14 complete. All 6 abilities implemented (5 full pipeline + 1 already done):
- Cipher (CR 702.99): encoded_cards Vec<(ObjectId, CardId)> on GameObject; CipherCombatDamage PendingTrigger; CipherTrigger SOK 56; KW 141, AbilDef 57; 7 tests; card: Call of the Nightwing; script 187 (pending_review — harness gap: no cast_spell_cipher action). Two fixes: PendingTrigger hash (HIGH), aftermath override (MEDIUM). Harness fix: enrich_spec_from_def propagates AbilityDefinition::Cipher → KW::Cipher.
- Haunt (CR 702.55): haunting_target: Option<ObjectId> on GameObject; HauntExileTrigger SOK 57 + HauntedCreatureDiesTrigger SOK 58; HauntExiled event 107; KW 142; 8 tests; card: Blind Hunter; script 188 (PASS). Fix: haunting_target cleared after trigger resolves (MEDIUM).
- Reconfigure (CR 702.151): is_reconfigured: bool on GameObject; AbilityDefinition::Reconfigure (AbilDef 58); Effect::DetachEquipment; Layer 4 type removal; KW 143; 8 tests (clean review); card: Lizard Blades; script 189 (PASS).
- Blood Tokens (CR 111.10g): blood_token_spec(); ActivationCost.discard_card: bool; Command::ActivateAbility.discard_card_id; handle_activate_ability discard processing; 14 tests (clean); card: Voldaren Epicure; script 190 (5/5 PASS).
- Treasure Tokens: already fully validated (P2, script 073) — no work needed.
- Decayed Tokens (CR 702.147): zombie_decayed_token_spec(); 4 new token tests (12 total); card: Jadar, Ghoulcaller of Nephalia; script 191 (PASS). Two cross-cutting engine fixes: end_step_actions() generic AtBeginningOfYourEndStep CardDef sweep + resolution.rs card registry fallback for TriggeredAbility SOK.

1829+ tests. 175 validated. P4 82/88. Scripts 187-191. 6 new cards.

**Next**: Claim W1-B15. Check docs/ability-batch-plan.md for Batch 15 (Friends Forever, Choose a Background, Doctor's Companion — deck validation only, no new discriminants). After B15: Mutate mini-milestone. After Mutate: LegalActionProvider update.

**Discriminant chain end of B14**: KW 143, AbilDef 58, SOK 58. B15 needs no new discriminants.

**Hazards**:
- Cipher script 187 pending_review — harness lacks cast_spell_cipher action to wire encoding choice (LOW gap, same pattern as Gift's gift_opponent gap)
- Blood token helpers.rs export: blood_token_spec added to prelude — future cards can use it directly
- Two engine infra fixes in this batch (end_step + resolution.rs) benefit all future CardDef triggers via PendingTriggerKind::Normal
- W3 claim (2026-03-03) was stale — reset to available

**Commit prefix used**: W1-B14:

## Handoff History

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
