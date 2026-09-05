---
name: remedy
description: Initialize an SR remediation session — read the plan, pick the next SR task, dispatch a worker
user-invocable: true
allowed-tools: Read, Grep, Bash
argument-hint: "[SR-<N> | status]"
---

# SR Remediation Session

Run `/remedy` to start a coordinator session for the SR remediation track
(senior-review findings, tasks `scutemob-53`..`scutemob-62`). This track runs
**outside** the `/start` / `/eot` skills and the W1–W6 workstream system.

The authoritative protocol is `docs/sr-remediation-plan.md`. This skill
orchestrates it: orient → select the next task → dispatch a worker → monitor →
collect → log. The coordinator does NOT implement SR tasks inline (same rule as
`/dispatch` — trivial fixes under 10 lines excepted).

Arguments:
- `/remedy` — full flow: orient, pick next task per the plan's sequencing, dispatch
- `/remedy SR-<N>` — skip selection; dispatch that specific task (still run the
  collision check and refuse if it fails)
- `/remedy status` — report only: SR task statuses, session log tail, what's next;
  dispatch nothing

## Procedure

### 1. Read the plan

Read `docs/sr-remediation-plan.md` in full. It contains the task inventory with
sequencing, hard constraints, collision rules, per-task gotchas, verification
gates, and the Session Log (newest-first handoffs). Do not proceed from memory
of this skill alone — the doc is updated between sessions and wins on conflict.

### 2. Orient

1. Confirm clean state: `git status` clean on `main`, `git pull` current.
2. **Collision check (mandatory):** read `memory/workstream-state.md`
   *read-only*. If any W6 card-authoring or other worker session is active:
   - Wide-blast-radius tasks (SR-3, SR-6, SR-7) are **blocked** — do not
     dispatch them; pick another SR task or report and stop.
   - Other SR tasks may proceed.
3. Start the ESM session with the track's coordinator identity:
   ```bash
   esm session start --project scutemob --agent sr-coordinator
   ```
4. Get SR task state:
   ```bash
   esm task list --project scutemob
   ```
   Filter to `SR-` titled tasks (scutemob-53..62 plus any SR tasks added since).

### 3. Handle in-flight work first

Before dispatching anything new, in this order:

- Any SR task **in_review** → collect it now via the `/collect` procedure
  (worktree check → merge → done transition), then append its Session Log entry
  (step 6) before moving on.
- Any SR task **in_progress** → a worker may already be running. Check for a
  live kitty tab (`kitty @ ls`, tab title `worker: {task_id}`) and recent task
  comments. If the worker is alive, resume the monitoring loop (step 5) instead
  of dispatching a duplicate. If it is dead/stale, tell the user what you found
  and ask before re-dispatching — do not silently double-assign.
- Any SR task **blocked** → report it; unblocking needs admin approval.

### 4. Select and dispatch the next task

**Selection:** explicit `/remedy SR-<N>` argument wins. Otherwise take the
lowest-numbered SR task still in `backlog`, honoring the plan's hard
constraints (SR-1 first; SR-2 before card-authoring resumes; SR-8 before M10
work) and the collision rules from step 2. SR-9 must be split into 2–3 subtasks
at dispatch time per the plan — create the subtasks (SR-9a/9b/9c titles, `SR-`
prefix) and dispatch the first rather than dispatching scutemob-61 whole.

**Dispatch:** follow the `/dispatch` skill's procedure with these deviations:

- **Skip task creation** — the ESM task already exists. Start at worktree
  creation: `esm worktree create <task_id>` (capture the absolute `worktree`
  path), transition to `in_progress` with the standard attestations
  (`branch_exists=true`, `acceptance_criteria_defined=true`,
  `working_branch=<branch from worktree create>`), agent `sr-coordinator`,
  then unlock.
- **Worker prompt** — use the `/dispatch` kitty launch command, but replace the
  worker prompt with:

  > Read `.esm/worker.md` and follow its task/acceptance criteria. THEN read
  > `docs/sr-remediation-plan.md` — you are working the SR remediation track;
  > its verification gates, per-task gotchas, and conventions are binding.
  > Use commit prefix `SR-<N>:`. Do NOT modify `memory/workstream-state.md`.
  > Before implementing, use TaskCreate to build a visible task list derived
  > from the acceptance criteria (one item per concrete step, including the
  > verification gates: cargo test --all, clippy --all-targets -D warnings,
  > fmt --check, cargo build --workspace) and keep it updated live. Delegate
  > to specialized project agents via the Agent tool where one fits (see the
  > Agents table in CLAUDE.md); implement directly otherwise. Satisfy every
  > acceptance criterion via `esm task satisfy` before signaling ready, run
  > /review for larger tasks (the plan says which), then follow the
  > Completion Sequence.

- Report the dispatch to the user in the `/dispatch` format.

### 5. Monitor and collect

Run the `/dispatch` background polling loop (10-minute Bash timeout, restart on
timeout — this is expected, not an error; state file survives restarts). When
the task reaches `in_review`, `/collect` it.

### 6. End-of-session bookkeeping (replaces /eot — do NOT run /eot)

After each collect (or when stopping for the session):

1. Append one entry to the **Session Log** in `docs/sr-remediation-plan.md`
   (newest first, format specified there): date, task, outcome, hazards
   discovered, pointer for the next session.
2. Update the plan's inventory table if sequencing knowledge changed.
3. Update CLAUDE.md "Current State" **only** for snapshot-material changes
   (CI live, card-def gating, crate layout) — routine SR progress stays out.
4. Do **not** touch `memory/workstream-state.md`.
5. `esm session end` (or let the idle timeout close it).

## Notes

- One SR task in flight at a time by default. The wide-blast tasks (SR-3/6/7)
  must never run concurrently with each other or with card-authoring workers.
- If a worker discovers a new problem, it creates a new `SR-`-prefixed ESM task
  rather than expanding scope; surface these to the user at collect time.
- If kitty remote control is unavailable, fall back to reporting the manual
  worker launch command, as `/spawn` does.
