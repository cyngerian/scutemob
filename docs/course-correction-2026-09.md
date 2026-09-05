# Course Correction — September 2026

<!-- last_updated: 2026-09-05 -->

**Status**: REVIEWED AND APPROVED, section by section, by the owner on
2026-09-05 (interactive review), including the reconciliation with
`docs/course-correction-2026-09-addendum.md` (§9). Written the same day from an
independent audit of the tree at `8604207e` (main, PB-DX56 collected; PB-DX57
in flight on `scutemob-236`). §4.4's open questions are answered below. Tasks
are filed in ESM; IDs in §10.

**Owner ruling it serves**: `docs/end-state.md` — playable matches with the
owner's 6-person pod. This document does not re-rank seeds, close seeds, or
override any standing invariant. It changes what the project measures and what
the next tasks are.

---

## 0. Summary

The engine is technically strong and the process around it is rigorous to the
point of having become the product. Measured against the end state, the project
has not moved since July: card coverage is flat at 63%, the engine has been
played by a human once (2026-08-01/02), the pod's decks are not checked in, and
every commit since the 2026-08-15 ruling has been seed-queue work. The roadmap
document (`docs/mtg-engine-roadmap.md`) is the February plan and is not what is
being executed; the v4 seed queue is, and its ranking criterion does not
reference the end state.

The fix is mostly documents and process, costs no engine work, and fits between
collecting PB-DX57 and the next dispatch. It is ordered so that the efficiency
win comes first, because every later task gets cheaper.

| Do now | Do not |
|---|---|
| Let PB-DX57 finish and collect it | Dispatch v4 ranks 22 (PB-DX9) or 23 (PB-DX38) |
| §3 context diet, one chore commit | Big-bang refactor of the engine |
| §4 anchor: decks in, pod-coverage report | Retire any gate that catches real bugs |
| §5 play: hot-seat seats, play session two | Start M10 networking before a pod match |

---

## 1. Audit findings (measured 2026-09-05)

Every figure below was taken from the tree or from `git`; the recipe is in §8 so
the next reader re-measures rather than trusts.

### 1.1 What is working

- **The test suite reproduces its pin.** `cargo test --workspace --no-fail-fast`
  on `8604207e`: 5,316 passed / 0 failed / 5 ignored across 72 targets, wall
  time ~101 s of test execution. This matches CLAUDE.md's published figure.
- **Machine-enforced invariants are real**: pinned toolchain (SR-11), `-D
  warnings` in the workspace lint table, the seal gate, the HASH/PROTOCOL
  fingerprint gates, `[profile.fuzz]` with assertions on, 299 golden scripts,
  and a fuzzer whose HARD bucket reads 0 on the standard invocation.
- **The review-fix cycle finds real defects.** The batch records show gates
  defeated by execution and repaired; that is engineering, not theatre.
- **Engine hygiene**: 94k source lines, no `todo!`/`unimplemented!`, 70
  `unwrap`/`expect`/`panic` sites, 4 `unsafe`, six external dependencies, no IO,
  and a CR citation on nearly every rule.
- **Two crates are already the right shape for pod play**: `crates/view-model`
  redacts per seat (Invariant 7), and `tools/play-server` runs 2..=6 seats.

### 1.2 Process has eclipsed product

| Signal | Value |
|---|---|
| CLAUDE.md, loaded into every session and every worker | 5,080 lines ≈ 112k tokens |
| Total context loaded before any work starts | ≈ 121k tokens |
| `memory/` markdown | 265k lines, 20 MB |
| `memory/workstream-state.md` alone | 8,280 lines |
| September non-merge commits touching no code | 165 of 295 |
| Lines added, last 30 days: engine src / tests / memory | 10.9k / 84.1k / 37.8k |
| Commit message length, September | median 550 bytes, max 6.1 KB |
| Seed registry rows (`docs/audits/decision-point-audit.md`) | 510 (245 CLOSED, 19 LIVE, 13 OPEN, 13 DEFERRED, 16 PARTIAL, rest parked/recorded) |
| Distinct OOS ids anywhere in the repo | 620 |
| Test files that pin the HASH/PROTOCOL version literal | 48 |
| Tests that grep engine source text ("source gates") | 486 of 5,276 (≈9%), in 53 files |

