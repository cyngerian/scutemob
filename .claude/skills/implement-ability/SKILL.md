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

Create `memory/ability-wip.md`:

```markdown
# Ability WIP: <Name>

ability: <Name>
cr: <CR number>
priority: P<N>
started: <today's date>
phase: plan
plan_file: memory/ability-plan-<lowercase-name>.md

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

Spawn the `ability-impl-planner` agent (Opus):

```
Task tool:
  subagent_type: ability-impl-planner
  prompt: "Plan the implementation of the <Name> ability (CR <number>). Read memory/ability-wip.md for context. Write the plan to memory/ability-plan-<name>.md."
  model: opus
```

After the planner completes:
- Verify `memory/ability-plan-<name>.md` was created
- Update `memory/ability-wip.md`: set `phase: implement`
- Report: "Plan written. Run `/implement-ability` to start implementation."

### Phase: implement

Spawn the `ability-impl-runner` agent (Sonnet):

```
Task tool:
  subagent_type: ability-impl-runner
  prompt: "Implement the <Name> ability. Read memory/ability-wip.md and memory/ability-plan-<name>.md. Execute unchecked steps 1-4, run tests after each, and check off steps in ability-wip.md."
  model: sonnet
```

After the runner completes:
- Read `memory/ability-wip.md` to confirm steps 1-4 are checked off
- Update `phase: review`
- **Continue immediately to Phase: review** (do not stop and report).

### Phase: review

Spawn the `ability-impl-reviewer` agent (Opus):

```
Task tool:
  subagent_type: ability-impl-reviewer
  prompt: "Review the <Name> ability implementation. Read memory/ability-wip.md and memory/ability-plan-<name>.md. Verify against CR <number>. Write findings to memory/ability-review-<name>.md."
  model: opus
```

After the reviewer completes:
- Read `memory/ability-review-<name>.md`
- Add review reference to `ability-wip.md`:
  ```
  ## Review
  findings: <count>
  review_file: memory/ability-review-<name>.md
  ```
- Check the verdict:
  - If `needs-fix`: update `phase: fix`, **continue immediately to Phase: fix** (do not stop).
  - If `clean`: update `phase: card` and stop. Report implementation + review summary.

### Phase: fix

Spawn the `ability-impl-runner` agent (Sonnet) in fix mode:

```
Task tool:
  subagent_type: ability-impl-runner
  prompt: "Fix the <Name> ability review findings. Read memory/ability-wip.md and memory/ability-review-<name>.md. Apply all HIGH and MEDIUM fixes, run tests."
  model: sonnet
```

After the runner completes:
- Update `phase: card` in `ability-wip.md`
- Stop and report: implementation summary, review findings, and what was fixed.
- User runs `/implement-ability` to continue to card phase.

### Phase: card

Spawn the `card-definition-author` agent (Sonnet):

The plan file (`memory/ability-plan-<name>.md`) has a "Step 5: Card Definition" section
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
- Report: "Card definition added. Run `/implement-ability` to generate a game script."

### Phase: script

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
- Report: "Game script generated. Run `/implement-ability` to close out."

### Phase: close

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

3. Report a summary:

```
## Ability Complete: <Name>

**CR**: <number>
**Phase**: closed
**Steps completed**: 7/7
**Review findings**: <count> (<count> fixed)
**Card defined**: <card name>
**Script**: <script path>
**Coverage status**: validated
```

## Important Notes

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
