# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | Batch 5 complete; Batch 6 next |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | LOW remediation — T2/T3 items | ACTIVE | 2026-03-03 | Phase 0 complete; T2 done; working T2/T3 LOWs |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-02 (session end)
**Workstream**: W1: Abilities — Batch 5
**Task**: Implement Batch 5: Alt-cast hand/exile (Dash, Blitz, Plot, Prototype, Impending)
**Completed**:
- W3 structural refactor: CastSpell 13 booleans → `alt_cost: Option<AltCostKind>`, PendingTrigger 21 booleans → `kind: PendingTriggerKind`, GameObject `was_evoked/was_escaped/was_dashed` → `cast_alt_cost: Option<AltCostKind>` — commit 201bc48
- Dash (CR 702.109): ETB haste, EOT return trigger, 7 tests, Zurgo Bellstriker, script 132 — commit 54f6ea9
- Blitz (CR 702.152): ETB haste + EOT sacrifice + inline draw-on-death, 9 tests (SBA lethal path), Riveteers Requisitioner, script 133 — commit 4499bda
- Plot (CR 702.170): new Command::PlotCard special action + free cast (AltCostKind::Plot), 20 tests, Slickshot Show-Off, script 134 — commit 9750a51
- Prototype (CR 702.160/718): NOT an AltCost — separate `prototype: bool` on CastSpell; zone-change revert (CR 718.4), copy propagation (CR 718.3c); 2 HIGH fixes; 10 tests, Blitz Automaton, script 135 — commit aa46447
- Impending (CR 702.176): AltCostKind::Impending, Layer 4 type-removal inline, time counter ETB + end-step removal; clean review (4 LOW test gaps); 11 tests, Overlord of the Hauntwoods, script 136 — commit c2d30fd
- helpers.rs: added ManaColor + ManaAbility to DSL prelude (enables Everywhere token mana_abilities)
- replay_harness.rs: cast_spell_impending action type + "time" in parse_counter_type
- 1421 tests passing; 122 validated total; P4 30/88
**Next**: Claim W1-B6. Check docs/ability-batch-plan.md for Batch 6 contents.
**Hazards**: Discriminant chain: KeywordAbility 95-99, AbilityDefinition 28-32, StackObjectKind 31-33. StackObject still has per-ability was_X fields (was_dashed, was_blitzed etc.) — not consolidated, deferred. Prototype's `prototype: bool` on CastSpell still causes ~85-file update when new Prototype cards added — could use Default+struct-update eventually.
**Commit prefix used**: `W1-B5:`, `W3:` (structural refactor)

## Handoff History

### 2026-03-01 (session end) — W1: Abilities — Batch 4
- Retrace, Jump-Start, Aftermath, Embalm, Eternalize, Encore; 1336 tests; 117 validated; P4 25/88; scripts 126-131; cards: Flame Jab, Radical Idea, Cut//Ribbons, Sacred Cat, Proven Combatant, Briarblade Adept; commits cada8d5–3991065

### 2026-03-01 (session end) — W1: Abilities — Batch 3
- Melee, Poisonous, Toxic, Enlist, Ninjutsu/CommanderNinjutsu; 1295 tests; P4 19/88; scripts 121-125; cards: Wings of the Guard, Poisonous Viper, Pestilent Syphoner, Coalition Skyknight, Ninja of the Deep Hours; commits 3e695b4–17e19fd

### 2026-03-01 (session end) — W1: Abilities — Batch 2
- Flanking, Bushido, Rampage, Provoke, Afflict, Renown, Training; 1254 tests; P4 13/88; scripts 114-120; cards: Suq'Ata Lancer, Devoted Retainer, Wolverine Pack, Goblin Grappler, Khenra Eternal, Topan Freeblade, Gryff Rider; commit 92f1265

### 2026-03-01 (session end) — W1: Abilities — Batch 1
- Horsemanship, Skulk, Devoid, Decayed, Ingest; 1177 tests; P4 6/88; scripts 109-113; cards: Shu Cavalry, Furtive Homunculus, Forerunner of Slaughter, Shambling Ghast, Mist Intruder; commit 9cc5672

### 2026-02-28 (session end) — W1: Abilities — Batch 0
- Bolster, Adapt, Shadow, Partner With, Overload; 1166 tests; scripts 104-108; P3 36/40, P4 1/88; commit 2729c3d
