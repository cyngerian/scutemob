# Strategic Review: Path to Playable (2026-03-07)

> **Purpose**: Document findings from a full project review and define actionable changes
> to the roadmap that shorten the time-to-playable. These changes should be implemented
> before M10 begins.
>
> **Status**: HISTORICAL SNAPSHOT (2026-03-07). Metrics below reflect state at time of writing.
> For live metrics see `CLAUDE.md` "Current State" section.
> **Updated 2026-03-09**: Type consolidation complete; metrics refreshed below.

---

## Context

The engine core (M0-M9.5) is complete and well-tested. As of 2026-03-07: ~52k lines of
engine code, ~131k lines of tests, 1641 tests, 193 card definitions, 229 game scripts,
153 validated abilities, 29/36 corner cases covered. *(As of 2026-03-09: 1934 tests, 125 hand-authored cards,
~105 approved scripts, ~187 validated abilities, 32/36 corner cases covered — see
CLAUDE.md for current numbers.)* The architecture is sound.

The problem is the **critical path from here to a human-playable game**:

```
Current plan:  M10 (network) -> M11 (UI) -> M12 (card pipeline) -> M13 (full UI) -> M14 (assets) -> M15 (alpha)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
               5 milestones, estimated 6+ months before a human can play through a GUI
```

This review identifies changes to compress that path without sacrificing quality.

---

## Finding 1: Decouple M11 (UI) from M10 (Networking)

**Problem**: The roadmap says M11 depends on M10 "for multi-window testing," but this is a
soft dependency. The Tauri app can render game state and accept player input without a
WebSocket server — it can drive the engine directly in-process.

**Change**: M11 should be executable in parallel with or before M10. The app starts as a
local single-player experience (1 human + 3 bots from the simulator crate), then gains
networking when M10 lands.

**Impact**: Humans can play games months earlier. The UI gets tested and iterated on before
networking adds debugging complexity.

**Files to update**:
- `docs/mtg-engine-roadmap.md` — remove M10 dependency from M11; add local-play mode
- `docs/workstream-coordination.md` — note M11 is independent of M4/W4

---

## Finding 2: Consider Web UI Instead of Tauri

**Problem**: The Tauri app cannot build on the current dev environment (headless Debian,
missing webview/GTK libs). The replay viewer already has a working axum + Svelte 5 stack
that serves a web UI. Building a second UI framework (Tauri IPC) creates duplication.

**Change**: Evaluate extending the replay viewer into an interactive game client instead of
building a separate Tauri app. The axum server already:
- Serves Svelte components that render game state
- Has API endpoints for loading and stepping through games
- Runs on the current dev environment

An interactive web client would add:
- WebSocket endpoint for real-time state updates
- Command submission (cast spell, pass priority, declare attackers, etc.)
- Bot opponents via the simulator crate (server-side)

This could be the M10 server and M11 UI in a single crate.

**Trade-off**: Loses Tauri's native desktop packaging, but gains:
- Single codebase (not two UI frameworks)
- Works on any device with a browser
- Buildable on the current dev environment
- Faster iteration (no Tauri rebuild cycle)

**Decision (2026-07-26, user) — WEB-FIRST**: extend the axum + Svelte 5 replay-viewer stack.
Tauri v2 remains a later *packaging wrapper* option, not a parallel UI framework. Recorded in
`memory/decisions.md`; applied to `docs/mtg-engine-roadmap.md` (M11-local).

> **Correction to this finding's premise**: "the Tauri app cannot build on the current dev
> environment (headless Debian)" is **stale** — development moved to a full desktop
> (skylarch) and Tauri builds fine there. The web-first call was made on *iteration speed*
> and *single-stack* grounds, not on the environment constraint. The rest of the finding
> stands.

**Files to update** (if web-first chosen):
- `docs/mtg-engine-architecture.md` — update Section 1 system architecture diagram
- `docs/mtg-engine-roadmap.md` — merge M10/M11 scope; redefine M11
- `CLAUDE.md` — update architecture invariants if needed
- `memory/decisions.md` — record the decision