Every batch appends a 100–200 line narrative to CLAUDE.md, a second copy to
`workstream-state.md`, and a third to its own execution-notes file. The
`/eot` skill's own "≤250 lines" size guard for CLAUDE.md has been ignored
since at least July. `docs/end-state.md` proposed four process amendments on
2026-08-15 (operator-delta line, play sessions as queue input, net-negative
re-ranks, releases in player terms). None has been adopted; the repo has zero
tags.

### 1.3 Code shape

| Function | Lines |
|---|---|
| `rules/resolution.rs::resolve_top_of_stack_inner` | 8,934 |
| `effects/mod.rs::execute_effect_inner` | 6,957 |
| `rules/casting.rs::handle_cast_spell` | 5,043 |
| `rules/abilities.rs::check_triggers_with_timing` | 3,876 |

20 engine functions exceed 500 lines; 63 exceed 200. The four above are single
`match` statements over 106 `Effect` variants and 27 stack-object kinds. They
cannot be reviewed as units, which is why each batch's review keeps finding a
site the batch's own census missed. The zone-change replacement block
(`check_zone_change_replacement` plus its Redirect/Proceed/ChoiceRequired arms)
is copied 23 times. The batch histories' recurring "a fifth hand-rolled copy"
is the same disease.

### 1.4 Fitness to the end state

- **Played once.** One human playtest (2026-08-01/02). It found zero engine
  bugs and seventeen client, simulator and card-def defects. Nothing since.
- **Authoring stopped in April.** New card-def files per month: Feb 137,
  Mar 172, Apr 11, since then 1. Coverage 63.1% on 2026-07-26, 63.2% today.
  The end-state doc says authoring "is expected to become the critical path".
- **No decks, no metric.** `decks/` does not exist; the proposed pod-coverage
  number cannot be computed. Commits since the ruling mention "pod" or "deck"
  four times.
- **No multi-human surface.** `play-server` is one human plus bots. Six humans
  need hot-seat seats (small) or networking (M10, not started;
  `crates/network` is a 4-line doc comment).

### 1.5 Roadmap

`docs/mtg-engine-roadmap.md` is the February plan. M10-pre's five checkboxes
have been unticked since March; it routes through Tauri (untouched since
2026-02-20) and M14 asset polish to reach alpha; its risk register has no row
for "process consumes the schedule". The plan actually being executed is
`memory/primitives/seed-rerank-2026-08-14.md` §4: 41 ranked entries, 20
shipped, a user-approved chain to rank 23, then 18 more ranked by
correctness-first seed yield. Ranks 24–41 include "back-face starting loyalty"
and "edgar return-transformed", with no named player behind them.

---

## 2. In-flight work

**PB-DX57 (`scutemob-236`, v4 rank 21)** has a live worker, ten commits, and is
already writing its handoff. **Let it finish and collect it normally.**

**Do not dispatch rank 22 (PB-DX9) or rank 23 (PB-DX38).** Rank 23 is a
CR-citation sweep across 97 files; rank 22's surviving content is one inert
field plus multi-card search. Neither has a pod customer. The approved
five-task chain is therefore closed at task 3 of 5 by owner decision (record
that decision in the v4 memo banner when §4 lands, so the next `/dispatch`
cannot re-pick them).

The direction change is documents and process. It can happen in the gap
between collecting DX57 and the next dispatch.

---

## 3. Context diet (efficiency first)

**Goal**: a session or worker loads under 15k tokens of project context before
work starts, down from ~121k. Nothing that catches bugs is removed.

### 3.1 Changes

1. **CLAUDE.md → under 250 lines.** Keep: the nine Architecture Invariants, the
   SR gate pointer table, key paths, the Agents table (trimmed per §6), one
   headline metric line (pod coverage once §4 lands, `docs/authoring-status.md`
   until then), one "next dispatch" line, and pointers. Move the entire
   "Current State" section (4,694 lines) **verbatim** to
   `memory/archive/claude-md-current-state-2026-09-05.md`. Nothing is lost.
