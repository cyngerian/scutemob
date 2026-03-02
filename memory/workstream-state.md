# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | Batch 4 complete; Batch 5 next |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | Phase 0 complete; T2 done; Phase 1 (abilities) next |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-01 (session end)
**Workstream**: W1: Abilities — Batch 4
**Task**: Implement Batch 4: Alt-cast graveyard (6 abilities)
**Completed**:
- Retrace (CR 702.81): additional cost (discard land), returns to graveyard (not exile), escape auto-detection fix, disc 89, 11 tests, Flame Jab, script 126 — commit 04565a6
- Jump-Start (CR 702.133): discard-any-card cost, exile on resolve, Madness-aware discard routing, disc 90, 12 tests, Radical Idea, script 127 — commit 27107db
- Aftermath (CR 702.127): AbilityDefinition::Aftermath{name,cost,card_type,effect,targets} (disc 24), cast_with_aftermath flag, first-half hand-only / second-half graveyard-only, disc 91, 12 tests, Cut // Ribbons, script 128 — commit cada8d5
- Embalm (CR 702.128): exile-as-cost (vs. Unearth's resolve-time exile), CardId stored (not ObjectId, CR 400.7), White token + Zombie, supertypes preserved (review fix), disc 92, 12 tests, Sacred Cat, script 129 — commit 4a7757d
- Eternalize (CR 702.129): Black token + 4/4 P/T override + Zombie, source_name in StackObjectKind for TUI, disc 93, 12 tests, Proven Combatant, script 130 — commit 87390ca
- Encore (CR 702.141): per-opponent token creation with Haste, EncoreSacrificeTrigger EOC pattern, encore_activated_by field (control-change ruling fix), 2 new StackObjectKinds (discs 29/30), disc 94, 10 tests, Briarblade Adept, script 131 — commit 3991065
- Harness fix: card_name_to_id strips " // " for split card names
- 1336 tests passing; 117 abilities validated total; P4 25/88
**Next**: Batch 5 — Claim W1-B5. Check docs/ability-batch-plan.md for Batch 5 contents.
**Hazards**: Encore LOW #1 deferred (encore_must_attack not cleared at EOT if sacrifice countered). Discriminant chain: KeywordAbility 89-94, AbilityDefinition 24-27, StackObjectKind 27-30.
**Commit prefix used**: `W1-B4:`

## Handoff History

### 2026-03-01 (session end) — W1: Abilities — Batch 3
- Melee, Poisonous, Toxic, Enlist, Ninjutsu/CommanderNinjutsu; 1295 tests; P4 19/88; scripts 121-125; cards: Wings of the Guard, Poisonous Viper, Pestilent Syphoner, Coalition Skyknight, Ninja of the Deep Hours; commits 3e695b4–17e19fd

### 2026-03-01 (session end) — W1: Abilities — Batch 2
- Flanking, Bushido, Rampage, Provoke, Afflict, Renown, Training; 1254 tests; P4 13/88; scripts 114-120; cards: Suq'Ata Lancer, Devoted Retainer, Wolverine Pack, Goblin Grappler, Khenra Eternal, Topan Freeblade, Gryff Rider; commit 92f1265

### 2026-03-01 (session end) — W1: Abilities — Batch 1
- Horsemanship, Skulk, Devoid, Decayed, Ingest; 1177 tests; P4 6/88; scripts 109-113; cards: Shu Cavalry, Furtive Homunculus, Forerunner of Slaughter, Shambling Ghast, Mist Intruder; commit 9cc5672

### 2026-02-28 (session end) — W1: Abilities — Batch 0
- Bolster, Adapt, Shadow, Partner With, Overload; 1166 tests; scripts 104-108; P3 36/40, P4 1/88; commit 2729c3d

### 2026-02-28 (late night) — W5: Card Authoring (second batch)
- 7 cards (Demonic Tutor, Worldly Tutor, Vampiric Tutor, Mana Confluence, Phyrexian Altar, Impact Tremors, Skullclamp); 3 new DSL gaps identified; commit `c3e80e0`
