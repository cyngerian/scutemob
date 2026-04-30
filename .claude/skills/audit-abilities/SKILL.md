---
name: audit-abilities
description: Refresh the ability coverage doc by scanning the engine, card defs, and scripts
---

# Audit Abilities

Refresh `docs/mtg-engine-ability-coverage.md` by scanning the engine source, card definitions,
and game scripts for current implementation status.

## Procedure

Use the Task tool to invoke the `ability-coverage-auditor` agent with subagent_type
`ability-coverage-auditor`.

If `$ARGUMENTS` is provided (e.g., "evergreen", "equipment", "flashback"), pass it to the
agent to scope the audit to that section or ability only:

> "Audit ability coverage for: $ARGUMENTS. Read docs/mtg-engine-ability-coverage.md, scan
> the engine source and scripts for the relevant rows, update statuses, and recompute the
> summary table."

If no arguments, run a full audit:

> "Run a full ability coverage audit. Read docs/mtg-engine-ability-coverage.md, scan the
> engine source, card definitions, and scripts for every row in every section. Update all
> statuses, recompute the summary table, and refresh the Priority Gaps section."

After the agent finishes, report the summary to the user.
