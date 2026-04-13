---
title: Cleanup Retention Policy
status: active
last_updated: 2026-04-12
---

# Cleanup Retention Policy

Defines what lives where, what gets archived when, and what protocol governs
any cleanup pass on the scutemob project. Read by the `/cleanup` skill on
startup. This is the policy document for end-of-milestone cleanup.

## 1. The two-tier reversibility ladder

Two tiers only. No soft-delete. No expiry calendar.

- **Live** — file is in its working location (`docs/`, `memory/`, `.claude/`,
  repo root). Discoverable by every agent and skill.
- **Archive** — file is under `<area>/archive/<year>-<month>/`. Still in the
  repo, still discoverable by `grep` and `ls`, recoverable by `git mv` back
  to live without git archaeology.

There is no third tier. Archive *is* the soft-delete tier. Files in archive
that turn out to be truly dead may be promoted to delete (`git rm`) by a
future cleanup pass with explicit oversight approval per item.

## 2. Where things live

| Location | Purpose | Examples |
|----------|---------|----------|
| `docs/` | Active design and reference docs referenced from CLAUDE.md or skills | `mtg-engine-architecture.md`, `mtg-engine-roadmap.md` |
| `docs/archive/<year>-<month>/` | Snapshots and superseded docs that may still have research value | dated strategic reviews, codebase snapshots |
| `memory/` | Active project memory: gotchas, conventions, decisions, workstream state, WIP files | `gotchas-rules.md`, `primitive-wip.md` |
| `memory/abilities/` | **Untouchable corpus** | Used as research corpus by `ability-impl-planner` / `ability-impl-runner` |
| `memory/primitives/` | **Untouchable corpus** | Used as research corpus by `primitive-impl-planner` / `primitive-impl-reviewer` |
| `memory/card-authoring/review-*.md` | **Untouchable corpus** (glob-protected) | Globbed by `card-fix-applicator` |
| `memory/archive/<year>-<month>/` | Closed session artifacts with no active inbound references | `etb-trigger-fix-plan.md`, `w3-low-s*-review.md` |
| `memory/cleanup/` | Cleanup pass deliverables: runtime reference maps, dry-run plans | `runtime-reference-map-YYYY-MM-DD.md`, `cleanup-plan-YYYY-MM-DD.md` |
| `.claude/` | Agent + skill definitions, settings, hooks | All agent `.md` + skill `SKILL.md` files |
| Repo root | `CLAUDE.md`, README, config | `CLAUDE.md`, `Cargo.toml`, `.gitignore` |

## 3. Untouchable corpus rules (permanent)

These rules apply to every cleanup pass forever, until oversight explicitly
retires the agent that uses the corpus:

- **`memory/abilities/`** is used by `ability-impl-planner` (parent reference,
  "study similar abilities") and `ability-impl-runner` (glob). Untouchable.
- **`memory/primitives/`** is used by `primitive-impl-planner` and
  `primitive-impl-reviewer` as a research corpus for sibling PB plans and
  reviews. Untouchable.
- **`memory/card-authoring/review-*.md`** is globbed by `card-fix-applicator`.
  Untouchable until that agent is retired. Other files in
  `memory/card-authoring/` (named hard references like `consolidated-fix-list.md`,
  `dsl-gap-audit-v2.md`, `triage-summary.md`) are managed individually.
- **`crates/engine/src/cards/defs/`** is parent-referenced from every card
  authoring agent. Out of doc-cleanup scope entirely.
- **`test-data/generated-scripts/`** is parent-referenced from at least 5
  sources. Out of doc-cleanup scope entirely.

If oversight ever decides to retire one of these as corpus, the right move
is a **content distillation pass** (extract patterns into a summary doc, keep
the raw plans for reference) — that is a separate project, not a cleanup pass.

## 4. Year-month archive convention

Archive subdirectories use `<year>-<month>` format, e.g. `2026-04/`. Every
cleanup pass writes into the current month's directory. The directory contains:

- Archived files preserving their original filenames
- A `README.md` listing what was archived and why, with reference to the
  dry-run plan that authorized each move (one README per batch, sparse,
  curated by hand only when discovery value justifies the curation cost)

This stops the archive from becoming a flat dump and provides chronological
audit. Archive subdirectories are checked into git (Q4 ruling: discoverability
matters more than repo size).

## 5. Cleanup cadence

Cleanup runs at **end-of-milestone**, by the `/cleanup` skill, with this
policy as its protocol document.

- Mid-workstream cleanup is **forbidden**. It creates collision with worker
  sessions. The 2026-04-12 cleanup pass demonstrated the cost: half the work
  had to be deferred until the active workstream closed.
- Cleanup is **event-driven**, not calendar-driven. There is no tickler. The
  trigger is "milestone-reviewer agent has just closed its pass and the next
  milestone hasn't started yet."

## 6. Tier rules

### Tier 1 (T1) — EDIT-IN-PLACE

Always permitted, lowest risk. File stays in place; content is trimmed,
links are fixed, or stale references are corrected.

### Tier 2 (T2) — ARCHIVE

`git mv <path> <area>/archive/<year>-<month>/<original-name>`. Permitted when:
- File has any prior reference, even stale
- Content has design rationale value (captured user preferences, historical
  decisions, completed-workstream review findings)
- File is a session-scoped artifact for a closed workstream with no remaining
  active inbound references

### Tier 3 (T3) — DELETE

