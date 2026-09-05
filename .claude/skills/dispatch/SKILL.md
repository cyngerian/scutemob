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
coordinator's one way to hand out implementation work: it launches the worker directly and
watches it with the Monitor tool.

## Procedure

### 1–7. Create the task and its worktree

(Formerly "same as `/spawn`"; `/spawn` was retired by CC-14. These steps are the whole recipe.)

1. **Project context**: ESM Project ID and Agent ID are under "Project Info" in `CLAUDE.md`.
2. **Coordinator state**: `git branch --show-current` must be `main`; otherwise abort. Any doc
   the brief will cite must already be COMMITTED on main (dispatch hygiene 2 — the worktree forks
   from main and cannot see untracked coordinator files). Land any `workstream-state.md` chore
   commit BEFORE the next step (dispatch hygiene 9).
3. **Clarify the task**: propose acceptance criteria if none were given. Always include a testing
   criterion. Name the **change class** (`memory/conventions.md` → "## Change-class acceptance table")
   so the criteria ask for that class's ritual and no more.
4. **Create the ESM task** — `--criteria` is REPEATABLE, one flag per criterion (dispatch hygiene 4;
   a pipe-joined string becomes one mega-criterion):
   ```bash
   esm task create --project <project_id> --title "<title>" --criteria "<c1>" --criteria "<c2>"
   ```
5. **Create the worktree**: `esm worktree create <task_id>`. Parse the JSON only from
   UNTRUNCATED output; capture the `worktree` (absolute path) and `branch` fields VERBATIM
   (dispatch hygiene 1/2/9). It writes `.esm/worker.md` with the criteria and the CLI recipe.
6. **Transition to in_progress** with the literal branch string from step 5:
   ```bash
   esm task transition <task_id> in_progress --agent primary \
     --attest branch_exists=true --attest acceptance_criteria_defined=true \
     --attest working_branch=<branch>
   ```
7. **Release the lock** so the worker can transition later: `esm task unlock <task_id> --agent primary`.

### 7b. Write the self-contained brief

Write `<worktree_abs>/.esm/brief.md`, **at most 80 lines** (`docs/course-correction-2026-09.md`
§3.1 item 7). It is what the worker works from; `.esm/worker.md` carries the ESM mechanics and
the brief carries the engineering:

- The task in two sentences, and the criteria restated with any measured figure RE-DERIVED at
  HEAD (brief cites drift within a chain — dispatch hygiene 9).
- The **change class** and its required ritual — cite it by section:
  `memory/conventions.md` → "## Change-class acceptance table" (row 1–4), so the worker reads
  the table rather than a paraphrase of it.
- The files and functions to start from, with paths verified to exist at HEAD.
- Pointers, not prose: the relevant `memory/gotchas-*.md` entries, the seed registry rows, the
  plan or notes file to append to. Known-site lists are FLOORS — say so and ask for an
  inverse-method census (dispatch hygiene 6).
- What the worker must NOT do: no `git add -A`; stage by explicit path (dispatch hygiene 10);
  no `sleep` polling under `run_in_background` (dispatch hygiene 13); bench scratch and target
  dirs under the scratchpad and deleted before finishing (dispatch hygiene 11).

The brief is untracked (`.esm/` is excluded from git), so copy anything durable into the task's
notes file as well.

### 8. Launch the worker

Instead of reporting "launch the worker" to the user, launch it directly via the
`esm worker-tab` CLI command (esm-21). It opens a split kitty tab — worker session on
the left, a live glance pane (`esm task glance`) on the right — and handles tab
titling, cwd verification, retry, and a manual-instructions fallback when kitty
remote control is unavailable.

`{worktree_abs}` is the absolute path returned in the `worktree` field of
`esm worktree create`'s JSON response in step 5 — pass it verbatim.

