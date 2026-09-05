# CHANGELOG

One entry per shipped batch, newest first, **at most ten lines each** (`docs/course-correction-2026-09.md`
§3.1 item 2). Each entry names the task, the merge commit, the seeds closed, the wire/test/coverage
deltas, and the notes file that holds the full record. `CLAUDE.md` and `memory/workstream-state.md`
carry pointers only, never narrative. Entries before 2026-09-05 are in
`memory/archive/claude-md-current-state-2026-09-05.md` and `memory/archive/claude-md-changelog-2026-0{7,8}.md`.

## 2026-09-05 — CC-15 (`scutemob-252`) — simulator raw-characteristics ceiling ratchet + split-on-touch

- New `crates/simulator/tests/cc15_raw_characteristics_ratchet.rs`: two-sided per-file pins on raw `.characteristics.`
  reads in `crates/simulator/src` (47 at HEAD: 2/14/28/1/1/1) plus a directory walk so an unrostered file must count 0;
  five executed defeats recorded in its module doc; paired per CC-17 with the SR-38 channel probes.
- The addendum's "43 at HEAD" was a grep-LINE count: it included 3 comment mentions and missed 6 line-wrapped chains.
- `memory/conventions.md` gains "Split on touch": engine §5.3 rule, extended to `legal_actions.rs` / `targeting.rs`
  with `rules/queries.rs` as the destination and the battlefield qualifier (hand/library raw reads are correct).
- Tests 5,330 → **5,333** (+3, 73 targets, 0 leavers); workspace clippy + fmt clean; 0 engine lines.

## 2026-09-05 — CC-17 (`scutemob-254`) — pair-or-demote rule for source-text gates

- `memory/conventions.md` gains the rule (addendum A3, accepted as written): a new source gate standing in for a
  behaviour ships with a behavioural probe under the same revert or a one-line reason + seed ID; no retroactive sweep —
  an existing unpaired gate gets its probe when re-keyed after a defeat; a paired gate is a backstop, no bypass matrix.
- Exemptions (SR-3/5/8/36, count ratchets) and the A5 must-not-retire list restated; `docs/engine-invariants.md`
  preamble cross-references it. Markdown only.

## 2026-09-05 — CC-4 (`scutemob-240`) — change-class acceptance table recorded

- `memory/conventions.md` gains "## Change-class acceptance table": the four rows verbatim from the course-correction doc
  §3.1 item 6 (diff-verified), the hot-path file list spelled out, and the "suite / clippy / fmt" definitions.
- `/dispatch` step 3 and the brief template cite the table by section; conventions' clippy line raised to the CI bar.
- Markdown only; no Rust touched.

## 2026-09-05 — CC-3 (`scutemob-239`) — skills and agents tuning

- `/collect`, `/eot`, `/implement-primitive`, `/implement-ability`, `/audit-cards` now write `CHANGELOG.md` + the batch
  notes file and touch only CLAUDE.md's three snapshot lines; `/eot` enforces the 250-line guard and carries the operator-delta line.
- `/dispatch`: steps 1–7 inlined (no `/spawn`), a ≤80-line `.esm/brief.md` step, worker prompt uses an `esm task comment`
  task list + change class; step 10 is the Monitor recipe (hygiene 5/12/13), the sleep polling loop is gone.
- Reviewer agents get read-only `Bash`; `/review` ranks findings (HIGH/MEDIUM fixed in-cycle, LOW logged).
- Stale refs corrected: primitive-card-plan, dsl-gap-closure-plan, view_model path, oos-retriage-plan, `_authoring_plan.json`
  (`/author-wave --cards` added for CC-13), `"Task"` → `"Agent"`, 7 clippy invocations to the CI bar. `.claude/docs.yaml`
  had no CLAUDE.md writer (the §6.1 row was wrong). Markdown only; no Rust touched.

## 2026-09-05 — CC-2 (`scutemob-238`) — scattered HASH/PROTOCOL version sentinels deleted

- 53 literal `assert_eq!(HASH_SCHEMA_VERSION|PROTOCOL_VERSION, <n>)` sites in 43 test files: 33 sentinel-only
  tests deleted, 11 assertions deleted in place, 4 misnamed tests renamed; 40 unused imports and 30 orphan banners gone.
- Only `core hash_schema` / `core protocol_schema` pin a version literal now; rule recorded in `docs/engine-invariants.md` SR-8.
- Tests 5,363 → **5,330** (−33 == deleted tests; name diff 37 leavers / 4 additions, all accounted); 0 engine lines;
  HASH 85 / PROTOCOL 44 untouched (A5). Notes: `memory/primitives/cc-2-execution-notes.md`.