`git rm <path>`. Permitted only when **all** of:
- Gate A (runtime reference map) returns zero references anywhere
- Content is purely scratch/ephemeral with no design rationale
- Oversight explicitly approves the delete per item

**Default tier for ambiguous items: T2 (archive).** The cleanup agent may not
promote an item to T3 without explicit oversight approval per item.

## 7. Commit budget

Every cleanup pass declares a worker-disruption budget upfront:

- **Target**: ≤2 commits during any active workstream (low-collision file
  types only — `.claude/`, root cruft, memory ephemera)
- **Target**: ≤4 commits deferred until workstreams complete
- **Hard limit**: 6 total commits per pass

If a pass needs more than 6 commits, the work is too entangled — split or
defer items rather than weakening the budget.

## 8. The B.7 hard rules (standing protocol for any in-flight workstream)

These rules apply to any cleanup work conducted while any workstream is
ACTIVE (per `memory/workstream-state.md`):

1. **Never** use `git add -A`, `git add .`, or `git add memory/` while
   uncommitted worker changes are present in the working tree. Always use
   explicit per-file `git add <path>`.
2. **Re-run `git status`** immediately before every commit. If the worker has
   added new modified files since the last status check, re-evaluate
   collision surface before committing.
3. **Never edit** the worker's WIP file (`memory/primitive-wip.md`,
   `memory/ability-wip.md`), the worker's row in `memory/workstream-state.md`,
   or any file the worker is actively writing (e.g., a `pb-plan-N.md` that is
   untracked). Even an EDIT-IN-PLACE typo fix is forbidden — those files
   belong to the workstream.
4. **Do not touch** active-plan files for in-flight workstreams. For W6, this
   means `docs/primitive-card-plan.md` Phase 1.8 is off-limits for the entire
   PB duration.
5. **Do not edit CLAUDE.md "Current State", "Active Milestone", "Active Plan",
   or "Last Updated" lines** for the duration of any in-flight workstream.
   Other CLAUDE.md sections are fair game with care.
6. **Each cleanup commit must rebase cleanly** against any worker commit
   landed since the previous cleanup commit. Verify with `git status` before
   commit and `git log --oneline -5` after commit to confirm linear history.
7. **If the worker session commits a new file into a corpus directory** during
   the cleanup pass, the file becomes part of the protected corpus
   immediately and the rule in §3 takes over.
8. **Halt and re-evaluate** if `git status` ever shows a modified file the
   cleanup agent did not modify and the worker has not yet committed. That
   signals either a third concurrent session or a worker mid-edit; in either
   case, cleanup pauses until the working tree is understood.
8a. **Implement-phase halts default to defer, not proceed.** If rule #8 fires
    during a workstream's implement phase (the worker has uncommitted source
    code or test changes present in the working tree, not just plan/wip
    files), default to deferring the entire cleanup pass until the workstream
    closes — even when collision math shows clean isolation between the
    cleanup-permitted scope and the worker's uncommitted set. The implement
    phase is the moment of highest worker volatility; the marginal benefit of
    landing cleanup commits during it does not justify the residual risk of
    interleaved worker stop-and-flag, context juggling, or commits that need
    to be reverted because they conflict with something the worker hadn't yet
    staged. Plan phase is contemplative and may proceed with explicit
    per-file `git add`. Implement phase defers. Recorded after the 2026-04-12
    cleanup pass: B.7 #8 fired during PB-N implementation, the cleanup agent
    recommended Option A (proceed with provably-clean isolation), oversight
    overrode to Option B (defer everything), and the override is now standing
    rule.

## 9. What is NOT in scope for any cleanup pass

- **Auto-memory MEMORY.md** at
  `/home/airbaggie/.claude/projects/-home-airbaggie-scutemob/memory/MEMORY.md`
  — lives outside the repo, has its own oversize warning system and cleanup
  protocol. The cleanup pass may update *references* to it from `.claude/` or
  CLAUDE.md but does not edit the file itself.
- **Source code** under `crates/`, `tools/`, `benches/` — code is governed
  by its own review process (`milestone-reviewer`, code-review fix sessions).
- **Card definition files** under `crates/engine/src/cards/defs/` — too
  numerous, too volatile, governed by the card-authoring pipeline.
- **Test data and generated scripts** under `test-data/` — owned by the
  testing system.
- **Gitignored files** at the repo root or anywhere else — not version-
  controlled, the user owns them. The cleanup pass may remove dangling
  *references* to gitignored files (e.g., a CLAUDE.md table row for a
  gitignored snapshot) but never touches the file itself.

## 10. Protocol summary

Every cleanup pass follows these gates in order:

1. **Gate A** — Runtime reference map. Scans `.claude/`, root config,
   CLAUDE.md, MEMORY.md, and produces an untouchable file index. Halt for
   oversight review.
2. **Gate B** — Workstream collision check. Reads
   `memory/workstream-state.md` and the WIP files of any ACTIVE workstream;
   produces a collision report. Halt for oversight review.
3. **Gate C/D/E/F** — Dry-run plan. Per-item action table with reversibility
   tiers, per-commit unified diffs, post-state grep verification, worker-
   disruption budget. Halt for oversight review.
4. **Execute** — One commit at a time, with re-verification grep before and
   after each. Halt on any unexpected state change.

The `/cleanup` skill orchestrates this protocol. The cleanup agent may not
combine gates and may not skip the halt-for-oversight checkpoints.