```bash
esm worker-tab {task_id} "{worktree_abs}" --prompt 'Read .esm/worker.md, then .esm/brief.md, and follow both. BEFORE you start implementing, post a task list as an `esm task comment` on your task AND write it to memory/primitives/<batch>-task-list.md — one item per concrete step (each site, each card-def edit, each test, build/clippy/fmt, /review). This build exposes no TaskCreate to workers; the comment plus the file is the task list. Update it as you go: repost the comment with items checked off at every milestone, never one batch at the end. The brief names the CHANGE CLASS; do that class'"'"'s acceptance ritual (memory/conventions.md, section "Change-class acceptance table") and no more. Delegate the heavy lifting to the specialized project agents via the Agent tool rather than implementing everything inline: primitive batches use primitive-impl-runner then primitive-impl-reviewer; card authoring uses bulk-card-author then card-batch-reviewer; game scripts use game-script-generator; CR coverage questions use cr-coverage-auditor. Only implement directly when no agent fits. Subagent briefs must forbid git outright (the stash stack is shared across worktrees). Never `git add -A`; stage by explicit path. Never wait with `sleep` under run_in_background — use the Monitor tool or one foreground until-loop. Keep bench scratch under your scratchpad and delete it before finishing. When done: satisfy every criterion with `esm task satisfy`, run /review (HIGH and MEDIUM findings fixed in-cycle, LOW logged to the notes file unless trivial), write the ≤10-line CHANGELOG.md entry and the notes file, then follow the Completion Sequence in .esm/worker.md.'
```

The `--prompt` value above is this project's customized worker prompt (task-list
discipline, the brief, the change-class ritual, the specialized-agent roster) — keep its agent
roster in sync with the Agents table in CLAUDE.md, and do not drop it in favor of the stock prompt: `esm update` skips this
skill precisely because of that customization (see `.esm/migration.json`), and
`esm update --force` would clobber it.

Check the command's JSON output: `cwd_verified` must be `true`. If the command reports
kitty remote control unavailable, relay its manual launch instructions to the user
(`cd <worktree_abs> && claude "<prompt>"`) and verify `kitty @ ls` works before the next dispatch.

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

### 10. Wait for completion — one persistent Monitor per worker, never a sleep loop

After dispatching, watch each task with the **Monitor tool** (dispatch hygiene 3/5/13). Do NOT
write a bash `while … sleep 30` loop: under the 10-minute Bash cap it dies within minutes, under
`run_in_background` a `sleep` returns immediately and burns turns for zero wall-clock, and the
restart ritual it forces is exactly what the Monitor tool exists to remove.

**Recipe** (dispatch hygiene 5/6/7/8/12):

1. Put the parser in a **scratchpad FILE**, not an inline `python3 -c` — a `\"` inside an
   f-string is a SyntaxError that surfaces only as an empty MONITOR ERROR (hygiene 8) — and feed
   it the task JSON on **STDIN**, never by interpolating `$out` into the script (hygiene 6):
   ```bash
   cat > "$SCRATCHPAD/watch_{task_id}.py" <<'EOF'
   import sys, json
   d = json.load(sys.stdin)
   t = d.get("task", {}); acs = d.get("acceptance_criteria", [])
   sat = sum(1 for c in acs if c.get("satisfied"))
   last = (d.get("comments") or [{}])[-1].get("content", "")[:200].replace("\n", " ")
   print(t.get("current_status"), f"{sat}/{len(acs)} AC", "|", last)
   EOF
   ```
2. Start ONE Monitor per worker with an `until` loop, IP-pinned because `tower` DNS blips
   (hygiene 7), with a quiet threshold before emitting errors:
   ```bash
   ESM_URL=http://192.168.1.223:8765 esm task get {task_id} | python3 "$SCRATCHPAD/watch_{task_id}.py"
   ```
   Condition: the printed status is `in_review` or `done`. Emit a line only when the status,
   the satisfied-AC count or the last comment CHANGES; after ~5 consecutive fetch failures emit
   one error line, not five.
3. **Stall check at 30 minutes of quiet** — BEFORE assuming a hang, check for a permission prompt
   (hygiene 12: PB-DX56 lost 70 minutes to one):
   ```bash
   kitty @ get-text --match 'title:^worker: {task_id}' --extent screen | grep -n 'Do you want to proceed'
   ```
   Approve a throwaway-scratch prompt with `kitty @ send-text --match id:<win> '1\r'`. A quiet
   worker with a running delegated agent is NOT a stall: check `git log main..HEAD` and dirty-file
   mtimes in the worktree first (hygiene 9).
4. When the Monitor reports `in_review`, `/collect {task_id}`. Do not dispatch the next task
   without explicit owner approval (`feedback_queue_autonomous_chaining` is RETRACTED).

The coordinator stays free for user interaction while the Monitors run; there is no timeout to
restart and no state file to re-read.

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