## 2026-09-05 — CC-1 (`scutemob-237`, merge `4815467c`) — context diet

- `CLAUDE.md` 5,281 → 244 lines; `memory/workstream-state.md` 8,438 → 54; this file seeded with the last three batches.
- Everything moved is verbatim (diff-verified) in `memory/archive/{claude-md-current-state,claude-md-reference-sections,workstream-state}-2026-09-05.md`.
- Docs only; suite 5,363 / 0 / 6 unmoved.

## 2026-09-05 — PB-DX57 (`scutemob-236`, merge `cb6980f2`) — the gate-widening cluster

- Closed **OOS-DX28-1** and **OOS-DX28-6** as classes, plus **OOS-DX28-5**, **OOS-DX26-3**, **OOS-DX21-7**;
  **OOS-ADJ-2** (already taken by PB-DX42b) verified by execution. Filed OOS-DX57-1..5.
- 35 hand-written declaration mirrors censused by a test; 20 pinned against the declaration they mirror,
  three found already stale (one a subtraction gate that cancels against an over-declaration).
- OOS-DX21-7: 216 functions read, 17 vacuous assert-on-unchanged-state sites repaired (19 guard-removal
  proofs RED); the new gate then found an 18th in `rules/commander.rs`.
- Every gate the batch wrote was defeated and re-keyed: 10 adversarial defeats + 4 from `/review` (16 findings, all taken).
- **0 engine lines**; HASH 85 / PROTOCOL 44 unmoved; coverage unmoved 1,140/1,803; tests 5,316 → **5,363** (+47, 0 leavers).
- Notes: `memory/primitives/pb-DX57-execution-notes.md`. Chain CLOSED here by owner decision.

## 2026-09-05 — PB-DX56 (`scutemob-235`, merge `8604207e`) — fuzz violations made diagnosable, then diagnosed

- Closed **OOS-FB1-1** (crash→seed→replay evidence), **OOS-DX32-1**, **OOS-DX22-8**, rider **OOS-DP9-19(b)**. Filed OOS-DX56-1..15.
- Fuzzer HARD bucket **291 → 0** on the standard invocation; `--stop-on-error` no longer halts on an undiagnosed class.
- `player_consistency` was two arms with opposite CR dispositions (CR 800.4j permits the active-player arm;
  CR 800.4a forbids the priority arm); two extra-turn / cleanup-grant holes fixed (CR 800.4k).
- `attachment_validity` watched the direction that heals; the never-healing direction (attacher leaves,
  host keeps the dead id in a HASHED field) had no check and is now fixed one-directionally (CR 400.7f).
- Engine +82/−20 in four files; HASH 85 / PROTOCOL 44 unmoved; coverage unmoved; tests 5,287 → **5,316** (+29).
- Eleven gates defeated by execution and re-keyed (8 bypass pass + 3 `/review`; 15 findings, all taken).
- Notes: `memory/primitives/pb-DX56-execution-notes.md`; census `pb-DX56-mechanism-census.md`.

## 2026-09-05 — PB-DX55 (`scutemob-234`, merge `e0da3cc9`) — the whole bot/human refusal surface

- Filed and closed **OOS-SIM6-3**, **OOS-SIM5-3**, **OOS-SIM5-5** (none had a registry row) plus rider **OOS-DX51-3**. Filed OOS-DX55-1..10.
- Bot refusal surface **70 → 9**, every survivor one parked class (OOS-SIM5-4); PB-DX32 gate-config rejection rate 1.843‰ → **0**, pinned.
- Auto-tap now funds activations, not only casts; `command_mana_cost` is an exhaustive 45-arm match with no wildcard.
- Two hand-rolled block predicates inside `handle_declare_blockers` collapsed into `check_block_pair`
  (`combat.rs` net −131 lines); `LegalAction::DeclareBlockers` now carries per-attacker legal blocks.
- Browser half proven by a real `POST /api/game/action` drive (the 422 reproduces under revert).
- HASH 85 / PROTOCOL 44 unmoved; coverage unmoved; tests 5,243 → **5,287** (+44); `npm run build` NOT run (OOS-DX55-7).
- Notes: `memory/primitives/pb-DX55-execution-notes.md`. Revert harness itself was wrong first (OOS-DX55-4, mtime).
