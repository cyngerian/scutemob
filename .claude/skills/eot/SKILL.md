---
name: eot
description: End-of-turn / end-of-session — combined ESM session close + scutemob workstream-state rotation + memory routing. MTG-flavored shorthand for the old /end + /end-session pair.
user-invocable: true
allowed-tools: Read, Edit, Write, Bash, Glob, Grep
---

# /eot — End of Turn (End of Session)

Run `/eot` when you're done working. This is the consolidated scutemob session-close
skill: it does what the global `/end` does (ESM lifecycle + uncommitted check) AND the
load-bearing parts of the old `/end-session` (workstream-state rotation, memory routing).

> Replaces both `/end` and `/end-session` for scutemob. The global `/end` still works
> if you forget, but `/eot` is the canonical close — it handles project-specific
> bookkeeping that `/end` doesn't know about.

## Procedure

Steps 1–3 are session state. Steps 4–6 are project bookkeeping. Step 7 ends ESM.
Step 8 reports + asks for confirmation.

### 1. Gather session state

Run these in parallel:
- `git status` — uncommitted changes
- `git log --oneline -15` — what was committed this session
- `git diff --stat` if uncommitted changes exist
- `esm worktree list` — active worker worktrees

### 2. Check for uncommitted work + active worktrees

If `git status` shows uncommitted changes:
- Warn the user; ask whether to commit before closing.
- If declined, note the uncommitted files in the summary.

If `esm worktree list` shows worktrees:
- Warn the user; list each with task ID, branch, and ESM status.
- Do NOT remove them — user decides whether to wait, collect, or leave.

### 3. Documentation check

If `.claude/docs.yaml` exists, run a session-scoped doc check (changed files vs.
trigger patterns, `frequency: task` or `frequency: session`). Report stale docs in
2-3 lines and let the user decide. Skip if no docs.yaml.

### 4. Rotate workstream-state.md (project bookkeeping — Step 0 from old /end-session)

Read `memory/workstream-state.md`. If any workstream is `ACTIVE`, set its Status
to `available` (clean stopping point) or `paused` (mid-task; next session resumes).
Clear the Task and Claimed fields (`—`).

Update the "Last Handoff" section with a fresh entry following this template:

```markdown
## Last Handoff

**Date**: <YYYY-MM-DD> (worker session OR oversight session)
**Workstream**: W<N>: <name>
**Task**: <what was being worked on, merge commit if applicable>

**Completed**:
- <bullet list of engine surface, cards, tests, OOS seeds, review verdict>

**Not done / deferred**:
- <bullet list>

**Next session candidates**:
- <bullet list, highest-yield first>

**Hazards** (carrying forward):
- <bullet list of gotchas this session surfaced or reinforced>

**Commit prefix used**: <e.g., scutemob-N: + chore: for end-session>
```

**Rotate history**:
1. Move the existing "Last Handoff" content into "Previous Handoff (preserved for chain context)" IF this session is part of a multi-PB chain and the immediate predecessor handoff is still load-bearing. Otherwise rotate it straight to "Handoff History" as the most recent entry.
2. If the existing "Previous Handoff" content rotates into "Handoff History", drop the oldest history entry to maintain a 5-entry window.

If no workstream was ACTIVE (exploratory session), skip the rotation but still
write a one-line "Last Handoff" entry summarizing what happened.

### 4.5. Card authoring operations checkoff (W6 only)

If this session worked on W6 card authoring AND the HISTORICAL
`docs/card-authoring-operations.md` still has unchecked items, you may check off completed
tasks (`- [ ]` → `- [x]`) as a historical-reference courtesy. Record the real completion in
the live campaign tracker `memory/card-authoring/campaign-plan-2026-05-16.md` (§0), and
reference the task IDs in the Last Handoff "Completed" bullet.

> **Note (DOCB-2, `scutemob-132`):** `docs/card-authoring-operations.md` is bannered
> HISTORICAL — it is not the live status source; `campaign-plan-2026-05-16.md` §0 is.
>
> **Fallback**: Once all items in the operations plan are checked off (X-7
> complete), this step becomes a no-op. Skip it for all future sessions.

If no card authoring work was done, skip.

### 5. Memory topic file routing (Step 1 from old /end-session)

Review what happened this session. Route new content to the appropriate file:

