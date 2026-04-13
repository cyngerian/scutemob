---
name: start-milestone
description: Load only the relevant milestone section from the roadmap, check for a session plan, and orient the developer for the milestone ahead — without reading the full roadmap.
---

# Start Milestone

Given a milestone number (`$ARGUMENTS`, e.g. "8" or "M8"), do the following:

## Step 1: Check for a session plan

Check if `memory/m<N>-session-plan.md` exists (where `<N>` is the milestone number).

**If the session plan exists:**
- Read it.
- Report: "Session plan found — reading `memory/m<N>-session-plan.md` instead of the full roadmap."
- Skip to Step 4 after reading the plan.

**If no session plan exists:**
- Continue to Step 2. After Step 4, automatically invoke the `rules-implementation-planner` agent to create one (see Step 5).

## Step 2: Find the milestone section in the roadmap

Do NOT read the full `docs/mtg-engine-roadmap.md`. Instead:

1. Use Grep to find the line number of the milestone heading:
   - Pattern: `### M<N>:` (e.g. `### M8:`)
   - File: `docs/mtg-engine-roadmap.md`
   - Output mode: `content` with line numbers enabled

2. Use Grep to find the line number of the NEXT milestone heading:
   - Pattern: `### M<N+1>:` (e.g. `### M9:`)
   - Same file, same output mode

## Step 3: Read only the milestone section

Use the Read tool with `offset` set to the start line from Step 2 and `limit` set to
(next milestone line - start line). This reads only the relevant section — typically
40–100 lines, not the full 951-line document.

## Step 4: Report

Present a concise summary:
1. Milestone name and number
2. Deliverables list (what needs to be built)
3. Acceptance criteria
4. Dependencies on previous milestones (if any)
5. Recommended first steps

If a session plan was used (Step 1), also note:
- How many sessions are planned
- Which session to start with (first unchecked session)

## Step 5: Create session plan (only when no plan existed)

If Step 1 found no session plan, use the Task tool to invoke the `rules-implementation-planner`
agent with subagent_type `rules-implementation-planner`. Pass a prompt like:

> "Create a session plan for M<N>. The milestone roadmap section has just been summarized above.
> Research the relevant CR rules, design the implementation architecture, and write the plan to
> `memory/m<N>-session-plan.md`."

The agent will do deep CR research (Opus), produce a sequenced session breakdown with architecture
notes and CR citations, and write the file. When it finishes, report the plan file path and the
first session to work on.

## Important

- Never read `docs/mtg-engine-roadmap.md` in full. Always use Grep + offset/limit.
- The session plan (if it exists) is the primary source — it was authored by Opus with
  full context and contains sequenced implementation steps. Prefer it over the raw
  roadmap section.
