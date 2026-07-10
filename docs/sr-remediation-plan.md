# SR Remediation Track — Operations Guide

<!-- last_updated: 2026-07-10 -->

> **Audience:** any session (human-driven or agent) working the SR-prefixed tasks.
> This track runs **outside** the `/start` / `/eot` skills and outside the W1–W6
> workstream system. Follow this doc instead. Do not modify
> `memory/workstream-state.md` — handoffs live in the Session Log at the bottom
> of this file.
>
> **Entry point:** run `/remedy` — it reads this doc, handles in-flight SR work,
> selects the next task per the sequencing below, and dispatches a worker
> (coordinator mode; `.claude/skills/remedy/SKILL.md`). The manual protocol
> below is for doing an SR task inline yourself without dispatching; the
> collision rules, verification gates, gotchas, and bookkeeping apply to both
> modes.

## Background

On 2026-07-10 a full senior-Rust-engineer review of the project was performed
(architecture, dispatch, card DSL, test infrastructure, plus direct measurement
of CI history and compile times). Ten remediation tasks were created in ESM as
`scutemob-53` … `scutemob-62`, titled `SR-1` … `SR-10`. The unifying theme of
the findings: the project's guarantees are increasingly maintained by **process**
(worker discipline, review passes, memory files) rather than by **machine**
(CI, registry gates, exhaustive matches, cross-regime tests). Each SR task
converts a process guarantee into a machine guarantee.

Full evidence (file:line) is in each task's ESM description — run
`esm task get scutemob-<N>` before starting one. This doc does not repeat it.

## Task inventory and sequencing