2. **One narrative per batch, in one place.** A batch writes
   `memory/primitives/pb-<id>-execution-notes.md` (already the convention) and
   a ≤10-line entry at the top of a new `CHANGELOG.md`. CLAUDE.md and
   `workstream-state.md` get a one-line pointer each, never a narrative.
3. **Rotate `workstream-state.md`.** Move everything below "Active Claims" to
   `memory/archive/workstream-state-2026-09-05.md`. Keep the claims table and
   the last handoff, capped at 60 lines. `/eot` already has a 5-entry history
   window rule; enforce it.
4. **Commit messages**: title line plus at most ten body lines. Detail goes in
   the notes file the message points at.
5. **Delete the scattered version sentinels.** 48 test files pin the
   `HASH_SCHEMA_VERSION` / `PROTOCOL_VERSION` literal; they exist only to be
   re-pinned, and the "re-pin across 49 files, survivor-scan both axes" ritual
   appears in nearly every batch record. Keep exactly the two canonical gate
   tests (`hash_schema`, `protocol_schema`) plus their history/frozen-prefix
   companions. Everything else asserts against the constant, not a literal.
6. **Scale the acceptance ritual to the change class.**

   | Change class | Required | Not required |
   |---|---|---|
   | Engine behaviour (`crates/engine/src`, `crates/card-types/src`) | suite, clippy, fmt, revert-proven probe per fix, wire prediction before code | bench A/B unless a hot-path file is touched (`layers.rs`, `sba.rs`, `priority.rs`, `combat.rs`) |
   | Card defs only | suite, `check-defs-fmt.sh`, regenerate authoring status, batch review | revert matrix, wire prediction, bench |
   | Tests / docs / tooling only | suite, clippy | everything else |
   | New source gate added | one executed defeat of the gate, recorded in the test's own doc | bypass matrix over every other gate in the batch |

7. **Self-contained worker brief.** `/dispatch` writes an ≤80-line
   `.esm/worker.md`-adjacent brief (task, criteria, files, gotcha pointers).
   The worker prompt stops instructing "read CLAUDE.md" as step one; the
   trimmed CLAUDE.md is small enough that this no longer matters, but the brief
   is what the worker works from.
8. **Registry hygiene**: `docs/audits/decision-point-audit.md` gets a
   machine-parseable status column (the audit's own tally had to be derived by
   grepping bold words). No content change.

### 3.2 What is deliberately kept

CI and the toolchain pin; every SR gate; the two fingerprint gates and their
history rows; golden scripts; the fuzzer with its HARD-equals-zero ratchet; the
CR-citation rule (Invariant 8); ESM task tracking; the `/review` fix cycle; the
registry as ground truth.

**Hard constraint (addendum A5, binding on CC-2, CC-16 and CC-17)**: the
`HASH_SCHEMA_HISTORY` and PROTOCOL history rows and both
`FROZEN_HISTORY_PREFIX_DIGEST` pins; the declaration and stream fingerprint
gates themselves; `[profile.fuzz]` and the HARD-equals-zero ratchet; the SR-3
seal gate. Nothing in this document or the addendum touches them.

### 3.3 Draft tasks

- **CC-1** (doc-only, one chore commit, coordinator): CLAUDE.md rewrite +
  Current State archive + workstream-state rotation + `CHANGELOG.md` seeded with
  the last three batches. Acceptance: `wc -l CLAUDE.md` < 250; archive files
  byte-contain the moved text; `/start` still orients.
- **CC-2** (tests only): delete the 46 non-canonical version-literal
  assertions; both fingerprint gates still green; document the rule in
  `docs/engine-invariants.md` SR-8.
- **CC-3** (skills): update `/collect`, `/eot`, `/dispatch`,
  `/implement-primitive` per §6.3 so they write to `CHANGELOG.md` and the notes
  file instead of CLAUDE.md, and carry the change-class table.
- **CC-4** (doc): add the change-class table to `memory/conventions.md`.

**Review**: [x] approved by owner 2026-09-05 (interactive review)  [ ] amended

---

## 4. Anchor: decks in, one metric

