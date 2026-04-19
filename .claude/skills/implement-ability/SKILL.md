---
name: implement-ability
description: Orchestrate the ability implementation pipeline — pick, plan, implement, review, fix, card, script, close
---

# Implement Ability

Orchestrate the full ability implementation pipeline. Manages `memory/ability-wip.md` as
a state file and dispatches the right agent for the current phase.

## Arguments

- No args: continue from current phase in `memory/ability-wip.md`
- `<ability name>`: start a new ability (overwrites any WIP)
- `--status`: show current WIP state and exit

## Task List

**Every pipeline run MUST create a task list using TaskCreate** so the user can follow progress
in the Claude Code TUI. Create tasks at pipeline start, update them as phases complete.

Create these tasks immediately after determining what to do:

1. `Plan <Name> ability` -- "Research CR <number>, study similar abilities, write plan"
2. `Implement <Name>` -- "Enum variant, rule enforcement, trigger wiring, unit tests"
3. `Review <Name>` -- "Opus reviewer: verify against CR <number>"
4. `Fix <Name> findings` -- "Apply HIGH/MEDIUM fixes from review (if any)"
5. `Author <Name> card def` -- "Card definition for showcase card"
6. `Generate <Name> script` -- "Game script for golden test"
7. `Close <Name>` -- "Update coverage doc, CLAUDE.md, MEMORY.md"

Set up dependencies: each task blockedBy the previous one.

### Task updates during execution:

- Set task to `in_progress` (with activeForm) BEFORE spawning each agent
- Set task to `completed` AFTER the phase succeeds
- If fix phase is skipped (clean verdict), set fix task to `completed` with subject "Fix <Name> -- skipped (clean)"
- If resuming a pipeline, check TaskList first -- reuse existing tasks, don't create duplicates

## Procedure

### Step 0: Read Current State

Read `memory/ability-wip.md` if it exists. Determine the current phase.

If `$ARGUMENTS` is `--status`:
- If `ability-wip.md` exists: display its contents and stop.
- If not: report "No ability in progress" and stop.

### Step 1: Determine What to Do

**If `$ARGUMENTS` names a specific ability** (e.g., "ward"):
- Start fresh — create/overwrite `ability-wip.md` (go to "Create WIP" below).

**If no arguments and `ability-wip.md` exists with `phase:` that is NOT `closed`**:
- Continue from the current phase (go to the matching phase handler below).

**If no arguments and no WIP (or `phase: closed`)**:
- Run the `/next-ability` logic to pick the top gap.
- Use the result to create `ability-wip.md` (go to "Create WIP" below).

### Create WIP

Look up the ability's CR number in `docs/mtg-engine-ability-coverage.md`.

**Create task list** (7 tasks with sequential dependencies). Check TaskList first to avoid duplicates.

Create `memory/ability-wip.md`:

```markdown
# Ability WIP: <Name>

ability: <Name>
cr: <CR number>
priority: P<N>
started: <today's date>
phase: plan
plan_file: memory/abilities/ability-plan-<lowercase-name>.md

## Step Checklist
- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update
```

Then proceed to phase: plan.

### Phase: plan

**TaskUpdate**: Set plan task to `in_progress`, activeForm: "Planning <Name> ability"

Spawn the `ability-impl-planner` agent (Opus):

```
Task tool:
  subagent_type: ability-impl-planner
  prompt: "Plan the implementation of the <Name> ability (CR <number>). Read memory/ability-wip.md for context. Write the plan to memory/abilities/ability-plan-<name>.md."
  model: opus
```

After the planner completes:
- Verify `memory/abilities/ability-plan-<name>.md` was created
- Update `memory/ability-wip.md`: set `phase: implement`
- **TaskUpdate**: Set plan task to `completed`
- Report: "Plan written. Run `/implement-ability` to start implementation."

### Phase: implement

**TaskUpdate**: Set implement task to `in_progress`, activeForm: "Implementing <Name>"

