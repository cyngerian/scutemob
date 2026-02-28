---
name: start-session
description: Session orientation — git log, workstream state, dispatch table. Fast and read-only.
---

# Start Session

CLAUDE.md and MEMORY.md are already loaded in your system prompt — do NOT re-read them or any other files except the ones listed below.

Do NOT run cargo test, cargo build, cargo clippy, or any build/test commands. Tests are run before/after code changes, not at session start.

## Step 1: Git log

Run ONLY this command:
- `git log --oneline -5`

## Step 2: Workstream state

Read `memory/workstream-state.md` and report:
- Which workstreams are **ACTIVE** (another session is working on them — do NOT touch)
- Which are **available** (free to claim)
- Which are **paused** (partially done, safe to resume)
- The **Last Handoff** section (what the previous session did and what's next)

If any workstream is ACTIVE, print a warning:
> **W<N> is ACTIVE** — another session is working on it. Pick a different workstream or coordinate with the other session.

## Step 3: Progress checkboxes

Read the **Progress checkboxes** section of `docs/workstream-coordination.md` (grep for `#### Phase` to find it — it's near the end of the file). Report:

1. **Current phase**: Which phase are you in? (Phase 0 if any Phase 0 boxes are unchecked, otherwise Phase 1, etc.)
2. **Next unchecked item**: The specific checkbox that should be worked on next
3. **Recommended workstream + task**: Based on the unchecked item, which workstream to claim and what to do

Example output:
> **Phase 0: Stabilize** — 0/5 items checked
> Next item: `W3 T1: 10 new tests written`
> **Recommendation**: Claim W3, write the 10 missing T1 tests from `docs/mtg-engine-low-issues-remediation.md`

Also check `memory/ability-wip.md` — if an ability is in-progress, that takes priority over everything:
> **WIP ability found**: `<name>` in phase `<phase>` — finish this first with `/implement-ability`

## Step 4: Brief summary

Give a brief summary (5-8 lines max) covering:
- What the last few commits worked on
- Current project status from memory
- The recommended task from Step 3 (don't just say "pick a workstream" — give the specific next action)

## Step 5: Dispatch table

Print the dispatch table so the developer knows which files to load for their work:

---

**When to Load What:**

| Task | Load before starting |
|------|----------------------|
| Touching any file in `rules/` | `memory/gotchas-rules.md` |
| Touching any file in `state/`, `cards/`, `effects/` | `memory/gotchas-infra.md` |
| Writing or modifying tests | `memory/gotchas-infra.md` (testing gotchas) |
| Writing new code or tests | `memory/conventions.md` |
| Questioning a design decision | `memory/decisions.md` |
| Implementing a new subsystem | `docs/mtg-engine-corner-cases.md` (full) |
| Starting a new milestone | Use `/start-milestone <N>` |
| Writing golden tests | `docs/mtg-engine-game-scripts.md` |
| Implementing network features (M10+) | `docs/mtg-engine-network-security.md` |
| Implementing replay viewer (M9.5) | `docs/mtg-engine-replay-viewer.md` |
| Deciding what to work on | `docs/workstream-coordination.md` |
| Fixing LOW issues | `docs/mtg-engine-low-issues-remediation.md` |
| Working on abilities | `docs/ability-batch-plan.md` |

Use `/review-subsystem <name>` to load the right file and see open issues in one step.

---

## Step 6: Claim reminder

Print:
> **Next step**: Run `/start-work <workstream>` to claim your workstream before starting.
> Examples: `/start-work W1-B3`, `/start-work W2`, `/start-work W3`

## Step 7: Session plan check

Check if a session plan file exists in `memory/` (e.g., `m8-session-plan.md`). If one exists, call it out prominently: **"Session plan found: `memory/m<N>-session-plan.md` — use `/start-milestone <N>` to load it without touching the roadmap."** Do not read it unless the developer asks.

Do not read any files beyond `memory/workstream-state.md`, the progress checkboxes section of `docs/workstream-coordination.md`, and `memory/ability-wip.md`. Do not run any commands beyond the single git log above.
