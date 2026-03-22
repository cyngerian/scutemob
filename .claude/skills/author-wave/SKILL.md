---
name: author-wave
description: Orchestrate the full author-review-fix-commit cycle for one card authoring group
---

# Author Wave

Orchestrate the full author → review → fix → commit cycle for one authoring group
from the card authoring operations plan (`docs/card-authoring-operations.md`).

## Arguments

- `<group-name>`: Author the specified group (e.g., `body-only`, `mana-creature`, `removal-destroy`)
- `A-<N>`: Author by operations plan item number (e.g., `A-01`, `A-18`)
- No args: find the next unchecked A-* item and suggest it
- `--status`: show wave progress and exit

## State File

`memory/card-authoring/wave-progress.md` — tracks per-group status:

```markdown
# Wave Progress

| Group | Status | Sessions Done | Cards Authored | Cards Reviewed | Findings | Committed |
|-------|--------|---------------|----------------|----------------|----------|-----------|
| body-only | complete | 3/3 | 55 | 55 | 0H 0M | yes |
| mana-creature | in-progress | 1/2 | 10 | 0 | — | no |
```

## Procedure

### Step 0: Read State

1. Read `memory/card-authoring/wave-progress.md` if it exists
2. Read `docs/card-authoring-operations.md` to find the target group

**If `$ARGUMENTS` is `--status`**:
- Display wave progress table. Stop.

**If no arguments**:
- Find the first unchecked A-* item in the operations plan
- Suggest: "Next group: `<group>` (A-<N>, <cards> cards, <sessions> sessions). Run `/author-wave <group>` to start."
- Stop.

### Step 1: Pre-check

1. Read `test-data/test-cards/_authoring_plan.json` to find all sessions for the target group
2. Check which sessions are `ready` vs `blocked`
3. For ready sessions, check which cards already have def files (skip unless skeleton)
4. Report:
   - Total sessions: N ready, M blocked
   - Cards to author: N new, M to update (skeletons)
   - Cards to skip: N (already complete)

If all sessions are blocked, stop: "All sessions for `<group>` are blocked. Skip to next group."

### Step 2: Author Sessions

For each ready session in the group:

1. Launch the `bulk-card-author` agent:
   ```
   Agent tool:
     subagent_type: bulk-card-author
     prompt: "Author session <ID> from _authoring_plan.json. Group: <group>. Read the session data, look up each card via MCP, read a reference def, and write all card files."
   ```

2. **Parallel execution**: Launch up to 2 agents simultaneously for different sessions.
   Wait for both to complete before launching the next pair.

3. After each agent completes, note files created and any errors.

### Step 3: Build Check

Run:
```bash
~/.cargo/bin/cargo build --lib -p mtg-engine
```

If compile errors: read the errors, fix the card defs inline (Edit tool), rebuild.

### Step 4: Review Authored Cards

Batch all newly authored cards into groups of 5. For each batch:

1. Launch the `card-batch-reviewer` agent:
   ```
   Agent tool:
     subagent_type: card-batch-reviewer
     prompt: "Review these 5 card definitions against oracle text: <card1>, <card2>, ... Write findings to memory/card-authoring/review-<group>-batch-<N>.md"
   ```

2. **Parallel execution**: Launch up to 3 reviewer agents simultaneously.

3. After all reviewers complete, collect findings.

### Step 5: Fix Findings

If any HIGH or MEDIUM findings:

1. Launch the `card-fix-applicator` agent:
   ```
   Agent tool:
     subagent_type: card-fix-applicator
     prompt: "Apply fixes from review findings for the <group> group. Read memory/card-authoring/review-<group>-batch-*.md files. Fix all HIGH and MEDIUM findings."
   ```

2. After fixes, if any HIGH findings were fixed, re-review those specific cards.

### Step 6: Final Build + Test

```bash
~/.cargo/bin/cargo build --workspace && ~/.cargo/bin/cargo test --all 2>&1 | tail -5
```

If failures: fix and re-test.

### Step 7: Update State and Commit

1. Update `memory/card-authoring/wave-progress.md` with group status
2. Check off the A-* item in `docs/card-authoring-operations.md`
3. Stage all new/changed card def files
4. Commit: `W6-cards: author <group> (<N> cards)`

### Step 8: Report

```
## Wave Complete: <group> (A-<N>)

**Cards authored**: N new + M updated
**Cards skipped**: N (already complete) + M (blocked)
**Review findings**: N HIGH, M MEDIUM, L LOW
**Fixes applied**: N
**Total card defs**: <new total>
**Next group**: <next-group> (A-<next>)
```

## Notes

- **Do not overlap fix phases** between groups. Fix group N before starting group N+1's
  authoring. Authoring can overlap (group N in review while group N+1 is authoring).
- **Blocked sessions**: Skip them. Note in the report. They'll be addressed in Phase 3.
- **MCP budget**: Each bulk-card-author gets 30 calls. Each reviewer gets 15 calls.
  Plan agent launches accordingly.
- **Build early, build often**: Build after authoring (Step 3) AND after fixing (Step 6).
  Don't let compile errors accumulate across agents.
