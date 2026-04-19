---
name: spawn
description: Create a task and worktree for a worker agent
user-invocable: true
allowed-tools: Read, Bash
argument-hint: "<title>"
---

# Spawn a worker in a worktree

Run `/spawn <title>` to create an ESM task and a git worktree with a worker context file. The user then launches a separate Claude Code session in the worktree to do the implementation. The coordinator (you) stays on `main`.

## Procedure

### 1. Get project context

Read the project's `CLAUDE.md` for:
- **ESM Project ID** (under "Project Info")
- **Agent ID**

### 2. Verify coordinator state

Run `git branch --show-current`. You must be on `main` (or the project's base branch). If not, warn the user and abort.

### 3. Clarify the task

Same as `/task`: if the user provided a title but no details, ask for acceptance criteria. If the context is clear, propose criteria and proceed.

**Always include a testing criterion.**

### 4. Create the ESM task

Run via Bash:
```bash
esm task create --project <project_id> --title "<title>" --criteria "<criterion 1>" --criteria "<criterion 2>"
```

Note the returned `task_id`.

### 5. Create the worktree

Run via Bash:
```bash
esm worktree create <task_id>
```

This handles everything in one step:
- Derives branch name from task title
- Creates git worktree at `.worktrees/<task_id>/`
- Copies `.mcp.json` and `.claude/` into worktree
- Replaces coordinator skills with worker skills (review only)
- Writes `.esm/worker.md` with task details, criteria, and CLI-based instructions
- Excludes `.esm/` from git tracking

### 6. Transition to in_progress

Run via Bash:
```bash
esm task transition <task_id> in_progress --agent primary \
  --attest branch_exists=true \
  --attest acceptance_criteria_defined=true \
  --attest working_branch=<branch>
```

### 7. Release the lock

Transitioning to `in_progress` auto-locks the task to you (the coordinator). Release the lock so the worker can transition it later:

```bash
esm task unlock <task_id> --agent primary
```

### 8. Report

```
## Worker ready

**Task**: {task_id} — {title}
**Branch**: {branch}
**Worktree**: .worktrees/{task_id}/

### Acceptance criteria
{numbered list}

### Launch the worker
  cd .worktrees/{task_id} && claude

When launching, tell the worker to — BEFORE implementing — use TaskCreate to
build a visible task list from the acceptance criteria and any referenced plan
file (one item per concrete step), and to update each item's state live as it
progresses. The coordinator follows this list to track progress.

Also tell the worker to delegate the heavy lifting to specialized project
agents via the Agent tool rather than implementing inline: primitive batches
(PB-*) use `primitive-impl-runner` for implementation and
`primitive-impl-reviewer` for review; keyword abilities use
`ability-impl-runner` + `ability-impl-reviewer`; card authoring uses
`bulk-card-author` + `card-batch-reviewer`; LOW fix sessions use
`fix-session-runner`; game scripts use `game-script-generator`. See the Agents
table in CLAUDE.md. Only implement directly when no specialized agent fits.
```

## Notes

- If there's already an active task in `in_progress` with no worktree, warn — it may be a `/task`-created single-agent task.
- Multiple `/spawn` calls create multiple worktrees. This is expected for parallel work.
- If the worktree path already exists, report it and ask if the user wants to resume or start fresh.
