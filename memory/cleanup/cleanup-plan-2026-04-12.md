# Cleanup Dry-Run Plan — 2026-04-12

> Gate C/D/E/F deliverable for the cleanup counter-proposal.
>
> **Status**: APPROVED with override. All 6 commits DEFERRED until PB-N close.
> Cleanup agent is halted. No execution until oversight pings to resume.
> **Predecessor gates**: A (runtime reference map) and B (workstream collision report)
> already approved. See `memory/cleanup/runtime-reference-map-2026-04-12.md`.
> **Active workstream during planning**: W6 / PB-N (worker session in implement
> phase as of plan approval — 76+ uncommitted source/test/card-def files).
> **Commit budget**: 0 safe + 6 deferred = 6 total. AT BUDGET.

## OVERSIGHT OVERRIDE — 2026-04-12 (post-Gate-C approval)

After this dry-run plan was drafted and Gate B was approved, the W6 worker
phase-shifted from **planning** to **implementing** PB-N. `git status` showed
76+ uncommitted files across `crates/engine/src/cards/defs/`,
`crates/engine/src/`, and `crates/engine/tests/`. B.7 rule #8
("halt and re-evaluate on unexpected state change") fired correctly.

The cleanup agent re-evaluated and recommended **Option A** (proceed with
C1+C2 since the collision math was provably zero — none of the C1/C2 files
overlapped the worker's uncommitted set). Oversight **overrode to Option B**:
defer all 6 commits until PB-N closes. Reasons recorded:

1. Worker is mid-implement, not mid-plan — highest volatility moment.
2. Marginal time cost of waiting is near zero (PB-N hours from close).
3. Cleaner git history — single coherent post-PB-N batch reads as one event.
4. Risk asymmetry favors deferral — cost of waiting is marginal; cost of A
   failing is worker disruption during hot implementation.
5. The protocol fired correctly — "halt and re-evaluate" does not have to
   mean "proceed." Conservative read of unprecedented working-tree churn is
   "wait until churn settles."
6. **New standing rule** recorded: if B.7 #8 fires during a workstream's
   implement phase (uncommitted source code or test changes present), default
   to deferring the entire cleanup pass even when collision math shows clean
   isolation. Plan phase is contemplative and may proceed with explicit
   per-file `git add`. Implement phase defers.

This new rule is encoded as **Rule 8a** in the retention policy doc draft
(§4 of this plan) and will land with C3.

**Trigger condition for execution**:
- PB-N close commit visible in `git log` (worker commits with message
  matching `PB-N close` or `W6.*PB-N.*close`)
- AND `git status` clean modulo any subsequent worker activity
- AND oversight pings the cleanup agent to resume

**Cleanup agent action right now**: halt completely. Do not execute any
commit. Do not modify the dry-run plan beyond this header note and the
two follow-up edits noted below. Wait for the trigger.

**When the trigger fires**: cleanup agent re-runs Gate B (single section
append to `runtime-reference-map-2026-04-12.md` titled "post-PB-N collision
recheck"), and only then begins execution of C1 → C6 in order with
re-verification before each.

## Table of contents

1. Per-item action table (Gate C reversibility classification)
2. Per-commit dry-run diffs (Gate D index-first sequencing + Gate F simulated diff)
3. Re-verification protocol (per-commit safety harness)
4. Retention policy doc draft (full content of `docs/cleanup-retention-policy.md`)
5. `/cleanup` skill draft (full content of `.claude/skills/cleanup/SKILL.md`)
6. Post-state grep verification (Gate F)
7. Worker-disruption budget
8. Items requiring explicit oversight approval (AWAITING APPROVAL queue)
9. Items demoted or removed from the original audit
10. Open questions captured for the next cleanup pass

---

## 1. Per-item action table

Every item touched by this cleanup, classified per the Gate C reversibility ladder
and tagged with the Gate B PB-N collision tag.

| # | Path | Action | Tier | Tag | Commit | Dependency notes |
|---|------|--------|------|-----|--------|------------------|
| 1 | `.claude/agents/ability-coverage-auditor.md` | Edit line 60: `crates/engine/src/cards/definitions.rs` → `crates/engine/src/cards/defs/` | T1 | deferred until PB-N closes (override) | C1 | none |
| 2 | `.claude/skills/next-ability/SKILL.md` | Edit line 51: `crates/engine/src/cards/definitions.rs` → `crates/engine/src/cards/defs/` | T1 | deferred until PB-N closes (override) | C1 | none |
| 3 | `.claude/agents/rules-implementation-planner.md` | Edit line 10: `memory/m8-session-plan.md` → `memory/m<N>-session-plan.md` (template form) | T1 | deferred until PB-N closes (override) | C1 | none |
| 4 | `.claude/skills/start-session/SKILL.md` | Edit line 136: replace `m8-session-plan.md` parenthetical with `m<N>-session-plan.md` | T1 | deferred until PB-N closes (override) | C1 | none |
| 5 | `.claude/skills/start-milestone/SKILL.md` | Edit line 13: drop `e.g. memory/m8-session-plan.md` literal example | T1 | deferred until PB-N closes (override) | C1 | none |
| 6 | `memory/feedback_feature_branch_workflow.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes (override) | C2 | content has design rationale ("user explicitly requested for PB-20") |
| 7 | `memory/feedback_worktree_parallel.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes (override) | C2 | content has design rationale ("happened during W3-LC + W6-PB19 parallel work") |
| 8 | `memory/archive/2026-04/README.md` | Create new file (archive batch index) | T1 (new file) | deferred until PB-N closes (override) | C2 | none |
| 9 | `docs/cleanup-retention-policy.md` | Create new file (full policy) | T1 (new file) | deferred until PB-N closes | C3 | C4 adds CLAUDE.md row for it |
| 10 | `.claude/skills/cleanup/SKILL.md` | Create new file (skill that runs the protocol) | T1 (new file) | deferred until PB-N closes | C3 | references file from #9 |
| 11 | `memory/archive/2026-04/README.md` | Edit: append reference to retention policy doc now that it exists | T1 | deferred until PB-N closes | C3 | depends on #9 |
| 12 | `CLAUDE.md` line 67 | Remove `Codebase Analysis` row from Primary Documents table | T1 | deferred until PB-N closes | C4 | **AWAITING OVERSIGHT APPROVAL** — file is gitignored, see §8 |
| 13 | `CLAUDE.md` Primary Documents table | Add `Cleanup Retention Policy` row | T1 | deferred until PB-N closes | C4 | depends on #9 |
| 14 | `CLAUDE.md` Agents table | Add `/cleanup` row OR add `Cleanup` entry to "When to Load What" | T1 | deferred until PB-N closes | C4 | depends on #10 |
| 15 | `CLAUDE.md` line 172 | Reword `memory/MEMORY.md` reference in §MCP Resources per Q3 ruling option (b) | T1 | deferred until PB-N closes | C4 | none |
| 16 | `memory/etb-trigger-fix-plan.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes | C5 | zero external refs |
| 17 | `memory/forage-dsl-plan.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes | C5 | zero external refs |
| 18 | `memory/w3-low-s1-review.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes | C5 | zero external refs |
| 19 | `memory/w3-low-s2-review.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes | C5 | zero external refs |
| 20 | `memory/w3-low-s3-review.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes | C5 | zero external refs |
| 21 | `memory/w3-low-s4-review.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes | C5 | zero external refs |
| 22 | `memory/w3-low-s5-review.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes | C5 | zero external refs |
| 23 | `memory/w3-low-s6-review.md` | `git mv` → `memory/archive/2026-04/` | T2 | deferred until PB-N closes | C5 | zero external refs |
| 24 | `docs/engine_explanation.md` | Read in detail; add audience-disambiguating header + cross-ref to `docs/mtg-engine-architecture.md` | T1 | deferred until PB-N closes | C6 | judgment call at execution time |
| 25 | `docs/mtg-engine-architecture.md` | Add reciprocal cross-ref to `docs/engine_explanation.md` | T1 | deferred until PB-N closes | C6 | depends on #24 |
| 26 | `docs/scutemob-architecture-review.md` | Read in detail; default action: T2 archive to `docs/archive/2026-04/` (date-and-archive) | T2 | deferred until PB-N closes | C6 | judgment call at execution time |

**Counts**: 26 items / 6 commits / 2 safe + 4 deferred / no T3 deletes proposed.

## 2. Per-commit dry-run diffs

### C1 [deferred until PB-N closes — override] — Stale-path fixes in agents/skills

**Commit message**:

```
chore: cleanup pre-flight — fix 5 stale path references in agents/skills

Per the cleanup counter-proposal Q2 ruling: three references to the long-moved
crates/engine/src/cards/definitions.rs file (the path is now defs/) and three
literal "memory/m8-session-plan.md" example references (M8 is shipped, the
template form is m<N>-session-plan.md).

No CLAUDE.md or memory/ touches. Zero W6/PB-N collision surface.

Refs: memory/cleanup/cleanup-plan-2026-04-12.md item #1-5
```

**Files touched** (5 .claude/ files, 0 docs/, 0 memory/, 0 root):

#### Edit 1: `.claude/agents/ability-coverage-auditor.md`

```diff
@@ -57,7 +57,7 @@ Record the file(s) and line numbers where the ability is implemented or referen
 #### b. Check Card Definitions

 ```
-Grep for the ability name in crates/engine/src/cards/definitions.rs
+Grep for the ability name in crates/engine/src/cards/defs/
 ```

 Record which card definitions use this ability.
```

#### Edit 2: `.claude/skills/next-ability/SKILL.md`

```diff
@@ -48,7 +48,7 @@
    - **Step 3 (trigger wiring)**: Only applies to trigger-based abilities. Grep
      `crates/engine/src/` for a matching `TriggerCondition` variant and check if it's wired
      to runtime dispatch.
    - **Step 4 (unit tests)**: Grep `crates/engine/tests/` for the ability name or CR number.
-   - **Step 5 (card definition)**: Grep `crates/engine/src/cards/definitions.rs` for the
+   - **Step 5 (card definition)**: Grep `crates/engine/src/cards/defs/` for the
      ability name.
    - **Step 6 (game script)**: Grep `test-data/generated-scripts/` for the ability name.
```

#### Edit 3: `.claude/agents/rules-implementation-planner.md`

```diff
@@ -7,7 +7,7 @@ description: |

   <example>
   Context: M8 is the active milestone and no session plan exists yet
   user: "plan M8 implementation"
-  assistant: "I'll research CR 614-616 (replacement/prevention effects), audit the codebase for interception sites, and produce a session plan at memory/m8-session-plan.md."
+  assistant: "I'll research CR 614-616 (replacement/prevention effects), audit the codebase for interception sites, and produce a session plan at memory/m<N>-session-plan.md."
   <commentary>Triggered by explicit planning request for a milestone.</commentary>
   </example>
```

**Note**: this change replaces a literal `m8` example inside an agent description string with the template form. The surrounding context still references "M8 is the active milestone" because that's the example scenario, not a literal file path. The fix is minimal — only the file path string changes.

#### Edit 4: `.claude/skills/start-session/SKILL.md`

```diff
@@ -133,7 +133,7 @@

 ## Step 6: Session plan check

-Check if a session plan file exists in `memory/` (e.g., `m8-session-plan.md`). If one exists, call it out prominently: **"Session plan found: `memory/m<N>-session-plan.md` — use `/start-milestone <N>` to load it without touching the roadmap."** Do not read it unless the developer asks.
+Check if a session plan file exists in `memory/` (e.g., `m<N>-session-plan.md` for the active milestone). If one exists, call it out prominently: **"Session plan found: `memory/m<N>-session-plan.md` — use `/start-milestone <N>` to load it without touching the roadmap."** Do not read it unless the developer asks.
```

#### Edit 5: `.claude/skills/start-milestone/SKILL.md`

```diff
@@ -10,8 +10,7 @@ Given a milestone number (`$ARGUMENTS`, e.g. "8" or "M8"), do the following:

 ## Step 1: Check for a session plan

-Check if `memory/m<N>-session-plan.md` exists (where `<N>` is the milestone number, e.g.
-`memory/m8-session-plan.md`).
+Check if `memory/m<N>-session-plan.md` exists (where `<N>` is the milestone number).
```

**Bash commands** (run in order):

```bash
git status                                                # verify clean working tree (modulo W6 in-flight)
# (5 Edit tool calls — see diffs above)
git status                                                # verify only the 5 .claude/ files are staged
git add .claude/agents/ability-coverage-auditor.md \
        .claude/skills/next-ability/SKILL.md \
        .claude/agents/rules-implementation-planner.md \
        .claude/skills/start-session/SKILL.md \
        .claude/skills/start-milestone/SKILL.md
git commit -m "chore: cleanup pre-flight — fix 5 stale path references in agents/skills

[full message above]"
git log --oneline -3                                      # verify linear history
```

**Re-verification grep (Gate F)**:

```bash
# Expect zero results in any of these:
Grep pattern="cards/definitions\.rs" path=".claude/"
Grep pattern="memory/m8-session-plan" path=".claude/"
```

---

### C2 [deferred until PB-N closes — override] — Archive 2 misplaced auto-memory feedback files + create archive index

**Commit message**:

```
chore: cleanup — archive 2 misplaced auto-memory feedback files

Both files use auto-memory frontmatter (name/description/type) but live in the
project memory/ directory rather than the auto-memory path. Neither is indexed
in the auto-memory MEMORY.md. Content has design rationale (captured user
preferences) so T2 archive, not T3 delete. The content may be migrated into
the auto-memory system in a separate session.

Also creates memory/archive/2026-04/README.md as the per-batch index.

Refs: memory/cleanup/cleanup-plan-2026-04-12.md items #6, #7, #8
```

**Files touched** (2 mvs in memory/, 1 new file in memory/archive/):

#### Move 1
`git mv memory/feedback_feature_branch_workflow.md memory/archive/2026-04/feedback_feature_branch_workflow.md`

#### Move 2
`git mv memory/feedback_worktree_parallel.md memory/archive/2026-04/feedback_worktree_parallel.md`

#### New file: `memory/archive/2026-04/README.md`

```markdown
# Archive batch — 2026-04

Files moved here as part of the cleanup pass on 2026-04-12. This batch
contains misplaced auto-memory feedback files that lived in the project
memory directory but never made it into the auto-memory index. Content
preserved for possible later migration to the auto-memory system at:

  /home/airbaggie/.claude/projects/-home-airbaggie-scutemob/memory/MEMORY.md

The dry-run plan that authorized each move is in
`memory/cleanup/cleanup-plan-2026-04-12.md`.

The retention policy doc that defines the archive convention will land in
`docs/cleanup-retention-policy.md` after PB-N closes.

## Files in this batch

- `feedback_feature_branch_workflow.md` — captured user preference for PB feature-branch workflow
- `feedback_worktree_parallel.md` — captured user preference for git worktrees in parallel sessions
```

**Bash commands**:

```bash
git status                                                # re-verify W6 state unchanged
mkdir -p memory/archive/2026-04                           # parent dir for the moves
git mv memory/feedback_feature_branch_workflow.md \
       memory/archive/2026-04/feedback_feature_branch_workflow.md
git mv memory/feedback_worktree_parallel.md \
       memory/archive/2026-04/feedback_worktree_parallel.md
# Write the new README.md (Write tool)
git status                                                # verify only the 2 mvs and 1 new file
git add memory/archive/2026-04/README.md                  # explicit add for the new file
git commit -m "chore: cleanup — archive 2 misplaced auto-memory feedback files

[full message above]"
git log --oneline -3
```

**Re-verification grep (Gate F)**:

```bash
# Expect: matches only inside memory/archive/2026-04/ and memory/cleanup/
Grep pattern="feedback_feature_branch_workflow"
Grep pattern="feedback_worktree_parallel"

# Expect: file no longer at original path
ls memory/feedback_feature_branch_workflow.md  # should error
ls memory/feedback_worktree_parallel.md         # should error
ls memory/archive/2026-04/                      # should list 3 entries: 2 .md + README.md
```

---

### C3 [deferred until PB-N closes] — Create retention policy doc + /cleanup skill

**Commit message**:

```
chore: cleanup retention policy + /cleanup skill

Adds the foundation for future cleanup passes:
- docs/cleanup-retention-policy.md — two-tier ladder, year-month archive
  convention, end-of-milestone cadence, B.7 hard rules
- .claude/skills/cleanup/SKILL.md — runs the gated protocol (Gate A → B →
  dry-run → execute) and references the policy doc

Also updates memory/archive/2026-04/README.md to reference the now-existing
policy doc.

Does NOT touch CLAUDE.md — Primary Documents row + Agents/Skills row land in
C4 to bundle all CLAUDE.md edits into one commit.

Refs: memory/cleanup/cleanup-plan-2026-04-12.md items #9, #10, #11
```

**Files touched** (2 new files, 1 edit):

- `docs/cleanup-retention-policy.md` — full content in §4 of this plan
- `.claude/skills/cleanup/SKILL.md` — full content in §5 of this plan
- `memory/archive/2026-04/README.md` — append a "see policy" line

#### Edit to `memory/archive/2026-04/README.md`

```diff
@@ -8,8 +8,7 @@ preserved for possible later migration to the auto-memory system at:
 The dry-run plan that authorized each move is in
 `memory/cleanup/cleanup-plan-2026-04-12.md`.

-The retention policy doc that defines the archive convention will land in
-`docs/cleanup-retention-policy.md` after PB-N closes.
+The retention policy doc that defines the archive convention is at
+`docs/cleanup-retention-policy.md`. Read it before running another cleanup pass.
```

**Bash commands**:

```bash
# CRITICAL: verify PB-N has closed before this commit fires
git log --oneline -10 | grep "PB-N close"                # must show a PB-N close commit
git status                                                # verify clean working tree
mkdir -p .claude/skills/cleanup
# Write tool: docs/cleanup-retention-policy.md (content from §4 of this plan)
# Write tool: .claude/skills/cleanup/SKILL.md      (content from §5 of this plan)
# Edit tool: memory/archive/2026-04/README.md      (diff above)
git add docs/cleanup-retention-policy.md \
        .claude/skills/cleanup/SKILL.md \
        memory/archive/2026-04/README.md
git commit -m "chore: cleanup retention policy + /cleanup skill

[full message above]"
```

---

### C4 [deferred until PB-N closes] — All CLAUDE.md edits in one commit

**Commit message**:

```
chore: CLAUDE.md cleanup — index updates + MEMORY.md reword + retention policy entry

Bundles all CLAUDE.md edits into one commit to minimize merge-conflict surface
with the W6 worker (which touches CLAUDE.md on PB-N close):

1. Add Cleanup Retention Policy row to Primary Documents table (post-C3)
2. Add /cleanup row to the appropriate skills/agents table
3. Reword memory/MEMORY.md reference in §MCP Resources per Q3 ruling option (b)
4. Remove Codebase Analysis row from Primary Documents table — file is gitignored
   and not present in fresh clones, so the table row is structurally broken
   (AWAITING OVERSIGHT APPROVAL per dry-run plan §8)

Item 4 is conditional on explicit oversight approval. If approval is withheld,
this commit lands without item 4 and the codebase_analysis row stays as-is.

Refs: memory/cleanup/cleanup-plan-2026-04-12.md items #12, #13, #14, #15
```

#### Edit 1: Remove `Codebase Analysis` row (CONDITIONAL on oversight approval)

```diff
@@ -64,7 +64,6 @@
 | Card Authoring Operations | `docs/card-authoring-operations.md` | Ordered task list for triage → fix → author → audit (68 tasks) |
 | Runtime Integrity | `docs/mtg-engine-runtime-integrity.md` | Watchdog, recovery, bug reporting — pre-alpha requirement |
 | Type Consolidation Plan | `docs/mtg-engine-type-consolidation.md` | Pre-M10 refactoring: CastSpell, SOK triggers, AbilityDef, Designations — 8 sessions |
-| Codebase Analysis | `codebase_analysis_220260228.md` | Comprehensive codebase snapshot (2026-02-28): architecture, file inventory, stats |
 | This file | `CLAUDE.md` | Current project state; session context |
```

#### Edit 2: Add `Cleanup Retention Policy` row

```diff
@@ -64,6 +64,7 @@
 | Card Authoring Operations | `docs/card-authoring-operations.md` | Ordered task list for triage → fix → author → audit (68 tasks) |
 | Runtime Integrity | `docs/mtg-engine-runtime-integrity.md` | Watchdog, recovery, bug reporting — pre-alpha requirement |
 | Type Consolidation Plan | `docs/mtg-engine-type-consolidation.md` | Pre-M10 refactoring: CastSpell, SOK triggers, AbilityDef, Designations — 8 sessions |
+| Cleanup Retention Policy | `docs/cleanup-retention-policy.md` | Two-tier ladder, year-month archive convention, /cleanup skill protocol |
 | This file | `CLAUDE.md` | Current project state; session context |
```

#### Edit 3: Reword line 172 §MCP Resources MEMORY.md reference

Per Q3 ruling: option (b) for CLAUDE.md (human-facing reword without literal path).

```diff
@@ -169,7 +169,7 @@ MCP Resources
 - **Rules search**: query by rule number ("613.8") or concept ("dependency continuous effects")
 - **Card lookup**: query by exact card name for oracle text, types, rulings
 - **Rulings search**: query by interaction concept ("copy effect on double-faced card")
-- **rust-analyzer**: semantic code navigation — hover, definition, references, implementations, incoming/outgoing calls, workspace symbols. Call `rust_analyzer_stop` when done to free ~2.5GB RAM. First call triggers ~70s indexing warmup. Results default to 50 max; pass `limit` to override. See `memory/MEMORY.md` "rust-analyzer MCP Server" section for details.
+- **rust-analyzer**: semantic code navigation — hover, definition, references, implementations, incoming/outgoing calls, workspace symbols. Call `rust_analyzer_stop` when done to free ~2.5GB RAM. First call triggers ~70s indexing warmup. Results default to 50 max; pass `limit` to override. See your auto-memory MEMORY.md index (rust-analyzer MCP Server section) for details.
```

#### Edit 4: Add `/cleanup` row to a CLAUDE.md table

This requires deciding which table to add it to. Looking at the existing structure:

- **Agents table** (line 194-212 area): lists named agents and their triggers/purposes. The `/cleanup` skill is not an agent.
- **When to Load What table** (line 74 area): maps tasks to "load before starting" file references. A `/cleanup` entry fits here as `Task: end-of-milestone cleanup → Load: docs/cleanup-retention-policy.md`.

Default: add to "When to Load What" table.

```diff
@@ -85,6 +85,7 @@ Use `/review-subsystem <name>` to load the right file and see open issues in one
 | Implementing a keyword ability | `docs/mtg-engine-ability-coverage.md` |
 | Checking ability gaps | Use `/audit-abilities` or `/ability-status` |
 | Implementing a single ability end-to-end | Use `/implement-ability` — orchestrates plan → implement → review → fix → card → script → close |
+| End-of-milestone cleanup pass | Use `/cleanup` — reads `docs/cleanup-retention-policy.md` and runs Gate A → B → dry-run → execute |
 | Fixing LOW issues | `docs/mtg-engine-low-issues-remediation.md` |
 | Authoring card definitions | `docs/card-authoring-operations.md` (operations plan with ordered tasks); `docs/mtg-engine-card-pipeline.md` (DSL reference) |
```

**Bash commands**:

```bash
# CRITICAL: verify PB-N has closed AND CLAUDE.md is at known state
git log --oneline -5                                      # confirm PB-N close commit landed
git status                                                # working tree must be clean
git diff CLAUDE.md                                        # should be empty
# 4 Edit tool calls (or 3 if oversight withholds approval for item 4)
git diff CLAUDE.md                                        # review the bundled changes
git add CLAUDE.md
git commit -m "chore: CLAUDE.md cleanup — index updates + MEMORY.md reword + retention policy entry

[full message above]"
```

**Re-verification grep (Gate F)**:

```bash
Grep pattern="codebase_analysis" path="CLAUDE.md"        # 0 results IF item 4 lands; otherwise 1
Grep pattern="cleanup-retention-policy" path="CLAUDE.md" # 1 result (the new row)
Grep pattern="memory/MEMORY\.md" path="CLAUDE.md"        # 0 results (rewording removes the literal)
Grep pattern="/cleanup" path="CLAUDE.md"                 # 1 result (the new row)
```

---

### C5 [deferred until PB-N closes] — Archive 8 closed session artifacts

**Commit message**:

```
chore: archive 8 closed session artifacts to memory/archive/2026-04/

T2 archive (not delete) — content has historical value but no remaining
inbound references. Files moved:

  memory/etb-trigger-fix-plan.md          (B14 ETB fix, merged 2026-03-09)
  memory/forage-dsl-plan.md               (PB-6 Forage primitive, shipped)
  memory/w3-low-s1-review.md              (W3 LOW sprint S1 review)
  memory/w3-low-s2-review.md              (W3 LOW sprint S2 review)
  memory/w3-low-s3-review.md              (W3 LOW sprint S3 review)
  memory/w3-low-s4-review.md              (W3 LOW sprint S4 review)
  memory/w3-low-s5-review.md              (W3 LOW sprint S5 review)
  memory/w3-low-s6-review.md              (W3 LOW sprint S6 review)

Three other historical artifacts (benchmark-results.md,
engine-core-complete-checkpoint.md, w3-layer-audit.md) are NOT archived this
pass — they have inbound references from active docs or from the protected
memory/primitives/ corpus that this cleanup may not touch. Defer to a future
pass after the references are themselves cleaned up.

Refs: memory/cleanup/cleanup-plan-2026-04-12.md items #16-23
```

**Bash commands**:

```bash
git status                                                # working tree clean
git mv memory/etb-trigger-fix-plan.md  memory/archive/2026-04/etb-trigger-fix-plan.md
git mv memory/forage-dsl-plan.md       memory/archive/2026-04/forage-dsl-plan.md
git mv memory/w3-low-s1-review.md      memory/archive/2026-04/w3-low-s1-review.md
git mv memory/w3-low-s2-review.md      memory/archive/2026-04/w3-low-s2-review.md
git mv memory/w3-low-s3-review.md      memory/archive/2026-04/w3-low-s3-review.md
git mv memory/w3-low-s4-review.md      memory/archive/2026-04/w3-low-s4-review.md
git mv memory/w3-low-s5-review.md      memory/archive/2026-04/w3-low-s5-review.md
git mv memory/w3-low-s6-review.md      memory/archive/2026-04/w3-low-s6-review.md
git status                                                # verify exactly 8 renames staged
git commit -m "chore: archive 8 closed session artifacts to memory/archive/2026-04/

[full message above]"
```

**Note on `memory/archive/2026-04/README.md`**: this commit does NOT update the
README to list the 8 newly archived files. The README is a hand-curated index;
amending it on every archive commit creates merge friction. Let the README be
sparse and let `ls memory/archive/2026-04/` be the discovery mechanism. Future
cleanup passes can re-curate the README at the end of the pass.

**Re-verification grep (Gate F)**:

```bash
# Expect: 0 results for each in non-archive locations (excluding cleanup/, archive/)
Grep pattern="etb-trigger-fix-plan" --glob="!memory/archive/**" --glob="!memory/cleanup/**"
Grep pattern="forage-dsl-plan"      --glob="!memory/archive/**" --glob="!memory/cleanup/**"
Grep pattern="w3-low-s[1-6]-review" --glob="!memory/archive/**" --glob="!memory/cleanup/**"

# Files at expected new locations
ls memory/archive/2026-04/etb-trigger-fix-plan.md  # exists
ls memory/archive/2026-04/forage-dsl-plan.md       # exists
ls memory/archive/2026-04/w3-low-s*.md             # 6 files
```

---

### C6 [deferred until PB-N closes] — engine_explanation/architecture disambiguation + scutemob-architecture-review decision

**Commit message** (placeholder — actual content depends on the read-and-decide step):

```
chore: disambiguate engine_explanation vs architecture (audience headers + cross-refs)

Per Gate A oversight ruling on the original audit's "duplicate docs" finding:
read both files in detail at execution time and decide whether they serve
distinct audiences. Default action: add audience-disambiguating headers and
reciprocal cross-references; do NOT delete or merge.

Also handles docs/scutemob-architecture-review.md per the same ruling — read
the content, default to T2 archive to docs/archive/2026-04/ as a dated
snapshot if the content does not serve a current audience.

Refs: memory/cleanup/cleanup-plan-2026-04-12.md items #24, #25, #26
```

**Files touched**:

- `docs/engine_explanation.md` — add header (~3 lines), add cross-ref (~1 line)
- `docs/mtg-engine-architecture.md` — add reciprocal cross-ref (~1 line)
- `docs/scutemob-architecture-review.md` — `git mv` to `docs/archive/2026-04/scutemob-architecture-review.md` (default action), OR add disambiguating header (alternate action)

**Note**: this commit's diff cannot be precomputed because it depends on a
read-and-decide step at execution time. The runner agent (or the cleanup
session) reads both files in detail, makes the audience call, and produces the
diff at execution time. The dry-run plan documents the *intent* and the
*default action*, not the literal diff.

**Bash commands** (intent only):

```bash
git status                                                # working tree clean
# Read tool: docs/engine_explanation.md (full)
# Read tool: docs/mtg-engine-architecture.md (full)
# Read tool: docs/scutemob-architecture-review.md (full)
# Decide audience split
# Edit tool: docs/engine_explanation.md (add header + cross-ref)
# Edit tool: docs/mtg-engine-architecture.md (add cross-ref)
# Either: git mv docs/scutemob-architecture-review.md docs/archive/2026-04/...
# Or:     Edit tool: docs/scutemob-architecture-review.md (add header)
git status
git add docs/engine_explanation.md docs/mtg-engine-architecture.md \
        docs/scutemob-architecture-review.md docs/archive/2026-04/  # whichever applies
git commit -m "..."
```

**Stop-and-flag triggers** for this commit:
- If reading reveals the two files are in fact substantially duplicate (>70% overlap), halt and escalate to oversight rather than picking a merge strategy unilaterally.
- If `scutemob-architecture-review.md` turns out to be a draft of `engine_explanation.md` (origin order matters), default action changes to `git rm` after explicit oversight approval — but per the §5 rule, T3 deletes need oversight per item.

---

## 3. Re-verification protocol

**Before every safe-during-PB-N commit fires**, the cleanup agent runs:

```bash
# 1. Re-check git status — has the worker added new modified files?
git status

# 2. Re-check workstream-state W6 row — is W6 still ACTIVE on PB-N?
Grep pattern="W6:" path="memory/workstream-state.md" output_mode="content"

# 3. Re-check primitive-wip phase — is the worker still mid-pipeline?
Read memory/primitive-wip.md

# 4. Re-run the Gate A targeted grep for this commit's files
#    (e.g., before C1, grep .claude/ for any references to files C1 touches)
Grep pattern="<file-being-edited>" path=".claude/" path="docs/" path="memory/"
```

If any of (1)–(4) reveal a state change since the dry-run plan was approved,
the cleanup agent **halts** and escalates to oversight rather than proceeding.

**Before every deferred commit fires**, in addition to the above:

```bash
# 5. Verify PB-N has closed — look for the close commit signature
git log --oneline -20 | grep -iE "PB-N (close|complete)|W6.*PB-N.*close"

# 6. Re-check that no other workstream went ACTIVE while PB-N was closing
Grep pattern="ACTIVE" path="memory/workstream-state.md"
```

If PB-N is still in flight, the deferred commit waits.

**After every commit lands**, the cleanup agent runs:

```bash
# 7. Verify linear history (no merges, no rebases)
git log --oneline -5

# 8. Re-run the Gate F grep verification for this commit's specific concerns
#    (the per-commit "Re-verification grep" sections above)
```

If (7) or (8) reveal a problem, the cleanup agent **halts** before the next
commit and reports to oversight.

---

## 4. Retention policy doc draft (`docs/cleanup-retention-policy.md`)

Full content. Lands in C3.

```markdown
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
```

---

## 5. `/cleanup` skill draft (`.claude/skills/cleanup/SKILL.md`)

Full content. Lands in C3.

```markdown
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
```

---

## 6. Post-state grep verification (Gate F summary)

After **all 6 commits land** (assuming oversight approves everything including
the AWAITING APPROVAL items), the following greps should return the indicated
results across the full repo:

| Pattern | Path | Expected | Why |
|---------|------|---------:|-----|
| `cards/definitions\.rs` | repo-wide | 0 | Stale path eliminated by C1 |
| `memory/m8-session-plan` | repo-wide (excluding `memory/cleanup/`) | 0 | Literal example removed by C1 |
| `memory/feedback_feature_branch_workflow` | non-archive (excluding `memory/cleanup/`) | 0 | Archived in C2 |
| `memory/feedback_worktree_parallel` | non-archive (excluding `memory/cleanup/`) | 0 | Archived in C2 |
| `cleanup-retention-policy` | `CLAUDE.md` | 1 | Added in C4 |
| `cleanup-retention-policy` | `docs/` | 1 (the file itself) | Created in C3 |
| `/cleanup` (skill mention) | `CLAUDE.md` | 1 | Added in C4 |
| `codebase_analysis` | `CLAUDE.md` | 0 | Removed in C4 (CONDITIONAL on approval) |
| `memory/MEMORY\.md` (literal) | `CLAUDE.md` | 0 | Reworded in C4 |
| `etb-trigger-fix-plan` | non-archive (excluding `memory/cleanup/`) | 0 | Archived in C5 |
| `forage-dsl-plan` | non-archive (excluding `memory/cleanup/`) | 0 | Archived in C5 |
| `w3-low-s[1-6]-review` | non-archive (excluding `memory/cleanup/`) | 0 | Archived in C5 |
| `engine_explanation\.md` (cross-ref) | `docs/mtg-engine-architecture.md` | ≥1 | Added in C6 |
| `mtg-engine-architecture\.md` (cross-ref) | `docs/engine_explanation.md` | ≥1 | Added in C6 |
| `scutemob-architecture-review` | repo-wide | depends on C6 decision | Default: file moves to `docs/archive/2026-04/` |

Files that **do NOT** appear in this verification (because they were deferred
out of the pass):

- `memory/benchmark-results.md` — referenced from
  `engine-core-complete-checkpoint.md`; deferred
- `memory/engine-core-complete-checkpoint.md` — referenced from
  `docs/mtg-engine-milestone-reviews.md`; deferred
- `memory/w3-layer-audit.md` — referenced from
  `memory/primitives/pb-review-S.md` (protected corpus, untouchable);
  **permanently deferred from this pass** until pb-review-S is itself
  archived (which requires retiring the primitive corpus rule, a separate
  decision)

---

## 7. Worker-disruption budget

| Bucket | Budget | Plan (post-override) |
|--------|-------:|-------:|
| Safe during PB-N | ≤2 commits | 0 (override defers C1+C2) |
| Deferred until PB-N closes | ≤4 commits | 6 (C1, C2, C3, C4, C5, C6) — exceeds soft target but stays under hard limit |
| **Total** | **≤6 commits** | **6** ✓ AT HARD LIMIT |

**Override note**: oversight overrode the original 2+4 split to 0+6 because
W6 phase-shifted to implement during plan drafting. The total commit count
is unchanged; only the timing changes. The deferred bucket exceeds its soft
target of 4 but the hard limit of 6 total commits is preserved.

**Files touched per commit**:

| Commit | Tag (post-override) | Files | Touches CLAUDE.md? | Touches memory/? | Worker collision risk |
|--------|---------------------|------:|:------------------:|:----------------:|:---------------------:|
| C1 | deferred (override) | 5 | no | no | none — fires post-PB-N |
| C2 | deferred (override) | 3 (2 mvs + 1 new) | no | yes (2 mvs into archive/, 1 new in archive/) | none — fires post-PB-N |
| C3 | deferred | 3 (2 new + 1 edit) | no | yes (1 edit in archive/) | none — fires post-PB-N |
| C4 | deferred | 1 (CLAUDE.md) | yes | no | low (post-PB-N) — must still rebase against worker's PB-N close commit on CLAUDE.md if any conflict |
| C5 | deferred | 8 (mvs) | no | yes (8 mvs into archive/) | none |
| C6 | deferred | 3-4 (depends on decision) | no | no | low |

All commits land strictly after PB-N closes. C4 retains the only non-trivial
rebase consideration (the worker's PB-N close commit will touch CLAUDE.md
"Current State" / "Active Plan" / "Last Updated" lines). The cleanup agent
rebases C4 against the worker's close commit before applying.

---

## 8. Items requiring explicit oversight approval (AWAITING APPROVAL queue)

**Status**: both items resolved at plan-approval time on 2026-04-12. Recorded
here for the durable audit trail. Both resolutions land with their original
commits (C4 and C6) when execution resumes post-PB-N close.

### A1 — Remove `Codebase Analysis` row from CLAUDE.md Primary Documents table (commit C4, item #12)

**RESOLUTION (2026-04-12)**: **APPROVED**. Oversight rationale:
> "It points at a gitignored file that fresh clones lack. The row is
> structurally broken and serves no discoverability function. The 'keep as
> marker' alternative is sentimental, not load-bearing. Remove."

The C4 commit will remove the row as drafted. No conditional fallback needed.

**Verified context**: `codebase_analysis_220260228.md` is matched by
`.gitignore` line 40 (`codebase_analysis_*.md`). It is a local-only file,
not version-controlled. CLAUDE.md line 67 has a Primary Documents row pointing
at it. Fresh clones of the repo lack the file entirely, which means the table
row points at a file that does not exist on most checkouts.

**Proposed action**: remove the row. The local file (where it exists) is
untouched — that is the user's personal file.

**Alternative**: keep the row (oversight may want the table to acknowledge
the snapshot's existence even if the file is local-only).

**Why explicit approval**: the original audit flagged this for archival, but
the verification revealed there is nothing to archive — the file is gitignored.
The decision is "remove the dangling table row" or "keep it as a marker." Per
counter-proposal §5, explicit approval required.

**Default if oversight does not respond**: keep the row (the more
conservative choice).

### A2 — `docs/scutemob-architecture-review.md` archive vs disambiguating header (commit C6, item #26)

**RESOLUTION (2026-04-12)**: **DEFAULT APPROVED** (T2 archive). Oversight
rationale:
> "Cleanup agent reads at execution time and applies the default unless the
> read reveals a distinct audience worth preserving in place."

The C6 commit will:
1. Read the file in detail at execution time
2. Apply the default (T2 archive to `docs/archive/2026-04/scutemob-architecture-review.md`)
3. Override the default only if the read reveals a distinct audience
4. If overridden, add a disambiguating header in place instead and record
   the deviation in the commit message

---

## 9. Items demoted or removed from the original audit

The original audit's punch list flagged several items that verification
showed should be demoted or removed entirely. Recording these so the
decisions are durable and the next cleanup pass does not relitigate them.

| Original audit item | Original tier | New tier | Reason |
|---------------------|--------------|----------|--------|
| `zz_scratch.md` | T3 delete | **REMOVED FROM PLAN** | Matched by `.gitignore` line 38. Local-only personal scratchpad. The cleanup pass has no business touching it. The user owns it. |
| `codebase_analysis_220260228.md` archive | T2 archive | **EDIT-IN-PLACE on the CLAUDE.md row only** | Matched by `.gitignore` line 40. There is no file to archive — the file is local-only. The cleanup only removes the dangling CLAUDE.md table row (and only with explicit approval). |
| `feedback_feature_branch_workflow.md` | T3 delete (audit) | **T2 archive** | Content has design rationale ("user explicitly requested for PB-20"). Default tier for ambiguous content is T2 per retention policy §6. |
| `feedback_worktree_parallel.md` | T3 delete (audit) | **T2 archive** | Same reason. Content captures a real workflow preference learned from incident. |
| `memory/ability-wip.md` | T3 delete (audit) | **NOT TOUCHED** | The `/implement-ability` skill is not retired (Gate A oversight Q1). The wip file is the skill's entry-point sentinel. Same protection as `memory/primitive-wip.md` (which the audit did not flag). |
| `memory/abilities/` (329 files, 5.1 MB) | T2 bulk archive | **NOT TOUCHED** | Used as research corpus by `ability-impl-planner` and `ability-impl-runner`. Permanently untouchable per retention policy §3. |
| `memory/primitives/pb-review-*.md` (58 files) | T2 bulk archive | **NOT TOUCHED** | Used as research corpus by `primitive-impl-planner` and `primitive-impl-reviewer`. Permanently untouchable per retention policy §3. |
| `memory/card-authoring/review-*.md` (115 files) | T2 bulk archive | **NOT TOUCHED** | Globbed by `card-fix-applicator`. Permanently untouchable per retention policy §3. |
| `memory/benchmark-results.md` | T2 archive | **DEFERRED** | Referenced from `engine-core-complete-checkpoint.md`. Defer until that file is itself dealt with. |
| `memory/engine-core-complete-checkpoint.md` | T2 archive | **DEFERRED** | Referenced from `docs/mtg-engine-milestone-reviews.md:1349` (active doc with high write frequency). Defer to a future pass that batches milestone-reviews edits. |
| `memory/w3-layer-audit.md` | T2 archive | **PERMANENTLY DEFERRED THIS PASS** | Referenced from `memory/primitives/pb-review-S.md:185` (protected corpus). Cannot be archived without modifying pb-review-S, which the cleanup may not touch. |
| `docs/engine_explanation.md` ↔ `docs/mtg-engine-architecture.md` merge | T3 delete one | **T1 EDIT-IN-PLACE only** | Per Gate A oversight Q1: "duplicates" probably serve different audiences. Add cross-references and audience-disambiguating headers; do not merge or delete. |
| `docs/project-status.md` reconcile with CLAUDE.md | T1 EDIT-IN-PLACE | **REMOVED FROM PLAN** | Touches CLAUDE.md, which is in a protected section during PB-N. The B.6 list explicitly includes "docs/project-status.md workstream + test-count rows" as untouchable for PB-N duration. Defer to a post-PB-N cleanup pass that has explicit scope for this reconciliation. |
| `docs/mtg-engine-simulator.md` (W2 stalled doc) | T2 archive (audit) | **REMOVED FROM PLAN** | Gate A captured this as a candidate but did not verify W2's status as "permanently retired." W2 is "available, stalled" not "retired." Do not archive. |
| `docs/mtg-engine-tui-plan.md` (W2 stalled doc) | T2 archive (audit) | **REMOVED FROM PLAN** | Same reason. |
| `docs/mtg-engine-interaction-gaps.md` | T3 delete (audit) | **REMOVED FROM PLAN** | Audit said "superseded by corner-case-audit" but the verification step did not happen. Defer to a future cleanup pass that includes the read-and-compare step. |

---

## 10. Open questions captured for the next cleanup pass

These are not blocking the 2026-04-12 pass; they are captured here so the
next cleanup pass starts from a known position.

1. **`docs/project-status.md` reconciliation with CLAUDE.md "Current State"**:
   the original audit flagged this. Deferred this pass because the file is
   in the W6 protected set. Schedule for the first post-PB-N cleanup pass
   with explicit scope.
2. **`memory/engine-core-complete-checkpoint.md` and
   `memory/benchmark-results.md` archive**: deferred this pass because the
   first is referenced from `docs/mtg-engine-milestone-reviews.md`. The
   right approach is a single commit that updates the milestone-reviews
   inventory entry to point at the archive path and moves both files in
   the same commit. Do this when the next milestone-reviewer agent runs.
3. **`memory/w3-layer-audit.md` archive**: permanently deferred from this
   pass. Will be archivable only after `memory/primitives/pb-review-S.md` is
   itself archived, which requires retiring the primitive corpus rule (a
   separate, larger decision).
4. **`docs/mtg-engine-simulator.md` and `docs/mtg-engine-tui-plan.md` (W2
   stalled docs)**: status of W2 needs explicit oversight call before these
   can be touched. Either W2 is alive (these stay) or W2 is retired (T2
   archive). Surface to user at next milestone boundary.
5. **`docs/mtg-engine-interaction-gaps.md`**: needs a read-and-compare against
   `docs/mtg-engine-corner-case-audit.md` before any action. Schedule for
   next pass.
6. **Auto-memory MEMORY.md oversize** (33.1 KB > 24.4 KB limit): out of
   scope for project cleanup. Surface to the user as a separate session,
   not as part of any cleanup pass.
7. **First `/cleanup` invocation after this pass**: schedule for end-of-M10
   (the next milestone after the W6 PB queue closes). The first invocation
   tests whether the protocol embedded in the skill matches the cadence and
   whether 6 commits is the right budget.

---

## End of dry-run plan

This plan is the contract. Oversight reviews. If approved, the cleanup
agent executes commits in order (C1 → C2 during PB-N; C3 → C4 → C5 → C6
after PB-N close), with re-verification before each.

If oversight rejects any item, the item is dropped from its commit and the
rest of the commit lands. If oversight rejects an entire commit, that commit
and all its dependents are dropped from the pass.

If oversight wants the plan resequenced or rebudgeted, the cleanup agent
produces a new dry-run plan and re-submits.

No commits land until oversight signs off on this document.
