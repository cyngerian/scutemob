---
name: end
description: End the current ESM work session
user-invocable: true
allowed-tools: Read, Bash, Glob
---

# End the current ESM work session

Run `/end` when you're done working. This records a session summary in ESM, checks for uncommitted work, and ensures continuity for the next session.

## Procedure

### 1. Gather session state

Run these in parallel:
- `git status` — check for uncommitted changes
- `git log --oneline -15` — see what was committed this session
- `git diff --stat` — if there are uncommitted changes, show what they are

### 2. Summarize the session

Review what was accomplished by looking at:
- Git commits made during this session
- Tasks transitioned or criteria satisfied (from your conversation context)
- Any blockers or issues encountered

Write a concise but specific summary. Bad: "worked on features". Good: "added MLB schedule endpoint, fixed roster sync timeout, 3 tests added".

### 3. Check for uncommitted work

If `git status` shows uncommitted changes:
- **Warn the user** — uncommitted work won't be captured in ESM or git history
- Ask if they want to commit before ending
- If they say no, note the uncommitted files in your session summary

### 3b. Check for active worktrees

Run `esm worktree list`. If there are worktrees:
- **Warn the user** — worker sessions may still be running in these worktrees
- List each worktree with its task ID, branch, and ESM status
- Do NOT remove them — the user decides whether to wait, collect, or leave them

### 3c. Documentation check

If `.claude/docs.yaml` exists, run a session-scoped doc check:

1. Get all files changed this session from git log (all commits since session start).
2. Read `.claude/docs.yaml` and match changed files against trigger patterns.
3. Filter to templates with `frequency: task` or `frequency: session`.
4. Exclude docs already updated this session (check git log for changes to the docs directory).
5. Report any stale docs. The user decides whether to update now or defer.

Keep this brief — `/end` should not become a lengthy review. Report findings in
2-3 lines and let the user decide.

If `.claude/docs.yaml` doesn't exist, skip this step entirely.

### 4. End the session

Run via Bash:
```bash
esm session end <session_id> --summary "..."
```

Use the `session_id` from when you ran `esm session start` (check your conversation context).

If you don't have the session_id (e.g., session was auto-started), skip this step and note it.

### 5. Report

```
## Session ended

### Completed this session
{bulleted list of accomplishments}

### Open tasks
{any tasks still in_progress or in_review, with brief status}

### Active worktrees
{list any active worktrees and their branches, or "None"}

### Uncommitted changes
{clean working tree, or list of uncommitted files}

### Next session
{what should be done next — specific, actionable}
```

## Notes

- Always try to end sessions cleanly. If the user just closes the terminal, the session will auto-expire after 10 minutes without a heartbeat, but the summary will be lost.
- The session summary is stored in ESM and returned by `esm project bootstrap` in the next session — this is how continuity works.
- If the ESM server is unreachable, warn the user but don't block. The git history still captures what happened.
