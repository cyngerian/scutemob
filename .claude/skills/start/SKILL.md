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

This returns active tasks, recent activity, and last session's handoff summary.

If the server is unreachable, warn the user and continue with local-only context (git log, file reads).

### 3. Start a session

Run via Bash:
```bash
esm session start --project <project_id> --agent primary
```

The CLI records the returned `session_id` in `.esm/session`, so it is sent as a
heartbeat on every later `esm` call (the session no longer auto-expires while
you work) and `esm session end`/`heartbeat` default to it — you don't need to
pass it by hand. Note it in your context anyway as a fallback.

### 4. Check local state

Run these in parallel:
- `git status` — uncommitted changes
- `git branch` — confirm current branch
- `git log --oneline -10` — recent commits
- `esm worktree list` — check for active worktrees with ESM task status

### 4a. Check for provisioned-file drift

Run via Bash:
```bash
esm doctor --dir . --human
```

Look at the `skills` check. If it reports missing skills, ESM-provisioned files
have been lost from this project — most often deleted by a worker branch and
merged without anyone noticing. Tell the user which skills are missing and offer
to restore them with `esm update`. Then have them commit the restoration, or the
next merge will silently drop them again.

Other failing checks are informational at session start — mention them, don't act
on them unless the user asks.

### 5. Orient and report

Report to the user in this format:

```
## Session started

**Project**: {name} ({project_id})
**Branch**: {branch}
**Session**: {session_id}

### Context from ESM
{summary from bootstrap: active tasks, recent activity}

### Local state
- Uncommitted changes: {yes/no, brief summary}
- Recent commits:
  {last 5 relevant commits}

### Active worktrees
{output from esm worktree list, if any}
{if none, omit this section}

### Provisioned files
{if esm doctor's skills check failed: list the missing skills and say
 `esm update` restores them}
{if it passed, omit this section entirely}

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
- Heartbeats ride on every authenticated `esm` call automatically — but only once a session has been started with `esm session start`, which records the session id in `.esm/session` for the CLI to attach as the `X-ESM-Session-Id` header. The file is found by walking up from the current directory (like git finds `.git`), so calls from anywhere inside the project heartbeat fine. A session started before that plumbing existed has no `.esm/session` at all and won't heartbeat on its own; run `esm session heartbeat` explicitly, or start a fresh session.
- If this is the first session for the project, bootstrap will return empty context. That's normal — start by creating tasks.
