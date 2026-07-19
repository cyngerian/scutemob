# Doc Remediation — Execution Report for the Auditor — 2026-07-18

Closes the loop on `memory/doc-audit-2026-07-18.md` (F1–F8 + addendum). All six live
tasks executed and merged in one sitting (2026-07-18, between PB-OS1 and PB-OS2 on the
PB-OS queue). Written by the mainline coordinator; every acceptance criterion below was
**spot-checked on disk at collect** (byte counts, greps, link resolution), not taken from
worker attestations.

## Execution timeline

| # | Step | Who | Merge/commit | Outcome |
|---|------|-----|--------------|---------|
| 0 | Audit file committed (was untracked) | coordinator | `b1735e8a` | Available to branching workers, as you flagged |
| 1 | Collect scutemob-116 (PB-OS1) | coordinator | `db49a0b2` | Opened the zero-worktree gap |
| 2 | DOC-5 (121) + DOC-8 (124) | coordinator-inline, gated protocol | 4 commits `fc4c12a1`/`c46dc738`/`5ec8f607`/`6e47623a`, merge `e22a836f` | See §DOC-5/8 |
| 3 | DOC-1v2 (125) ∥ DOC-3 (119) | dispatched workers (parallel) | `4c7995b0` / empty branch | See §DOC-1v2, §DOC-3 |
| 4 | DOC-2 (118) | dispatched worker | `c0de9550` | See §DOC-2 |
| 5 | DOC-6v2 (126) | dispatched worker | `78a10cc0` | See §DOC-6v2 |
| 6 | Bookkeeping under the new recurrence rule (first use) | coordinator | `a0cf1eb8` | Detail → archive changelog; one-paragraph delta in CLAUDE.md |

Cancelled-superseded as reconciled: `scutemob-117`/`120`/`122`/`123`.
One ~10s ESM outage (connection refused) between worker completion and collection; recovered on retry, nothing lost.

## Per-task results

### DOC-5 (scutemob-121) + DOC-8 (scutemob-124) — gated cleanup, coordinator-inline
- **Gate A** (`memory/cleanup/runtime-reference-map-2026-07-18.md`): ~88 hard refs, 31
  glob/parent refs, 7 missing targets. **New finding beyond the audit**: `/implement-primitive`
  SKILL.md pointed at `memory/primitives/primitive-wip.md` — a **stale duplicate frozen at
  PB-AC9** — while the live WIP is `memory/primitive-wip.md`. A live-pipeline hazard, not
  hygiene: 12 references repointed, stale copy header-annotated (not moved — §3).
- **Gate B**: CLEAR (zero worktrees, zero ACTIVE workstreams). Report appended to the map.
- **Plan** (`memory/cleanup/cleanup-plan-2026-07-18.md`) with documented **deviations from
  the audit's ~26-file estimate**:
  1. Protection glob **widened to `*review*.md`** instead of renaming files —
     `card-fix-applicator`'s own `review-*.md` read-glob untouched (protection is a superset).
     Consequence: the 9 `f4-*review*` files the audit counted archivable are now protected-kept,
     consistent with the DOC-8 ruling. **Yield: 15 files, not 26.**
  2. Removed from candidates on Gate A evidence: `wave-progress.md` (author-wave + audit-cards
     skill hard refs), `dsl-gap-audit-v2.md` (policy-named), `dsl-gap-audit-2026-05-16.md`
     (companion of the ACTIVE campaign plan — should never have been a candidate).
- **Executed**: 4 commits (hard limit 6). 15 files → `memory/archive/2026-07/` with README:
  batch 1 (memory/ top-level): project-audit-2026-04-12, migration-to-skylarch, low-sweep-plan
  (workstream-state:15 repointed), w3-layer-audit, engine-core-complete-checkpoint,
  benchmark-results; batch 2 (card-authoring): a42-retriage, a42-tier4-diagnosis,
  bf1-retriage-report, todo-classification-2026-04-12, wave-001/002/003, f5-verification,
  dsl-gap-audit.md (v1). No T3 deletes anywhere.
- **DOC-8**: user ruling (c)+(b)-abilities-only recorded in `memory/decisions.md`; §3
  rewritten (abilities → distillation-authorized-until-executed; primitives reconfirmed live —
  OS retriage cited pb-plan-AC7/AC8 the same week; reviews stay). Follow-up task
  **`scutemob-127`** filed (abilities-corpus distillation, 3 ACs, opportunistic).

### DOC-1v2 (scutemob-125) — CLAUDE.md restructure (absorbs DOC-4)
- **CLAUDE.md 77,820 → 33,969 bytes**; Current State **56,906 → 9,379 bytes**.
- Three-way split exactly per the addendum: changelog **verbatim** →
  `memory/archive/claude-md-changelog-2026-07.md` (35,230 B; verified by content grep);
  invariant bullets → **`docs/engine-invariants.md`** (19,568 B), registered in Primary
  Documents + When-to-Load, one-line pointers remain in Current State.
- Verified at collect: SR-35 historical measurements untouched (`1,380 of 1,748` intact ×2 in
  the invariants doc); live structural claims now **1,798**; "Fifteen" → "Seventeen"; W5
  commit-prefix row gone; type-consolidation routing row fixed; **recurrence rule** written
  (collect-time detail appends to the archive file, one-paragraph delta in CLAUDE.md, new
  dated file per month); secondary docs index + one-line mentions for crew/new-doc/
  next-ability/remedy/start-stepper (DOC-4 scope).

