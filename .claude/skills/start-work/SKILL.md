---
name: start-work
description: RETIRED — the W1–W6 workstream-claim model is frozen. See the current flow below.
---

# Start Work — 🗄️ RETIRED (2026-07-18, DOCB-2 / `scutemob-132`)

> **This skill is retired. Do not use it to claim work.**
>
> `/start-work` was built on the **W1–W6 workstream-claim model**, which is **frozen**.
> Collision-avoidance no longer works by editing an "Active Claims" table in
> `memory/workstream-state.md`; it works by **git worktrees + ESM task locks** (each
> dispatched worker gets its own worktree and an ESM task that only one agent owns).
> The docs this skill used to read (`docs/project-status.md`, `docs/workstream-coordination.md`,
> `docs/ability-batch-plan.md`, `docs/primitive-card-plan.md`) are all RETIRED or HISTORICAL.

## What to do instead — current flow

1. **`/start`** — bootstrap ESM, load state (`esm project bootstrap` + auto-memory
   `MEMORY.md`), and orient. This replaces the old "read the claims table" step.
2. **Pick the next work item from the active queue plan** — the ranked queue lives in a
   dated file under `memory/primitives/`, named in the **CLAUDE.md "Current State"** section
   (as of 2026-07-18: `memory/primitives/oos-retriage-plan-2026-07-18.md`). Card-authoring
   work is driven by `memory/card-authoring/campaign-plan-2026-05-16.md` (§0). Coverage/health
   numbers come from `docs/authoring-status.md` (regenerate via `python3 tools/authoring-report.py`).
3. **`/dispatch <title>`** — create the ESM task + worktree and launch a worker. This is the
   real collision boundary: the worktree isolates the working tree and the ESM task lock
   prevents two agents claiming the same work. Use `/spawn` if you want to launch the worker
   by hand, or `/task` + `/done` for small self-assigned work.
4. **`/collect`** — merge the finished worker's branch back to main and close the task.

For the pipeline mechanics of a single primitive batch, use **`/implement-primitive`**.

## Why it was retired

`memory/doc-audit-2026-07-18b.md` (Theme 2, S2) found this skill wired entirely to retired
docs and built on the frozen W-model. The coordinator decision (DOCB-2) was **retire, not
rewire**: the claim-table mechanism it encodes has been fully superseded by the
worktree + ESM-task-lock model above, so there is nothing to rewire it onto. The file is kept
(not deleted) so an invocation lands on this banner instead of a "skill not found" error.
