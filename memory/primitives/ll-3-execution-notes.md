# LL-3 execution notes — `scutemob-257`

**Batch**: LL-3 (landscape lessons → project text). **Change class 0** (docs only; zero Rust
files touched). **Branch**: `feat/ll-3-lockstep-new-effect-checklist-conventions-additions-wor`.
**Base**: `9677fa0c`. **Source**: `docs/mtg-engine-landscape-assessment.md` §9 (rows a, b, the
Tilt row's exit-code contract, the `Completeness` row) + `crates/manabrew-compat/CLAUDE.md`
rules 1–2 of the phase.rs clone at `~/projects/scutemob-landscape/phase` (MIT/Apache-2.0, read-only).
**Licence note**: only phase.rs is permissive; the `manabrew` clone beside it is AGPL-3.0/GPL-3.0 — it may
be read, never copied from.

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
for a new *variant* (`Effect`, `AbilityDefinition` and `KeywordAbility` are re-exported;
`StackObjectKind` is deliberately absent from the prelude; only a brand-new payload *type* goes there); `tools/play-server/src/view.rs` defines no DTO mirroring any of the four enums (one safely
wildcarded `AbilityDefinition::Spell` picker); and `tools/replay-viewer` references none of them.
And `StackObjectKind` is **not** in the SR-8 protocol closure — no `Command` or `GameEvent` variant
carries one (only doc-comment mentions), so it bumps `HASH_SCHEMA_VERSION` alone.

## Acceptance — change class 0

- **Zero source files.** `git diff --name-only main...HEAD` returns ten paths, every one a `.md`;
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

## `/review` (Opus, read-only, 2026-09-05)

All five criteria PASS. Three MEDIUM findings, **all fixed in-cycle**; seven LOW, five of them
fixed because they were one-line accuracy defects in a file whose entire value is per-row accuracy.

### MEDIUM, fixed

- **M1 — a false COMPILE row.** The checklist claimed
  `state/stack_registry.rs::stack_index_for_announced_target` is a wildcard-free match over
  `StackObjectKind`. It is not: it is an eight-line `position()` closure that **delegates** to
  `card_in_stack_zone`, so a new variant needs no edit there and produces no compile error. Row
  deleted (the `card_in_stack_zone` row above it is the real registration). Verified by reading
  the function. Note the shape of this defect: the embedded verifier passed it, because a
  substring match confirms a symbol EXISTS and says nothing about what it does — which is the
  limit of any source-text gate, and exactly the pair-or-demote argument (CC-17).
- **M2 — the "unsupported" example over-claimed, in the rule about over-claiming.** It named
  three defs with a stale recipient blocker; only `stroke_of_midnight.rs` and
  `emergency_eject.rs` are. `saw_in_half.rs`'s blocker is a **different and still-open** gap
  (two copy-tokens with halved stats need `CreateTokenCopy` with per-stat modification), so its
  marker is correct. Inherited verbatim from `docs/mtg-engine-landscape-assessment.md` §2, which
  lumps all three — **the assessment table is corrected in this batch too**, as a new row, since
  it is the source a later batch would plan the def fixes from. The conventions example now
  carries `saw_in_half.rs` as the counter-example, which makes the rule sharper than it was.
- **M3 — the licence claim was too broad.** "The clones … (MIT)" is true of `phase` (MIT/Apache-2.0)
  and false of `manabrew` (AGPL-3.0/GPL-3.0). Corrected in `memory/conventions.md`, this file and
  the checklist. A blanket MIT on the directory is precisely the claim a later agent would rely on
  before copying code out of it.

### LOW, fixed (each a one-line accuracy defect)

- **L4** — "the four enums are already re-exported" in `helpers.rs`: three are; `StackObjectKind`
  is deliberately absent (a card-def author never constructs one).
- **L5** — a missing registration point, found by the reviewer applying this page's own
  floor-not-census rule to it: `crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs`
  `one_of_each_variant()` is a hand-maintained roster of all 27 `StackObjectKind` values with a
  hard-coded `assert_eq!(variants.len(), 27)` and **no forward pin** — its own message says it
  "does NOT detect a new StackObjectKind variant". Added as a SILENT row.
- **L6** — the `all_keywords` / `all_ability_definitions` rows were labelled "SILENT-ish" while
  the same cell explained that a roster test set-compares against the parsed enum. That is TEST,
  not silent; the label contradicted its own body and blunted the COMPILE/SILENT discipline.
- **L7** — `pb_dx28_chosen_object_roster.rs` was listed among the tests that parse `pub enum
  Effect`. Its own doc says it is "Pinned against the FUNCTION, not against `pub enum Effect`", so
  it catches a name the enum does not declare and a NEW variant slips past it. Same for
  `pb_dx26_attach_keyword_roster.rs` and `pb_dx39_source_relative_roster.rs`. Split into its own
  SILENT row; only `decision_gate.rs` and `pb_rs1_roster_sweep.rs` keep the forward-pin claim.
- **L8** — the acceptance evidence said nine changed paths; it is ten. The load-bearing half (no
  non-`.md` file) was and is correct.
- **L10** — the test `mod`-line row named `tests/core/main.rs` as if it were the file; each of the
  nine groups has its own `main.rs`. Reworded so `core` reads as the example it is.

### LOW, logged only (out of this batch's file scope)

- **L1** — `docs/engine-invariants.md` (SR-5) says the `AbilityDefinition` / `ZoneChangeAction`
  hazard "is not yet gated (`scutemob-67`)". The `AbilityDefinition` half **is** gated now, by
  `crates/engine/src/state/ability_definition_registry.rs::handling` plus its roster test. A
  one-line correction for whoever next edits that doc.
- **L2** — `docs/cleanup-retention-policy.md`'s directory table has no row for `memory/checklists/`.
  It falls under the generic `memory/` row, but a file cited by a skill and an agent has the
  property that makes `memory/primitives/` "untouchable corpus".
- **L9** — the committed `ll-3-task-list.md` mirror lagged the ESM comment thread (which was
  reposted at every milestone, as the worker prompt requires). Brought to final state in the fix
  commit; the lesson is that the FILE half of the task-list convention needs the same reposting
  discipline as the comment half, and it is worth one clause in the worker prompt next time.
