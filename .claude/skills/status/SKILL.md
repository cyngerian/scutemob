---
name: status
description: Show project and fleet status from ESM
user-invocable: true
allowed-tools: Read, Bash
---

# Show project and fleet status from ESM

Run `/status` for a quick snapshot of your project's tasks, active sessions, and fleet-wide context.

## Procedure

### 1. Get project context

Look for the ESM Project ID in the project's `CLAUDE.md` (under "Project Info").

### 2. Fetch status

Run these via Bash:
```bash
esm task list --project <project_id> --human
esm fleet status --human
esm worktree list --human
esm session list --project <project_id> --human
```

### 3. Report

```
## Project: {name} ({project_id})

### Tasks
| Task | Status | Criteria |
|------|--------|----------|
{each task with id, title, state, and criteria satisfied/total}

### Dispatched workers
{for each task in `in_progress` or `in_review` that has an active worktree:}
| Task | Status | Criteria | Signal |
|------|--------|----------|--------|
{task_id, title, satisfied/total criteria, "ready for collection" if in_review or "working" if in_progress}

If any tasks are in `in_review` state, highlight them:
**Ready for collection**: {task_id} — run `/collect {task_id}` to merge.

### Active worktrees
{from esm worktree list, or "None"}

### Active sessions
{from esm session list, or "None"}

### Alerts
{from fleet status for this project, or "None"}

### Fleet overview
{one-line-per-project summary from fleet status: name, priority, active sessions, stale flag}
```

## Notes

- This is read-only — it doesn't modify any state.
- If the ESM server is unreachable, fall back to showing local git state (`git status`, `git log --oneline -5`, current branch).
- The fleet overview helps the user see what else is happening across their projects. Keep it brief.
- **Watchdog**: if a dispatched worker's task has been in `in_progress` for a long time and the session heartbeat is stale, warn: "Worker for {task_id} may be stuck."
