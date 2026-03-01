# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | Batch 3 done; Batch 4 next (Retrace, Jump-Start, Aftermath, Embalm, Eternalize, Encore) |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | Phase 0 complete; T2 done; Phase 1 (abilities) next |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-01 (session end)
**Workstream**: W1: Abilities — Batch 3
**Task**: Implement Batch 3: Combat Modifiers & Ninjutsu (5 abilities)
**Completed**:
- Melee (CR 702.121): SelfAttacks trigger, MeleeTrigger (disc 23), counts distinct Player targets at resolution, disc 83, 7 tests, Wings of the Guard, script 121 — commit 3e695b4
- Poisonous (CR 702.70): CombatDamageDealt triggered ability, PoisonousTrigger (disc 24), N fixed counters additive (unlike Infect), disc 84, 6 tests, Poisonous Viper (test card), script 122 — commit 9a6961c
- Toxic (CR 702.164): STATIC ability inline in combat.rs, no trigger/stack, sum all Toxic(N) values, disc 85, 8 tests, Pestilent Syphoner, script 123 — commit 3dfc5a8
- Enlist (CR 702.154): enlist_choices on DeclareAttackers, 10-check validation, tap-as-cost, EnlistTrigger (disc 25), +X/+0 UntilEndOfTurn at resolution, disc 86, 8 tests, Coalition Skyknight, script 124 — commit 29c24a4
- Ninjutsu (CR 702.49) + CommanderNinjutsu (CR 702.49d): Command::ActivateNinjutsu, NinjutsuAbility (disc 26), discs 87/88, ETB site pattern + combat registration, attack target inheritance (CR 702.49c), 12 tests, Ninja of the Deep Hours, script 125 — commit 17e19fd
- 3 CR corrections: Melee=702.121 (not 702.122=Crew), Enlist=702.154 (not 702.155), Toxic=702.164 (not 702.156=Ravenous)
- 1295 tests passing; 111 abilities validated total; P4 19/88
**Next**: Batch 4 — Alt-cast graveyard (Retrace, Jump-Start, Aftermath, Embalm, Eternalize, Encore). Claim W1-B4.
**Hazards**: None — all committed; Toxic review found layer-bypass bug (fixed inline). Discriminant chain: KeywordAbility 83-88, StackObjectKind 23-26.
**Commit prefix used**: `W1-B3:`

## Handoff History

### 2026-03-01 (session end) — W1: Abilities — Batch 2
- Flanking, Bushido, Rampage, Provoke, Afflict, Renown, Training; 1254 tests; P4 13/88; scripts 114-120; cards: Suq'Ata Lancer, Devoted Retainer, Wolverine Pack, Goblin Grappler, Khenra Eternal, Topan Freeblade, Gryff Rider; commit 92f1265

### 2026-03-01 (session end) — W1: Abilities — Batch 1
- Horsemanship, Skulk, Devoid, Decayed, Ingest; 1177 tests; P4 6/88; scripts 109-113; cards: Shu Cavalry, Furtive Homunculus, Forerunner of Slaughter, Shambling Ghast, Mist Intruder; commit 9cc5672

### 2026-02-28 (session end) — W1: Abilities — Batch 0
- Bolster, Adapt, Shadow, Partner With, Overload; 1166 tests; scripts 104-108; P3 36/40, P4 1/88; commit 2729c3d

### 2026-02-28 (late night) — W5: Card Authoring (second batch)
- 7 cards (Demonic Tutor, Worldly Tutor, Vampiric Tutor, Mana Confluence, Phyrexian Altar, Impact Tremors, Skullclamp); 3 new DSL gaps identified; commit `c3e80e0`

### 2026-02-28 (night) — W5: Card Authoring (first batch)
- Batches A/B/C: authored 30, removed 22 with simplifications; 8 accurate cards remain
- Fixed generate_worklist.py: DSL_GAP_PATTERNS, blocked classification; commit prefix `W5-cards:`

### 2026-02-28 (night) — W2: TUI & Simulator (UX fixes)
- Fix 1-6: Hand scrolling, discard events, zone counters, zone browser overlay, CardDetail return-to, action hints; Esc bug fix