**Goal**: the question "is the project closer to pod play" has a number.

1. Create `decks/` with the six pod decklists: the pilot's exported file
   (Moxfield/Archidekt) kept verbatim under `decks/exports/`, and a generated
   plain decklist (`1 Card Name` per line, `Commander:` header line, the format
   `test-data/test-decks/fetch_decks.py` already consumes) under `decks/`. One
   pair per deck, named by pilot. The converter lives in
   `tools/authoring-report.py` (CC-6).
2. Extend `tools/authoring-report.py` to emit **pod coverage**: X of N
   distinct cards across the six decks are `Complete` and deck-legal, plus a
   per-deck table and a ranked list of the missing cards with their blocker
   note (`partial` note text or "no def"). Written to `docs/pod-coverage.md`,
   regenerated at every `/collect`.
3. Add a `pod_blocker` column to the seed registry rows that name a card. A
   seed with no pod blocker argues for rank from something else, explicitly.
4. Answer the end-state doc's open questions in this document's §4.4 (owner
   input required).

### 4.4 Owner answers (2026-09-05)

| Question | Answer |
|---|---|
| Format | **Commander.** Every engine assumption to date stands. |
| Play surface for match one | **Hot-seat on one machine.** P1 as written; P3 networking waits until co-location is the bottleneck. |
| Deck check-in format | **Both.** The Moxfield/Archidekt export is checked in as-is for provenance; a plain `1 Card Name` decklist is generated from it and is what the coverage report reads. CC-6 gains the converter. |
| Judge button | **Yes, and pull it forward.** Manual state adjustment while paused (the old M13 item) becomes a P1 deliverable; a "playable with known holes plus a judge button" match is the target for play session two. CC-9 gains it. |

### 4.5 Draft tasks

- **CC-5** (owner + coordinator): check in six decklists under `decks/`.
- **CC-6** (tooling): export-to-plain-decklist converter plus pod-coverage
  report in `tools/authoring-report.py`, output `docs/pod-coverage.md`,
  headline line copied into CLAUDE.md by `/collect`. Acceptance: report
  regenerates from `decks/` alone; missing-card list is sorted by number of
  decks that need the card; a card name in an export that has no def is
  reported, never silently dropped.
- **CC-7** (doc): registry `pod_blocker` column; the v4 memo's ranks 24–41
  re-triaged against it (see §5.3).

**Review**: [x] approved by owner 2026-09-05 (interactive review)  [ ] amended

---

## 5. The roadmap, in pod terms

Retire `docs/mtg-engine-roadmap.md` M10 through M15 as written (banner them
HISTORICAL the way earlier plans were; do not delete). Replace with four phases
whose gate is a played match.

### 5.1 Phases

| Phase | What | Gate to next |
|---|---|---|
| **P0 Anchor** | §3 + §4 | Pod-coverage number exists and is in CLAUDE.md |
| **P1 Play** | Hot-seat multi-human seats in `tools/play-server` (N human seats, seat switch re-renders through the existing seat-redacted view-model; bots fill the rest), **plus the judge button**: pause, manual state adjustment (life, zone moves, counters), resume — the old M13 item pulled forward so a match with known holes is playable. Then **play session two**: owner + at least one pod member, even a partial game. Defects found in play go to the front of the queue. | A match log from a played session, with its defect list filed |
| **P2 Author** | Card authoring resumes in pod-deck order using the existing author/review/fix agents (§6.2 dry run first). Engine seeds are admitted only with a `pod_blocker` or a fuzzer crash. v4 ranks 24–41 re-triaged: pod-blocker seeds keep their place, the rest become explicit won't-do rows. | Pod coverage above an owner-chosen threshold for the first two decks |
| **P3 Network** | Old M10a, unchanged in scope, started only when hot-seat matches have happened and co-location is the bottleneck. Tauri and M14 asset polish leave the critical path. | — |

### 5.2 Standing rules (from `docs/end-state.md`, now adopted)

- Every `/eot` handoff carries one line: *what can a player observe now that
  they could not at the last handoff?* Two consecutive empty lines force a
  P1/P2 item to the front.
