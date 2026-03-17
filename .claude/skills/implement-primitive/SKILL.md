---
name: implement-primitive
description: Orchestrate the primitive batch implementation pipeline -- plan, implement, review, fix, close
---

# Implement Primitive

Orchestrate the full primitive batch (PB-N) implementation pipeline. Manages
`memory/primitive-wip.md` as a state file and dispatches the right agent for the current phase.

## Arguments

- No args: continue from current phase in `memory/primitive-wip.md`
- `PB-<N>` or `<N>`: start a new primitive batch (overwrites any WIP)
- `--review-only PB-<N>`: retroactive review of an already-completed batch (skips plan + implement)
- `--status`: show current WIP state and exit

## Task List

**Every pipeline run MUST create a task list using TaskCreate** so the user can follow progress
in the Claude Code TUI. Create tasks at pipeline start, update them as phases complete.

### For full pipeline (plan -> implement -> review -> fix -> close):

Create these tasks immediately after determining what to do:

1. `Plan PB-<N> (<title>)` -- "Research CR rules, study engine, write implementation plan"
2. `Implement PB-<N>` -- "Engine changes, card def fixes, tests"
3. `Review PB-<N>` -- "Opus reviewer: verify engine + card defs against CR/oracle"
4. `Fix PB-<N> findings` -- "Apply HIGH/MEDIUM fixes from review" (mark description: "created if review finds issues")
5. `Close PB-<N>` -- "Update project-status.md, workstream-state.md, CLAUDE.md"

Set up dependencies: 2 blockedBy 1, 3 blockedBy 2, 4 blockedBy 3, 5 blockedBy 4.

### For review-only mode (review -> fix -> close):

Create these tasks:

1. `Review PB-<N> (<title>) [retroactive]` -- "Opus reviewer: verify engine + card defs against CR/oracle text"
2. `Fix PB-<N> findings` -- "Apply HIGH/MEDIUM fixes from review (if any)"
3. `Close PB-<N> review` -- "Update project-status.md review backlog, workstream-state.md"

Set up dependencies: 2 blockedBy 1, 3 blockedBy 2.

### Task updates during execution:

- Set task to `in_progress` (with activeForm) BEFORE spawning each agent
- Set task to `completed` AFTER the phase succeeds
- If fix phase is skipped (clean verdict), set fix task to `completed` with updated subject "Fix PB-<N> -- skipped (clean)"
- If resuming a pipeline, check TaskList first -- reuse existing tasks, don't create duplicates

## Procedure

### Step 0: Read Current State

Read `memory/primitive-wip.md` if it exists. Determine the current phase.

If `$ARGUMENTS` is `--status`:
- If `primitive-wip.md` exists: display its contents and stop.
- If not: report "No primitive batch in progress" and stop.

### Step 1: Determine What to Do

**If `$ARGUMENTS` includes `--review-only`** (e.g., "--review-only PB-17"):
- This is a retroactive review of a completed batch. Go to "Create Review-Only WIP" below.

**If `$ARGUMENTS` names a specific batch** (e.g., "PB-18" or "18"):
- Start fresh -- create/overwrite `primitive-wip.md` (go to "Create WIP" below).

**If no arguments and `primitive-wip.md` exists with `phase:` that is NOT `closed`**:
- Continue from the current phase (go to the matching phase handler below).
- Check TaskList -- reuse existing tasks if they exist.

**If no arguments and no WIP (or `phase: closed`)**:
- Read `docs/project-status.md` Review Backlog section.
- If there are `pending` reviews, suggest the next one:
  > **W6-review is the primary objective.** Next pending review: PB-<N> (<cards> cards).
  > Run `/implement-primitive --review-only PB-<N>` to start.
- If all reviews are complete, find the next `planned` PB batch from the Primitive Batches table.
- If none found, report "All primitive batches complete!" and stop.

### Create Review-Only WIP

This mode skips plan and implement phases. Used for retroactive review of completed batches.

1. Read the PB-N section from `docs/primitive-card-plan.md` for context.
2. Read `docs/project-status.md` Review Backlog to confirm PB-N is `pending`.
3. **Create task list** (3 tasks for review-only mode, with dependencies).

Create `memory/primitive-wip.md`:

```markdown
# Primitive WIP: PB-<N> -- <Title> (REVIEW-ONLY)

batch: PB-<N>
title: <Primitive name>
cards_affected: <count>
mode: review-only
started: <today's date>
phase: review
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-<N> implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.
```

4. Update `docs/project-status.md` Review Backlog: set PB-N status to `in-review`.
5. Proceed directly to **Phase: review (review-only mode)**.

### Create WIP (full pipeline)

Read the PB-N section from `docs/primitive-card-plan.md` to get:
- Primitive name/description
- Card count
- Dependencies
- Session estimate

Also read `memory/workstream-state.md` "Last Handoff" for deferred items from prior PBs.

**Create task list** (5 tasks for full pipeline, with dependencies).