| Order | ESM ID | Task | Size | Notes |
|-------|--------|------|------|-------|
| 1 | scutemob-53 | SR-1: Revive CI | S | **DONE 2026-07-10.** CI green; every later task now has a machine gate. |
| 2 | scutemob-54 | SR-2: Registry gate (invariant #9) | M | **DONE 2026-07-10.** Superseded archived scutemob-48. Card-authoring waves unblocked. Follow-up: `scutemob-64` (SR-12). |
| 3 | scutemob-55 | SR-3: Seal GameState | M–L | **DONE 2026-07-10.** Invariant #3 machine-enforced. CI gained `cargo build --workspace` — do not drop it. |
| 4 | scutemob-56 | SR-4: Silent-failure sweep | M–L | Mechanical but large; classification work. Can run any time after SR-1. |
| 5 | scutemob-57 | SR-5: KeywordAbility catch-all audit | M | Pairs naturally with SR-4 (same files). |
| 6 | scutemob-58 | SR-6: Extract card-defs crate | M | Wide blast radius. Coordinate with card authoring (collision rules). |
| 7 | scutemob-59 | SR-7: PendingTrigger → TriggerData cutover | M | Requires HASH_SCHEMA_VERSION bump; read `state/hash.rs` header first. |
| 8 | scutemob-60 | SR-8: Protocol versioning policy | M | Hard blocker before M10's first networked client. Design + implement. |
| 9 | scutemob-61 | SR-9: Test infra consolidation | L | Three sub-items (binaries / equivalence test / script triage). **Split into 2–3 ESM subtasks at dispatch time.** |
| 10 | scutemob-62 | SR-10: Dependency & lint hygiene | S–M | Four independent chores; safe filler work between larger tasks. |
| 11 | scutemob-63 | SR-11: Pin the Rust toolchain | S | Discovered during SR-1. CI floats to newest stable; new lints redden CI with no commit, and the local clippy gate can't reproduce them. Pairs well with SR-10. |
| 12 | scutemob-64 | SR-12: Unbypassable invariant-9 gate + marker anti-rot | M | Discovered during SR-2 review. `GameStateBuilder`/`start_game` skip `validate_deck`; only the Inert marker class has a rot guard. |

Order is a recommendation, not a dependency chain. Hard constraints only:

- **SR-1 first.** It is small and gates everything else.
- **SR-2 before resuming card-authoring waves** (each wave authored pre-gate adds ungated inventory).
- **SR-8 before M10 networking work begins.**
- SR-4 and SR-5 touch the same six rules files — do them adjacently (or as one
  session pair) to avoid re-learning the terrain twice.

## Session protocol (replaces /start and /eot for this track)

### Starting a session

1. `cd ~/projects/scutemob` and confirm `git status` is clean on `main`,
   `git pull` current.
2. **Collision check (mandatory):** read `memory/workstream-state.md`
   *read-only*. If a W6 card-authoring or other worker session is active:
   - SR-3, SR-6, SR-7 (wide-blast-radius refactors): **do not start** — pick
     another SR task or wait.
   - All other SR tasks: proceed, but stay out of `cards/defs/`.
3. Start an ESM session with the dedicated agent id so this track is
   distinguishable from `primary` and worker fleets:
   ```
   esm session start --project scutemob --agent sr-worker
   ```
   (Heartbeats: sessions auto-end after 10 min idle; long builds are fine,
   just re-heartbeat via any esm call if a session seems stale.)
4. Pick the task, read it fully: `esm task get scutemob-<N>`.
5. Load context: `memory/conventions.md`, plus `memory/gotchas-rules.md` if
   touching `rules/`, `memory/gotchas-infra.md` if touching `state/`, `cards/`,
   `effects/`, or tests.
6. Create the branch and claim the task with required attestations:
   ```
   git checkout -b sr/<N>-<slug>          # e.g. sr/1-revive-ci
   esm task transition scutemob-<id> in_progress --agent sr-worker \
     --attest branch_exists=true \
     --attest acceptance_criteria_defined=true \
     --attest working_branch=sr/<N>-<slug>
   ```

### During the session

- **Commit prefix:** `SR-<N>:` (e.g. `SR-1: fix CI branch filters master→main`).
  This track is not in the W-prefix table in CLAUDE.md; `SR-<N>:` is its
  convention.
- One SR task per branch. If a task reveals a separate problem, create a new
  ESM task (`SR-` prefix, mention discovery source) rather than expanding scope.
- Task comments are short status lines per project convention:
  `Completed: X. Next: Y.` / `Blocked: X. Tried: Y.`

### Verification gates (all must pass before in_review)

```
~/.cargo/bin/cargo test --all
~/.cargo/bin/cargo clippy --all-targets -- -D warnings
~/.cargo/bin/cargo fmt --all -- --check
~/.cargo/bin/cargo build --workspace     # catches TUI/replay-viewer exhaustive-match breaks
```

Once SR-1 lands, a green CI run on the pushed branch/main is an additional gate.

Task-specific extras:
- **SR-7 (and anything changing serialized state shape):** bump
  `HASH_SCHEMA_VERSION` per the checklist in `crates/engine/src/state/hash.rs`
  header comment; hash tests assert the expected value.
- **SR-3 / SR-6:** run the replay-viewer and TUI builds explicitly; they are the
  consumers most likely to break.
- **SR-10 rand upgrade:** dual-instance determinism and state-hashing tests are
  the regression canary — run `state_hashing` tests specifically.

### Finishing a task

1. Satisfy each acceptance criterion explicitly:
   ```
   esm task satisfy scutemob-<id> <criterion_id> --by sr-worker
   ```
   (Criterion IDs are in `esm task get` output. Do not skip this — signaling
   ready with 0 criteria satisfied is a known failure mode.)
2. Transition with attestations:
   ```
   esm task transition scutemob-<id> in_review --agent sr-worker \
     --attest tests_passing=true --attest implementation_complete=true
   ```
3. Optionally run `/review` (Opus reviewer against acceptance criteria) for the
   larger tasks (SR-3, SR-4, SR-6, SR-7, SR-8). Small ones (SR-1, SR-10 items)
   may self-review.
4. Merge to main, delete the branch, then:
   ```
   esm task transition scutemob-<id> done --agent sr-worker \
     --attest review_complete=true
   ```
5. **End-of-session bookkeeping (replaces /eot):**
   - Append one entry to the Session Log below (date, task, outcome, hazards
     discovered, next-session pointer).
   - Update this doc's inventory table if sequencing knowledge changed.
   - Update CLAUDE.md "Current State" **only** when an SR task materially
     changes the snapshot (e.g. SR-1 makes CI live; SR-2 changes card-def
     counts/gating; SR-6 changes crate layout). Routine SR progress does not
     belong in CLAUDE.md.
   - `esm session end` (or let the 10-min idle timeout close it).
   - Do **not** rotate `memory/workstream-state.md` — that file belongs to the
     W-workstream sessions.

## Gotchas inherited from the review (read before the relevant task)

- **SR-1: DONE (2026-07-10).** CI is live and green. The predicted first-run
  surprises were real but not the ones predicted — no missing system libs. What
  actually bit, in order:
  1. **Toolchain float.** `dtolnay/rust-toolchain@stable` resolves to the newest
     stable (1.97.0 on 2026-07-07); this dev box is on 1.95.0. Six clippy
     findings across 3 files were invisible locally. **A clean local
     `cargo clippy -- -D warnings` does not mean CI is green.** To reproduce CI
     exactly: `rustup toolchain install 1.97.0` then `cargo +1.97.0 clippy
     --all-targets -- -D warnings`. Filed as `scutemob-63` (SR-11) — pin the
     toolchain.
  2. **Disk exhaustion.** `cargo test --all` links ~300 test binaries; with
     debuginfo, `target/` is 68 GB and overruns the runner's 89 GB. It fails as
     `ld terminated with signal 7 [Bus error]` + an LLVM "file a bug" banner;
     the true cause is one line earlier, `No space left on device (os error 28)`.
     Fixed with `CARGO_PROFILE_{DEV,TEST}_DEBUG=0` (68 GB → 5.0 GB) plus a
     free-disk-space step. **Any future CI job that adds a build must keep
     debuginfo off**, or it will resurrect this in a form that looks like a
     compiler bug.
  3. `gh workflow run` immediately after `git push` can dispatch against the
     *previous* SHA — a run that silently re-tests old code. Poll
     `gh api repos/<o>/<r>/commits/<branch> --jq .sha` until it matches local
     `HEAD` before dispatching, and always check the run's `headSha`.

  The fallback (`cargo test -p mtg-engine`) was not needed: the full
  `cargo test --all` runs green, 3090 passed / 0 failed, ~13 min wall clock.
- **SR-1 scope cap (user direction, 2026-07-10): keep CI cheap.** One Ubuntu
  job, fmt + clippy + tests, nothing more. No OS matrix, no nightly benchmark
  runs, no Tauri builds — long/expensive Actions are worthless this far from
  playable alpha; revisit the full matrix around M10/M11. Related drift to
  reconcile: `.claude/CLAUDE.local.md` describes CI as Ubuntu/Windows/macOS
  with nightly benchmark regression alerts — that is aspirational, not real
  (actual workflow is a single Ubuntu job). Correct that doc to describe the
  minimal CI as the intended current state, noting the matrix as an M10/M11
  follow-up.
- **SR-2: DONE (2026-07-10).** The marker (`CardDefinition.completeness`) is the single
  source of truth; `tools/authoring-report.py` derives its empty/todo/clean buckets from it
  and reports marker-vs-comment drift. See the SR-2 session-log entry below for the three
  hazards it surfaced — chiefly that **you cannot detect `abilities: vec![]` with a regex**
  (it matches `mana_abilities: vec![]` and back faces), which had corrupted the authoring
  report's headline number for the whole campaign.
- **SR-3: DONE (2026-07-10).** The predicted harness problem was a non-problem:
  `replay_harness.rs` is *inside* the engine crate, so `pub(crate)` fields remain
  visible to it, and its only public state-producing function returns an owned
  `GameState` by value rather than lending `&mut`. It is already a constructor.
  What actually mattered, in order:
  1. **Sealing the fields is not the seal.** `player_mut`, `object_mut`,
     `zone_mut`, `add_object`, `move_object_to_zone`,
     `move_object_to_bottom_of_zone`, `next_object_id` and `next_replacement_id`
     were all `pub` and all hand out mutable access. Sealing only the fields
     leaves invariant #3 fully bypassable. Zero production consumers used any of
     them; all are now `pub(crate)`. **Before sealing a struct, enumerate its
     `pub fn`s that take `&mut self`, not just its fields.**
  2. **`cargo test --all` and `cargo clippy --all-targets` cannot prove a seal.**
     Both build dev-dependencies, and cargo unifies features across the
     workspace, so the engine's `test-util` feature is ON for *every* crate under
     those commands. Only `cargo build --workspace` (no dev-deps) can catch a
     production consumer using an escape hatch. It is now a CI step; do not drop
     it. Corollary: a `compile_fail` doctest can guard the *fields* (they are
     `pub(crate)` in every profile) but can never guard the *hatches*.
  3. **Accessors lose disjoint field borrows.** `state.objects.get_mut(&id)`
     alongside `state.card_registry` is legal — rustc borrows fields
     independently — but `state.objects_mut()` borrows all of `state`. One site
     (`tests/morph.rs`) had to hoist an `Arc::clone` of the registry above the
     mutable borrow. Expect a handful of these in any future sealing work.
  4. Mechanical migration (~2 000 sites across 287 files) is safe if it is
     **compiler-driven**, not regex-driven: a syntactic first pass guesses
     read-vs-write, then a loop over `cargo --message-format=json` spans corrects
     each guess (E0616 → add `()`; E0599 → the receiver was not a `GameState`,
     revert; E0594/E0596 → this site really mutates, use `_mut()`). Two bugs in
     the first-pass regex (an over-eager `&x.field()` strip, and multi-line
     method chains) were caught only because the tree was reset and redone rather
     than patched forward. Verify no *other* type in the workspace has a method
     named like a sealed field first, or a wrong rewrite compiles silently.
- **SR-4/SR-5:** "expected fizzle" classifications must cite the CR rule
  (608.2b etc.) per project test convention; use the mtg-rules MCP server, and
  remember CR text is authoritative over card rulings.
- **SR-6:** the defs import `crate::cards::helpers::*` — the DSL types and
  helpers prelude must move (or re-export) cleanly for defs to compile in the
  new crate. `build.rs` moves with the defs.
- **SR-9(b):** the equivalence test's whole point is that
  `enrich_spec_from_def` shadow-implements object construction — if hashes
  diverge, the harness is wrong until proven otherwise, not the engine.

## Session Log

_One entry per session, newest first. Format:_
`- YYYY-MM-DD — SR-<N> (scutemob-<id>) — <status: done / in progress / blocked> — <one-line outcome + hazards + pointer for next session>`

- 2026-07-10 — SR-3 (scutemob-55) — **done** — Invariant #3 is now a machine gate.
  All 38 `GameState` fields are `pub(crate)`, with one public read accessor each
  (by-ref for containers, by-value for `Copy` scalars). The eight `pub` methods that
  handed out mutable access are `pub(crate)` too — sealing the fields alone would have
  left the bypass wide open, and no production consumer used any of them. `zone_mut` had
  zero callers repo-wide once it stopped being `pub`; deleted. Mutable access now exists
  only in `state::test_util` (free functions, so `rg 'test_util::'` enumerates every use)
  plus `*_mut()` accessors, both gated on `cfg(any(test, feature = "test-util"))`; the
  engine enables the feature for its own tests/benches via a self dev-dependency.
  Migrated 287 files (tests, benches, simulator, tui, replay-viewer). 3106 tests pass
  (3104 baseline + 2 new doctests); all four gates clean.
  **Hazards discovered:** all four written up in the SR-3 gotcha above — chiefly
  (a) sealing fields without sealing `&mut self` methods proves nothing, and
  (b) **`cargo test --all` / `cargo clippy --all-targets` cannot detect a broken seal**,
  because dev-dependencies + feature unification turn `test-util` on workspace-wide.
  Only `cargo build --workspace` can. It was missing from CI, so the consumer-side
  guarantee was still process-only until this task added it — caught by `/review`, not
  by me. Also: accessors cost you rustc's disjoint field borrows (one `Arc::clone` hoist
  in `tests/morph.rs`).
  **Deliberately not closed:** `GameStateBuilder`, `#[derive(Deserialize)]` and
  `replay_harness::build_initial_state` still let a caller *construct* an arbitrary owned
  state. None can mutate a live one behind the command log's back, which is the invariant
  that rewind/replay actually depends on; documented on `GameState` rather than papered over.
  **Next session:** SR-4/SR-5 as a pair (same six rules files), or SR-11 (`scutemob-63`,
  pin the toolchain) as cheap filler — SR-3 added a CI step, so a toolchain float that
  reddens CI now costs one more build.

