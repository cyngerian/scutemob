# Guide to the Authoring Status Report

This guide explains how to read `docs/authoring-status.md` and the tooling behind it.
The report itself is auto-generated and overwritten on every run — never edit it.
This guide IS hand-written and meant to evolve as the report does.

## What the report is for

A single, regenerable source of truth for "how is card authoring going?" — backed
entirely by the codebase, the authoring plan JSON, and `git log`. When discussing
strategy ("which group should we tackle next?", "is engine work or authoring effort
the bottleneck?"), reference this report instead of stale prose docs.

## How to regenerate

```
python3 tools/authoring-report.py
```

Run from anywhere — the script resolves paths relative to itself. It writes:

- `docs/authoring-status.md` — the human-readable report (this is what you read)
- `docs/authoring-status-missing.txt` — sidecar list of cards still missing on disk,
  one per line, tab-separated `<group>\t<name>`. Use as a batch-author worklist.
- `docs/authoring-status-prev.json` — last run's numbers, for the "Δ since last run"
  column. Overwritten every run.

## Section-by-section: how to read it

### Headline

The summary table. Counts on disk, plan coverage, TODO totals.

- **"Plan cards with a def file"** is the percentage of the original 2026-03-10
  authoring plan that has a card definition file on disk. Apostrophes, accents, and
  double-faced cards are handled tolerantly — "Akroma's Will" matches `akromas_will.rs`.
- **"Plan cards still missing"** is the gap. The full list is in the sidecar txt.
- **"Bonus defs"** are card files that exist on disk but were not in the original plan.
  These are NOT junk — see the "Bonus defs outside the plan" section below.
- **"Effective coverage"** can exceed 100% because bonus cards count toward it.
- **"Clean / TODO / Empty"** classify each file by quality:
  - **Clean** — no `// TODO` comments, has at least one non-empty ability.
  - **TODO** — at least one `// TODO` comment present.
  - **Empty** — `abilities: vec![]` (a placeholder file, no real definition yet).
- **"Δ since last run"** shows what changed since the previous time you ran the report.
  `·` means no change, `+5` means it grew, `-3` means it shrank, `—` means there's
  no number to compare against.

### Authoring activity (git, by window)

How many cards were created or modified in 7d / 30d / 90d / 365d windows. Built from
`git log` so it's always live. "Modified" means the file was touched; that includes
TODO closures, bug fixes, and authoring on existing files.

### Bonus defs outside the plan

The 2026-03-10 plan was a one-shot snapshot. Bonus defs accrue from three sources:

1. **Pre-plan authoring** — cards split out of the old `definitions.rs` monolith
   (W2 split in February 2026) and W5-cards batches that pre-date the plan.
2. **Wave-1 ability batches** (`W1-B*`) — when a keyword ability was implemented,
   reference cards demonstrating that ability were authored. These weren't in the
   plan but are real cards.
3. **Primitive-batch sample cards** (`W6-prim`) — when a DSL primitive shipped, a
   handful of sample cards using it were authored alongside the engine change.

If you ever see `NO-GIT-RECORD` extras, that means a `.rs` file exists on disk but
has never been committed. Investigate — could be uncommitted scratch work.

### Coverage by authoring-plan group

Per-group breakdown. The plan organizes ~1,636 cards into ~43 groups
(`combat-keyword`, `draw`, `mana-land`, etc.). Each row shows:

- **Auth / Total**: how many of the group's cards have a file on disk.
- **Clean / TODO / Empty**: among the authored ones, how the quality splits.

If a group is `5/7` authored but `0 clean / 5 todo / 0 empty`, every card in that
group has a definition that's blocked on engine primitives. That's an engine-work
problem, not an authoring-effort problem.

### Lagging groups

Groups with ≥5 cards in the plan but <50% authored. For each laggard, the report
prints the cards that ARE authored with their quality bucket and a verdict:

- **`unwritten`** — most authored cards are clean. The remaining work is just
  authoring the cards that don't exist yet.
- **`engine-blocked`** — most authored cards are `todo` or `empty`. Authoring more
  cards in this group will produce more TODO debt; the bottleneck is missing
  engine primitives.
- **`mixed`** — split, possibly worth reviewing case-by-case.

This verdict is useful when planning the next batch: tackle `unwritten` groups
with `/author-wave`; tackle `engine-blocked` groups with a primitive batch (PB-N).

### TODO classification

Every `// TODO` line in every card def is matched against pattern buckets like
"CDA / dynamic P/T", "TriggerCondition::* missing variant", "X-scaled tokens".
Lines that match no pattern fall into "OTHER (unclassified)". The classifier is
in `TODO_BUCKETS` at the top of `tools/authoring-report.py` — extend it by adding
`(re.compile(r"...", re.I), "bucket name")` tuples.

The "Δ since last run" column tells you which gap categories are growing,
shrinking, or stable. If a primitive batch closed 12 TODOs of one bucket, you'll
see `-12` after re-running the report.

### Raw OTHER samples

Twelve unclassified TODO lines, deterministically chosen by sorting all OTHER
lines by slug and taking an evenly-spaced sample. Read them. If you spot a
pattern across 3+ lines, that's a new bucket — add it to `TODO_BUCKETS` and
rerun.

### Missing card-defs sidecar

Pointer to `docs/authoring-status-missing.txt`. The full list of plan cards with
no on-disk file, sorted by group. Use it as a worklist for the next authoring
session.

### Sentinel — files outside git

Only appears if there's something to flag. NO-GIT-RECORD means uncommitted scratch
files; ERROR means git lookup failed. Investigate before committing or running
further authoring batches.

## What is NOT in this report (and why)

These were considered and deliberately excluded. If you want any of them, file an
issue or extend the script — they're just not in scope today.

### Plan staleness vs. external data sources

The plan was generated from EDHREC + deck data on 2026-03-10. New high-popularity
Commander cards have been printed since (Aetherdrift, etc.) and the plan does not
know about them. Detecting "what cards SHOULD be in the plan but aren't" requires
fetching fresh Scryfall and EDHREC data and re-running the planner. That's a
separate workflow — `tools/scryfall-import/` for the import side. Not part of
this report.

### Oracle-text drift

A card's printed Oracle text occasionally changes via errata. The plan stores a
snapshot of Oracle text from 2026-03-10. Detecting "this plan card's text has
changed" requires MCP `lookup_card` calls per card (1,636 network requests).
Not part of this report — could be a separate `tools/oracle-drift-check.py`
script if the need arises.

### Slug aliases (a "lookup file" of name → filename)

The script's slug matcher handles apostrophes, accents, and double-faced cards
internally. Earlier I considered exporting that mapping as a sidecar JSON file
("Akroma's Will" → "akromas_will") so other scripts wouldn't have to duplicate
the matcher logic. Decided against because:

1. There are no other scripts today that need this mapping.
2. The matching logic is 8 lines (`slugify` + `all_face_slugs`); easier to
   import than to maintain a generated lookup.

If you ever build a card-search tool, a card-loading tool, or anything else
that needs name-to-slug translation, lift the two functions instead of
re-deriving.

### Per-card rule-coverage validation

The report says how many cards have a definition file. It does NOT say whether
that definition correctly implements the card's Oracle text. Verifying
correctness is what `card-batch-reviewer` and `/audit-cards` do — separate
tools, separate workflow.

### Game-script test coverage per card

Whether each authored card has a corresponding game-script test. That lives in
`docs/mtg-engine-game-scripts.md` and the `/audit-cards` workflow.

## Extending the report

Common extensions:

- **Add a TODO bucket**: edit `TODO_BUCKETS` near the top of `tools/authoring-report.py`.
  Put more-specific patterns BEFORE more-general ones — first match wins.
- **Add an activity window**: edit the `WINDOWS` tuple.
- **Bump the OTHER sample count**: edit `SAMPLE_OTHER_N`.
- **Change the laggard threshold**: edit the inline `r[0] >= 5 and r[1] / r[0] < 0.5`
  filter in the Lagging groups section.

If you add a new section, document it here too.

## When does the report drift?

The report is regenerated from current state every run, so it never drifts. But:

- **`docs/authoring-status-prev.json`** is overwritten every run. If you want a
  longer-window comparison, copy it to a baseline file before re-running:
  `cp docs/authoring-status-prev.json docs/authoring-status-prev.frozen.json`
  (then manually point the script at the frozen file or compute deltas yourself).
- **The plan JSON** at `test-data/test-cards/_authoring_plan.json` is itself a
  one-shot snapshot. The report flags its `generated` timestamp in the headline
  so you know how stale the universe target is.
