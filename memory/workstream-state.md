# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | Batch 0+1+2 complete; Batch 3 next (Ninjutsu, Bushido+, Exalted, etc.) |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | Phase 0 complete; T2 done; Phase 1 (abilities) next |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-01 (session end)
**Workstream**: W1: Abilities — Batch 2
**Task**: Implement Batch 2: Combat Triggers — Blocking (7 abilities)
**Completed**:
- Flanking (CR 702.25): BlockersDeclared trigger, -1/-1 ModifyBoth UntilEndOfTurn, StackObjectKind::FlankingTrigger (disc 19), discriminant 76, 7 tests, Suq'Ata Lancer, script 114 — commit a9457c8
- Bushido (CR 702.45): SelfBlocks + SelfBecomesBlocked triggers, AddCounter, discriminant 77, TriggerEvent::SelfBecomesBlocked (disc 18), 7 tests, Devoted Retainer, script 115 — commit ca39b61
- Rampage (CR 702.23): SelfBecomesBlocked + blocker count at resolution, ModifyBoth(bonus), StackObjectKind::RampageTrigger (disc 20), discriminant 78, 8 tests, Wolverine Pack, script 116 — commit a9c38e0
- Provoke (CR 702.39): new CombatState::forced_blocks OrdMap, ProvokeTrigger (disc 21), untap + force-block with evasion checks, discriminant 79, 7 tests, Goblin Grappler, script 117 — commit 2700b56
- Afflict (CR 702.130): SelfBecomesBlocked + defending_player LoseLife, discriminant 80, 6 tests, Khenra Eternal, script 118 — commit 4bdbb6a
- Renown (CR 702.112): is_renowned: bool on GameObject (reset on zone change), SelfCombatDamageDealt + RenownTrigger (disc 22), CR 603.4 intervening-if, discriminant 81, 7 tests, Topan Freeblade, script 119 — commit d4fe4b5
- Training (CR 702.149): SelfAttacksWithGreaterPowerAlly (disc 19), builder.rs TriggeredAbilityDef, calculate_characteristics() power compare, discriminant 82, 7 tests, Gryff Rider, script 120 — commit 92f1265
- CR correction: Training=702.149 (not 702.150=Compleated); coordination docs updated — commit cf85ee6
- 1254 tests passing; 105 abilities validated total; P4 13/88
**Next**: Batch 3 — check `docs/ability-batch-plan.md` for the next batch. Claim W1-B3.
**Hazards**: None — all changes committed; working tree has only minor modification to test-data/test-decks/_authoring_worklist.json
**Commit prefix used**: `W1-B2:`

## Handoff History

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