- 2026-07-10 — SR-2 (scutemob-54) — **done** — Invariant #9 is now a machine gate.
  `Completeness` on `CardDefinition` (`Default` = `Complete`, so an unmarked def is
  playable); `Inert` / `Partial` / `KnownWrong` each carry a note, and `validate_deck`
  rejects them with `DeckViolation::IncompleteCard`, which `Display`s the card name and
  the defect. `CardRegistry::try_new` returns `RegistryError::DuplicateCardId`; `new`
  panics. Sweep marked 851 defs: 68 inert, 627 partial, 47 known-wrong. 3104 tests pass
  (was 3090); all four gates clean.
  **Hazards discovered:**
  (a) **`abilities: vec![]` cannot be detected by regex.** `re.search(r"abilities:\s*vec!\[\s*\]")`
  also matches a nested `mana_abilities: vec![]` and a back face's empty ability list.
  `tools/authoring-report.py` had this bug, so 51 fully-implemented cards were filed under
  "empty" and the headline clean number was wrong: it is **1,006 / 1,748 (57.6%)**, not the
  983 / 56.2% quoted in CLAUDE.md before this task. Any future scan of the defs must parse
  the top-level field (brace-depth), not grep. Same trap for `oracle_text`: a meld def
  (Hanweir) has an empty top-level `oracle_text` and a populated back face.
  (b) **Grep spelling silently narrows a hand-curated list.** The first KnownWrong pass
  searched `modelled as` (double-l) and so never even considered the defs that wrote
  `modeled as` / `approximated` — 28 of them, including one (`ingenious_prodigy`) whose
  own comment says `DEVIATION:`. The review caught it. When curating from a grep, print the
  candidate count and sanity-check it against an independent count.
  (c) A def's *first* comment block often describes a deviation that was later **removed**
  (`hazorets_monument`, `reforge_the_soul`, `ingenious_prodigy` line 9). Read the code, not
  the comment, before marking — and read past the first block.
  **Deferred:** `scutemob-64` (SR-12) — the gate binds only where `validate_deck` is called
  (`GameStateBuilder` / `start_game` / simulator bypass it), and only the Inert class has an
  anti-rot test. Both were raised by the review and are pre-existing in kind.
  **Next session:** SR-3 (`scutemob-55`, seal GameState) or SR-4/SR-5 as a pair. Card
  authoring waves are now unblocked — SR-2 was their gate.