---

## Finding 3: Rethink M12 (Card Pipeline)

**Problem**: M12 envisions a three-stage pipeline (scripted converter -> LLM fallback ->
human review) for bulk card generation. But the project is already scaling card authoring
via Claude agents without this infrastructure:
- 125 hand-authored card definitions
- `card-definition-author` agent generates definitions
- `generate_skeleton.py` creates starting points
- W5 workstream has a worklist of ~941 ready cards

The scripted converter's claimed 70-80% coverage is optimistic given the DSL gaps documented
in MEMORY.md (targeted_trigger blocking 57 cards, return_from_graveyard blocking 17, etc.).

**Change**: Downscope M12 from "build a pipeline crate" to:
1. Close DSL gaps in the engine (already happening via W1 ability batches)
2. Improve the `card-definition-author` agent prompt
3. Batch-run the agent against high-priority cards
4. Add deck builder enforcement (reject games with undefined cards) — this is a small
   engine feature, not a full milestone

The pattern library / scripted converter can be revisited post-alpha if agent-based
generation proves too slow or expensive at scale.

**Files to update**:
- `docs/mtg-engine-roadmap.md` — downscope M12
- `memory/decisions.md` — record rationale

---

## Finding 4: Prioritize Morph/Transform (Unblock 9 Ability Batches) — DONE

**Problem**: Morph (5 batches blocked) and Transform (4 batches blocked) were unimplemented,
blocking 9 ability batches in `docs/ability-batch-plan.md`.

**Resolution (2026-03-08)**: Both implemented as pre-M10 mini-milestones:
1. Transform/DFC (CR 712) — complete: Daybound/Nightbound, Disturb, Craft
2. Morph/Manifest (CR 702.37/701.40) — complete: Morph, Megamorph, Disguise, Manifest, Cloak

All 9 previously-blocked batches are now unblocked. See CLAUDE.md for details.

---

## Finding 5: The Simulator Already Has Bots

**Discovery**: The `crates/simulator/` crate (1,698 lines) already contains:
- `RandomBot` — plays random legal moves (fuzzing)
- `HeuristicBot` — makes smarter decisions
- `GameDriver` — runs complete games with bots
- `LegalActionProvider` — enumerates legal actions
- `mana_solver` — greedy mana payment
- Random deck builder from available CardDefinitions

This is more advanced than expected. The infrastructure for "1 human + 3 bots" already
exists. What's missing is the **human input bridge** — a way for a human to be one of the
players in a `GameDriver` game, receiving state views and submitting commands.

**Change**: The first playable milestone should be a human playing against 3 bots through
a web UI, using the existing simulator infrastructure. This is the fastest path to a
playable game.

**Files to update**:
- `docs/mtg-engine-roadmap.md` — reference simulator in M11 deliverables

---

## Finding 6: Split M10 Into Two Parts

**Problem**: M10's scope is large for a single milestone: WebSocket server, room management,
hidden info filtering, reconnection, state history ring buffer, safe checkpoints, rewind,
pause. This is 6-8 weeks of work.

**Change**: Split into:
- **M10a: Basic Multiplayer Server** (~2-3 weeks)
  - WebSocket server accepting player connections
  - Room manager (create/join by code, 2-6 player slots)
  - Command relay and event broadcast
  - One engine instance per room
  - Out-of-turn command rejection
- **M10b: Resilience & Social Features** (~2-3 weeks)
  - Hidden info filtering (private_to())
  - Reconnection with state sync
  - State history ring buffer
  - Rewind (unanimous consent)
  - Pause/resume

**Rationale**: M10a gets multiplayer working. M10b adds polish. If time is tight, M10b
features can ship in alpha without blocking multiplayer play.

**Files to update**:
- `docs/mtg-engine-roadmap.md` — split M10 section

---

## Finding 7: Uncommitted Changes Need Commit

