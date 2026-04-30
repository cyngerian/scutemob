---
name: cleanup
description: Run an end-of-milestone cleanup pass following the gated protocol — runtime reference map → workstream collision check → dry-run plan → execute. Reads docs/cleanup-retention-policy.md as the protocol document. Never run mid-workstream.
---

# Cleanup

Run a project cleanup pass at end-of-milestone. This skill orchestrates the
gated protocol from `docs/cleanup-retention-policy.md`. The protocol exists
because cleanup work has high blast radius (affects what every future agent
and skill loads) and must not collide with in-flight workstreams.

## When to run

- **End-of-milestone**: after `milestone-reviewer` closes its pass and before
  the next milestone starts. The working tree is clean, no workstreams are
  active, and the cleanup has zero collision surface.
- **Never mid-workstream**: collision risk. If a workstream is ACTIVE per
  `memory/workstream-state.md`, the protocol forces nearly all real work into
  the deferred bucket, and the cost of running the gates exceeds the cleanup
  yield.
- **Never on a tickler**: cadence is event-driven, not calendar-driven.

## First steps

1. Read `docs/cleanup-retention-policy.md` in full. This is the protocol
   document; do not skim.
2. Read `memory/workstream-state.md` to confirm no workstreams are ACTIVE.
   If any are, **halt** and report to the user — recommend re-running after
   the workstream closes.
3. Read `git status` and `git log --oneline -10`. Verify the working tree is
   clean and the last several commits are coherent.

## Procedure

### Gate A — Runtime reference map

Produce `memory/cleanup/runtime-reference-map-<YYYY-MM-DD>.md`:

1. Scan every `.claude/agents/*.md` and `.claude/skills/**/SKILL.md` for path
   strings that point at files in the repo. Classify each as **hard** (literal
   complete path), **glob** (pattern with wildcards), or **parent** (directory
   used as a search root).
2. Scan `.claude/settings.json` and `.claude/settings.local.json` for hooks
   and any path strings.
3. Scan `CLAUDE.md` Primary Documents table, "When to Load What" table, MCP
   Resources section, Milestone Checklist, Agents table, and any inline
   references.
4. Scan the auto-memory MEMORY.md (at the long absolute path) for index lines
   that reference repo files.
5. Use the `Glob` tool to expand every glob pattern and to enumerate the
   files in every parent reference. Use `Read` (or existence check) to verify
   every hard reference points at a real file. Flag missing-target references
   as pre-existing bugs but **do not fix them inside Gate A**.
6. Output the map in the format from the 2026-04-12 reference map at
   `memory/cleanup/runtime-reference-map-2026-04-12.md`. Sections:
   1. Untouchable file index (alphabetical)
   2. Glob and parent references (highest risk)
   3. Per-source breakdown
   4. Missing targets (existing bugs)
   5. Questions surfaced
   6. Scan completeness checklist
7. **Halt** for oversight review. Do not proceed to Gate B until oversight
   approves.

### Gate B — Workstream collision check

Append a "Workstream Collision Report" section to the same file (or write a
sibling file `memory/cleanup/workstream-collision-report-<YYYY-MM-DD>.md`):

1. Read `memory/workstream-state.md` and list every workstream by status.
2. For every ACTIVE workstream, read its WIP file (e.g.
   `memory/primitive-wip.md`, `memory/ability-wip.md`) and enumerate the
   files it is currently mutating.
3. Run `git status` and capture every uncommitted modification. Each
   uncommitted file the cleanup agent did not modify is a worker-owned file.
4. Cross-reference against the Gate A untouchable index. Files that appear
   in both are highest-risk paths.
5. Document the B.7 hard rules from the retention policy as the standing
   protocol for the cleanup pass.
6. **Halt** for oversight review.

### Gate C/D/E/F — Dry-run plan

Produce `memory/cleanup/cleanup-plan-<YYYY-MM-DD>.md`. Required sections:

1. Per-item action table with: path, action, tier (T1/T2/T3), tag (safe
   during workstream / deferred until workstream closes), commit number,
   dependency notes
2. Per-commit dry-run diffs — exact `git mv`, `git rm`, file edits as proposed
   unified diff, commit message draft
3. Re-verification protocol — before/after grep for each commit
4. Retention policy doc updates if needed (usually not — the policy is
   stable)
5. Post-state grep verification (Gate F)
6. Worker-disruption budget — total commit count split between safe and
   deferred (target: ≤2 + ≤4, hard limit 6)
7. Items requiring explicit oversight approval (AWAITING APPROVAL queue)
8. Items demoted or removed from any input audit

**Halt** for oversight review. The plan is the contract. Do not execute
until oversight signs off.

### Execute

One commit at a time, with re-verification grep before and after each.

For each commit:
1. Re-run the per-commit re-verification grep from the dry-run plan
2. Check `git status` — has anything changed since the plan was approved?
3. Check `memory/workstream-state.md` — is any workstream still ACTIVE?
4. Make the file edits / `git mv` / `git rm` operations from the plan
5. `git status` — verify only the planned files are staged
6. **Explicit per-file `git add <path>`** — never `git add -A` or `git add .`
7. `git commit -m "..."` with the planned message
8. `git log --oneline -5` — verify linear history
9. Re-run the post-commit grep verification
10. **Halt** before the next commit if anything is unexpected

## Hard rules

The B.7 hard rules from `docs/cleanup-retention-policy.md` §8 apply to every
commit during any in-flight workstream. They are repeated in the policy doc;
do not duplicate them here.

## Halting points

- After Gate A: oversight must approve before Gate B
- After Gate B: oversight must approve before the dry-run plan
- After the dry-run plan: oversight must approve before any commit
- After each commit: re-run the grep verification before the next commit
- On any unexpected state change at any point: halt and report

## Stop-and-flag triggers

- A file flagged for deletion that turns out to be referenced from `.claude/`
- A "duplicate" that on read serves a distinct audience
- A workstream collision the worker has not yet committed
- A file the input audit classified as orphaned that is actually indexed
  somewhere the audit did not scan
- A `.gitignore` entry covering a file the input audit flagged for deletion
- Ambiguity about whether an artifact has historical value

When in doubt, demote one tier (T3 → T2 → T1) rather than guessing.
