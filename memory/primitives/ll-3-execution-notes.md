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

### F4 — the census found two registration points the commissioning brief did not name

Dispatch hygiene 6 says a brief's site list is a floor, and it was one again. The brief named ten
sites; the census (delegated to a read-only Explore agent, its load-bearing claims re-verified by
hand) added:

1. **`crates/engine/src/state/stack_registry.rs`** — `card_in_stack_zone`, `source_of` and
   `stack_index_for_announced_target` are all wildcard-free over `StackObjectKind`, plus the
   **deliberately duplicated** `mtg_simulator::invariants::stack_card_of`, which must NOT delegate
   to the engine (the check exists to catch the engine getting the classification wrong, so reading
   the engine's own answer back would make it agree with the bug). The module's own doc records the
   defect that created it: `Effect::CounterSpell` matched the literal `Spell` variant, fell through
   `MutatingCreatureSpell`, and no-opped (PB-DX25, `OOS-SIM3-5`).
2. **`crates/engine/src/rules/mana.rs::is_mana_producing_effect`** — the dangerous one. A
   `matches!` over an allow-list of the ten `AddMana*` variants: a new mana-producing `Effect` not
   added simply returns `false`, so the ability is not a triggered mana ability under CR 605.1b, it
   uses the stack, and it can be responded to. Nothing reddens.
3. **`crates/engine/src/state/ability_definition_registry.rs::handling`** — the `AbilityDefinition`
   twin of SR-5's keyword registry. Its module doc says that without it "a newly added variant
   compiles everywhere and is silently inert". `docs/engine-invariants.md` (SR-5) still says this
   hazard on `AbilityDefinition` "is not yet gated (`scutemob-67`)" — **stale**; the registry exists
   and is exhaustive. Not fixed here (out of this batch's file scope); worth a one-line correction.

The census also **corrected the brief** on three points, all verified: `helpers.rs` needs no edit
for a new *variant* (all four enums are already re-exported; only a brand-new payload *type* goes
there); `tools/play-server/src/view.rs` defines no DTO mirroring any of the four enums (one safely
wildcarded `AbilityDefinition::Spell` picker); and `tools/replay-viewer` references none of them.
And `StackObjectKind` is **not** in the SR-8 protocol closure — no `Command` or `GameEvent` variant
carries one (only doc-comment mentions), so it bumps `HASH_SCHEMA_VERSION` alone.

## Acceptance — change class 0

- **Zero source files.** `git diff --name-only main...HEAD` returns nine paths, every one a `.md`;
  `grep -Ev '\.md$'` over that list is empty. No `.rs`, no `Cargo.toml`, no `test-data/`, so the
  suite and clippy read nothing this branch changed (see F3).
- **The checklist verifies itself.** `memory/checklists/new-effect-variant.md` carries a paste-able
  verifier and passes it **31/31 rows, 0 stale** at `9677fa0c` — run both from scratchpad and
  exactly as the file prints it.
- **Counts re-derived independently.** `Effect` 106, `AbilityDefinition` 68, `KeywordAbility` 166,
  `StackObjectKind` 27, by a brace-depth parse of the enum bodies — agreeing with the census, and
  with `pb_rs1_roster_sweep.rs`'s "106-variant `Effect`" comment and SR-5's "166 variants".
- **CLAUDE.md guards.** 249 lines (`/eot` fails at 250); four ESM-guarded headings present; three
  `## Current State` keys present.
- **CHANGELOG.** One entry, 10 body lines, newest first.

## LOW findings

- **L1** — `docs/engine-invariants.md` (SR-5) says the `AbilityDefinition` / `ZoneChangeAction`
  hazard "is not yet gated (`scutemob-67`)". The `AbilityDefinition` half **is** gated, by
  `crates/engine/src/state/ability_definition_registry.rs::handling` + its test. Out of this
  batch's file scope; a one-line correction for whoever next touches that doc.
- **L2** — `docs/cleanup-retention-policy.md`'s directory table has no row for `memory/checklists/`.
  It falls under the generic `memory/` row today, but the new file is cited by a skill and an agent,
  which is the property that makes `memory/primitives/` and `memory/abilities/` "untouchable
  corpus". Worth a row the next time that policy is edited.
- **L3** — `memory/conventions.md` line 52 said golden scripts live in `test-data/golden-games/`;
  the directory is `test-data/generated-scripts/<group>/`. Fixed in this batch (it is in a file the
  brief named, and a stale path in the conventions file is exactly what the new rules are about).

## `/review`

(filled in after the review pass)
