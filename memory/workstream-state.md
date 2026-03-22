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
| W3: LOW Remediation | — | available | — | **W3 LOW sprint DONE** (S1-S6): 83→29 open (119 closed total). TC-21 done. 2233 tests. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | T-1: Refresh DSL gap audit | ACTIVE | 2026-03-22 | Phase 0 triage — scan all card defs for TODOs |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: Card authoring infrastructure (I-1 through I-6)

**Completed**:
- I-1: Created `card-fix-applicator` agent (`.claude/agents/card-fix-applicator.md`) — reads review findings, applies corrections to card defs
- I-2: Updated `bulk-card-author` agent — expanded reference table (16→33 groups), added PB-0 through PB-22 DSL patterns, 13 known-issue patterns, re-authoring rule for skeletons, MCP budget 20→30
- I-3: Updated `card-batch-reviewer` agent — checks 10→12, known-issue patterns 10→19, added "Now-Expressible Patterns" table (26 entries), MCP budget 10→15
- I-4: Created `/triage-cards` skill — orchestrates T-1 through T-7 (gap audit, session reclassification, finding consolidation)
- I-5: Created `/author-wave` skill — orchestrates author→review→fix→commit for one group with parallel agents
- I-6: Created `/audit-cards` skill — orchestrates X-1 through X-7 (full re-scan, fix, certify)
- Updated CLAUDE.md (agent table, "When to Load What" table)
- Updated MEMORY.md (agent list, skills list)
- Updated `docs/card-authoring-operations.md` status DRAFT→ACTIVE, all I-* items checked off

**Next**:
1. T-1: Refresh DSL gap audit — run `/triage-cards` to start Phase 0

**Hazards**:
- New agent (`card-fix-applicator`) requires session restart to be usable as subagent_type
- No code changes — only agent/skill/doc files modified

**Commit prefix used**: `W6-cards:`

## Handoff History

### 2026-03-22 — W6: Card Authoring Infrastructure (I-1 through I-6)
- Created card-fix-applicator agent, 3 new skills (triage-cards, author-wave, audit-cards)
- Updated bulk-card-author (33 groups, 13 KI patterns, MCP 30) and card-batch-reviewer (12 checks, 19 KI patterns, Now-Expressible table)
- Operations plan status: DRAFT → ACTIVE

### 2026-03-21 — W6: PB-22 S7 (Adventure CR 715 + Dual-Zone Search)
- AltCostKind::Adventure + adventure_face on CardDefinition + exile-on-resolution
- also_search_graveyard on Effect::SearchLibrary (dual-zone search)
- 2 new cards (Bonecrusher Giant, Lovestruck Beast), 3 fixed (Monster Manual, Finale, Lozhan)
- Review: 1H 3M, all fixed. 9 new tests, 2281 total. **PB-22 COMPLETE**

### 2026-03-21 — W6: PB-22 S6 (Emblem Creation, CR 114)
- Effect::CreateEmblem + GameEvent::EmblemCreated + emblem trigger scanning
- 6 planeswalker card defs authored/fixed. Review: 5H 6M 2L, all fixed. 2272 total

### 2026-03-21 — W6: PB-22 S5 (Copy/Clone Primitives)
- Effect::BecomeCopyOf + Effect::CreateTokenCopy + Condition::CardTypesInGraveyardAtLeast
- Review: 0H 2M 3L. 5 new tests, 2265 total

### 2026-03-20 — W6: PB-22 S1 (activation_condition)
- activation_condition on activated abilities (CR 602.5b). 305 files, 2236 total