Create `memory/primitive-wip.md`:

```markdown
# Primitive WIP: PB-<N> -- <Title>

batch: PB-<N>
title: <Primitive name>
cards_affected: <count>
started: <today's date>
phase: plan
plan_file: memory/primitives/pb-plan-<N>.md

## Deferred from Prior PBs
<list of deferred items that apply to this batch, or "none">

## Step Checklist
- [ ] 1. Engine changes (new types/variants/dispatch)
- [ ] 2. Card definition fixes
- [ ] 3. New card definitions (if any)
- [ ] 4. Unit tests
- [ ] 5. Workspace build verification
```

Then proceed to phase: plan.

### Phase: plan

**TaskUpdate**: Set plan task to `in_progress`, activeForm: "Planning PB-<N>"

Spawn the `primitive-impl-planner` agent (Opus):

```
Agent tool:
  subagent_type: primitive-impl-planner
  prompt: "Plan the implementation of PB-<N> (<title>). Read memory/primitive-wip.md for context. Write the plan to memory/primitives/pb-plan-<N>.md."
  model: opus
```

After the planner completes:
- Verify `memory/primitives/pb-plan-<N>.md` was created
- Update `memory/primitive-wip.md`: set `phase: implement`
- **TaskUpdate**: Set plan task to `completed`
- Report: "Plan written. Run `/implement-primitive` to start implementation."

### Phase: implement

**TaskUpdate**: Set implement task to `in_progress`, activeForm: "Implementing PB-<N>"

Spawn the `primitive-impl-runner` agent (Sonnet):

```
Agent tool:
  subagent_type: primitive-impl-runner
  prompt: "Implement PB-<N> (<title>). Read memory/primitive-wip.md and memory/primitives/pb-plan-<N>.md. Execute all steps, run tests, and check off completed steps in primitive-wip.md."
  model: sonnet
```

After the runner completes:
- **Verification gate** -- run `~/.cargo/bin/cargo build --workspace 2>&1 | tail -10` directly.
  If it fails, the runner hallucinated its changes. Do NOT advance the phase. Instead:
  - Report the failure to the user with the compiler error.
  - Re-invoke the runner with the same prompt plus: "The previous run did not write its changes.
    Start from scratch -- implement all unchecked steps now."
  - Run the build check again before proceeding.
- Read `memory/primitive-wip.md` to confirm steps are checked off
- **Commit**: Stage all changed files and create a commit:
  `W6-prim: PB-<N> implement <title>`
  Include a brief body listing what was added (engine changes, card fixes, tests).
- Update `phase: review`
- **TaskUpdate**: Set implement task to `completed`
- **Continue immediately to Phase: review** (do not stop and report).

### Phase: review

**TaskUpdate**: Set review task to `in_progress`, activeForm: "Reviewing PB-<N>"

**For review-only mode** (retroactive review of completed batch):

The reviewer has no plan file. Instead, instruct it to:
1. Read the PB-N section of `docs/primitive-card-plan.md` for the batch specification
2. Use `git log --oneline --all` to identify the commit(s) for PB-N
3. Grep for the engine changes and card defs described in the batch spec
4. Verify engine changes against CR rules
5. Verify every card def against oracle text via MCP lookup

Spawn the `primitive-impl-reviewer` agent (Opus):

```
Agent tool:
  subagent_type: primitive-impl-reviewer
  prompt: "RETROACTIVE REVIEW of PB-<N> (<title>). This batch was already implemented but never reviewed.
  There is no plan file. Instead:
  1. Read docs/primitive-card-plan.md PB-<N> section for the batch specification
  2. Grep crates/engine/src/ for the engine changes described (new types, variants, dispatch logic)
  3. Read every card def file listed in the batch spec
  4. For each card def, look up oracle text via mcp__mtg-rules__lookup_card and verify correctness
  5. Verify engine changes against CR rules via mcp__mtg-rules__get_rule
  6. Write findings to memory/primitives/pb-review-<N>.md
  Focus on: oracle text mismatches, remaining TODOs, wrong game state, CR violations, missing match arms."
  model: opus
```

**For normal mode** (review after implementation):

Spawn the `primitive-impl-reviewer` agent (Opus):

```
Agent tool:
  subagent_type: primitive-impl-reviewer
  prompt: "Review the PB-<N> (<title>) implementation. Read memory/primitive-wip.md and memory/primitives/pb-plan-<N>.md. Verify engine changes against CR rules. Verify every card def against oracle text. Write findings to memory/primitives/pb-review-<N>.md."
  model: opus
```

After the reviewer completes (both modes):
- Read `memory/primitives/pb-review-<N>.md`
- **TaskUpdate**: Set review task to `completed`
- Add review reference to `primitive-wip.md`:
  ```
  ## Review
  findings: <count> (HIGH: <n>, MEDIUM: <n>, LOW: <n>)
  verdict: <clean / needs-fix>
  review_file: memory/primitives/pb-review-<N>.md
  ```