Spawn the `ability-impl-runner` agent (Sonnet):

```
Task tool:
  subagent_type: ability-impl-runner
  prompt: "Implement the <Name> ability. Read memory/ability-wip.md and memory/abilities/ability-plan-<name>.md. Execute unchecked steps 1-4, run tests after each, and check off steps in ability-wip.md."
  model: sonnet
```

After the runner completes:
- **Verification gate** — run `~/.cargo/bin/cargo build -p mtg-engine 2>&1 | tail -5` directly.
  If it fails, the runner hallucinated its changes. Do NOT advance the phase. Instead:
  - Report the failure to the user with the compiler error.
  - Re-invoke the runner with the same prompt plus: "The previous run did not write its changes.
    Start from scratch — implement all unchecked steps now."
  - Run the build check again before proceeding.
- Read `memory/ability-wip.md` to confirm steps 1-4 are checked off
- Update `phase: review`
- **TaskUpdate**: Set implement task to `completed`
- **Continue immediately to Phase: review** (do not stop and report).

### Phase: review

**TaskUpdate**: Set review task to `in_progress`, activeForm: "Reviewing <Name>"

Spawn the `ability-impl-reviewer` agent (Opus):

```
Task tool:
  subagent_type: ability-impl-reviewer
  prompt: "Review the <Name> ability implementation. Read memory/ability-wip.md and memory/abilities/ability-plan-<name>.md. Verify against CR <number>. Write findings to memory/abilities/ability-review-<name>.md."
  model: opus
```

After the reviewer completes:
- Read `memory/abilities/ability-review-<name>.md`
- Add review reference to `ability-wip.md`:
  ```
  ## Review
  findings: <count>
  review_file: memory/abilities/ability-review-<name>.md
  ```
- **TaskUpdate**: Set review task to `completed`
- Check the verdict:
  - If `needs-fix`: update `phase: fix`, **continue immediately to Phase: fix** (do not stop).
  - If `clean`: update `phase: card`.
    - **TaskUpdate**: Set fix task to `completed`, subject: "Fix <Name> -- skipped (clean)"
    - Stop and report implementation + review summary.

### Phase: fix

**TaskUpdate**: Set fix task to `in_progress`, activeForm: "Fixing <Name> review findings"

Spawn the `ability-impl-runner` agent (Sonnet) in fix mode:

```
Task tool:
  subagent_type: ability-impl-runner
  prompt: "Fix the <Name> ability review findings. Read memory/ability-wip.md and memory/abilities/ability-review-<name>.md. Apply all HIGH, MEDIUM, and LOW fixes, run tests."
  model: sonnet
```

After the runner completes:
- **Verification gate** — run `~/.cargo/bin/cargo build -p mtg-engine 2>&1 | tail -5` directly.
  If it fails, re-invoke the runner with the fix findings and the build error. Run the check again.
- Update `phase: card` in `ability-wip.md`
- **TaskUpdate**: Set fix task to `completed`
- Stop and report: implementation summary, review findings, and what was fixed.
- User runs `/implement-ability` to continue to card phase.

### Phase: card

**TaskUpdate**: Set card task to `in_progress`, activeForm: "Authoring <Name> card def"

Spawn the `card-definition-author` agent (Sonnet):

The plan file (`memory/abilities/ability-plan-<name>.md`) has a "Step 5: Card Definition" section
with a suggested card name. Use that card.

```
Task tool:
  subagent_type: card-definition-author
  prompt: "Add a card definition for <suggested card name>"
  model: sonnet
```

After it completes:
- Check off step 5 in `ability-wip.md`
- Update `phase: script`
- **TaskUpdate**: Set card task to `completed`
- Report: "Card definition added. Run `/implement-ability` to generate a game script."

### Phase: script

**TaskUpdate**: Set script task to `in_progress`, activeForm: "Generating <Name> game script"

Spawn the `game-script-generator` agent (Sonnet):