- A play session after every three collected batches, as queue input.
- Any future re-rank retires at least as many seeds as it admits, with
  won't-do rows.
- A git tag every five collected batches, with notes written for the pod.

### 5.3 Engine shape

No big-bang refactor. Adopt **split on touch**: a batch that edits an arm of
any of the four functions in §1.3 first moves that arm into its own module
(mechanical move, behaviour-neutral, suite-protected), then makes its change.
Schedule one dedicated mechanical pass on `execute_effect_inner` after P1,
when a quiet window exists; the 23-copy replacement block is extracted in the
same pass.

### 5.4 Draft tasks

- **CC-8** (doc): banner roadmap M10–M15 HISTORICAL; add a "Pod phases"
  section pointing here; strike v4 ranks 22–23 with the owner decision.
- **CC-9** (play-server + frontend): hot-seat multi-human seats. Acceptance:
  two humans complete a turn cycle each on one machine with hands hidden from
  the other; the Invariant-7 gates cover the new channel; `npm run build` run
  (not declared N/A).
- **CC-9b** (play-server + frontend + one engine entry point): the judge
  button. Pause; edit life totals, move an object between zones, add/remove
  counters; resume. Every adjustment enters the engine as a `Command` (Invariant
  3) and is logged as an event (Invariant 4) so the replay stays complete.
  Acceptance: a paused game accepts one of each adjustment kind and resumes with
  SBAs and triggers consistent; the HASH/PROTOCOL wire prediction is written
  before code (a new `Command` variant is a wire change).
- **CC-10** (owner): play session two; defects filed as tasks, not seeds.
- **CC-11** (doc): re-triage v4 ranks 24–41 with the `pod_blocker` column;
  won't-do rows written.
- **CC-12** (engine, later): `execute_effect_inner` split into per-family
  modules plus replacement-block extraction. Not before P1.

**Review**: [x] approved by owner 2026-09-05 (interactive review)  [ ] amended

---

## 6. Agents and skills review

Seventeen agents under `.claude/agents/`, twenty-seven skills under
`.claude/skills/`. Read in full 2026-09-05. The pipeline that has been running
(`/dispatch` → worker → `primitive-impl-runner` / `primitive-impl-reviewer` →
`/review` → `/collect`) works and should be kept. What follows is tuning.

### 6.1 Findings

**Stale references (cheap fixes, do them in CC-3).**

| Where | Stale item | Correct target |
|---|---|---|
| `primitive-impl-planner` step 3 | reads `docs/primitive-card-plan.md` (HISTORICAL) and `docs/dsl-gap-closure-plan.md` (SUPERSEDED); "122 dangerous cards" | the active queue file named in CLAUDE.md; `docs/authoring-status.md` |
| `primitive-impl-runner` step 1 | `tools/replay-viewer/src/view_model.rs` as an exhaustive-match site | `crates/view-model/src/lib.rs` (moved 2026-08-01) |
| `/implement-primitive` | names `oos-retriage-plan-2026-07-18.md` as the active queue (two re-ranks old) | resolve from CLAUDE.md; after §4, from `docs/pod-coverage.md` |
| `bulk-card-author` | session data from `_authoring_plan.json` (a 2026-03-10 snapshot at `test-data/test-cards/`) | pod-deck missing-card list from `docs/pod-coverage.md` |
| `/dispatch` step 10 | a `sleep 30` polling loop under `run_in_background` | contradicted by dispatch hygiene 3/5/13 in memory: use the Monitor tool, never a sleep loop |
| `/dispatch` worker prompt | "use TaskCreate" | dispatch hygiene 10: workers in this build have no TaskCreate; the accepted substitute is an `esm task comment` task list |
| runner agents' `tools` | `"Task"` | the tool is `Agent` |
| 7 agents/skills | `cargo clippy -- -D warnings` | `cargo clippy --workspace --all-targets -- -D warnings` (the CI bar) |
| `.claude/docs.yaml`, `/collect` step 7.3, `/implement-primitive` close step 3, `/eot` step 6 | all write into CLAUDE.md "Current State" | `CHANGELOG.md` + the batch notes file (§3.1 item 2) — **this is where the bloat comes from** |

