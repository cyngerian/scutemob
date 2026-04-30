---
name: review-subsystem
description: Load the appropriate memory topic file for a subsystem and surface open issues from the milestone reviews doc.
---

# Review Subsystem

Given a subsystem name (`$ARGUMENTS`), do the following:

## Step 1: Determine which topic file to load

Route by subsystem name:
- `rules`, `layers`, `sbas`, `sba`, `combat`, `stack`, `targeting`, `casting`, `resolution`, `priority` → Read `memory/gotchas-rules.md`
- `infra`, `state`, `builder`, `hash`, `cards`, `effects`, `engine`, `tests`, `testing` → Read `memory/gotchas-infra.md`
- Unknown or ambiguous → Read BOTH files and note the ambiguity

## Step 2: Search for open issues

Use Grep to search `docs/mtg-engine-milestone-reviews.md` for lines containing the subsystem name (case-insensitive). Look for lines that appear to be open/unresolved:
- Lines with issue IDs like `MR-M<n>-<nn>`
- Lines with "OPEN" status
- Avoid lines marked "CLOSED" or "resolved"

## Step 3: Report

Summarize:
1. Which topic file(s) were loaded
2. Count of gotchas in the loaded file(s)
3. List of open issues in the reviews doc for this subsystem (or "none found")
