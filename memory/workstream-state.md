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
| W5: Card Authoring | — | available | — | Wave 2 DONE (187 cards, committed d83ac94+01e3b52); Wave 3 next (Mana Lands, 92 cards) |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-13
**Workstream**: W5: Card Authoring
**Task**: Phase 2 Wave 2 — combat-keyword (187 cards) — full cycle: author → review → fix → commit

**Completed**:
- Authored 187 cards across 14 sessions (26–39) via bulk-card-author agents (2 in parallel)
- Compile fixes during authoring: `EffectLayer::PowerToughness` → `PtModify`, no `KeywordAbility::CantBlock` variant, `ManaCost` missing `colorless` field
- 38 review batches run in parallel (4 at a time) via card-batch-reviewer agents
- Fix pass (13 HIGHs): World supertype, CantBeBlocked on Invisible Stalker, wrong target filter (Tamiyo's Safekeeping), Concordant Crossroads, approximate-targeting cards emptied (Ram Through, Legolas), Markov Baron Convoke+Madness, Hammer of Nazahn implemented, Crown of Skemfar Enchant+Reach, Nullpriest Kicker, Nezumi Prowler Ninjutsu cost, Mindleecher MutateCost, Ajani CMC, */* CDA creatures `Some(0)→None`
- Committed `d83ac94` (author) + `01e3b52` (fixes) — 1944 tests pass; 640 total card defs
- Wave plan: `memory/card-authoring/wave-002-combat-keyword.md` (COMPLETE)

**Next**:
1. W5 Wave 3: Mana Lands (92 cards, 7 sessions, batch 16) — sessions from `_authoring_plan.json` group `mana-land`; create `memory/card-authoring/wave-003-mana-land.md`; reference: `command_tower.rs`
2. W3 T3: ManaPool::spend() encapsulation (last unchecked Phase 3 item)

**Hazards**: `*/*` CDA creatures must use `power: None, toughness: None` (not `Some(0)`) — engine SBA uses `toughness?` which skips None. Aura cards need `Enchant(EnchantTarget::Creature)` keyword. Ninjutsu/Mutate cards need BOTH the keyword marker AND the cost `AbilityDefinition`.

**Commit prefix used**: `W5-cards:`

## Handoff History

### 2026-03-13 — W5: Wave 2 combat-keyword (187 cards) complete
- 14 sessions (26–39); 38 review batches; 13 HIGH fixes; commits d83ac94+01e3b52; 640 total card defs; 1944 tests

### 2026-03-12 — W5 recovery: Wave 1 recovered, reviewed, fixed, committed
- Recovered lost session (82 cards on disk); 17 review batches; fix pass (39 files); commit e04ce0d; 453 total card defs

### 2026-03-10 — W5: Card Authoring (Phase 1)
- bulk_generate.py: 114 template card defs (371 total); 20 review batches; all HIGH/MEDIUM fixed; 1972 tests

### 2026-03-10 — W1 (B16 closeout) + W5 (card authoring planning)
- B16 complete: Dungeon + Ring; 24 card defs; EDHREC data; 1,743 card universe; authoring plan + 2 new agents

### 2026-03-09 — Cross-cutting: Ability Validation Sprint + B16 closeout
- P4 93/105 validated; 6 abilities promoted; harness: gift_opponent, enrich_spec_from_def Gift fix; 4 card defs + 7 scripts; docs updated

