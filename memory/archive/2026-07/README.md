# Archive batch 2026-07 — cleanup pass 2026-07-18 (scutemob-121 / DOC-5)

Authorized by `memory/cleanup/cleanup-plan-2026-07-18.md`. All T2 (recoverable via
`git mv` back). EF-queue milestone boundary; zero active worktrees.

## Batch 1 — memory/ top-level one-shots
- project-audit-2026-04-12.md — one-shot audit, superseded by doc-audit-2026-07-18
- migration-to-skylarch.md — hardware migration record, self-scheduled for archive
- low-sweep-plan.md — LOW Sweep campaign plan, campaign COMPLETE 2026-05-16 (workstream-state W3 row repointed)
- w3-layer-audit.md — W3-LC audit, complete; findings absorbed into gotchas
- engine-core-complete-checkpoint.md — dated checkpoint snapshot
- benchmark-results.md — dated benchmark snapshot (paired with checkpoint)

## Batch 2 — memory/card-authoring/ closed non-review artifacts
- a42-retriage-2026-04-10.md, a42-tier4-diagnosis-2026-04-10.md — A-42 retriage, closed
- bf1-retriage-report.md — BF1 retriage, closed
- todo-classification-2026-04-12.md — April TODO taxonomy, superseded by marker sweep 2026-07-16
- wave-001-land-etb-tapped.md, wave-002-combat-keyword.md, wave-003-mana-land.md — completed wave plans
- f5-verification.md — one-shot verification artifact
- dsl-gap-audit.md — v1, two generations behind dsl-gap-audit-2026-05-16.md (active) / v2 (skill-referenced)

NOT archived despite audit listing: wave-progress.md (skill hard-refs), dsl-gap-audit-v2.md
(policy-named), dsl-gap-audit-2026-05-16.md (active campaign companion), all f4-*review*
files (protected by §3 glob widened to *review*.md this pass).