**Reviewers cannot execute.** `primitive-impl-reviewer` and
`ability-impl-reviewer` have no `Bash`. The PB-DX56 record says so explicitly
("Bash is disabled for this session"), and the batches' strongest findings are
the ones proven by execution. Give both reviewers `Bash` with a "read-only
use: run tests and reverts in a scratch worktree, never commit" instruction.

**`/review` is generic and that is fine.** It spawns a general-purpose Opus
reviewer with acceptance criteria. It has found real defects every batch (15
in PB-DX56). Keep it. Add one line: findings are ranked, HIGH and MEDIUM are
fixed in the cycle, LOW are logged to the notes file and not fixed unless
trivial. The current "all N taken" habit is a driver of batch length.

**Dormant agents.** Their descriptions are injected into every session's
context. With no milestone running and all abilities shipped, eight agents
have not been invoked since July: `rules-implementation-planner`,
`session-runner`, `milestone-reviewer`, `fix-session-runner`,
`ability-impl-planner`, `ability-impl-runner`, `ability-impl-reviewer`,
`ability-coverage-auditor`. Move them to `.claude/agents-dormant/` (restore
with a `git mv` when a milestone or ability needs them). `cr-coverage-auditor`
and `game-script-generator` stay; both are useful in P1/P2.

**Dormant or retired skills.** `start-work` (self-declared RETIRED), `end`
(replaced by `/eot`), `spawn` (superseded by `/dispatch`), `remedy` (SR track
closed 2026-07-16), `start-milestone`, `next-ability`, `ability-status`,
`audit-abilities`. Delete `start-work`, `end`, `spawn`; move the rest to
`.claude/skills-dormant/`. Keep `cleanup`, `crew`, `triage-cards`,
`author-wave`, `audit-cards`, `docs`, `new-doc`, `review-subsystem`,
`start-stepper` and the session-loop skills.

### 6.2 Card-authoring agents (P2 depends on these)

`card-definition-author` is tight and well-scoped (`maxTurns: 16`, one file
per card, never edits engine code, MCP for oracle text). `card-batch-reviewer`
carries the legal-but-wrong checklist and the multiplayer controller/owner
check. `card-fix-applicator` is sound. `bulk-card-author` is 515 lines, half
of it a DSL quick reference that duplicates `helpers.rs` and has not been
exercised since April against a DSL that has since gained variants (106
`Effect`, 53 `Condition`, 50 `TriggerCondition`, 33 `LayerModification`).

Before P2 starts, run **one dry-run wave of five pod cards** through
`author-wave` and read the review file. Expect the quick reference to need
updating; fix it from the review rather than by rewriting it speculatively.
Also: `bulk-card-author` must be pointed at `docs/pod-coverage.md`'s
missing-card list (item in §6.1), and both authors must be told the
`Completeness` marker rule (new defs are `Complete` or carry a marked note;
SR-2) — it is enforced by a gate but neither prompt states it.

### 6.3 Skill edits to make (all in CC-3)

1. `/collect` step 7: strike the queue row and write the `CHANGELOG.md` line
   and the pod-coverage headline; stop editing CLAUDE.md Current State.
2. `/eot` step 6: enforce the existing ≤250-line guard (fail the step if
   exceeded); add the operator-delta line to the handoff template.
3. `/dispatch` step 10: replace the polling loop with the Monitor recipe from
   memory dispatch hygiene 5/12/13; fix the worker prompt (`esm task comment`
   task list; brief file; change-class table).
4. `/implement-primitive`: resolve the active queue from CLAUDE.md's one
   "next dispatch" line; close phase writes `CHANGELOG.md`, not CLAUDE.md.
5. Reviewer agents: add `Bash` with the read-only instruction.
6. All: correct the stale paths in the table above; `clippy` invocation to the
   CI bar; `"Task"` → `"Agent"`.

### 6.4 Draft tasks

- **CC-3** (as in §3.3) carries every edit in §6.1 and §6.3.
- **CC-13** (authoring dry run): five pod cards through `/author-wave`; the
  review file is the deliverable; `bulk-card-author`'s quick reference
  corrected from it.
