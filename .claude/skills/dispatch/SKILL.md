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

Instead of reporting "launch the worker" to the user, launch it directly in a kitty pane.

**CRITICAL: `--cwd` MUST be an absolute path.** `kitty @ launch --cwd` resolves
relative paths against the kitty process cwd (typically `$HOME`), NOT the cwd of
the calling shell or the focused pane. A relative path like `.worktrees/{task_id}`
will launch the worker in the wrong directory (usually `$HOME`) and it will start
trashing files there. Use the absolute `worktree` path returned by `esm worktree
create` in step 5.

Verify the absolute path exists before launching:
```bash
test -d "{worktree_abs}/.esm" && echo ok || { echo "worktree missing"; exit 1; }
```

Then launch:
```bash
kitty @ launch --type=tab --tab-title "worker: {task_id}" --keep-focus --cwd "{worktree_abs}" -- bash -c 'export PATH="$HOME/.local/bin:$PATH" ESM_API_KEY="'"$ESM_API_KEY"'" ESM_URL="'"$ESM_URL"'"; claude --model opus[1m] --dangerously-skip-permissions "Read .esm/worker.md and follow its instructions. BEFORE you start implementing, use TaskCreate to build a visible task list derived from the acceptance criteria and any referenced plan file — one item per concrete step (enum add, each dispatch site, each card-def edit, each test, build/clippy/fmt checks, /review). Mark each item in_progress when you start it and completed as soon as it is done (do not batch completions at the end). The coordinator follows this task list to track progress. THEN delegate the heavy lifting to specialized project agents via the Agent tool rather than implementing everything inline: primitive batches (PB-*) use primitive-impl-runner for implementation and primitive-impl-reviewer for review; keyword abilities use ability-impl-runner + ability-impl-reviewer; card authoring uses bulk-card-author + card-batch-reviewer; LOW issue fix sessions use fix-session-runner; game scripts use game-script-generator. See the Agents table in CLAUDE.md. Only implement directly when no specialized agent fits the work. Satisfy all acceptance criteria, run /review (spawning the review agent if one fits), then follow the Completion Sequence."; exec bash'
```

Where `{worktree_abs}` is the absolute path returned in the `worktree` field of
`esm worktree create`'s JSON response (e.g.
`/home/skydude/projects/garden69/.worktrees/garden69-150`).

After launching, confirm the new window's cwd matches:
```bash
kitty @ ls | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); [print(w['id'], t.get('title'), w.get('cwd')) for o in d for t in o.get('tabs',[]) for w in t.get('windows',[]) if 'worker: {task_id}' in (t.get('title','')+w.get('title',''))]"
```
If the cwd is not the worktree path, close the window with
`kitty @ close-window --match id:<id>` and retry.

Notes on the launch command:
- `--type=tab` opens a new tab — does NOT affect existing tabs or split layouts
- `--keep-focus` prevents the new tab from stealing focus from the coordinator
- `--tab-title` labels it with the task ID for easy identification
- `--cwd` sets the working directory to the worktree (absolute path only)
- `--dangerously-skip-permissions` allows autonomous operation
- `--model opus[1m]` matches the user's standard model config
- The prompt is passed as an argument (NOT `-p`), for interactive streaming output
- `; exec bash` keeps the tab open after Claude exits

If `kitty @` is not available (not running in kitty, or remote control disabled), fall
back to reporting the manual launch command as `/spawn` does.

Test kitty availability first:
```bash
kitty @ ls >/dev/null 2>&1
```

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

## Notes

- Multiple `/dispatch` calls create multiple workers. The coordinator decides whether
  to dispatch sequentially or in parallel.
- The worker runs interactively with the prompt as an argument. It executes autonomously
  and exits when done. `exec bash` keeps the tab open for inspection.
- If the worker fails or gets stuck, the kitty tab remains open. Check via `esm task get`.
- The coordinator's context stays clean — no implementation detail leaks back. Only
  the task state (criteria, comments, signal ready) is visible via ESM.