- New MTG rules / game logic gotcha → `memory/gotchas-rules.md`
- New infra / testing / builder / im-rs gotcha → `memory/gotchas-infra.md`
- New convention or style decision → `memory/conventions.md`
- New design decision → `memory/decisions.md`
- Cross-cutting operational detail (applies nearly every session) → `memory/MEMORY.md`

**Anti-bloat rules**:
- Do NOT duplicate content already in CLAUDE.md or any memory topic file.
- Do NOT add detailed structural inventories of files/types/functions.
- Remove or correct outdated entries.
- MEMORY.md target: ≤80 lines.

"No changes needed" is a valid outcome. Be honest — most sessions reuse existing
patterns and need no new memory entries.

### 5.5. Topic file freshness (lightweight)

For each memory topic file **read or modified this session**:
- If the file has a `Last verified: M<N>` header and was modified, update it.
- If the file contains a stale entry (gotcha no longer applies, convention changed), correct or remove it.

Only check files already in context. Do not read topic files solely for this step.

### 6. CLAUDE.md minimal update (Step 2 from old /end-session)

Apply only these specific changes if applicable:
- Update **Current State** header (active milestone, status, test count, date) — but
  only the snapshot fields; detailed PB history goes to `memory/workstream-state.md`,
  not here.
- Remove stale content (completed milestones, finished plans).

**Do NOT**:
- Add new gotchas, conventions, or decisions to CLAUDE.md — those go to memory topic files (see Step 5 routing).
- Add detailed file/type/function inventories.
- Expand sections already comprehensive.

**Size guard**: CLAUDE.md target ≤250 lines. If a change would push it over, route
the content to a topic file instead. (Current state: file may be over the guard
due to durable reference tables — Architecture Invariants, Agents, Primary Documents.
Don't reduce those without explicit user direction.)

### 7. End the ESM session

```bash
esm session end <session_id> --summary "<text>"
```

Use the `session_id` from when `/start` ran `esm session start`. If unavailable
(auto-started, lost to compaction, etc.), skip with a note.

The summary should be 1-2 paragraphs, specific. Bad: "worked on features". Good:
"PB-EWC shipped (scutemob-20, merge 9ea3ba8c): EntersWithCounters u32→Box<EffectAmount>
per CR 614.1c. 2 cards, HASH 17→18, +5 tests, 3 OOS-EWC seeds filed."

### 8. Confirm and close

Present a brief summary:

```
## Session ended

### Completed this session
{bulleted list}

### Memory / CLAUDE.md updates
{what was written, or "no changes needed"}

### Open tasks / worktrees
{any in_progress/in_review tasks, or "None"}

### Uncommitted changes
{clean, or list}

### Next session (highest-yield first)
{specific, actionable picks from the handoff "Next session candidates"}

### Commit prefix used
{e.g., scutemob-N: / merge: / chore:}
```

Then ask: **"Ready to close this session?"** — wait for the user's confirmation
before considering the close final. (ESM session may already be ended at this
point; the confirmation is for any final coordinator-side commits.)

## Commit Prefix Reference

| Workstream | Prefix | Example |
|------------|--------|---------|
| W1: Abilities | `W1-B<N>:` | `W1-B3: implement Ninjutsu` |
| W2: TUI & Simulator | `W2:` | `W2: fix blocker declaration UI` |
| W3: LOW Remediation | `W3:` | `W3: add debug_assert to sba.rs` |
| W4: M10 Networking | `W4:` | `W4: add GameServer skeleton` |
| W6: Primitive + Card Authoring | `scutemob-N:` (worker) / `W6-triage:` / `W6-cards:` / `W6-audit:` | `scutemob-20: PB-EWC ...` |
| Cross-cutting / End-session | `chore:` | `chore: end-session — handoff + history rotation` |
| Merge commits | `merge:` (auto-prefixed by `esm worktree merge`) | `merge: scutemob-20 — PB-EWC ...` |

## Notes

- If the ESM server is unreachable, warn but don't block. Git history still
  captures the work; the workstream-state rotation still happens.
- Session auto-expires after 10 minutes without heartbeat. Running `/eot`
  explicitly is preferred so the summary is recorded.
- Worker sessions write their own handoff via `esm task transition` + notes;
  coordinator `/eot` consolidates multi-PB chains in `Previous Handoff`.
- Do NOT touch `~/.claude/skills/end/` or `~/.claude/skills/start/` — those are
  globals managed externally. This skill is the project-local consolidation.
