---
name: start-work
description: Claim a workstream before starting work. Prevents parallel session collisions.
---

# Start Work

Claim a workstream so parallel sessions know it's taken. Run this after `/start-session`
and before doing any actual work.

## Arguments

- `W1`, `W1-B3`, `W1-B8.7`, etc. -- Claim W1 (abilities), optionally with batch detail
- `W2` -- Claim W2 (TUI & simulator)
- `W3` -- Claim W3 (LOW remediation)
- `W4` -- Claim W4 (M10 networking)
- `W5`, `W5-cards` -- Claim W5 (card authoring -- RETIRED, use W6)
- `W6`, `W6-PB18`, `W6-cards` -- Claim W6 (primitive + card authoring)
- No args -- Show current state and ask which workstream to claim

## Procedure

### Step 1: Read state

Read `memory/workstream-state.md`. Display the Active Claims table.

Also read `docs/project-status.md` to show current progress summary:
- Primitive batch progress (done/active/planned counts)
- Card health summary
- Path to alpha status

### Step 2: Validate

Check the target workstream's current status:

- **If `ACTIVE`**: STOP. Print a warning:
  > **COLLISION WARNING**: W<N> is already ACTIVE (claimed <timestamp>).
  > Another session may be working on it. Check with the other session before proceeding.
  > To force-claim anyway, run `/start-work W<N> --force`.

- **If `not-started`**: STOP. Print:
  > W<N> is marked `not-started` (blocked or deferred). Check `docs/workstream-coordination.md`
  > for prerequisites.

- **If `available` or `paused`**: Proceed to Step 3.

### Step 3: Claim

Update `memory/workstream-state.md`:

1. Set the target workstream's **Status** to `ACTIVE`
2. Set **Task** to the specific task from `$ARGUMENTS` (e.g., "PB-18: Stax/restrictions")
   or a general description if no detail given (e.g., "TUI hardening")
3. Set **Claimed** to today's date and time
4. Set **Notes** to any relevant context

### Step 4: Load context

Based on the workstream, load relevant files and report context:

| Workstream | Auto-load |
|------------|-----------|
| W1 | `docs/ability-batch-plan.md` -- find the target batch |
| W2 | `tools/tui/` source, `crates/simulator/` |
| W3 | `docs/mtg-engine-low-issues-remediation.md` |
| W4 | `docs/mtg-engine-roadmap.md` M10 section |
| W5 | RETIRED -- redirect to W6 |
| W6 | `docs/project-status.md` + `docs/primitive-card-plan.md` |

#### W6-Specific Context Loading

**If claiming W6 with a PB number** (e.g., `W6-PB18`):

1. Read the PB-<N> section from `docs/primitive-card-plan.md`
2. Read `docs/project-status.md` to check:
   - PB-<N> status (should be `planned`)
   - Any deferred items from prior PBs that apply
   - Review backlog status
3. Read `memory/workstream-state.md` "Last Handoff" for deferred items from the prior PB
4. Check if `memory/primitive-wip.md` exists (WIP from a previous session)
5. Report:
   - Batch title and card count
   - Dependencies (and whether they're met)
   - Deferred items carried forward
   - Whether `/implement-primitive` should be used (recommended for all remaining PBs)

**If claiming W6 without a PB number** (e.g., just `W6`):

1. Read `docs/project-status.md` to find the next `planned` PB batch
2. Suggest claiming that specific batch
3. If all PBs are `done`, suggest Phase 2 card authoring or Phase 3 audit

**If claiming `W6-cards`** (Phase 2 bulk authoring):

1. Read `docs/project-status.md` to verify all PBs are `done`
2. Read `docs/primitive-card-plan.md` Phase 2 wave plan
3. Report which authoring wave is next

#### W1-Specific Context Loading (legacy)

If claiming W1 with a batch number, also report:
- Which abilities are in that batch
- Whether any are already done
- The previous handoff notes for W1 (if any)

### Step 5: Report

Print confirmation:
> **Claimed W<N>**: <task description>
> **Commit prefix**: `W6-prim:` (primitives) or `W6-cards:` (authoring) or `W<N>:`
> **Recommended workflow**: `/implement-primitive` for PB batches
> **Remember**: Run `/end-session` when done to release the claim and write a handoff.

## Force-claim

If `$ARGUMENTS` includes `--force`:
- Skip the collision check in Step 2
- Set the workstream to ACTIVE regardless of current status
- Print a warning that the previous claim was overridden
