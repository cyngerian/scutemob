---
name: dispatch
description: Spawn a task and auto-launch a worker agent in a kitty pane
user-invocable: true
allowed-tools: Read, Bash
argument-hint: "<title>"
---

# Dispatch a worker

Run `/dispatch <title>` to create an ESM task, a git worktree, launch a worker Claude
session in a new kitty terminal pane, and begin monitoring for completion. This is the
automated version of `/spawn` — instead of telling the user to launch a worker manually,
the coordinator launches it directly.

## Procedure

### Steps 1–7: Same as /spawn

Follow the exact same procedure as `/spawn` (steps 1 through 7):
1. Get project context from CLAUDE.md
2. Verify coordinator is on main
3. Clarify the task and acceptance criteria
4. Create the ESM task (`esm task create`)
5. Create the worktree (`esm worktree create <task_id>`) — **capture the absolute
   `worktree` path from the JSON response**; you need it verbatim in step 8
6. Transition to in_progress (`esm task transition`)
7. Release the lock (`esm task unlock`)

### 8. Launch the worker

Instead of reporting "launch the worker" to the user, launch it directly via the
`esm worker-tab` CLI command (esm-21). It opens a split kitty tab — worker session on
the left, a live glance pane (`esm task glance`) on the right — and handles tab
titling, cwd verification, retry, and a manual-instructions fallback when kitty
remote control is unavailable.

`{worktree_abs}` is the absolute path returned in the `worktree` field of
`esm worktree create`'s JSON response in step 5 — pass it verbatim.

```bash
esm worker-tab {task_id} "{worktree_abs}" --prompt 'Read .esm/worker.md and follow its instructions. BEFORE you start implementing, use TaskCreate to build a visible task list derived from the acceptance criteria and any referenced plan file — one item per concrete step (enum add, each dispatch site, each card-def edit, each test, build/clippy/fmt checks, /review). Mark each item in_progress when you start it and completed as soon as it is done (do not batch completions at the end). The coordinator follows this task list to track progress. THEN delegate the heavy lifting to specialized project agents via the Agent tool rather than implementing everything inline: primitive batches (PB-*) use primitive-impl-runner for implementation and primitive-impl-reviewer for review; keyword abilities use ability-impl-runner + ability-impl-reviewer; card authoring uses bulk-card-author + card-batch-reviewer; LOW issue fix sessions use fix-session-runner; game scripts use game-script-generator. See the Agents table in CLAUDE.md. Only implement directly when no specialized agent fits the work. Satisfy all acceptance criteria, run /review (spawning the review agent if one fits), then follow the Completion Sequence.'
```

The `--prompt` value above is this project's customized worker prompt (task-list
discipline + the specialized-agent roster) — keep it in sync with the Agents table in
CLAUDE.md, and do not drop it in favor of the stock prompt: `esm update` skips this
skill precisely because of that customization (see `.esm/migration.json`), and
`esm update --force` would clobber it.

Check the command's JSON output: `cwd_verified` must be `true`. If the command reports
kitty remote control unavailable, relay its manual launch instructions to the user as
`/spawn` does.

### 9. Report and begin monitoring

Report to the user:

```
## Worker dispatched

**Task**: {task_id} — {title}
**Branch**: {branch}
**Worktree**: .worktrees/{task_id}/
**Status**: Worker launched in kitty tab "worker: {task_id}"

### Acceptance criteria
{numbered list}

### Monitoring
Watching for task to reach `in_review`. Will notify when ready to collect.
Use `/status` to check progress, or `/collect {task_id}` to collect manually.
```

### 10. Wait for completion

After dispatching one or more workers, enter an autonomous monitoring loop.
Do NOT ask the user to check manually — handle it yourself.

**Run the polling loop in the background** using the Bash tool's `run_in_background: true`
parameter and `timeout: 600000` (the 10-minute maximum). This keeps the coordinator free
to handle user interactions while waiting.

