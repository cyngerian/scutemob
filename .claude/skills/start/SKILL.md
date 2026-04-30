---
name: start
description: Start an ESM-managed work session
user-invocable: true
allowed-tools: Read, Bash, Glob
---

# Start an ESM-managed work session

Begin every session by running `/start`. This bootstraps project context from ESM, starts a tracked session, and orients you on what to do next.

## Procedure

### 0. Detect context — portfolio root vs. project

Before anything else, check the current working directory:

```bash
pwd
```

**If `pwd` is exactly `/home/skydude/projects`** (the portfolio root, not a subdirectory):

This is NOT an ESM-managed project. Do NOT run `esm project bootstrap` or `esm session start` — there is no project ID at this level and both commands will fail.

Instead:
- Read `/home/skydude/projects/CLAUDE.md` to confirm portfolio-root context and review routing guidance.
- If the user's prompt suggests a cross-project scan ("progress", "check-in", "what's changed", "portfolio update", "/report"), follow the procedure in auto-memory `portfolio-checkin.md` (7 deterministic steps).
- If the user's intent is unclear, ask whether they want (a) a portfolio check-in, or (b) to enter a specific project session. Do not guess.
- Skip steps 1–6 below. They are project-scoped.

**Otherwise (you are inside a specific project directory)**, continue with step 1.

### 1. Read project config

Look for the ESM Project ID in the project's `CLAUDE.md` (under "Project Info"). You need this for all `esm` commands.

### 2. Bootstrap from ESM

Run via Bash:
```bash
esm project bootstrap <project_id>
```

This returns active tasks, recent activity, pending approvals, alerts, and last session's handoff summary.

If the server is unreachable, warn the user and continue with local-only context (git log, file reads).

### 3. Start a session

Run via Bash:
```bash
esm session start --project <project_id> --agent primary
```

Save the returned `session_id` — you need it for `esm session end` later.

### 4. Check local state

Run these in parallel:
- `git status` — uncommitted changes
- `git branch` — confirm current branch
- `git log --oneline -10` — recent commits
- `esm worktree list` — check for active worktrees with ESM task status

### 5. Orient and report

Report to the user in this format:

```
## Session started

**Project**: {name} ({project_id})
**Branch**: {branch}
**Session**: {session_id}

### Context from ESM
{summary from bootstrap: active tasks, recent activity, any alerts}

### Local state
- Uncommitted changes: {yes/no, brief summary}
- Recent commits:
  {last 5 relevant commits}

### Active worktrees
{output from esm worktree list, if any}
{if none, omit this section}

### Documentation
{if .claude/docs.yaml exists, read it and check each template:}
  {count} docs configured, {count} current, {count} stale, {count} missing
  {if any stale: list the stale ones with one-line reason}
{if .claude/docs.yaml does not exist:}
  No docs config found. Run `/docs init` to set up documentation management.
{This is informational only — no prompts, no blocking. Mention it and move on.}

### Suggested next steps
{based on task states and bootstrap context — what should the agent work on}
```

### 6. Recreate task visibility

If the bootstrap response includes active tasks (in_progress or in_review), mention them explicitly so the user knows what's in flight.

## Notes

- If `esm project bootstrap` fails (server unreachable), warn the user and continue with local-only context.
- Heartbeats happen automatically on authenticated requests to the server.
- If this is the first session for the project, bootstrap will return empty context. That's normal — start by creating tasks.