- 2026-07-10 — SR-1 (scutemob-53) — **done** — CI revived and green for the first
  time in project history (run `29075466877`: fmt + clippy + 3090 tests, 0 failed).
  `ci.yml` now triggers on `main` (push + PR) and `workflow_dispatch`; raw
  `actions/cache` replaced with `Swatinem/rust-cache@v2`; `timeout-minutes: 45`;
  concurrency group with `cancel-in-progress`. Getting there required fixing 6
  real clippy findings only visible on CI's rustc 1.97.0 (local is 1.95.0) and
  a disk-exhaustion failure that masquerades as an LLVM/linker crash — both
  written up in the SR-1 gotcha above; read it before touching CI.
  **Hazards discovered:** (a) the local clippy gate is only as authoritative as
  the local toolchain — filed `scutemob-63` (SR-11) to pin it, and until that
  lands, treat a green local clippy as necessary but not sufficient; (b) the
  `git push` → `gh workflow run` race can test a stale SHA.
  **Note for the collector:** AC 4432 asks for a green run *on main via push*.
  A worker cannot push to main, so the green run above was `workflow_dispatch`
  on the feature branch at commit `d5b023ae`. Any commits after that are
  docs-only — verify with `git diff d5b023ae..HEAD -- .github/`, which is empty.
  The push-to-`main` trigger is what merging this branch exercises — confirm
  that run goes green at collection. *(Collector 2026-07-10: confirmed — merge
  `e9742dc2` push run `29076083859` on main completed green.)*
  **Next session:** SR-2 (`scutemob-54`, registry gate) — it now has a working
  machine gate behind it, which was the entire point of doing SR-1 first.
