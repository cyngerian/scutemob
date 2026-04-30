---
name: done
description: Complete a task — transition to done, merge branch to main, clean up
user-invocable: true
allowed-tools: Read, Bash
argument-hint: "[task_id]"
---

# Complete a task

Run `/done` (or `/done <task_id>`) to finalize a task: transition it to `done`, merge the feature branch to main, and clean up.

## Procedure

### 1. Identify the task

If a task_id was provided as an argument, use that. Otherwise:

Run via Bash:
```bash
esm task list --project <project_id> --status in_review
```

- If exactly one task is in_review, use that
- If multiple tasks are in_review, ask the user which one
- If no tasks are in_review, check for `in_progress` tasks and warn that they should be reviewed first

### 2. Verify readiness

Run via Bash:
```bash
esm task get <task_id>
```

Check:
- **All acceptance criteria are satisfied** — if any are not, list the unsatisfied ones and ask the user if they want to proceed anyway
- **Task is in `in_review` state** — if it's in `in_progress`, offer to transition through `in_review` first

### 3. Ensure clean state

Run `git status`. If there are uncommitted changes:
- Warn the user
- Ask if they want to commit first
- Do not proceed until working tree is clean or user explicitly says to continue

### 4. Merge to main

Run these steps:

```bash
# Get the current feature branch name
git branch --show-current

# Switch to main and pull latest
git checkout main
git pull --ff-only 2>/dev/null || true

# Merge the feature branch
git merge --no-ff <feature-branch> -m "merge: <task_id> — <task_title>"

# Delete the feature branch
git branch -d <feature-branch>
```

If the merge has conflicts:
- Report the conflicting files to the user
- Do NOT force-resolve or abort — let the user decide how to handle it
- Stop the procedure and tell them to run `/done` again after resolving

### 5. Transition to done

Run via Bash:
```bash
esm task transition <task_id> done --agent primary \
  --attest review_complete=true \
  --attest merged_to_main=true \
  --attest merge_commit=<merge-commit-hash>
```

### 6. Update project knowledge

Check if this task established anything that future sessions should know — new conventions, dependencies, architectural patterns, or important decisions. If so, update CLAUDE.md. Don't add task-specific details or anything derivable by reading the code.

If nothing new was established, skip this step.

### 6b. Documentation check

If `.claude/docs.yaml` exists, check whether this task's changes affect any docs:

1. Run `git diff --name-only main..HEAD` to get the list of files changed in this task.
2. Read `.claude/docs.yaml` and match changed files against trigger patterns.
3. Filter to templates with `frequency: task` (or no frequency, since `task` is the default).
4. For each matched template:
   - If the doc was already updated in this task (check git diff for changes to the doc file), skip it.
   - Otherwise, report: "{doc} may need updating ({trigger file} changed)"
5. Update stale docs or note why no update is needed (e.g., "cosmetic change, no doc impact").
6. When updating a doc, update the `<!-- last_updated: YYYY-MM-DD -->` comment to today's date.

If `.claude/docs.yaml` doesn't exist, skip this step entirely.

This step is advisory. Use judgment about whether trigger-matched changes actually
affect doc content. A renamed variable in `app/api/routes.py` doesn't mean `api.md`
needs rewriting.

### 7. Report

```
## Task completed

**Task**: {task_id} — {title}
**Merge commit**: {hash}
**Branch**: {feature-branch} (deleted)

### Criteria satisfied
{list of all criteria with checkmarks}

### Next
{suggest what to work on next — run `esm task list --project <project_id>` for remaining backlog}
```

## Notes

- If the task is already `done`, report that and do nothing.
- The `--no-ff` flag ensures a merge commit is created even for fast-forward merges, keeping the branch history visible.
- If `git pull` fails (no remote, or no tracking branch), that's fine — just merge locally.
- After merging, stay on `main`. The next task will create a new feature branch via `/task`.
