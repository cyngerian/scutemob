---
name: task
description: Create a task and start working on it
user-invocable: true
allowed-tools: Read, Bash
argument-hint: "<title>"
---

# Create a task and start working on it

Run `/task <title>` to create an ESM task, set up a feature branch, and transition to `in_progress` in one step. **This is for self-assigned work** — you will implement the task yourself on the feature branch. For delegating work to a worker agent, use `/dispatch` instead.

## Procedure

### 1. Get project context

Read the ESM Project ID from the project's `CLAUDE.md` (under "Project Info").

### 2. Clarify the task

If the user provided a title but no details, ask what the acceptance criteria should be. If the context is clear enough to infer criteria, propose them and proceed.

Good acceptance criteria are specific and verifiable:
- "GET /api/schedule returns 7-day game schedule" (not "add schedule endpoint")
- "Tests cover empty and populated responses" (not "add tests")

**Always include a testing criterion.** Every task should have at least one criterion
that requires tests — e.g., "Tests cover happy path and error cases for {feature}."

### 3. Create the task

Run via Bash:
```bash
esm task create --project <project_id> --title "<title>" --criteria "<criterion 1>" --criteria "<criterion 2>"
```

Note the returned `task_id`.

### 4. Create a feature branch

Derive a branch name from the task title:
- Lowercase, hyphens for spaces, strip special characters
- Prefix with `feat/`, `fix/`, or `refactor/` as appropriate
- Keep it short: `feat/schedule-endpoint`

Run `git checkout -b <branch-name>` from the appropriate base branch (usually `main`).

### 5. Transition to in_progress

Run via Bash:
```bash
esm task transition <task_id> in_progress --agent primary \
  --attest branch_exists=true \
  --attest acceptance_criteria_defined=true \
  --attest working_branch=<branch-name>
```

### 6. Report

```
## Task ready

**Task**: {task_id} — {title}
**Branch**: {branch-name}
**Status**: in_progress

### Acceptance criteria
{numbered list of criteria}

Ready to implement.
```

## Notes

- If there's already an active task in `in_progress`, warn the user. They may want to finish or pause it first.
- If the branch already exists (e.g., resuming work), check it out instead of creating a new one.
- The user can skip criteria by saying something like "just start, I'll define criteria later" — in that case, create the task with a single generic criterion and note that it should be refined.