**Problem**: There are ~20 modified files in the working tree (see git status). These include
changes to `card_definition.rs`, `effects/mod.rs`, `rules/` files, `state/` files, and
`testing/replay_harness.rs`. The workspace builds clean and snapshot_perf.rs already has
the `paired_with` field, but these changes are not committed.

**Change**: Commit the outstanding changes before starting new work.

---

## Pre-M10 Type Consolidation — COMPLETE (2026-03-09)

A pre-M10 refactoring effort (`docs/mtg-engine-type-consolidation.md`) consolidated
sprawling engine types before networking work begins. Results:

| Type | Before | After | Method |
|------|--------|-------|--------|
| `CastSpell` fields | 32 | 13 | `AdditionalCost` vec replaces ~20 one-off fields |
| `StackObjectKind` variants | ~62 | ~20 | `KeywordTrigger` absorbs ~25 trigger variants |
| `AbilityDefinition` variants | 64 | 55 | `AltCastAbility` replaces 10 alt-cost variants |
| `Designations` | 8 booleans | 1 u16 bitfield | `bitflags` crate |

All 1934 tests passing throughout. 0 HIGH/MEDIUM issues from consolidation (~22 LOW
stale doc comments deferred). See `docs/mtg-engine-type-consolidation.md` for session
details.

---

## Revised Critical Path

```
Before (serial):
  M10 ---------> M11 ---------> M12 ---------> M13 ---------> M14 -> M15

After (parallel, compressed):
  Transform/Morph (W1)            -- DONE (2026-03-08)
  Type consolidation (pre-M10)    -- DONE (2026-03-09)
  |
  +-- M11-local (web UI + bots, no networking)     -- humans can play here
  |
  +-- M10a (basic multiplayer server)               -- parallel with M11
  |
  +-- M10b (resilience, rewind, pause)              -- after M10a
  |
  +-- Card scaling via agents (continuous, replaces M12)
  |
  M13 (UI polish) + M14 (assets) -- merged, after M11+M10a
  |
  M15 (alpha)
```

**Key difference**: Humans can play (against bots, locally) after M11-local, which has
no dependency on networking. Multiplayer arrives when M10a finishes, in parallel.
Transform/Morph and type consolidation are complete — M10a/M11-local are ready to start.

---

## Action Items

All nine are now resolved — eight done, one (#9) obsoleted by a doc retirement. The last
five were closed on 2026-07-26 (`scutemob-147`).

| # | Action | Priority | Blocks | Status |
|---|--------|----------|--------|--------|
| 1 | Commit outstanding working tree changes | Immediate | Clean baseline | ✅ DONE (2026-03-07) |
| 2 | Update roadmap: decouple M11 from M10 | High | M11 start | ✅ DONE 2026-07-26 — M11 is now **M11-local**, dependencies are M9 + M9.5 only |
| 3 | Update roadmap: split M10 into M10a/M10b | High | M10 planning | ✅ DONE 2026-07-26 — plus an M10-pre engine-correctness block |
| 4 | Update roadmap: downscope M12 | Medium | M12 planning | ✅ DONE 2026-07-26 — pipeline crate cancelled; continuous agent-authoring track; original design kept in a collapsed HISTORICAL block |
| 5 | Record decisions in `memory/decisions.md` | High | Future sessions | ✅ DONE 2026-07-26 (commit `f50eabba`) |
| 6 | Decide: web-first vs Tauri-first vs both | High | M11 architecture | ✅ DONE 2026-07-26 — **web-first**, see Finding 2 above |
| 7 | ~~Schedule Transform/Morph in ability plan~~ | Medium | 9 ability batches | ✅ DONE (2026-03-08) |
| 8 | Update CLAUDE.md current state and milestone | Medium | Session orientation | ✅ DONE 2026-07-26 — Current State shows M11-local ACTIVE (web-first) |
| 9 | Update workstream-coordination.md | Medium | Parallel work | ✅ N/A — that doc was retired 2026-03-08 (the W1–W6 model is historical); work selection now lives in CLAUDE.md → Current State |
