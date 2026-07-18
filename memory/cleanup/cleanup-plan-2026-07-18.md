# Cleanup Dry-Run Plan — 2026-07-18 (DOC-5 / scutemob-121, DOC-8 / scutemob-124 folded)

Trigger: EF-queue milestone boundary (closed 2026-07-18); zero-worktree gap after
scutemob-116 collect. Gate A: `runtime-reference-map-2026-07-18.md` (incl. Gate B
section — CLEAR). Input audit: `memory/doc-audit-2026-07-18.md` F5 + addendum.
Oversight: coordinator, executing inline under user authorization (hold-lift directive
2026-07-18); halts observed as self-review checkpoints, stop-and-flag triggers escalate
to user.

## Deviations from the input audit (approved at plan time)

1. **Glob widened, not files renamed**: §3 protection becomes `*review*.md` (was
   `review-*.md`). `card-fix-applicator`'s own glob (`review-*.md`, agent line 57) is
   untouched — protection is a superset of what the agent reads. Consequence: the 9
   `f4-*review*` files the audit counted as archivable are now **protected — kept**,
   consistent with the DOC-8 ruling that review corpus stays. Yield drops ~26 → 15 files.
2. **Removed from candidates**: `wave-progress.md` (hard-ref'd by author-wave +
   audit-cards skills), `dsl-gap-audit-v2.md` (policy-named individually-managed ref),
   `dsl-gap-audit-2026-05-16.md` (companion of the ACTIVE campaign plan).
3. **Gate A hazard fix folded in as T1**: `/implement-primitive` SKILL.md points at
   `memory/primitives/primitive-wip.md` — a stale duplicate frozen at PB-AC9 — while the
   live WIP is `memory/primitive-wip.md`. Repoint the skill; header-annotate the stale
   duplicate (one line; §3 forbids moving it, not annotating a hazard).

## Action table

| # | Path | Action | Tier | Commit |
|---|------|--------|------|--------|
| 1 | memory/cleanup/runtime-reference-map-2026-07-18.md | add (Gate A+B artifact) | — | C1 |
| 2 | memory/cleanup/cleanup-plan-2026-07-18.md | add (this file) | — | C1 |
| 3 | memory/project-audit-2026-04-12.md | git mv → memory/archive/2026-07/ | T2 | C2 |
| 4 | memory/migration-to-skylarch.md | git mv → archive (self-scheduled) | T2 | C2 |
| 5 | memory/low-sweep-plan.md | git mv → archive + repoint workstream-state:15 | T2+T1 | C2 |
| 6 | memory/w3-layer-audit.md | git mv → archive (MEMORY.md:71 repoint → DOC-3/scutemob-119; pb-review-S.md:185 dangles per T2) | T2 | C2 |
| 7 | memory/engine-core-complete-checkpoint.md | git mv → archive (milestone-reviews:1349 is historical inventory — dangles per T2) | T2 | C2 |
| 8 | memory/benchmark-results.md | git mv → archive (paired with #7 per Gate A) | T2 | C2 |
| 9 | memory/archive/2026-07/README.md | add (batch listing, both batches) | — | C2 (appended C3) |
| 10 | card-authoring/a42-retriage-2026-04-10.md | git mv → archive (pb-plan-X refs historical, dangle per T2) | T2 | C3 |
| 11 | card-authoring/a42-tier4-diagnosis-2026-04-10.md | git mv → archive (zero refs) | T2 | C3 |
| 12 | card-authoring/bf1-retriage-report.md | git mv → archive | T2 | C3 |
| 13 | card-authoring/todo-classification-2026-04-12.md | git mv → archive (primitive-card-plan:735 gets DOC-2 banner) | T2 | C3 |
| 14 | card-authoring/wave-001-land-etb-tapped.md | git mv → archive | T2 | C3 |
| 15 | card-authoring/wave-002-combat-keyword.md | git mv → archive | T2 | C3 |
| 16 | card-authoring/wave-003-mana-land.md | git mv → archive | T2 | C3 |
| 17 | card-authoring/f5-verification.md | git mv → archive | T2 | C3 |
| 18 | card-authoring/dsl-gap-audit.md (v1, 2 gens stale) | git mv → archive (MEMORY.md:69 repoint → DOC-3) | T2 | C3 |
| 19 | docs/cleanup-retention-policy.md §3 | edit: glob `review-*.md` → `*review*.md`; abilities/ → distillation-approved wording (DOC-8); last_updated bump | T1 | C4 |
| 20 | memory/decisions.md | append DOC-8 decision entry | T1 | C4 |
| 21 | .claude/skills/implement-primitive/SKILL.md | repoint 3 refs → memory/primitive-wip.md | T1 | C4 |
| 22 | memory/primitives/primitive-wip.md | one-line STALE-DUPLICATE header note | T1 | C4 |

No T3 (delete) items. Nothing in this pass touches: `memory/abilities/`,
`memory/primitives/` pb-plan/pb-review files, any `*review*.md`, defs, test-data,
auto-memory MEMORY.md (out of scope §9 — repoints delegated to scutemob-119/DOC-3),
CLAUDE.md (owned by DOC-1v2/scutemob-125).

## Per-commit verification protocol

Before each commit: `git status --short` (only planned files); after: `git log --oneline -3`
linear + post-grep:
- C2/C3 post-grep: `ls memory/archive/2026-07/` contains the moved names; originals gone;
  `grep -rn "low-sweep-plan" memory/workstream-state.md` shows archive path at :15.
- C4 post-grep: `grep -n "\*review\*" docs/cleanup-retention-policy.md` hits §3;
  `grep -n "memory/primitive-wip" .claude/skills/implement-primitive/SKILL.md` ≥3 hits,
  zero remaining `memory/primitives/primitive-wip` refs in that file.

## Budget

4 commits total (target ≤4 deferred-bucket, hard limit 6). No active workstream — the
"during workstream" ≤2 sub-budget does not apply.

## Follow-ups filed by this pass

- ESM task (new): abilities-corpus distillation pass per DOC-8 ruling (b) — distill
  memory/abilities/ (329 files, 5.1MB, W1 closed 2026-03) into gotchas/conventions,
  then archive; separate project per policy §3, NOT a cleanup.
- Comment on scutemob-119 (DOC-3): repoint auto-memory MEMORY.md:71 (w3-layer-audit) and
  :69 (dsl-gap-audit v1 → dsl-gap-audit-2026-05-16) to post-archive reality.
- Gate A missing-target residue (audit-report/certification write-targets, eot:103 path)
  documented in the reference map §4; no action this pass.
