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

### 7. Update project knowledge

Same as `/done`: check if this task established anything that future sessions should know. If so, update CLAUDE.md. If nothing new, skip.

### 8. Report

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