- **CC-14** (housekeeping): dormant agents and skills moved/deleted; CLAUDE.md
  Agents table trimmed to the nine active ones.

**Review**: [x] approved by owner 2026-09-05 (interactive review)  [ ] amended

---

## 7. Sequencing

```
collect DX57 ──► CC-1 CC-2 CC-3 CC-4 CC-14 (one coordinator session, no engine code)
              ──► CC-5 (owner: decks)  ──► CC-6 CC-7 CC-8
              ──► CC-9 (hot-seat)      ──► CC-10 (play session two)
              ──► CC-13 (authoring dry run) ──► P2 authoring waves + CC-11
              ──► CC-12 (executor split) when a quiet window exists
```

CC-1 through CC-4 and CC-14 are one session's work and need no dispatch.
CC-6, CC-9, CC-9b, CC-13 are dispatchable tasks in the existing pipeline. CC-5
and CC-10 need the owner.

**Filing gate (owner, 2026-09-05)**: no task is filed until the owner's
addendum document has been read and reconciled with this one. Reconciliation
notes go in §9 below.

---

## 9. Reconciliation with the addendum

`docs/course-correction-2026-09-addendum.md` (a second independent audit of the
same tree) was read by this document's author on 2026-09-05. Its §1
re-measurements all reproduce against the figures here (one drift: 19
layer-resolved `calculate_characteristics(` reads in `crates/simulator/src`,
not 25; immaterial). Its five recommendations were reviewed interactively with
the owner the same day. Dispositions:

| Item | Disposition | Amendment |
|---|---|---|
| **A1** offer layer is a second legality implementation | **Accepted** | The rule applies to **battlefield** reads only. Several of the 43 raw `.characteristics.` reads are on cards in hand or library (e.g. `legal_actions.rs:984/999/1141`, "is this a land I can play"), where the raw value is correct because no layer effect applies off the battlefield. The ratchet is a per-file ceiling pinned at today's counts, lowered on touch, never raised. → **CC-15** |
| **A2** derive-based `HashInto` | **Accepted in principle, scheduled after P1** | Not in the P0/P1 gap. Card authoring rarely adds `GameState` fields, so "before P2" is weak, and the parent's rule that the first pod-facing result lands before any refactor holds. Needs a new proc-macro crate (none exists in the workspace); 3 `imbl` container impls stay manual. Blocked on CC-10, paired with CC-12. → **CC-16** |
| **A3** pair-or-demote source gates | **Accepted as written** | → **CC-17** |
| **A4** reconcile M10-pre before bannering | **Accepted** (verified: 0 diagnostic events, no `stress-tests/` dir, 21 `lki_object_snapshot` sites, PB-DP9's `EffectChoiceQuestion` shipped) | Folded into **CC-8**. |
| **A5** must-not-retire constraint | **Accepted** | Restated in §3.2 as binding on CC-2, CC-16 and CC-17. |

The addendum's two process endorsements (rank `/review` findings and log LOW;
one executed defeat per new gate) are already §3.1 item 6 and §6.1.

### 9.1 Folded tasks

- **CC-15** (simulator, tests, coordinator batch): raw-`characteristics`
  ceiling ratchet over `crates/simulator/src`, per file, pinned at measured
  counts (43 total at HEAD); §5.3 split-on-touch extended to
  `legal_actions.rs` and `targeting.rs` with `rules/queries.rs` as the
  destination; rule recorded in `memory/conventions.md` with the battlefield
  qualifier. Acceptance: ratchet green at HEAD; lowering any ceiling by one
  reddens it; the rule text states that off-battlefield raw reads are correct.
- **CC-16** (engine, after CC-10, with CC-12): derive-based `HashInto` with
  `skip`/`private` attributes in a new `crates/hash-derive` proc-macro crate;
  151 impls transcribed; the source-parsing coverage half of `hash_schema.rs`
  deleted with the reason recorded in `docs/engine-invariants.md` SR-8.
  Acceptance: `hash_schema` and `protocol_schema` green; exactly one new
  history row per gate; `public_state_hash` and `private_state_hash` of the
  canonical fixture and five fuzz seeds byte-identical before and after except
  for the version byte; declaration fingerprint, history rows, frozen prefix
  and PROTOCOL untouched (A5).
