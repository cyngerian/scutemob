# Dormant skills (moved aside 2026-09-05, CC-14 / `scutemob-251`)

Moved here, not deleted, because their track is closed rather than wrong
(`docs/course-correction-2026-09.md` §6.1): `start-milestone` (no milestone running; the
roadmap's M10–M15 are HISTORICAL under the pod-first plan), `next-ability` / `ability-status` /
`audit-abilities` (all keyword abilities shipped), `remedy` (SR remediation track closed
2026-07-16). Their agents, where they have them, are in `.claude/agents-dormant/`.

Deleted outright in the same task, not moved: `start-work` (self-declared RETIRED),
`end` (replaced by `/eot`), `spawn` (superseded by `/dispatch`, whose steps 1–7 now carry the
recipe). **`end` and `spawn` are ESM-provisioned skills, so `esm doctor` will list them as
missing from now on — that is expected. Do NOT run `esm update` to "fix" it**: it would
re-add them (and `esm update --force` would clobber the customized `/collect` and `/dispatch`).

**To restore a skill**: `git mv .claude/skills-dormant/<name> .claude/skills/<name>`; the skill
is picked up at the next session start. Re-read it first — it was written against a queue and
document set that no longer exists (`oos-retriage-plan-2026-07-18.md`, `_authoring_plan.json`,
`docs/primitive-card-plan.md`), so expect to repoint its inputs.
