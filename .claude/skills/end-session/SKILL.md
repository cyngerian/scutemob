---
name: end-session
description: End-of-session cleanup — release workstream claim, write handoff, update memory, revise CLAUDE.md.
---

# End Session

Run this checklist in order. After each step, briefly report what was done. At the end, ask the user to confirm before closing.

## Step 0: Release workstream claim and write handoff

Read `memory/workstream-state.md`. If any workstream has Status = `ACTIVE`:

1. **Set Status** to `available` (if work is done or at a clean stopping point) or `paused`
   (if mid-task and the next session should resume this specific work).
2. **Clear the Task and Claimed fields** (set to `—`).
3. **Write a handoff note** — replace the "Last Handoff" section with:

```markdown
## Last Handoff

**Date**: <today's date and approximate time>
**Workstream**: W<N>: <name>
**Task**: <what was being worked on>
**Completed**: <bullet list of what got done>
**Next**: <what the next session should do>
**Hazards**: <files with uncommitted changes, potential conflicts, gotchas>
**Commit prefix used**: <e.g., W1-B3, W2, W3>
```

4. **Rotate history** — move the previous "Last Handoff" content to the top of
   "Handoff History". Keep only the 5 most recent entries. Delete older ones.

If no workstream was ACTIVE (e.g., the session was exploratory), skip this step.

## Step 1: Update memory topic files

Review what happened this session. Route new content to the appropriate file:
- New MTG rules / game logic gotcha -> `memory/gotchas-rules.md`
- New infra / testing / builder / im-rs gotcha -> `memory/gotchas-infra.md`
- New convention or style decision -> `memory/conventions.md`
- New design decision -> `memory/decisions.md`
- Cross-cutting operational detail (applies nearly every session) -> `memory/MEMORY.md`

**Rules to prevent bloat:**
- Do NOT duplicate content already in CLAUDE.md or any memory topic file
- Do NOT add detailed structural inventories of files/types/functions — Claude can read files on demand
- Remove or correct anything that's now outdated
- MEMORY.md target: <=80 lines

Only write changes if there's something genuinely new. "No changes needed" is a valid outcome.

## Step 1.5: Verify topic file freshness

For each memory topic file that was **read or modified during this session**:
- Check the `Last verified: M<N>` header
- If the file was modified this session, update its "Last verified" to the current milestone
- If the file was read and contains stale information (gotcha no longer applies, convention changed), update or remove the stale entry

This is lightweight — only check files already in context. Do not read topic files solely for this step.

## Step 2: Revise CLAUDE.md (minimal updates only)

Apply only these specific changes if applicable:
- Update the **Current State** header (active milestone, status line, test count, date)
- Remove stale content (e.g., completed milestone sections that are now done)

**Do NOT:**
- Add new gotchas, conventions, or decisions to CLAUDE.md — those go to the appropriate memory topic file (see Step 1 routing rules)
- Add detailed inventories of new files, types, or functions
- Add content that belongs in docs/ or memory topic files
- Expand sections that are already comprehensive

**Size guard**: CLAUDE.md must stay <=250 lines. If a change would push it over 250L, route the content to a topic file instead.

Report what was changed (or "no changes needed").

## Step 3: Commit convention reminder

If work was done this session, remind the user about the commit prefix convention:

| Workstream | Prefix | Example |
|------------|--------|---------|
| W1: Abilities | `W1-B<N>:` | `W1-B3: implement Ninjutsu` |
| W2: TUI & Simulator | `W2:` | `W2: fix blocker declaration UI` |
| W3: LOW Remediation | `W3:` | `W3: add debug_assert to sba.rs` |
| W4: M10 Networking | `W4:` | `W4: add GameServer skeleton` |
| Cross-cutting | `chore:` | `chore: update workstream-state.md` |

## Step 4: Confirm and close

Present a brief summary to the user:
- What was accomplished this session
- What was updated in memory/CLAUDE.md (if anything)
- Workstream handoff written (or skipped)
- Any pending next steps

Then ask: **"Ready to close this session?"** — wait for the user's confirmation before ending.
