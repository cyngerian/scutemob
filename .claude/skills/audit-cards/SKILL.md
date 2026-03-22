---
name: audit-cards
description: Execute Phase 3 audit — scan all card defs for TODOs, empty abilities, known-issue patterns, and certify completion
---

# Audit Cards

Execute the Phase 3 audit and certification steps from `docs/card-authoring-operations.md`.
Scans every card def file for correctness issues and produces a certification report.

## Arguments

- No args: run full audit (X-1 through X-6)
- `X-<N>`: run a specific audit step only
- `--status`: show audit progress and exit
- `--quick`: scan for TODOs and empty abilities only (skip MCP oracle verification)

## Procedure

### Step 0: Check Prerequisites

Verify that Phase 2 authoring is substantially complete:
- Read `memory/card-authoring/wave-progress.md` — most groups should be `complete`
- Read `docs/card-authoring-operations.md` — most A-* items should be checked

If fewer than 80% of A-* items are checked, warn:
"Phase 2 is incomplete (<N>% done). Audit results will be partial."

### X-1: Full Re-scan

Scan every card def file in `crates/engine/src/cards/defs/`:

1. **TODO scan**: Grep for `TODO` comments. For each:
   - Is the TODO still valid? (check against current DSL)
   - If DSL now supports it: flag as "fixable"
   - If DSL doesn't: flag as "legitimate gap" (candidate for KNOWN_GAP)

2. **Empty abilities scan**: Grep for `abilities: vec![]`. For each:
   - Is this a vanilla creature (no oracle text abilities)? → OK
   - Does oracle text have abilities? → flag as "missing implementation"

3. **Known-issue pattern scan**: Check for all KI-1 through KI-19 patterns from
   the `card-batch-reviewer` agent's known-issue list.

4. **ETB tapped cross-check** (lands only):
   - For every Land type card def, compare ETB-tapped status against oracle text
   - Flag mismatches (missing ETB-tapped, spurious ETB-tapped)

5. **Mana cost spot-check**: Sample 10% of card defs, verify mana cost against oracle

6. **Type line spot-check**: Sample 10% of card defs, verify type line against oracle
   - Pay special attention to supertypes (Legendary, Basic, Snow)

Write results to `memory/card-authoring/audit-report.md`:

```markdown
# Card Audit Report

**Date**: <today>
**Total card defs scanned**: <N>
**Clean cards**: <N>
**Cards with issues**: <N>

## Issues by Category

| Category | Count | Severity |
|----------|-------|----------|
| Fixable TODOs (DSL supports it) | N | HIGH |
| Legitimate TODOs (DSL gap) | N | INFO |
| Missing implementations (empty abilities) | N | MEDIUM |
| Known-issue pattern matches | N | varies |
| ETB-tapped mismatches | N | HIGH |
| Mana cost errors (sample) | N | HIGH |
| Type line errors (sample) | N | HIGH |

## Per-Card Issues

### Card Name (file.rs)
- Issue 1: <description> (severity)
- Issue 2: <description> (severity)
...
```

### X-2: Fix Remaining Gaps

For each issue found in X-1:

1. **Fixable TODOs**: Implement the ability using current DSL. Remove the TODO.
2. **Empty abilities on non-vanilla cards**: Implement if DSL supports it.
3. **Known-issue patterns**: Apply the standard fix from the KI table.
4. **ETB-tapped mismatches**: Add or remove the replacement effect.
5. **Legitimate gaps**: If a micro-primitive would unblock 5+ cards, implement it.
   Otherwise, replace `TODO` with `// KNOWN_GAP: <description>` so it's explicitly
   documented as a permanent limitation.

Work in batches of 10-15 cards. Build + test after each batch.
Commit: `W6-audit: fix <N> audit findings — <brief description>`

### X-3: Re-scan Verification

Re-run X-1 to verify all fixes took effect. The report should show:
- Zero fixable TODOs
- Zero known-issue pattern matches
- Zero ETB-tapped mismatches
- Only KNOWN_GAP comments remain (all justified)

If new issues found, loop back to X-2.

### X-4: Final Build + Test

```bash
~/.cargo/bin/cargo build --workspace && ~/.cargo/bin/cargo test --all && ~/.cargo/bin/cargo clippy -- -D warnings
```

All must pass.

### X-5: Update Documentation

1. Update `CLAUDE.md` "Current State" with final card counts
2. Update `docs/project-status.md` Card Health section
3. Update `docs/workstream-coordination.md` Phase 5 checkboxes
4. Update `memory/workstream-state.md` W6 status

### X-6: Write Certification

Write `memory/card-authoring/audit-certification.md`:

```markdown
# Card Authoring Certification

**Date**: <today>
**Certified by**: audit-cards skill

## Summary

| Metric | Count |
|--------|-------|
| Total card def files | <N> |
| Complete (no TODOs, correct game state) | <N> |
| KNOWN_GAP (documented, justified) | <N> |
| Remaining TODOs | 0 |
| Wrong game state | 0 |

## KNOWN_GAP Inventory

| Card | Gap Description | Blocked Until |
|------|----------------|---------------|
| ... | ... | ... |

## Certification

All card definitions have been audited against oracle text. Every card either:
1. Has a complete, correct implementation matching oracle text, OR
2. Has a documented KNOWN_GAP with justification for why the DSL cannot express it

No card produces wrong game state. No stale TODOs remain.
```

### X-7: Check Off and Commit

1. Check off X-1 through X-6 in `docs/card-authoring-operations.md`
2. Commit: `W6-audit: card authoring complete — <N> cards, zero TODOs`

## Notes

- **X-1 is the most expensive step** — scanning 700+ files. Use `--quick` for a
  fast pass that skips MCP oracle verification.
- **MCP budget for oracle checks**: Use sparingly. Sample 10% for spot-checks.
  Only look up cards with flagged issues.
- **KNOWN_GAP vs TODO**: TODOs imply "we intend to fix this." KNOWN_GAP means
  "we know about this and it's acceptable for now." The distinction matters for
  the alpha readiness assessment.
