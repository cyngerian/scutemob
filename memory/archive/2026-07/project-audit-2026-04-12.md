# Project Audit — 2026-04-12

## Executive Summary

The MTG Commander Rules Engine is a mature, well-tested codebase that has been **blocked on card authoring for 6+ weeks** (since early March). The engine core (M0-M9.5) is complete and stable. The project has accumulated significant documentation debt and organizational complexity that will slow post-authoring development.

**Key metrics**:
- **223,546 lines of Rust** across all crates
- **2,625 tests** (2,607 `#[test]` + integration scripts)
- **1,749 card definitions** (780 with TODOs, 969 clean)
- **408 commits since March 1** — almost all card authoring/primitives
- **~50 primitive batches** completed (PB-0 through PB-37, plus PB-A through PB-X)

**What's working**: The engine, the primitive pipeline, the review process, the agent system.

**What's not**: Too many documents, stale planning artifacts, unclear next steps after W6.

---

## Document Inventory

### Core Documents (KEEP — actively used)

| Document | Purpose | Last Updated | Status |
|----------|---------|--------------|--------|
| `CLAUDE.md` | Session context, current state | 2026-04-12 | **ACTIVE** — primary context file |
| `docs/project-status.md` | Machine-parseable progress | 2026-04-12 | **ACTIVE** — tracks PBs and workstreams |
| `docs/primitive-card-plan.md` | PB slate and ordering | 2026-04-12 | **ACTIVE** — drives W6 work |
| `memory/workstream-state.md` | Cross-session coordination | 2026-04-12 | **ACTIVE** — handoffs live here |
| `memory/gotchas-infra.md` | Engine implementation gotchas | 2026-04-11 | **ACTIVE** — updated during PB work |
| `memory/gotchas-rules.md` | CR rules gotchas | 2026-04-12 | **ACTIVE** — updated during PB work |
| `memory/conventions.md` | Code style, patterns | 2026-04-11 | **ACTIVE** — updated during PB work |
| `memory/primitive-wip.md` | Current PB state | 2026-04-12 | **ACTIVE** — drives primitive-impl-* agents |

### Secondary Documents (KEEP — reference)

| Document | Purpose | Last Updated | Status |
|----------|---------|--------------|--------|
| `docs/mtg-engine-architecture.md` | System design, rationale | Mar 8 | **STABLE** — rarely changes |
| `docs/mtg-engine-corner-cases.md` | 36 known edge cases | Feb 23 | **STABLE** — reference |
| `docs/mtg-engine-milestone-reviews.md` | Code review findings | Mar 20 | **STABLE** — historical record |
| `docs/mtg-engine-roadmap.md` | Full milestone plan | Mar 27 | **STALE** — needs post-W6 update |
| `docs/mtg-engine-strategic-review.md` | 2026-03-07 decision record | Mar 9 | **HISTORICAL** — snapshot only |
| `memory/decisions.md` | Architectural decisions | Mar 7 | **STALE** — needs recent decisions |

### Likely Stale (ARCHIVE or DELETE)

| Document | Purpose | Last Activity | Recommendation |
|----------|---------|---------------|----------------|
| `docs/card-authoring-operations.md` | W5 operations plan | Apr 11 | **ARCHIVE** — W5 retired, replaced by primitive-card-plan |
| `docs/workstream-coordination.md` | 4-workstream tracking | Mar 21 | **CONSOLIDATE** into project-status.md |
| `docs/ability-batch-plan.md` | W1 ability batches | Mar 9 | **ARCHIVE** — W1 complete, all abilities done |
| `docs/mtg-engine-ability-coverage.md` | Ability audit | Mar 13 | **ARCHIVE** — abilities complete |
| `docs/dsl-gap-closure-plan.md` | PB-23 through PB-37 | Mar 29 | **ARCHIVE** — gap closure done |
| `docs/mtg-engine-low-issues-remediation.md` | W3 LOW tracking | Apr 10 | **CONSOLIDATE** — LOW sprint done, residuals in project-status |
| `docs/mtg-engine-interaction-gaps.md` | Interaction tracking | Mar 13 | **ARCHIVE** — outdated by TODO classification |
| `docs/mtg-engine-tui-plan.md` | W2 TUI plan | Feb 26 | **STALE** — W2 stalled since Feb 28 |
| `docs/mtg-engine-simulator.md` | Simulator design | Feb 28 | **STALE** — not touched in 6 weeks |
| `docs/mtg-engine-type-consolidation.md` | M9.5 refactor | Mar 9 | **ARCHIVE** — consolidation complete |
| `docs/mtg-engine-runtime-integrity.md` | Pre-alpha requirement | Mar 21 | **DEFERRED** — M10+ concern |
| `docs/mtg-engine-network-security.md` | P2P security | Feb 23 | **DEFERRED** — M10+ concern |
| `docs/mtg-engine-replay-viewer.md` | M9.5 viewer design | Feb 22 | **ARCHIVE** — viewer done |