- **CC-17** (doc, coordinator batch): the pair-or-demote rule in
  `memory/conventions.md`, cross-referenced from `docs/engine-invariants.md`.
- **CC-8** amended: before the HISTORICAL banner, tick the two shipped
  M10-pre items with pointers (resolution suspension → PB-DP9; LKI snapshot →
  SR-24 / PB-LKI-CC), leave the two absent items unticked with a one-line
  disposition each, and resolve the layer-bypass row by checking the audit's
  nine sites against HEAD once and recording the answer in the audit doc.

### 9.2 Sequencing after reconciliation

```
collect DX57 ──► CC-1 CC-2 CC-3 CC-4 CC-14 CC-15 CC-17   (one coordinator session)
              ──► CC-5 (owner: decks)  ──► CC-6 CC-7 CC-8
              ──► CC-9 CC-9b (hot-seat + judge button) ──► CC-10 (play session two)
              ──► CC-13 (authoring dry run) ──► P2 authoring waves + CC-11
              ──► CC-12 + CC-16 (executor split, derive hasher) after CC-10
```

**Filing**: all tasks above were filed in ESM on 2026-09-05 (owner decision at
the interactive review); the coordinator batch starts once DX57 is collected.
Task IDs are recorded in §10.

---

## 8. How to re-measure

All commands from the repo root.

- Tests: `cargo test --workspace --no-fail-fast > log 2>&1; grep -E '^test result' log | awk '{p+=$4;f+=$6;i+=$8} END{print p,f,i}'`
- Context load: `wc -c CLAUDE.md` ÷ 4.
- Docs-only commit share: for each `git log --since=<date> --format=%h --no-merges`, test whether `git show --numstat --format='' $c` names a path under `crates/` or `tools/`.
- Function lengths: brace-matching walk over `crates/engine/src/**/*.rs` (script in the 2026-09-05 audit session; trivially recreated).
- Replacement-block copies: `grep -rn 'check_zone_change_replacement(' crates/engine/src | wc -l`.
- Version-literal sentinels: `grep -rln -E 'HASH_SCHEMA_VERSION, *[0-9]+|PROTOCOL_VERSION, *[0-9]+|== *8[0-9]u8' crates tools | wc -l`.
- Source gates: `grep -rln -E 'read_to_string\(|include_str!\(' crates/engine/tests crates/simulator/tests | wc -l`.
- New card files per month: `git log --diff-filter=A --format=%ad --date=format:%Y-%m -- crates/card-defs/src/defs | sort | uniq -c`.
- Coverage trend: `docs/authoring-status.md` headline at each `/collect`; after §4, `docs/pod-coverage.md`.

---

## 10. Filed tasks (ESM, 2026-09-05)

| Item | Task |
|---|---|
| CC-1 | `scutemob-237` |
| CC-2 | `scutemob-238` |
| CC-3 | `scutemob-239` |
| CC-4 | `scutemob-240` |
| CC-5 | `scutemob-241` |
| CC-6 | `scutemob-242` |
| CC-7 | `scutemob-243` |
| CC-8 | `scutemob-244` |
| CC-9 | `scutemob-245` |
| CC-9b | `scutemob-246` |
| CC-10 | `scutemob-247` |
| CC-11 | `scutemob-248` |
| CC-12 | `scutemob-249` |
| CC-13 | `scutemob-250` |
| CC-14 | `scutemob-251` |
| CC-15 | `scutemob-252` |
| CC-16 | `scutemob-253` |
| CC-17 | `scutemob-254` |

Blocked-on relationships as in §9.2: CC-6 on CC-5; CC-9b on CC-9; CC-10 on
CC-9 and CC-9b; CC-11 on CC-6 and CC-7; CC-13 on CC-6; CC-12 and CC-16 on
CC-10. The coordinator batch (CC-1, 2, 3, 4, 14, 15, 17) starts once PB-DX57
(`scutemob-236`) is collected. Nothing else is dispatched until then.
