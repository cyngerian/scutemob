---
name: ability-status
description: Show ability coverage summary stats
---

# Ability Status

Show a quick summary of ability coverage stats from the tracking document.

## Procedure

1. **Read `docs/mtg-engine-ability-coverage.md`**.

2. **Extract the summary table** from the top of the document.

3. **Count by priority and status** — verify the summary table matches by scanning the
   actual rows. If there's a discrepancy, note it.

4. **Find top 5 P1/P2 gaps**: List the 5 highest-priority abilities with status `none` or
   `partial`.

5. **Report**:

```
## Ability Coverage Status

Last audited: <date>

| Priority | Total | Validated | Complete | Partial | None | N/A |
|----------|-------|-----------|----------|---------|------|-----|
| P1       | ...   | ...       | ...      | ...     | ...  | ... |
| P2       | ...   | ...       | ...      | ...     | ...  | ... |
| P3       | ...   | ...       | ...      | ...     | ...  | ... |
| P4       | ...   | ...       | ...      | ...     | ...  | ... |
| Total    | ...   | ...       | ...      | ...     | ...  | ... |

### Top Gaps
1. <ability> (P<N>, <status>) — <what's missing>
2. ...
```

If `$ARGUMENTS` is "detailed", also list all `partial` abilities with their specific gaps.