### Memory Files — Archaeology

| Directory | Files | Status |
|-----------|-------|--------|
| `memory/abilities/` | 330 files | **ARCHIVE** — W1 complete, all ability plans/reviews |
| `memory/card-authoring/` | 124 files | **PARTIAL ARCHIVE** — keep `todo-classification-2026-04-12.md`, archive review batches |
| `memory/primitives/` | 94 files | **PARTIAL ARCHIVE** — keep recent PB plans (Q4, future), archive completed |
| `memory/w3-*.md` | 7 files | **ARCHIVE** — W3 LOW sprint complete |
| `memory/feedback_*.md` | 2 files | **KEEP** — process learnings |
| `memory/*.md` (misc) | ~10 files | Review individually |

---

## Organizational Issues

### 1. Document Sprawl

**Problem**: 25 docs in `docs/`, 20+ files in `memory/`, 330 ability plans, 124 card-authoring files, 94 primitive files. Total: **~600 files** of planning/tracking artifacts.

**Impact**: Session startup time, context confusion, stale references in CLAUDE.md.

**Fix**:
1. Create `docs/archive/` and `memory/archive/` directories
2. Move completed W1, W3, completed PB files to archive
3. Reduce active docs to ~10 files
4. Update CLAUDE.md to reference only active docs

### 2. Workstream Confusion

**Problem**: Five workstreams defined (W1-W5), one retired (W5), one added (W6). W1/W3 done, W2 stalled, W4 not started. Coordination doc tracks 4 workstreams but reality is "just W6".

**Impact**: New sessions see complex workstream state for simple reality.

**Fix**:
1. In `project-status.md`, simplify to: "W6 active, everything else blocked on W6 completion"
2. After W6: define M10/M11 workstreams fresh, don't inherit W1-W5 numbering
3. Archive `docs/workstream-coordination.md`

### 3. Card Authoring Pipeline Complexity

**Problem**: Multiple overlapping systems:
- `docs/card-authoring-operations.md` (W5 plan, 68 tasks)
- `docs/primitive-card-plan.md` (current PB slate)
- `memory/card-authoring/dsl-gap-audit*.md` (multiple versions)
- `memory/card-authoring/todo-classification-2026-04-12.md` (new)

**Impact**: Unclear which doc drives work.

**Fix**:
1. `primitive-card-plan.md` is the **single source of truth** for remaining PB work
2. `todo-classification-2026-04-12.md` is the **audit/reference** for blocked cards
3. Archive everything else in `docs/card-authoring-operations.md` and older audits
4. Add "Card Health" section to `project-status.md` (already exists, keep updated)

### 4. Agent Proliferation

**Problem**: 17 agents in `.claude/agents/`. Many for completed workflows (ability-impl-*, milestone-reviewer, session-runner). Only actively used: primitive-impl-{planner,runner,reviewer}.

**Impact**: Agent selection confusion, stale agent prompts.

**Fix**:
1. After W6: archive ability-impl-* agents (W1 done)
2. Keep: primitive-impl-*, card-definition-author, bulk-card-author
3. For M10: create new network-focused agents rather than retrofitting
4. Consider: single "impl-pipeline" meta-agent that handles plan→implement→review→fix

---

## Post-W6 Streamlining Plan

### Phase 1: Archive Completed Work (1 session)

```
mkdir docs/archive memory/archive

# Archive W1 (abilities)
mv memory/abilities/ memory/archive/
mv docs/ability-batch-plan.md docs/archive/
mv docs/mtg-engine-ability-coverage.md docs/archive/

# Archive W3 (LOW sprint)
mv memory/w3-*.md memory/archive/
# Keep LOW remediation doc but move to archive

# Archive completed PBs
mv memory/primitives/pb-{plan,review}-{0-37,A-X}.md memory/archive/primitives/

# Archive old card-authoring artifacts
mv docs/card-authoring-operations.md docs/archive/
mv memory/card-authoring/{dsl-gap-audit,review-*,wave-*,f4-*,f5-*}.md memory/archive/
```

