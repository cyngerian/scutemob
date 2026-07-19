---
name: collect
description: Collect a finished worker — merge worktree to main, clean up
user-invocable: true
allowed-tools: Read, Bash
argument-hint: "[task_id]"
---

# Collect a worker's completed work

Run `/collect` (or `/collect <task_id>`) to merge a finished worker's branch to main, tear down the worktree, and transition the ESM task to done. This is the counterpart to `/spawn`.

## Procedure

### 1. Verify coordinator state

Run `git branch --show-current`. You must be on `main` (or the project's base branch). If not, warn the user and abort.

### 2. Identify the task

If a task_id was provided as an argument, use that. Otherwise:

Run via Bash:
```bash
esm worktree list
```

This lists worktrees annotated with ESM task status. Look for tasks in `in_review` (ready to collect) or `in_progress` (still working).

- If exactly one task is in `in_review`, use that
- If multiple are in `in_review`, list them and ask which one to collect
- If none are in `in_review`, check for `in_progress` and warn the worker may not be finished

### 3. Verify ESM criteria

Run via Bash:
```bash
esm task get <task_id>
```

Check:
- **All acceptance criteria are satisfied** — if any are not, list the unsatisfied ones and ask the user if they want to proceed anyway
- Note the task title for the merge commit message

### 4. Pre-merge conflict check

Run via Bash:
```bash
esm worktree check <task_id>
```

If conflicts are detected, report them and ask the user how to proceed.

### 5. Merge and clean up

Run via Bash:
```bash
esm worktree merge <task_id> --no-ff
```

This handles everything in one step:
- Merges the task branch to main with `--no-ff`
- Force-removes the worktree (discards .esm/, .mcp.json, .claude/ artifacts)
- Deletes the task branch

If the merge fails due to conflicts, report them and tell the user to resolve manually, then run `/collect` again.

### 6. Transition to done

The ESM workflow requires `in_progress → in_review → done`. If the task is still in `in_progress`, transition through both steps.

Get the merge commit hash:
```bash
git rev-parse HEAD
```

If task is in `in_progress`, first transition to `in_review`:
```bash
esm task transition <task_id> in_review --agent primary \
  --attest tests_passing=true \
  --attest implementation_complete=true
```

Then transition to `done`:
```bash
esm task transition <task_id> done --agent primary \
  --attest review_complete=true \
  --attest merged_to_main=true \
  --attest merge_commit=<hash>
```

### 7. Sync queue-plan + workstream state (state-sync — do this even for a paused queue)

If the collected task is a **primitive batch (PB-*) or any item drawn from a ranked queue
plan**, the collect step MUST update the coordination state, because the worker's worktree
had *old skill copies* and its edits only touched its own branch — the queue-plan banner and
`memory/workstream-state.md` live on main and are the coordinator's to update.

**Root cause this prevents** (`memory/doc-audit-2026-07-18b.md`, finding **N4**): PB-OS1 was
collected *during* a doc-remediation interlude that paused the queue. Its plan-closure step
was skipped, so `memory/primitives/oos-retriage-plan-2026-07-18.md` kept showing PB-OS1 as
"RECOMMENDED FIRST DISPATCH" / un-struck while OS2/OS3 carried ✅ banners — a live
**re-dispatch hazard** for the next dispatch loop. A paused or interrupted queue is exactly
when this bookkeeping gets dropped, so it is mandatory here, not optional.

For a PB/queue collection, after the merge:

1. **Active queue-plan file** (the dated file under `memory/primitives/` named in CLAUDE.md
   "Current State"): strike/✅-mark the collected batch's row in its "Queue summary" table,
   flip its §-section header to `✅ SHIPPED` with the merge/task ref, and **remove any
   "RECOMMENDED FIRST DISPATCH" / "next" banner** so the next dispatch can't re-pick it.
2. **`memory/workstream-state.md`**: update the W6 row / Last Handoff to name the shipped
   batch and the next queue target.
3. **CLAUDE.md "Current State"**: make the leading active-queue bullet (~line 17) agree with
   the queue — don't leave it a generation stale against the same section (audit #2 N1).
4. If card defs changed, regenerate `docs/authoring-status.md` via
   `python3 tools/authoring-report.py`. Never edit `docs/project-status.md` (RETIRED).

If a full `/eot` will run right after this collection it rotates `workstream-state.md` for
you — but the queue-plan banner (item 1) is never touched by `/eot`, so always do item 1 here.

### 8. Update project knowledge

Same as `/done`: check if this task established anything that future sessions should know. If so, update CLAUDE.md. If nothing new, skip.

### 9. Report

```
## Task collected

**Task**: {task_id} — {title}
**Merge commit**: {hash}
**Worktree**: removed
**Branch**: {branch} (deleted)

### Criteria satisfied
{list with checkmarks}

### Next
{suggest what to work on next — run `esm task list --project <project_id>` for remaining backlog,
 or `esm worktree list` to check other active worktrees}
```

## Notes

- If the task is already `done`, report that and do nothing.
- The `--no-ff` flag ensures a merge commit is created, keeping the branch history visible.
- After collecting, run `esm worktree list` to show remaining active worktrees.
- **Never skip step 7 for a PB/queue collection**, even mid-pause. The worker's worktree
  carried old skill copies and could only edit its own branch; the queue-plan banner and
  `workstream-state.md` on main are the coordinator's job. Dropping this is audit #2 N4.