The plan file has a "Step 6: Game Script" section with a suggested scenario.

```
Task tool:
  subagent_type: game-script-generator
  prompt: "Generate a game script for <suggested scenario using the card from step 5>"
  model: sonnet
```

After it completes:
- Check off step 6 in `ability-wip.md`
- Update `phase: close`
- **TaskUpdate**: Set script task to `completed`
- Report: "Game script generated. Run `/implement-ability` to close out."

### Phase: close

**TaskUpdate**: Set close task to `in_progress`, activeForm: "Closing <Name> ability"

1. Spawn the `ability-coverage-auditor` agent scoped to this ability:

```
Task tool:
  subagent_type: ability-coverage-auditor
  prompt: "Audit coverage for the <Name> ability only. Update its row in docs/mtg-engine-ability-coverage.md to reflect current status."
  model: opus
```

2. After it completes:
   - Check off step 7 in `ability-wip.md`
   - Set `phase: closed` in `ability-wip.md`

3. **Update CLAUDE.md Current State** (inline — no agent needed):
   - Run `~/.cargo/bin/cargo test --workspace 2>&1 | grep "^test result" | grep -oP '\d+ passed' | awk '{s+=$1} END {print s}'` to get the current test count
   - Read the `## Current State` section of `CLAUDE.md`
   - Update the `Status:` line: bump test count, validated count, and P<N> count to match current reality
   - Update `Last Updated:` to today's date

4. **Update MEMORY.md** (inline — no agent needed):
   - Read the `## Milestone Review Tracking` section of `memory/MEMORY.md` (the auto-memory file at `/home/skydude/.claude/projects/-home-skydude-projects-scutemob/memory/MEMORY.md`)
   - Find the most recent "Batch N complete" line
   - If this ability belongs to the same batch: update the test count, validated count, and P4 count in that line
   - If this ability closes a new batch: prepend a new "Batch N complete" line with current stats

5. **TaskUpdate**: Set close task to `completed`

6. Report a summary:

```
## Ability Complete: <Name>

**CR**: <number>
**Phase**: closed
**Steps completed**: 7/7
**Review findings**: <count> (<count> fixed)
**Card defined**: <card name>
**Script**: <script path>
**Coverage status**: validated
**CLAUDE.md**: updated (tests: <N>, validated: <N>, P<X>: <N>/<total>)
**MEMORY.md**: updated
```

## Important Notes

- **Always create a task list.** Use TaskCreate at pipeline start and TaskUpdate at each
  phase transition. This lets the user track progress in the Claude Code TUI.
  When resuming, check TaskList first -- reuse existing tasks, don't create duplicates.
- **Always verify with `cargo build` after impl-runner.** The runner can hallucinate successful
  changes. Run `~/.cargo/bin/cargo build -p mtg-engine 2>&1 | tail -5` yourself after every
  implement and fix phase before advancing. A compile failure means the runner's changes were
  not written — re-invoke with an explicit "start from scratch" prompt.
- **One ability at a time.** The WIP file tracks a single ability.
- **Auto-chained phases**: implement → review → fix run in a single invocation without
  stopping. The chain ends after fix (advancing to `card`) so the user can review what
  was built before continuing. Plan, card, script, and close each require a separate
  invocation.
- **No re-review after fix.** The runner runs `cargo test` after each fix — that is the
  safety net. A second Opus review pass is not worth the cost for mechanical fixes.
- **The planner and reviewer use Opus.** The runner uses Sonnet. This mirrors the milestone
  workflow's proven Plan (Opus) → Implement (Sonnet) → Review (Opus) → Fix (Sonnet) cycle.
- **Existing agents are reused.** `card-definition-author`, `game-script-generator`, and
  `ability-coverage-auditor` are not modified — they're invoked as-is.
- **The state file is the source of truth.** Always read `memory/ability-wip.md` before
  doing anything. Always update it after each phase.
