---
name: Use git worktrees for parallel sessions
description: Two Claude sessions sharing one working directory causes constant interference — use git worktrees to isolate
type: feedback
---

Parallel Claude sessions MUST use separate git worktrees, not just separate branches.

**Why:** Two sessions on separate branches but the same directory corrupt each other's working
tree. Uncommitted files from one session bleed into the other, stash/checkout operations
conflict, and edits get lost. This happened during W3-LC + W6-PB19 parallel work (2026-03-19).

**How to apply:**
- `git worktree add ../scutemob-w3 -b w3-branch` creates an isolated checkout
- Each session gets its own directory with its own branch
- Merge from the main directory: `git merge w3-branch`
- Clean up: `git worktree remove ../scutemob-w3 --force && git branch -d w3-branch`
- The W6 agent should be told to work on a feature branch too
- Tell agents their working directory explicitly when starting in a worktree
