---
name: next-ability
description: Show the highest-priority unimplemented ability and what's needed
---

# Next Ability

Find the highest-priority unimplemented or partially implemented ability and show what's
needed to complete it, with a checklist from the Ability Addition Workflow.

## Procedure

0. **Check for work in progress**: Read `memory/ability-wip.md` if it exists.
   - If the file exists and `phase:` is NOT `closed`, display the WIP status:
     ```
     ## Ability In Progress: <Name>
     **Phase**: <current phase>
     **CR**: <number>
     **Started**: <date>

     Run `/implement-ability` to continue from the current phase.
     ```
     Then continue below to also show the next gap (in case the user wants to switch).
   - If the file doesn't exist or `phase: closed`, proceed normally.

1. **Read `docs/mtg-engine-ability-coverage.md`** — both the section tables and the
   "Ability Addition Workflow" section.

2. **Find the top gap**: Scan all sections for rows with status `none` or `partial`,
   prioritizing P1 > P2 > P3 > P4. Within the same priority tier, prefer `partial` over
   `none` (partial abilities are closer to done). Within the same status, prefer rows that
   appear earlier in the document.

3. **Look up the CR rule** using `mcp__mtg-rules__get_rule` with `include_children: true`
   for the ability's CR number.

4. **Check dependencies**: If the "Depends On" column lists another ability, check that
   ability's status too. If the dependency is also `none`, suggest implementing it first.

5. **Determine which workflow steps apply**: Read the "Ability Addition Workflow" section
   and check which steps are already done for this ability:

   - **Step 1 (enum variant)**: Grep `crates/engine/src/state/types.rs` for the ability name.
     If found, step is done.
   - **Step 2 (rule enforcement)**: Grep `crates/engine/src/rules/` for the ability name or
     CR number. If enforcement logic exists, step is done.
   - **Step 3 (trigger wiring)**: Only applies to trigger-based abilities. Grep
     `crates/engine/src/` for a matching `TriggerCondition` variant and check if it's wired
     to runtime dispatch.
   - **Step 4 (unit tests)**: Grep `crates/engine/tests/` for the ability name or CR number.
   - **Step 5 (card definition)**: Grep `crates/engine/src/cards/defs/` for the
     ability name.
   - **Step 6 (game script)**: Grep `test-data/generated-scripts/` for the ability name.
   - **Step 7 (coverage doc update)**: This happens after the other steps.

6. **Report**:

```
## Next Ability: <name>

**CR**: <number> — <rule text summary>
**Current Status**: <partial/none>
**Priority**: P<N>
**What exists**: <what's already implemented, if partial>
**What's missing**: <specific gaps to fill>
**Dependencies**: <other abilities needed first, or "none">

### Workflow Checklist

- [x] Step 1: Enum variant — exists in `types.rs:L<N>` (or "[ ] not yet added")
- [ ] Step 2: Rule enforcement — add to `rules/<file>.rs` (see workflow table for which file)
- [ ] Step 3: Trigger wiring — add `TriggerCondition::<Variant>` dispatch (or "n/a")
- [ ] Step 4: Unit tests — add to `tests/<file>.rs`
- [ ] Step 5: Card definition — use `card-definition-author` agent for <suggested card>
- [ ] Step 6: Game script — use `game-script-generator` agent
- [ ] Step 7: Update coverage doc — run `/audit-abilities <name>`

### Suggested Approach

<Brief implementation strategy — which file to start in, what similar abilities to
reference as patterns, any gotchas from memory/gotchas-rules.md or memory/gotchas-infra.md>
```

If `$ARGUMENTS` is provided (e.g., "P2", "combat"), filter to that priority tier or section
before selecting the top gap.
