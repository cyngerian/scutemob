# LL-3 execution notes — `scutemob-257`

**Batch**: LL-3 (landscape lessons → project text). **Change class 0** (docs only; zero Rust
files touched). **Branch**: `feat/ll-3-lockstep-new-effect-checklist-conventions-additions-wor`.
**Base**: `9677fa0c`. **Source**: `docs/mtg-engine-landscape-assessment.md` §9 (rows a, b, the
Tilt row's exit-code contract, the `Completeness` row) + `crates/manabrew-compat/CLAUDE.md`
rules 1–2 of the phase.rs clone at `~/projects/scutemob-landscape/` (MIT, read-only).

## FOR THE COORDINATOR — copy to auto-memory at collect

### Dispatch hygiene LL-3 (2026-09-05) — the exit-3 rule

The `/dispatch` worker prompt (`.claude/skills/dispatch/SKILL.md`, step 8 `--prompt`) now
carries one more sentence, alongside the never-`git add -A` and never-`sleep`-under-
`run_in_background` rules:

> A gate you could not reach or could not answer — server down, tool or fixture missing, disk
> quota, output truncated, a suite you never ran — is reported as "could not find out", naming
> the gate and what blocked it; never report it as a failure, and never let it read as a pass.

**Why.** phase.rs's `scripts/tilt-wait.sh` separates exit 1 ("your code is broken") from exit 3
("I could not find out — Tilt unreachable, or watching a different checkout") and says in its own
header that *"collapsing them is how a wrong-checkout or a stopped Tilt gets misread as a compile
error, which teaches callers to distrust the script — and a distrusted freshness gate gets
bypassed, which restores the very false green this script exists to prevent."* scutemob has run
both failure directions: 2026-08-02, a `| tail` pipe hid a compile failure and faked a green run
(a non-answer read as a pass — dispatch hygiene 1); 2026-09-05, `/tmp` hit its user quota and
every coordinator shell command failed with `Disk quota exceeded`, with an `esm worktree create`
half-completing (an unreachable tool that reads as broken work — dispatch hygiene 11).

## What landed

| File | Change |
|---|---|
| `memory/checklists/new-effect-variant.md` | NEW — every registration point for a new `Effect` / `AbilityDefinition` / `KeywordAbility` / `StackObjectKind` variant, path + what to add + what silently fails |
| `.claude/skills/implement-primitive/SKILL.md` | cites the checklist |
| `.claude/agents/primitive-impl-runner.md` | cites the checklist |
| `memory/conventions.md` | new `## Landscape rules` section: three rules, each with a **Why** and a scutemob example |
| `.claude/skills/dispatch/SKILL.md` | the exit-3 sentence in the step-8 worker prompt |
| `CLAUDE.md` | one bold line at the head of `## Critical Gotchas` (249 lines, guard passes) |
| `docs/mtg-engine-landscape-assessment.md` | §9 table: `scutemob-257` against each adopted row, plus two new rows for the manabrew-compat rules |

## Findings and flags

### F1 (for the coordinator, auto-memory correction) — a stale gotcha in `MEMORY.md`

`MEMORY.md` → "Behavioral Gotchas" says: *"SelfEntersBattlefield triggers (PartnerWith,
Hideaway, Exploit) are NOT doubled by Panharmonicon — `doubler_applies_to_trigger` only matches
`AnyPermanentEntersBattlefield`."* **That is false at HEAD.** The `ArtifactOrCreatureETB` arm
(`crates/engine/src/rules/abilities.rs`) matches `AnyPermanentEntersBattlefield` **and**
`SelfEntersBattlefield`, with the comment "CR 603.2d + Panharmonicon ruling 2021-03-19 … This
mirrors the CreatureDeath arm's dual-event pattern". `docs/card-authoring-operations.md:292`
records the fix: *"PB-M: Panharmonicon — 2 bug fixes (SelfEntersBattlefield matching,
entering_object_id)"*. The memory line describes the pre-PB-M engine. It was drafted into this
batch as a conventions example and caught by verification before it shipped — which is itself
the case for the "verify before you cite" half of the new rules. In-repo docs are clean; only
auto-memory carries the stale claim, and only the coordinator can edit it.

### F2 — provisioned-file tension, flagged rather than silently resolved

`.esm/worker.md` rule 8 puts `.claude/skills/` off-limits; criteria 7568 and 7570 require
additive edits to `implement-primitive` and `dispatch`. Both were **skip-worktree** in this
worktree (`git ls-files -v` shows `S`), so they were not even on disk. Resolution: cleared the
bit on exactly those two paths, checked them out from HEAD, made purely additive edits (one cite
line; one prompt sentence). Main's copies are clean at HEAD, so the merge applies without
conflict; `esm worktree check` may still report them as provisioned change. Recorded in a task
comment at the start, not discovered at collect.

### F3 — acceptance-ritual ambiguity in the class-0 designation

The brief names "change class 0" and says `cargo test` is not required. `memory/conventions.md`
→ "Change-class acceptance table" has no numbered rows; its nearest row, "Tests / docs / tooling
only", requires *suite, clippy*. This batch touches **zero** files that either gate reads: the
diff is `.md` files plus two skill/agent `.md` files. Evidence recorded in place of the suite:
`git diff --name-only` against the merge base, showing no `.rs`, no `Cargo.toml`, no
`test-data/`. Worth one line in the table on the next edit to say that a docs-only diff (no
compiled input) is the case where the row's "suite, clippy" is vacuous.

## LOW findings from `/review`

(filled in after the review pass)