For each batch of dispatched tasks, run this bash polling loop:

```bash
# Poll dispatched tasks until all reach in_review or done
TASKS="{task_id_1} {task_id_2} {task_id_3}"  # space-separated dispatched task IDs
STATE="/tmp/esm-dispatch-$$.ready"

while true; do
  ALL_READY=true
  for tid in $TASKS; do
    # Skip tasks already marked ready (survives timeout restarts)
    grep -q "^$tid " "$STATE" 2>/dev/null && continue
    status=$(esm task get $tid | python3 -c "import sys,json; t=json.loads(sys.stdin.read()); print(t.get('task',{}).get('current_status','unknown'))")
    if [ "$status" = "in_review" ] || [ "$status" = "done" ]; then
      echo "$tid $status" >> "$STATE"
      echo "READY: $tid ($status)"
    else
      ALL_READY=false
      echo "POLL: $tid ($status)"
    fi
  done
  if $ALL_READY; then
    echo "ALL TASKS READY"
    rm -f "$STATE"
    break
  fi
  sleep 30
done
```

Notes on the polling loop:
- **No `2>/dev/null`** — errors from `esm task get` must be visible, not swallowed.
  If the API is down, the error output tells you why. Silent failures cause missed transitions.
- **`POLL:` heartbeat lines** — printed every cycle so you can confirm the loop is alive.
  If you see no output for >60s, the loop died.
- **State file** (`/tmp/esm-dispatch-$$.ready`) — tracks which tasks already reached
  `in_review`/`done`. Survives timeout restarts: the new loop skips already-completed tasks
  without needing to parse previous stdout.

#### Timeout handling — THIS IS CRITICAL

The Bash tool has a **hard 10-minute maximum**. Workers routinely take 20-40 minutes.
The loop WILL time out. **This is expected, not an error.**

**You MUST restart the loop when it times out.** Do not ask the user. Do not move on to
other work. Do not forget. The background process completion notification is your cue —
when you receive it, IMMEDIATELY start a new polling loop for any tasks not yet collected.

Before restarting, read the state file to see what's already done:
```bash
cat /tmp/esm-dispatch-*.ready 2>/dev/null
```

Then restart the loop with the SAME task IDs. The state file ensures already-completed
tasks are skipped.

**If you are tempted to do something else instead of restarting the loop, don't.**
The user relies on you to monitor workers autonomously. A missed restart means the user
has to notice and prompt you manually, which defeats the purpose of `/dispatch`.

When the loop exits with "ALL TASKS READY", `/collect` each task that is in `in_review`,
then proceed to dispatch the next wave.

If a task stays in `in_progress` for over 30 minutes with no criteria progress,
warn the user that the worker may be stuck — but don't stop the loop.

## Collecting dispatched workers

When a dispatched worker signals ready (task is in `in_review`), the coordinator can
run `/collect {task_id}` as normal. The `/collect` skill handles:
- Pre-merge conflict check (`esm worktree check`)
- Merging and cleanup (`esm worktree merge`)
- Transitioning to done (`esm task transition`)
- **State-sync (`/collect` step 7)** — for a PB/queue item, updating the active queue-plan
  banner + `memory/workstream-state.md` even when the queue is paused. This is mandatory:
  skipping it is the N4 re-dispatch hazard from `memory/doc-audit-2026-07-18b.md` (a shipped
  PB left showing "RECOMMENDED FIRST DISPATCH" gets re-picked by the next dispatch loop).

## Notes

- Multiple `/dispatch` calls create multiple workers. The coordinator decides whether
  to dispatch sequentially or in parallel.
- The worker runs interactively with the prompt as an argument. It executes autonomously
  and exits when done. `exec bash` keeps the tab open for inspection.
- If the worker fails or gets stuck, the kitty tab remains open. Check via `esm task get`.
- The coordinator's context stays clean — no implementation detail leaks back. Only
  the task state (criteria, comments, signal ready) is visible via ESM.
