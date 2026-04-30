---
name: triage-cards
description: Execute Phase 0 triage — scan card defs for TODOs, reclassify blocked sessions, consolidate review findings
---

# Triage Cards

Execute the Phase 0 triage steps from `docs/card-authoring-operations.md`. Establishes
ground truth about what the DSL can express today and what needs fixing.

## Arguments

- No args: run full triage (T-1 through T-6)
- `T-<N>`: run a specific triage step only
- `--status`: show triage progress and exit

## Procedure

### Step 0: Check Prerequisites

Read `docs/card-authoring-operations.md` Implementation Order to verify:
- All I-* items are checked (infrastructure complete)
- PB-22 is complete (all primitives available)

If not, stop: "Infrastructure tasks must be complete before triage. Run I-* items first."

### Step 1: Determine What to Do

**If `$ARGUMENTS` is `--status`**:
- Check if `memory/card-authoring/dsl-gap-audit-v2.md` exists
- Check if `memory/card-authoring/consolidated-fix-list.md` exists
- Check if `memory/card-authoring/triage-summary.md` exists
- Report which steps are done and which remain. Stop.

**If `$ARGUMENTS` names a step** (e.g., "T-1"):
- Jump to that step directly.

**If no arguments**:
- Find the first unchecked T-* item in the operations plan. Start there.

### T-1: Refresh DSL Gap Audit

1. Grep all `TODO` lines from `crates/engine/src/cards/defs/*.rs`
2. For each TODO, classify against the current DSL (PB-0 through PB-22):
   - **Now expressible**: The DSL primitive exists. Card should be re-authored.
   - **Still blocked**: No DSL support. Document what's missing.
   - **Stale/wrong**: TODO claims something is missing that already exists.
3. Cross-reference against `crates/engine/src/cards/helpers.rs` exports and the
   Effect/AbilityDefinition/TriggerCondition/TargetRequirement enums
4. Write `memory/card-authoring/dsl-gap-audit-v2.md` with:
   - Per-gap-bucket table: count, DSL status (yes/no/partial), what's missing, effort
   - Per-card list of "now expressible" cards (these go to Phase 1 re-authoring)

### T-2: Re-evaluate Blocked Sessions

1. Read `test-data/test-cards/_authoring_plan.json`
2. For each session with `"status": "blocked"`:
   a. Read the card list and oracle text
   b. Check whether abilities are now expressible in the DSL
   c. Classify: Unblocked / Partially blocked / Still blocked
3. Update `_authoring_plan.json` session statuses (blocked → ready where applicable)
4. Write summary of changes to the triage output

### T-3: Re-evaluate Deferred Sessions

Same procedure as T-2, applied to sessions with `"status": "deferred"`.

### T-4: Consolidate Review Findings

1. Read all review files:
   - `memory/card-authoring/review-phase1-batch-{01..20}.md`
   - `memory/card-authoring/review-wave-002-batch-{01..38}.md`
   - `memory/card-authoring/review-wave-003-batch-{01..15}.md`
2. For each HIGH or MEDIUM finding, classify:
   - **Still valid**: Needs fixing
   - **Superseded by PB work**: A primitive batch already fixed this card
   - **Now expressible**: The TODO is now implementable
   - **Already fixed**: Card def was updated since the review
3. Write `memory/card-authoring/consolidated-fix-list.md` with:
   - One entry per card needing work
   - Severity, specific action needed, source review batch
   - Grouped by severity (HIGH first, then MEDIUM, then LOW)

### T-5: Inventory Pre-existing Defs Not in Plan

1. List all card def files in `crates/engine/src/cards/defs/`
2. Cross-reference against `_authoring_plan.json` card names
3. For defs not in the plan:
   a. Check for TODOs
   b. Classify per T-1 (now expressible vs still blocked)
   c. Check for wrong game state patterns (KI-2 from reviewer)
4. Append fixable items to the consolidated fix list

### T-6: Write Triage Summary

Write `memory/card-authoring/triage-summary.md` with:
- Total cards that can be fully authored today
- Cards with fixable TODOs (DSL now covers them)
- Cards with valid TODOs (DSL still lacks something)
- Truly blocked cards (and what blocks them)
- Updated session counts (ready / blocked / deferred)
- Estimated effort for Phase 1 fixes and Phase 2 authoring

### T-7: Check Off and Commit

1. Check off T-1 through T-6 in `docs/card-authoring-operations.md`
2. Stage all new/changed files
3. Commit: `W6-triage: refresh DSL gap audit, reclassify sessions, consolidated fix list`

## Notes

- The triage is read-only for card defs — it never modifies them.
- The triage may modify `_authoring_plan.json` (session status updates).
- The triage creates 3 new files in `memory/card-authoring/`.
- MCP calls are not needed for triage — it works from file scanning and DSL knowledge.
- This is a large task. If context is getting long, break T-4 into sub-steps
  (Phase 1 reviews, then Wave 002, then Wave 003).