- Check the verdict:
  - If `needs-fix`: update `phase: fix`, **continue immediately to Phase: fix** (do not stop).
  - If `clean`: update `phase: close`.
    - **TaskUpdate**: Set fix task to `completed`, subject: "Fix PB-<N> -- skipped (clean)"
    - Stop and report review summary.

### Phase: fix

**TaskUpdate**: Set fix task to `in_progress`, activeForm: "Fixing PB-<N> review findings"

Spawn the `primitive-impl-runner` agent (Sonnet) in fix mode:

```
Agent tool:
  subagent_type: primitive-impl-runner
  prompt: "Fix the PB-<N> review findings. Read memory/primitive-wip.md and memory/primitives/pb-review-<N>.md. Apply all HIGH, MEDIUM, and LOW fixes, run tests."
  model: sonnet
```

After the runner completes:
- **Verification gate** -- run `~/.cargo/bin/cargo build --workspace 2>&1 | tail -10` directly.
  If it fails, re-invoke the runner with the fix findings and the build error. Run check again.
- **Commit**: Stage all changed files and create a commit:
  `W6-prim: PB-<N> review fixes — <summary of what was fixed>`
- Update `phase: close` in `primitive-wip.md`
- **TaskUpdate**: Set fix task to `completed`
- Stop and report: review findings and what was fixed.
- User runs `/implement-primitive` to continue to close phase.

### Phase: close

**TaskUpdate**: Set close task to `in_progress`, activeForm: "Closing PB-<N>"

1. **Update `docs/project-status.md`** (inline -- no agent needed):

   **For review-only mode**:
   - Find PB-<N> in the Review Backlog table
   - Set Review Status to `clean` or `fixed` (based on whether fix phase ran)
   - Update Findings column with count summary (e.g., "2H 3M fixed" or "clean")
   - Update Progress counter (e.g., "1 / 20 reviewed")
   - Find PB-<N> in the Primitive Batches table
   - Set Review column to `clean` or `fixed`

   **For full pipeline mode**:
   - Find the PB-<N> row in the Primitive Batches table
   - Set status to `done`, review to `clean` or `fixed`
   - Update cards_fixed and cards_remaining counts

2. **Update `memory/workstream-state.md`** (inline -- no agent needed):
   - Update Last Handoff section with completed review
   - Note next review target from the backlog

3. **Update CLAUDE.md Current State** (inline -- no agent needed):
   - Run test count command
   - Update the Status line if tests changed (fixes may add/modify tests)
   - Update Last Updated to today's date

4. Set `phase: closed` in `primitive-wip.md`

5. **TaskUpdate**: Set close task to `completed`

6. Report a summary:

**For review-only mode:**
```
## Review Complete: PB-<N> -- <Title>

**Mode**: retroactive review
**Cards reviewed**: <count>
**Findings**: <HIGH count> HIGH, <MEDIUM count> MEDIUM, <LOW count> LOW
**Verdict**: <clean / fixed>
**Test count**: <total tests>
**Review progress**: <done> / 20
**Next review**: PB-<next> -- <title>
```

**For full pipeline mode:**
```
## Primitive Batch Complete: PB-<N> -- <Title>

**Primitive**: <what was added>
**Cards fixed**: <count>
**Cards authored**: <count>
**Tests added**: <count>
**Review findings**: <count> (<count> fixed)
**Verdict**: <clean / fixed>
**Test count**: <total tests>
**Next**: PB-<N+1> -- <title>
```

## Important Notes

- **Always create a task list.** Use TaskCreate at pipeline start and TaskUpdate at each
  phase transition. This lets the user track progress in the Claude Code TUI.
- **Always verify with `cargo build --workspace` after impl-runner.** The runner can
  hallucinate successful changes. Run the build yourself after every implement and fix
  phase before advancing. A compile failure means the runner's changes were not written --
  re-invoke with an explicit "start from scratch" prompt.
- **One PB at a time.** The WIP file tracks a single primitive batch.
- **Auto-chained phases**: review -> fix run without stopping. The chain ends after fix
  so the user can review before closing.
- **No re-review after fix.** The runner runs `cargo test` after each fix -- that is the
  safety net. A second Opus review pass is not worth the cost for mechanical fixes.
- **The reviewer uses Opus, the runner uses Sonnet.** This mirrors the proven
  Review (Opus) -> Fix (Sonnet) cycle from `/implement-ability`.
- **W6-review is the primary objective.** When no WIP exists and review backlog has pending
  items, always suggest the next review before suggesting new PB implementation.
- **Review-only mode produces the same review file format** as normal mode. The reviewer
  agent's output is identical -- only the input differs (no plan file, uses batch spec + code).
- **The state file is the source of truth.** Always read `memory/primitive-wip.md` before
  doing anything. Always update it after each phase.
- **When resuming, check TaskList first.** Don't create duplicate tasks -- reuse existing ones.