### DOC-3 (scutemob-119) — auto-memory fixes
- All edits outside the repo (empty branch, as predicted). Verified post-collect:
  **18/18 MEMORY.md links resolve**; the 5 broken links resolved by **unlink-with-fact-kept-
  inline** (nothing fabricated); helpers.rs prelude + Key File Locations →
  `crates/card-types/src/cards/helpers.rs` (post-SR-6); removed-skill refs cleaned (one
  deliberate rename annotation retained: "`/start`→`/eot` (was `/start-session`/`/end-session`)").
- Also applied the two DOC-5 hand-off repoints (w3-layer-audit → archive path; DSL-gap line →
  `dsl-gap-audit-2026-05-16.md`, the campaign-authoritative generation).

### DOC-2 (scutemob-118) — supersession banners
- All 7 docs open with accurate status banners naming successors:
  **`project-status.md` RETIRED OUTRIGHT** (user decision; banner forbids regeneration —
  no successor generator), `workstream-coordination.md` HISTORICAL/SUPERSEDED (frozen
  2026-03-08), `primitive-card-plan.md`, `dsl-gap-closure-plan.md`,
  `mtg-engine-low-issues-remediation.md`, `card-authoring-operations.md`,
  `ability-batch-plan.md`. CLAUDE.md routing rows repointed against the **post-DOC-1v2**
  restructured file (worker briefed not to trust the audit's line numbers).

### DOC-6v2 (scutemob-126) — docs.yaml + stamps (absorbs DOC-7)
- **`.claude/docs.yaml` created**, scoped to the living set (bannered historical docs
  excluded) — `/eot`/`/done` stale-doc checks are **armed for the first time** since the
  machinery was built. ~20 docs adopted the `<!-- last_updated -->` marker;
  `cleanup-retention-policy.md` dual-stamped.
- Absorbed DOC-7: milestone-reviews header stamp, sr-remediation-plan marker,
  corner-case-audit stamp, `layer-bypass-audit.md` disambiguating line (9 HIGH = its own
  M10-scheduled class, not live bugs), `ability-wip.md` cleared to an explicit
  **IDLE — no ability in progress** state. `ability-coverage.md` **stamped-and-deferred**
  with a dated note; `/audit-abilities` was not run (as re-scoped).

## Finding-by-finding status

| Finding | Status | Closed by |
|---------|--------|-----------|
| F1 changelog-bloat | **CLOSED** + recurrence rule (regrowth prevention) | DOC-1v2 |
| F2 stale-presented-as-current | **CLOSED** (7 docs bannered) | DOC-2 |
| F3 cross-reference drift | **CLOSED** (repo + auto-memory sides) | DOC-1v2 + DOC-3 |
| F4 registration gaps | **CLOSED** (index + 5 skills) | DOC-1v2 (abs. DOC-4) |
| F5 cleanup overdue + glob gap | **CLOSED** (15 archived; glob widened) | DOC-5 |
| F6 no docs.yaml | **CLOSED** (structural fix armed) | DOC-6v2 |
| F7 ledger stamp lag | **CLOSED**, one deferral (below) | DOC-6v2 (abs. DOC-7) |
| F8/DOC-8 corpus | **DECIDED** (c)+(b)-abilities-only; execution deferred | scutemob-124 → 127 |

## Corrections & additions the execution fed back

Already in your addendum: quarantine ≈10.3MB/86% (not 11.5MB/95%); glob gap = 9 files (not 3);
historical-numbers caveat. **Newly added by execution:**
1. The stale `primitive-wip.md` duplicate + 12 skill references (Gate A) — the sharpest find
   of the pass and absent from the original audit.
2. Gate A's 7 missing-target references: 2 are `audit-cards` *write* targets (not bugs);
   `eot` SKILL.md:103 cites a repo-relative `memory/MEMORY.md` that is actually auto-memory;
   a stale `test_live.sh` permission entry; two auto-memory-relative paths written
   repo-relative. Documented in the reference map §4, **not fixed** (out of DOC scope).
3. dsl-gap-audit generation split resolved: v1 archived, `-2026-05-16` canonical
   (campaign-referenced), `v2` retained (triage-cards skill + policy-named).

## Residuals / open items

- **`scutemob-127`** (abilities-corpus distillation) — backlog, opportunistic, own project.
- `/audit-abilities` refresh — deferred with dated note in `ability-coverage.md`.
- Gate A missing-target residue (item 2 above) — documented, unassigned.
- Intentional T2 danglers (grep-discoverable, per policy): `pb-review-S.md:185` (protected
  corpus), `milestone-reviews.md:1349` (historical inventory row), `workstream-state.md:136`
  (rotated handoff narrative) still cite pre-archive paths.

## Headline metrics

- CLAUDE.md: **77,820 → 33,969 bytes (−56%)**; Current State 56,906 → 9,379 bytes.
  Per-session context cost of CLAUDE.md drops roughly by half for every future coordinator
  and worker session.
- 15 files (~0.3MB) archived with README + full `git mv` reversibility; 0 deletions.
- Commit spend: 1 (audit) + 4 (cleanup, ≤6 budget) + 4 merges + 2 bookkeeping.
- All 20 acceptance criteria across the six tasks satisfied and disk-verified.
