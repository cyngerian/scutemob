---
name: crew
description: Spawn a task with a multi-agent coordinator (context-primer, test-writer, architect, implementer, reviewer, fixer, gate-runner) running in a kitty pane. Parallel alternative to /dispatch.
user-invocable: true
allowed-tools: Read, Write, Bash
argument-hint: "<title>"
---

# Dispatch a crew

Run `/crew <title>` to create an ESM task, a git worktree, install the crew's agent definitions, then launch an Opus **coordinator** in a new kitty tab. The coordinator orchestrates seven single-shot subagents (context-primer, test-writer, architect, implementer, reviewer, fixer, gate-runner) instead of doing all the work on Opus itself.

This is a parallel alternative to `/dispatch`. Both skills coexist — use `/crew` for the division-of-labor pattern; `/dispatch` for the single-worker pattern.

## Procedure

### Steps 1–7: Same as /spawn

Follow steps 1–7 of `/spawn`:

1. Get project context from `CLAUDE.md` (ESM project ID, agent ID).
2. Verify coordinator is on `main` (or the project's base branch). Abort if not.
3. Clarify the task and acceptance criteria. **Always include a testing criterion.**
4. Create the ESM task: `esm task create --project <project_id> --title "<title>" --criteria ...`. Capture the returned `task_id`.
5. Create the worktree: `esm worktree create <task_id>`. **Capture the absolute `worktree` path from the JSON response** — you need it verbatim in step 8.
6. Transition to in_progress: `esm task transition <task_id> in_progress --agent primary --attest branch_exists=true --attest acceptance_criteria_defined=true --attest working_branch=<branch>`.
7. Release the lock: `esm task unlock <task_id> --agent primary`.

### Step 7.5 — Install crew agents and coordinator instructions

`esm worktree create` installed `.esm/worker.md` and replaced coordinator skills with worker skills (only `/review`). Now layer on the crew-specific files.

**Resolve the ESM client source directory.** Try the Python-installed location first, fall back to the repo path:

```bash
ESM_CLIENT="$(python3 -c 'import esm, os; print(os.path.join(os.path.dirname(esm.__file__), "..", "client"))' 2>/dev/null)"
if [ ! -d "$ESM_CLIENT/worker-agents" ]; then
  ESM_CLIENT="/home/skydude/projects/esm/client"
fi
if [ ! -d "$ESM_CLIENT/worker-agents" ]; then
  echo "ERROR: cannot find crew agent sources" >&2; exit 1
fi
```

**Install agent definitions into `<worktree>/.claude/agents/`:**

```bash
AGENTS_SRC="$ESM_CLIENT/worker-agents"
AGENTS_DST="{worktree_abs}/.claude/agents"

mkdir -p "$AGENTS_DST"
cp -f "$AGENTS_SRC"/*.md "$AGENTS_DST"/
# Confirm seven agents installed (context-primer, test-writer, architect, implementer, reviewer, fixer, gate-runner)
test "$(ls "$AGENTS_DST" | wc -l)" = "7" || { echo "ERROR: expected 7 agents, got $(ls "$AGENTS_DST" | wc -l)" >&2; exit 1; }
```

**Write `.esm/crew.md` from the template, substituting task metadata:**

```bash
TEMPLATE="$ESM_CLIENT/worker-instructions/crew.md.template"
test -f "$TEMPLATE" || { echo "ERROR: crew.md.template missing at $TEMPLATE" >&2; exit 1; }

sed \
  -e "s|{task_id}|$TASK_ID|g" \
  -e "s|{branch}|$BRANCH|g" \
  -e "s|{project_id}|$PROJECT_ID|g" \
  "$TEMPLATE" > "{worktree_abs}/.esm/crew.md"
```

Where `$TASK_ID`, `$BRANCH`, and `$PROJECT_ID` come from earlier steps.

### Step 8 — Launch the coordinator

Same kitty mechanics as `/dispatch`. **`--cwd` MUST be absolute** — relative paths resolve to `$HOME`, not the caller's cwd (see commit 5640e3c).

Verify prerequisites:
```bash
test -d "{worktree_abs}/.esm" && echo ok || { echo "worktree .esm missing"; exit 1; }
test -f "{worktree_abs}/.esm/crew.md" && echo ok || { echo "crew.md missing"; exit 1; }
test -d "{worktree_abs}/.claude/agents" && echo ok || { echo "agents dir missing"; exit 1; }
```

Test kitty availability:
```bash
kitty @ ls >/dev/null 2>&1
```

Launch:
```bash
kitty @ launch --type=tab --tab-title "crew: {task_id}" --keep-focus --cwd "{worktree_abs}" -- bash -c 'export PATH="$HOME/.local/bin:$PATH" ESM_API_KEY="'"$ESM_API_KEY"'" ESM_URL="'"$ESM_URL"'"; claude --model opus[1m] --dangerously-skip-permissions "Read .esm/crew.md and follow its instructions. Coordinate the crew to implement the task. Do not edit files yourself — delegate all work to the agents listed in .esm/crew.md."; exec bash'
```

Confirm cwd post-launch:
```bash
kitty @ ls | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); [print(w['id'], t.get('title'), w.get('cwd')) for o in d for t in o.get('tabs',[]) for w in t.get('windows',[]) if 'crew: {task_id}' in (t.get('title','')+w.get('title',''))]"
```

If cwd is wrong: `kitty @ close-window --match id:<id>` and retry.

If `kitty @` is unavailable, fall back to reporting the manual launch command:
```
cd {worktree_abs} && claude --model opus[1m] --dangerously-skip-permissions "Read .esm/crew.md and follow its instructions..."
```

Notes on the launch command:
- `--type=tab` opens a new tab; does not affect existing layouts.
- `--keep-focus` prevents the new tab from stealing focus.
- `--tab-title "crew: ..."` distinguishes crew tabs from `worker: ...` tabs for dispatched workers.
- The prompt is passed as a positional argument (not `-p`) for interactive streaming.
- `; exec bash` keeps the tab open after Claude exits.

### Step 9 — Report

```
## Crew dispatched

**Task**: {task_id} — {title}
**Branch**: {branch}
**Worktree**: .worktrees/{task_id}/
**Status**: Coordinator launched in kitty tab "crew: {task_id}"
**Crew**: context-primer, test-writer, architect, implementer, reviewer, fixer, gate-runner

### Acceptance criteria
{numbered list}

### Monitoring
Watching for task to reach `in_review`. Will notify when ready to collect.
Use `/status` to check progress, or `/collect {task_id}` to collect manually.
```

### Step 10 — Wait for completion

Run the polling loop in the background with `run_in_background: true` and `timeout: 600000`.
Same shape as `/dispatch` — see `/dispatch` step 10 for full details on timeout handling.

```bash
TASKS="{task_id}"  # space-separated for multiple
STATE="/tmp/esm-dispatch-$$.ready"

while true; do
  ALL_READY=true
  for tid in $TASKS; do
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
  $ALL_READY && { echo "ALL TASKS READY"; rm -f "$STATE"; break; }
  sleep 30
done
```

The Bash tool has a **hard 10-minute maximum**. Crew tasks routinely take 30+ minutes.
**You MUST restart the loop when it times out** — do not ask the user, do not move on.
The state file tracks completed tasks across restarts.

If the crew pauses at the review cycle cap (Step 5 of crew.md), the task sits in `in_progress` with a coordinator comment explaining the findings. Check `esm task get` in that case. The polling loop will still be waiting for `in_review` — you'll need to either manually intervene in the worktree, `/collect` after fixing, or kill the task.

## Notes

- `/crew` and `/dispatch` coexist and can both be used in the same project without interference. Task and worktree infrastructure is identical.
- Expected cost profile vs `/dispatch`: lower for medium-sized tasks (most work is sonnet/haiku), possibly higher for trivial tasks (coordination overhead dominates).
- If `esm-cli` is ever updated to bake crew agents into `esm worktree create` directly, Step 7.5 becomes redundant and can be dropped. Until then the skill owns installation.
- Agent files in `<worktree>/.claude/agents/` are scoped to that worktree — they don't leak into the coordinator's project or other worktrees.
- The coordinator's context stays clean — no implementation detail leaks back. Only task state (criteria, comments, signal-ready) is visible via ESM.
