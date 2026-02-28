---
name: start-work
description: Claim a workstream before starting work. Prevents parallel session collisions.
---

# Start Work

Claim a workstream so parallel sessions know it's taken. Run this after `/start-session`
and before doing any actual work.

## Arguments

- `W1`, `W1-B3`, `W1-B8.7`, etc. — Claim W1 (abilities), optionally with batch detail
- `W2` — Claim W2 (TUI & simulator)
- `W3` — Claim W3 (LOW remediation)
- `W4` — Claim W4 (M10 networking)
- No args — Show current state and ask which workstream to claim

## Procedure

### Step 1: Read state

Read `memory/workstream-state.md`. Display the Active Claims table.

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
2. Set **Task** to the specific task from `$ARGUMENTS` (e.g., "Batch 3: Combat Modifiers")
   or a general description if no detail given (e.g., "TUI hardening")
3. Set **Claimed** to today's date and time
4. Set **Notes** to any relevant context

### Step 4: Load context

Based on the workstream, suggest which files to load:

| Workstream | Auto-load |
|------------|-----------|
| W1 | `docs/ability-batch-plan.md` — find the target batch |
| W2 | `tools/tui/` source, `crates/simulator/` |
| W3 | `docs/mtg-engine-low-issues-remediation.md` |
| W4 | `docs/mtg-engine-roadmap.md` M10 section |

If claiming W1 with a batch number, also report:
- Which abilities are in that batch
- Whether any are already done
- The previous handoff notes for W1 (if any)

### Step 5: Report

Print confirmation:
> **Claimed W<N>**: <task description>
> **Commit prefix**: `W<N>-B<N>:` (or `W<N>:`)
> **Remember**: Run `/end-session` when done to release the claim and write a handoff.

## Force-claim

If `$ARGUMENTS` includes `--force`:
- Skip the collision check in Step 2
- Set the workstream to ACTIVE regardless of current status
- Print a warning that the previous claim was overridden
