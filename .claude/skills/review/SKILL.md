---
name: review
description: Spawn an Opus reviewer to check your work against acceptance criteria
user-invocable: true
allowed-tools: Bash
---

# Review current work

Run `/review` to spawn an Opus subagent that reviews your implementation against the task's acceptance criteria. Commit all work before running this.

## Procedure

### 1. Ensure clean state

Run `git status`. If there are uncommitted changes:
- Warn the user: "You have uncommitted changes. Commit before review so the reviewer sees the full picture."
- Do not proceed until the working tree is clean or the user explicitly says to continue.

### 2. Gather context

Read `.esm/worker.md` for:
- Task ID, title
- Acceptance criteria (the checklist the reviewer will verify against)

Run `git log main..HEAD --oneline` to get the list of commits on this branch.

Run `git diff main..HEAD --stat` to get the list of changed files.

### 3. Spawn the reviewer

Use the Agent tool with `model: "opus"` and `subagent_type: "general-purpose"`. Give it this prompt:

```
You are a code reviewer. Review the implementation on this branch against the acceptance criteria below.

## Task
**ID**: {task_id}
**Title**: {title}

## Acceptance Criteria
{numbered list from worker.md}

## Commits on this branch
{output of git log main..HEAD --oneline}

## Changed files
{output of git diff main..HEAD --stat}

## Instructions
1. Read each changed file to understand the implementation.
2. For each acceptance criterion, assess whether it is met. Be specific — cite file names and line numbers.
3. **Check for tests.** Look for new or modified test files in the diff. If the implementation
   adds new functionality but no corresponding tests exist, flag this as a FAIL: "No tests
   written for new functionality." Run the test suite if a test command is apparent (e.g.,
   pytest, cargo test, npm test). Failing tests are a FAIL finding.
4. Check for:
   - Missing edge cases
   - Code that doesn't match the criterion's intent
   - Obvious bugs or regressions
   - Files that should have been changed but weren't

Report in this format:

### Criteria Assessment
| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
{one row per criterion: PASS, FAIL, or UNCLEAR with explanation}

### Issues
{numbered list of specific issues to fix, or "None found"}

### Summary
{one sentence: ready to merge, or what needs to change}
```

### 4. Report findings

When the reviewer returns, relay its report to the user. If there are issues:
- List them clearly
- Suggest fixes
- After fixing, the user can run `/review` again to verify

If all criteria pass and no issues found, report:
```
## Review passed

All {N} criteria satisfied. Ready — tell the user you're done.
```

## Notes

- The reviewer is read-only — it should not edit files. If it tries, that's a bug in the prompt.
- The reviewer runs as Opus regardless of what model the worker session uses.
- You can run `/review` multiple times — after fixing issues, run it again to confirm.
- If the project has a test command, the reviewer should run it. If tests fail, that's a finding.