### Phase 2: Consolidate Active Docs (1 session)

**Target: 8 active docs**

1. `CLAUDE.md` — project context (streamlined)
2. `docs/project-status.md` — all progress tracking
3. `docs/mtg-engine-architecture.md` — system design
4. `docs/mtg-engine-roadmap.md` — milestones (update for M10/M11)
5. `docs/primitive-card-plan.md` — PB slate (until W6 done)
6. `memory/workstream-state.md` — handoffs
7. `memory/gotchas-infra.md` — infra learnings
8. `memory/gotchas-rules.md` — rules learnings

**Remove from CLAUDE.md "When to Load What" table**:
- Ability coverage (W1 done)
- LOW remediation (W3 done)
- Workstream coordination (consolidate into project-status)
- Card authoring operations (archive)
- Type consolidation (complete)

### Phase 3: Simplify CLAUDE.md (1 session)

Current CLAUDE.md is ~350 lines. Target: ~200 lines.

Remove:
- W1-W5 workstream details (keep only "W6 active")
- Milestone completion checklist (move to roadmap)
- Agent table details (link to .claude/agents/README.md instead)
- Card authoring wave process (archive)
- Detailed PB status (link to project-status.md)

Keep:
- Current state summary
- Architecture invariants
- Critical gotchas (top 3)
- MCP resources
- When to load what (simplified)
- Commit prefix convention

---

## M10/M11 Preparation

### Decision: Web-First vs Tauri

From strategic review: the Tauri app can't build on current dev env (headless Debian). The replay viewer (axum + Svelte) works. **Recommendation**: Web-first.

**M10 scope (web server)**:
1. Extend replay viewer into interactive game server
2. WebSocket endpoint for real-time state
3. Command submission (cast, pass, declare, etc.)
4. Bot opponents (use existing simulator crate)
5. Room management (create/join)

**M11 scope (web UI)**:
1. Interactive game board (extend Svelte components)
2. Player input (targeting, mode choices)
3. Hidden info (hand, opponent's hidden zones)
4. Local play: 1 human + 3 bots

### Simplified Roadmap

```
Current:
  W6 (card authoring) ────────────────────────> [in progress]
       └── PB-Q4 ──> PB-R ──> PB-T ──> ... ──> W6 done

After W6:
  M10 (game server) + M11 (web UI) ─────────> [parallel]
       └── Server + WebSocket       └── Svelte game board
       └── Bot opponents            └── Player input
       └── Room management          └── Hidden info

Alpha:
  M10 + M11 + remaining cards ──────────────> human playable
```

---

## Metrics to Track Post-W6

| Metric | Current | Target for Alpha |
|--------|---------|------------------|
| Card defs with TODOs | 780 | <100 (blocking cards) |
| Clean card defs | 969 | >1600 |
| Tests | 2,625 | 3,000+ |
| Game scripts | 270 | 300+ |
| Corner cases covered | 32/36 | 34/36 |

---

## Immediate Actions

1. **Finish PB-Q4** — 5 cards, currently in implement phase
2. **Update project-status.md** — card health numbers from TODO classification
3. **Don't start archiving yet** — wait until W6 PBs are truly done
4. **Decision needed**: Which remaining PBs are worth doing vs deferring?
   - High-yield: SubtypeFilteredAttack (18 cards), DamagedPlayer (15 cards), Landfall (13 cards)
   - Low-yield: Interactive (M10+ blocked anyway), complex patterns (compound blockers)

---

## Summary

The project is healthy but has accumulated cruft from 2 months of intensive card authoring. Once W6 completes:

1. **Archive** ~500 planning files from completed workstreams
2. **Consolidate** docs from ~25 to ~8 active files
3. **Simplify** CLAUDE.md by 40%
4. **Update** roadmap for M10/M11 parallel work
5. **Start** M10/M11 with clean slate, not inherited complexity

The engine is solid. The card base is 84% authored. The path to alpha is clear:
finish high-yield PBs → M10 server → M11 UI → playable game.
