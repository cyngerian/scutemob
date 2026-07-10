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
| 4 | scutemob-56 | SR-4: Silent-failure sweep | M–L | **DONE 2026-07-10.** 398 sites classified; `state::diagnostics` vocabulary added. Follow-ups: `scutemob-65` (SR-13), `scutemob-66` (SR-14). |
| 5 | scutemob-57 | SR-5: KeywordAbility catch-all audit | M | **DONE 2026-07-10.** Premise was a misattribution — see the gotcha. `state::keyword_registry` is the compile gate. Follow-up: `scutemob-67` (SR-15). |
| 6 | scutemob-58 | SR-6: Extract card-defs crate | M | **DONE 2026-07-10.** Three crates now: `card-types` ← `card-defs` ← `engine`. Engine edits leave the defs `Fresh`. Card-authoring paths moved — see the gotcha. |
| 7 | scutemob-59 | SR-7: PendingTrigger → TriggerData cutover | M | **DONE 2026-07-10.** `HASH_SCHEMA_VERSION` 36 → 37. `PendingTrigger::blank` is now the only way to build one, enforced by `tests/pending_trigger_shape.rs`. |
| 8 | scutemob-60 | SR-8: Protocol versioning policy | M | **DONE 2026-07-10.** Strict lockstep; `Envelope`/`PROTOCOL_VERSION` in `rules/protocol.rs`. The M10 blocker is cleared. Policy: `docs/mtg-engine-protocol-versioning.md`. |
| 9 | scutemob-61 | SR-9: Test infra consolidation | L | **SPLIT 2026-07-10** into `scutemob-69` (SR-9a binaries), `scutemob-70` (SR-9b equivalence test), `scutemob-71` (SR-9c script triage). Umbrella closes when all three land. |
| 9a | scutemob-69 | SR-9a: Consolidate 291 integration-test binaries | M | Sub-item (a) of SR-9. 291 links dominate test-build time (46s warm). |
| 9b | scutemob-70 | SR-9b: Harness-vs-direct equivalence property test | M | Sub-item (b) of SR-9. Same scenario, same final hash. Gotcha SR-9(b) below applies. |
| 9c | scutemob-71 | SR-9c: Golden-script corpus triage | M | Sub-item (c) of SR-9. 95/271 approved; pending_review silently skipped. |
| 10 | scutemob-62 | SR-10: Dependency & lint hygiene | S–M | Four independent chores; safe filler work between larger tasks. |
| 11 | scutemob-63 | SR-11: Pin the Rust toolchain | S | Discovered during SR-1. CI floats to newest stable; new lints redden CI with no commit, and the local clippy gate can't reproduce them. Pairs well with SR-10. |
| 12 | scutemob-64 | SR-12: Unbypassable invariant-9 gate + marker anti-rot | M | Discovered during SR-2 review. `GameStateBuilder`/`start_game` skip `validate_deck`; only the Inert marker class has a rot guard. |
| 13 | scutemob-65 | SR-13: Damage-source characteristics must use LKI | M | Discovered during SR-4. Wither/infect "function no matter what zone" (CR 702.80c/702.90e) but the engine reads the source through a live lookup and treats a dead source as having neither. Real bug, not an assert. |
| 14 | scutemob-66 | SR-14: Extend the SR-4 diagnostics vocabulary to the rest of `rules/` | M | Discovered during SR-4, which scoped to its two named files. ~200 unswept `calculate_characteristics` sites; method is written up in `docs/sr-4-silent-failure-audit.md`. |
| 15 | scutemob-67 | SR-15: Catch-all audit for the *other* dispatch enums | M | Discovered during SR-5. The ~117 catch-alls SR-5 was sent to find are real but sit on `AbilityDefinition` (20), `ZoneId` (19), `ZoneChangeAction` (17), … — `AbilityDefinition` is a genuine dispatch table. Registry pattern from SR-5 transfers directly. |
| 16 | scutemob-68 | SR-16: `PendingTrigger` serde round-trip drops `kind`/`data`/`embedded_effect` | S–M | Discovered during SR-7. Three `#[serde(skip)]` fields mean a serialized pending keyword trigger deserializes as an anonymous `Normal` trigger with no payload. Harmless today; load-bearing for M10 state sync and for rewind/replay. |

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
- **SR-4: DONE (2026-07-10).** 398 sites classified; full record in
  `docs/sr-4-silent-failure-audit.md`. Read that before SR-5 or SR-14 — it is the
  method, not just the result. What mattered:
  1. **Establish the ground truths before classifying anything.** Three facts turned a
     400-site judgment call into mostly mechanical work: `GameState::players` never
     loses entries (so *every* missing `PlayerId` is a bug — 32 sites decided at once);
     `GameState::zones` never shrinks (8 more); and `calculate_characteristics` returns
     `None` **iff** the ObjectId is absent, every other step being total (so its
     `unwrap_or_*` fallbacks are either dead code or the fizzle branch, never a
     rounded-off partial result). Only `state.objects` genuinely loses entries.
  2. **The classification belongs in the code, not a comment.** `expect_object` vs
     `lki_object` (both returning `Option`) makes each site a one-token change, and the
     `debug_assert!` is then machine-checked by the existing suite. A comment saying
     "this can't be None" rots; a `debug_assert!` that fires does not.
  3. **`move_object_to_zone` has four error variants, not two** — `ObjectNotFound`,
     `ZoneNotFound(to)`, `ZoneNotFound(from)`, `ObjectNotInZone`. Only the first can be
     a legal fizzle. `Err(_) => {}` was collapsing all four. Splitting them is safe only
     because a stale id is removed from `state.objects` entirely, so LKI always surfaces
     as `ObjectNotFound`; that assumption now has a tripwire test. **Read the error enum,
     don't trust the `Err(_)` arm's comment.**
  4. **Believe the code over the classifier.** Three sites a classifier called
     IMPOSSIBLE had existing comments saying "card disappeared — nothing to cast." The
     comments were right. When the two disagree, the demotion to FIZZLE is free and the
     promotion to IMPOSSIBLE is a panic in a real game.
  5. Bias uncertain calls toward FIZZLE and let `cargo test --all` adjudicate: 31 of the
     222 IMPOSSIBLE verdicts were `med` confidence, and the suite ran every one.
  6. **The SR-3 disjoint-borrow hazard recurs immediately.** `state.expect_object_mut(id)`
     borrows all of `GameState`; `state.objects.get_mut(&id)` borrows one field. Five
     blocks needed `card_registry` / `timestamp_counter` alongside. Hence the
     `debug_assert_object_live!` macro — the assert without the borrow. Expect this in
     any future accessor migration.
- **SR-5: DONE (2026-07-10).** Full record in `docs/sr-5-keyword-catchall-audit.md`.
  What mattered:
  1. **The task's premise was wrong, and implementing it literally would have been a
     large useless diff.** "~117 `_ => {}` catch-alls on `KeywordAbility`" — the count
     is right, the enum is not. Group the catch-alls by the enum their arms *name* and
     only **2** are on `KeywordAbility`; the rest are on `AbilityDefinition` (20),
     `ZoneId` (19), `ZoneChangeAction` (17), … Filed as `scutemob-67` (SR-15).
     **Check a task's premise with a script before you build the fix it implies.**
  2. **The real hazard had no syntax to grep for.** A keyword is read via
     `keywords.contains(&KeywordAbility::Flying)` — ~350 reads, no `match`, no arm, no
     wildcard. A new variant isn't *swallowed*; it is simply never mentioned. All 13
     of the actual `KeywordAbility` catch-alls are `filter_map` **projections**
     (`Fabricate(n) => Some(n), _ => None`) where the wildcard is the correct answer.
     None were changed.
  3. **`hash.rs` was already an exhaustive compile gate** (166 arms), as is the replay
     viewer's `view_model.rs`. So "adding a variant fails to compile" was *already*
     true before this task — but it only forces you to assign a hash byte and a
     display string. Neither is behavior. The registry forces a behavior
     classification, which is the thing the acceptance criterion was actually after.
  4. **Every derived set needs a non-vacuity guard, and this is not theoretical.** On
     the first run `declared_variants()` had a broken state machine and returned zero
     variants — and `registry_sites_match_the_source_tree` reported **pass**, because
     it was comparing empty against empty for all 166 keywords. Only the two
     `*_is_not_vacuous` guards failed. Assert the denominator, always.
  5. **Demonstrate a gate adversarially, not existentially.** Not "a new variant fails
     to compile" but "here are the four cheapest ways to make it compile and pass, and
     here is what catches each." Four stages, four different failures.
  6. Original brief (still binding for any keyword classed as needing no dispatch):
     "expected fizzle" / "no dispatch needed" claims cite the CR rule via the mtg-rules
     MCP server; CR text is authoritative over card rulings. All 18 marker keywords'
     citations were verified against rule text and matched their `types.rs` doc
     comments.
- **SR-6: DONE (2026-07-10).** The briefed hazard (defs import
  `crate::cards::helpers::*`; the prelude must move or re-export cleanly) was real and
  took about ten minutes. What actually mattered:
  1. **The task's own justifying document specified a layout that cannot work.**
     `docs/mtg-engine-card-pipeline.md` had `card-defs` depending on `engine` "for the
     DSL types". A crate rebuilds whenever a dependency changes, so defs stacked *above*
     the engine would recompile on every rules edit — the exact cost the split exists to
     remove. The DSL had to move *below* the engine into a third crate (`card-types`).
     **The arrow direction is the entire mechanism; the crate count is not.** Second SR
     task running whose brief was wrong in a load-bearing way (cf. SR-5's misattributed
     enum) — check the premise before building what it implies.
  2. **The closure is what makes it possible, so measure it first.** `helpers.rs` reaches
     into 11 `state/` modules. All 11 turned out to be pure data: zero `GameState`
     references, zero `pub(crate)` members, zero inherent impls outside their own files.
     Three greps decided the whole design. Had even one held a `GameState` method, the
     split would have needed a trait or a much bigger cut. (`hash.rs` impls `StateHash for
     <moved type>` ~100 times — not an orphan violation, because `StateHash` is local.)
  3. **Module re-exports made a 287-file refactor a 2-file refactor.** `pub use
     mtg_card_types::state::{game_object, types, …};` in the engine's `state/mod.rs` keeps
     every `crate::state::game_object::X` path resolving. `rules/` and `effects/` were not
     touched at all. Symmetrically, `pub(crate) use mtg_card_types::{cards, state};` in
     `card-defs/src/lib.rs` keeps `use crate::cards::helpers::*;` resolving, so **all 1,749
     def files moved with zero content edits** (git records pure renames).
  4. **`all_cards()` was not the only consumer of `defs`.** 14 test files reach individual
     card modules as `cards::defs::hardened_scales::card()`. Re-export the whole `defs`
     module, not just the collector. A `grep 'defs::' src/` misses this; it only shows up
     in `tests/`.
  5. **A file move silently breaks every non-compiled path reference.** The compiler found
     nothing here — but `include_str!` targets, the SR-5 registry's declared site strings,
     four Python tools, the TUI dashboard parser, seven `.claude/agents/` prompts and three
     skills all named the old directory. **The write-targets are the dangerous ones**: an
     authoring agent told to create `crates/engine/src/cards/defs/<slug>.rs` gets no error;
     `build.rs` simply never sees the card. Grep for the old path across *every* file type,
     not just `*.rs`, and sort the hits into live instructions (fix) vs historical records
     (leave — rewriting a PB review falsifies it).
  6. **`git mv` breaks `git log -- <path>` and therefore any tool that mines history.**
     `tools/authoring-report.py` attributes each card to its earliest add commit. Post-move,
     **all 1,748** were attributed to the SR-6 commit; the activity table read
     "1,749 cards added this week." Fixed with `--follow` (per-file, single pathspec) and
     with `-M` plus *both* pathspecs (windowed queries), so the move reads as renames rather
     than additions. Verify by re-running the tool **after** committing the move — before the
     commit it reports zeros and looks like a different bug.
  7. **SR-5's registry needed re-anchoring, not just repointing.** Its site paths were
     crate-relative, and `src/state/dungeon.rs` stopped naming a unique file the moment a
     second crate had a `src/state/`. Paths are now workspace-relative and the scan spans
     `engine` + `card-types`; `card-defs` is excluded by being outside every scan root
     (stronger than the old path filter). Demonstrated adversarially, per SR-5's own lesson:
     dropping the `card-types` scan root, admitting `card-defs`, and staling one declared
     site each fail the suite — and the per-root non-vacuity assertion is what catches the
     first. A cross-crate derived set needs a *per-root* denominator guard, not just a
     total-count guard.
- **SR-7: DONE (2026-07-10).** The briefed hazard (a bump of `HASH_SCHEMA_VERSION`, per the
  `state/hash.rs` header) was real and cost about five minutes. What actually mattered:
  1. **The migration was already finished; only the corpse remained.** All 13 legacy
     fields were `None` at **every one of their 33 construction sites** and were **read
     nowhere** outside `hash.rs` — `grep -rn '\.poisonous_n\b'` returns zero. The payloads
     had long since moved into `PendingTrigger.data`, which `flush_pending_triggers` reads
     and threads into `StackObjectKind::KeywordTrigger { keyword, data }`. So the task was
     a pure deletion with **zero behavior change**, and the risk was never "will the
     trigger still fire" but "will I delete a field that is secretly live." **Establish
     write-only-ness with a read-grep before touching anything** — it converts a scary
     refactor into a mechanical one, exactly as SR-4's ground-truth pass did.
  2. **The copy-paste literal is what kept the corpse warm.** 32 sites hand-spelled all 28
     fields, so every field added since the `blank()` helper existed propagated itself into
     32 more `None`s, and no field could ever be removed by a compiler error. That is why a
     "mid-migration" state persisted across many PBs. The 32 sites are now
     `..PendingTrigger::blank(source, controller, kind)` (nine of them collapsed to a bare
     `blank(..)` call because they overrode nothing); the four rules files shrank by ~850
     lines. **Prefer a `blank()`-style base to an all-fields literal precisely because it
     makes deletion possible, not because it is shorter.**
  3. **Do the 32-site edit with a script, then prove nothing was lost.** A regex over
     `^\s*(field): None,$` deleted the initializers; a brace-depth parser rewrote the
     literals. The safety net was a second script that re-read the *pre-change* files,
     extracted every field whose value differed from `blank()`'s default, and asserted each
     such value still appears in the post-change file. Compiling proves the code is
     well-formed, not that an override survived. (Two mechanical defects the compiler *did*
     catch: `impl PendingTrigger {` and `-> PendingTrigger {` both look like literals to a
     naive `PendingTrigger\s*\{` scan, and `ability_index: ability_index` tripped clippy's
     `redundant_field_names` at three sites.)
  4. **Deleting a resolution arm compiles with zero errors.** Verified adversarially:
     removing the entire `StackObjectKind::KeywordTrigger { keyword: Enlist, data:
     TriggerData::CombatEnlist { .. } }` arm from `resolution.rs` leaves `cargo check`
     completely green — the match has a catch-all — and the trigger silently resolves as a
     no-op. This is SR-5's hazard one enum over, and it is the reason
     `replacement_trigger_data_variants_are_still_consumed` exists: the *justification* for
     deleting the 13 fields is that their `TriggerData` variants have live consumers, so
     that justification is now itself a test. (Renaming a variant, by contrast, *is* a
     compile error — so the gate must be phrased against deletion, not against renaming.)
  5. **Only 11 of the 13 fields were hashed.** `haunt_source_object_id` and
     `haunt_source_card_id` were never fed to the hasher — a pre-existing determinism hole
     that was harmless only because both were always `None`. Deleting them made it moot.
     **When you remove fields from a `HashInto` impl, diff the impl against the struct
     rather than assuming they agree**; nothing enforces that they do.
  6. Per SR-5's lesson, all four new gates were demonstrated adversarially rather than
     existentially — re-add a field, hand-roll a literal, smuggle a literal into the one
     excluded file, blind the scanner, orphan a consumer. Five attacks, five distinct
     failures. The scanner-blinding attack is the one that matters: it fires the
     non-vacuity guard (`checked >= 30`), without which an absence-shaped assertion passes
     forever.
  7. **Left deliberately open:** `PendingTrigger` derives `Serialize`/`Deserialize`, but
     `kind`, `data` and `embedded_effect` are all `#[serde(skip)]`. A serialized-then-
     deserialized `GameState` therefore loses every keyword trigger's identity *and*
     payload, silently. Harmless today (triggers are flushed before priority, so a
     serialized state never carries a pending one in practice), but it is a live trap for
     M10 networking and for rewind/replay. Filed as `scutemob-68` (SR-16) rather than
     widened into this task.
- **SR-8: DONE (2026-07-10).** The brief's premise held up — unusually, for this track.
  `crates/network` really was a 4-line doc-comment stub, no replay log existed, and nothing
  in production serialized a `Command` or `GameEvent` anywhere. So SR-8 was greenfield: no
  migration, no compatibility shims, just get the policy right before M10 makes it
  expensive. What mattered:
  1. **The wire surface is 90 types, not 2.** `GameEvent::CreatureDied` carries
     `Option<Characteristics>` (added for LKI, CR 603.10a), and `Characteristics` holds
     `Vector<AbilityInstance>`, which reaches `Effect` → `TargetFilter` → **the entire card
     DSL**, across both `engine` and `card-types`. Consequence: **adding an `Effect` variant
     is a wire change**, so most PBs will bump `PROTOCOL_VERSION`. Measure the closure before
     reasoning about "the protocol"; the two enums named in the task are the roots, not the
     surface. It bottoms out cleanly — `GameState` is *not* reachable — which is the only
     reason protocol versioning and `HASH_SCHEMA_VERSION` can stay separate concerns, and
     that boundary is now asserted (`CLOSURE_MUST_NOT_CONTAIN`).
  2. **A hand-bumped version constant is the disease, not the cure.** The task as written
     ("implement the tag, define mismatch handling") would have delivered exactly the process
     guarantee this track exists to delete: a `PROTOCOL_VERSION: u32 = 1` that is correct only
     while every future author remembers it — the same shape as `HASH_SCHEMA_VERSION`, whose
     sentinel tests force you to *notice* a bump but never force you to *make* one. So the
     version is backed by `PROTOCOL_SCHEMA_FINGERPRINT`, a blake3 digest of the closure's
     normalized declaration text, recomputed from source. **When a task hands you a constant,
     ask what makes it true.**
  3. **The compiler is not the gate, and believing it is was the trap.** Adding a `GameEvent`
     variant *does* fail to compile (`hash.rs` is exhaustive). But that error only tells the
     author to assign a hash byte; once they do, the workspace builds clean, all 3 100+ other
     tests pass, and the wire has silently changed. Verified by doing it. This is SR-5's
     finding one enum over. Conversely, the three attacks `rustc` cannot see at all —
     `#[serde(skip)]`, `#[serde(rename)]`, `#[serde(rename_all)]` — are caught *only* by the
     digest, which is why container and field attributes are hashed rather than stripped.
  4. **Two under-inclusion holes surfaced while writing the guards, not after.**
     `pub type RoomIndex = usize;` is wire-bearing but is neither enum nor struct — caught by
     `every_referenced_type_resolves`, which refuses to let an unresolved type name pass.
     And `EnchantControllerConstraint`'s `#[derive(...)]` is **rustfmt-wrapped across three
     lines**, so a line-based attribute walk saw `)]`, concluded "not an attribute", and
     dropped that type's entire serde config out of the digest, silently. Caught by
     `every_closure_type_shows_its_serialize_derive`. **Write the denominator guard first; it
     finds the bug in the scanner you were about to trust.**
  5. **Stage the version check ahead of the payload parse.** The obvious one-pass
     `from_str::<Envelope<T>>` also rejects old messages — as `unknown variant 'Foo' at line 1
     column 42`. A client can act on `VersionMismatch`; it can only guess at that. The staging
     is pinned by `version_is_checked_before_the_payload_is_parsed`.
  6. **`/review` found the frame nobody was pointed at.** The closure was rooted at `Command`
     and `GameEvent` — "the two enums that *are* the protocol" — but `encode_replay_log` puts
     a **third** frame on the wire, `ReplayLog`, which nothing reachable from those two
     mentions. Its `Vec<Command>` contents were covered; its own two-field frame was not, so
     adding a field to it changed the replay-log format and tripped nothing. Sixth consecutive
     SR task where the review found a hole in the *gate*. The pattern is stable enough to
     name: **the author verifies the gate fires on the thing they were thinking about, and
     never enumerates the things the gate is not pointed at.** Also generic: `Envelope<T>`
     cannot be walked (`T` resolves to nothing), so its field names are pinned by a separate
     test — renaming `payload` to `body` was invisible to everything else.
  7. **One legitimate reason to re-pin without bumping**: widening the closure's *definition*
     (a new `SCAN_ROOTS` / `PROTOCOL_ROOTS` / `EXTERNAL_TYPES` entry) moves the digest because
     coverage grew, not because the wire did. Written down in both `protocol.rs` and the doc,
     because otherwise it is indistinguishable from cheating.
- **SR-9(b):** the equivalence test's whole point is that
  `enrich_spec_from_def` shadow-implements object construction — if hashes
  diverge, the harness is wrong until proven otherwise, not the engine.

## Session Log

_One entry per session, newest first. Format:_
`- YYYY-MM-DD — SR-<N> (scutemob-<id>) — <status: done / in progress / blocked> — <one-line outcome + hazards + pointer for next session>`

- 2026-07-10 — SR-8 (scutemob-60) — **done** — The M10 blocker is cleared: `Command` /
  `GameEvent` / replay-log streams now carry a version tag, and the version tag is itself
  machine-checked. **Policy is strict lockstep** — a message declares `protocol_version` and is
  accepted iff it equals `PROTOCOL_VERSION` exactly; older *and newer* are rejected with a typed
  `ProtocolError::VersionMismatch`. No negotiation, no forward compatibility. The rationale is
  invariant #9: a client that tolerates an unknown `GameEvent` variant holds a history it cannot
  correctly rewind and *cannot tell that it does*, so the corruption surfaces arbitrarily far
  from its cause. A refused connection is legible; a corrupted history is not. And in a
  trusted-playgroup single-server deployment all clients ship from one build anyway, so lockstep
  costs nothing and buys a loud failure. Decoding is **staged** (probe the version → reject →
  only then parse the payload), so a mismatch never surfaces as an opaque serde error.
  `ReplayLog` carries two versions and checks both: `protocol_version` answers "can I read these
  commands", `hash_schema_version` answers "can my state hashes be compared with the recorded
  ones" — passing the first does not imply passing the second.
  New: `crates/engine/src/rules/protocol.rs` (`PROTOCOL_VERSION`, `PROTOCOL_SCHEMA_FINGERPRINT`,
  `Envelope<T>`, `ProtocolError`, `ReplayLog`, `encode`/`decode`), `docs/mtg-engine-protocol-versioning.md`.
  **New gates:** `tests/protocol_schema.rs` (11) + `tests/protocol_roundtrip.rs` (17). 3162 tests
  pass (3134 baseline + 28), 316 suites. All four verification gates clean; `cargo build
  --workspace` explicitly re-run.
  **The gate is the point.** `PROTOCOL_SCHEMA_FINGERPRINT` is a blake3 digest of the normalized
  declaration text — attributes included — of the **transitive type closure** of the three wire
  frames, parsed out of `crates/{engine,card-types}/src`. Change the wire and it fails, names the
  drift, prints the new digest. Without it, `PROTOCOL_VERSION` would be exactly the hand-maintained
  constant this track exists to eliminate.
  **Hazards discovered:** all seven written up in the SR-8 gotcha above — chiefly
  (a) **the wire surface is 90 types, not 2.** `GameEvent::CreatureDied` carries
  `Option<Characteristics>` → `AbilityInstance` → `Effect` → the whole card DSL. Adding an
  `Effect` variant is a wire change, so most PBs will bump the version; that is what strict
  lockstep *means*, not gate noise. `GameState` is not in the closure, which is the whole reason
  this and `HASH_SCHEMA_VERSION` remain separate — now asserted. (b) **The compiler is not the
  gate.** Adding a `GameEvent` variant fails to compile (`hash.rs` is exhaustive), but that error
  only demands a hash byte; satisfy it and the workspace builds clean with every other test green
  while the wire has silently moved. Verified by actually doing it. The three attacks `rustc`
  cannot see at all (`serde(skip)` / `rename` / `rename_all`) are caught by the digest alone.
  (c) **Two under-inclusion holes were found by the denominator guards while those guards were
  being written**: `pub type RoomIndex = usize` is wire-bearing but is neither enum nor struct,
  and `EnchantControllerConstraint`'s `#[derive(...)]` is rustfmt-wrapped across lines, so the
  original line-based attribute walk dropped its entire serde config out of the digest, silently.
  **Demonstrated adversarially** (per SR-5's lesson), eleven attacks, eleven distinct outcomes —
  and the table records which attacks `rustc` kills *before* they reach the gate, because a demo
  that stops at a compile error measures the compiler, not the gate. Both are in the doc.
  **`/review` (Opus) returned 3/3 PASS, 0 HIGH, 1 MEDIUM, 4 LOW** — and, for the **sixth**
  consecutive SR task, every substantive finding was a hole in the *gate*, not a bug in the code.
  The MEDIUM is the one worth remembering: the closure was rooted at `Command` and `GameEvent`,
  but `encode_replay_log` puts a **third** frame on the wire, `ReplayLog`, which neither root
  reaches. Adding a field to it compiled clean and tripped nothing — in the one place the
  acceptance criteria explicitly named ("replay-log compatibility"). Now a `PROTOCOL_ROOTS` entry,
  fix verified by re-running the attack. Two LOWs became machine guards rather than comments
  (`declared_type_names_are_unique`; `no_workspace_type_shadows_an_external_type_name`), because
  each was a way to satisfy the gate *without* firing it. `Envelope<T>` is generic and cannot be
  walked, so its field names are pinned by their own test.
  **The pattern, stated:** the author verifies the gate fires on the thing they were thinking
  about, and never enumerates the things the gate is not pointed at. Six for six. Next SR author:
  before `/review`, list every serialized frame / dispatch site / derived set your gate does *not*
  cover, and justify each omission in writing.
  **Deliberately not closed:** `scutemob-68` (SR-16) — `PendingTrigger`'s `#[serde(skip)]` fields.
  `PendingTrigger` is *not* in the `Command`/`GameEvent` closure, so it is a **state-sync** bug,
  not a protocol one; SR-8's gate cannot see it and should not. M10 state sync will hit it. Also
  open by design: `im`'s serialized shape is allowlisted (`EXTERNAL_TYPES`) and a `Cargo.toml`
  bump could move the wire without moving the digest — flag it when SR-10 touches deps. And the
  staged decode's field-name probe is JSON-specific: if the roadmap's optional MessagePack upgrade
  lands, use named-field mode or move the tag into the WebSocket subprotocol string.
  **Note for the collector:** the 26 `.claude/skills/*/SKILL.md` deletions SR-7 flagged were
  *still* present in this worktree at session start. Left unstaged again. Worktree provisioning
  is dropping them; worth a look before it bites someone who runs `git add -A`.
  **Next session:** SR-9 (`scutemob-61`, test-infra consolidation — split into 2–3 ESM subtasks at
  dispatch time per the inventory table), or SR-11 (`scutemob-63`, pin the toolchain) — which SR-8
  has just made materially more valuable, and this was **measured, not assumed**. The digest hashes
  *normalized declaration text*; `normalize_ws` collapses whitespace runs, so I expected rustfmt
  churn to be absorbed. It is not: rewrapping a long field type inserts a **trailing comma**
  (`Vector<\n AbilityInstance,\n>`), which is a token, so the digest moves with no wire change.
  Confirmed by rewrapping `Characteristics::abilities` and watching the gate fire. It errs in the
  safe direction (a spurious bump, never a missed one) and `cargo fmt --check` keeps the tree
  canonical — so this can *only* fire when rustfmt's version changes, i.e. exactly the toolchain
  float SR-11 is chartered to kill. Third accepted false positive, documented in both the test
  header and the policy doc.

- 2026-07-10 — SR-7 (scutemob-59) — **done** — The `PendingTrigger` → `TriggerData` cutover
  is finished and can no longer un-finish itself. All 13 per-keyword `Option` fields deleted
  (`ingest_target_player`, `flanking_blocker_id`, `rampage_n`, `renown_n`, `poisonous_n`,
  `poisonous_target_player`, `enlist_enlisted_creature`, `recover_cost`, `recover_card`,
  `cipher_encoded_card_id`, `cipher_encoded_object_id`, `haunt_source_object_id`,
  `haunt_source_card_id`); the struct is 29 fields → 16, all of which are either identity or
  *generic* trigger context. **Zero behavior change, and this was verifiable up front:** every
  one of the 13 was `None` at all 33 construction sites and read at zero sites outside
  `hash.rs`. The payloads already travelled in `PendingTrigger.data`. `HASH_SCHEMA_VERSION`
  36 → 37 (removal-only: 11 of the 13 were hashed, so the byte stream shortens; the two
  `haunt_*` never were, a pre-existing hole made moot by deletion). 28 sentinel tests bumped.
  All 32 hand-rolled literals now build on `..PendingTrigger::blank(source, controller, kind)`
  — nine collapsed to a bare `blank(..)` call, having overridden nothing — and the four rules
  files shrank by ~850 lines net. 3134 tests pass (3129 baseline + 5 new), 314 suites. All four
  verification gates clean; `cargo build --workspace` explicitly re-run.
  **New gate:** `crates/engine/tests/pending_trigger_shape.rs` (5 tests). `PendingTrigger`'s field set is
  pinned against the struct declaration parsed out of `stubs.rs` (so re-adding a keyword field
  is a test failure that names the field and points at `TriggerData`); every
  `PendingTrigger { .. }` literal under `engine/src`, `card-types/src` and `engine/tests` must
  contain `..PendingTrigger::blank(`; `stubs.rs` — the one file excluded from that scan — is
  pinned to exactly one literal, `blank()`'s own body; and each of the 10 replacement
  `TriggerData` variants is asserted still present in *both* `abilities.rs`
  (`flush_pending_triggers`) and `resolution.rs` (`resolve_stack_object`).
  **Hazards discovered:** all seven written up in the SR-7 gotcha above — chiefly
  (a) **the copy-paste literal was the disease, not a symptom.** 32 sites spelling all 28
  fields meant every new field propagated itself into 32 more `None`s and no field could ever
  be removed by a compiler error; that is exactly how a "mid-migration" state survived many PBs.
  (b) **Deleting a `resolution.rs` arm compiles with zero errors** — verified by actually
  deleting the Enlist arm — because the `StackObjectKind` match has a catch-all. So the claim
  that justified this whole deletion ("the `TriggerData` variants have live consumers") is now
  itself a test rather than a thing I checked once. This is SR-5's hazard one enum over, and it
  confirms `scutemob-67` (SR-15) is pointed at something real. (c) The struct and its `HashInto`
  impl were **already out of sync** and nothing enforced agreement.
  **Demonstrated adversarially** (per SR-5's lesson), seven attacks, seven distinct failures:
  re-add `poisonous_n` → field-set gate; hand-roll a literal in `turn_actions.rs` → literal
  gate; smuggle a literal into the excluded `stubs.rs` → the exclusion's own pin; blind the
  scanner → the `checked >= 30` non-vacuity guard (an absence-shaped assertion passes forever
  without it); delete the Enlist resolution arm → the consumer gate; **hand-roll via `Self { .. }`
  inside `impl PendingTrigger`** → the fifth test, added after review; **a raw string containing
  an unbalanced `"` plus a fake literal** → no longer a spurious red. Renaming a variant, by
  contrast, is a compile error, so the gates are deliberately phrased against *deletion*.
  **`/review` (Opus) returned 4/4 PASS, 0 HIGH, 0 MEDIUM, 3 LOW** — and, for the fifth SR task
  running, all three LOWs were holes in the *gate*, none a bug in the code. Two were closed:
  (i) Gate 2 keys on the token `PendingTrigger`, so a `Self { .. }` literal inside an `impl`
  block was invisible — now forbidden by `no_pending_trigger_impl_block_uses_a_self_literal`,
  which pins the impl-block count at 2 so it cannot itself go vacuous; (ii)
  `strip_comments_and_strings` did not understand raw strings, so an unescaped `"` inside
  `r#"…"#` desynced quote-blanking and left a phantom literal visible (verified: the old
  stripper does surface one). The third is documented in the test rather than fixed:
  `replacement_trigger_data_variants_are_still_consumed` is string-presence, not reachability,
  so a producer-only mention would satisfy it. Making it precise means parsing match arms; the
  regression it actually catches — an arm deleted outright — is caught today.
  **Deliberately not closed:** `scutemob-68` (SR-16) — `PendingTrigger`'s `kind`, `data` and
  `embedded_effect` are all `#[serde(skip)]`, so a serialized pending keyword trigger
  deserializes as an anonymous `Normal` trigger with no payload, silently. Harmless today
  (triggers flush before priority) but a live trap for M10 state sync and for the rewind/replay
  history invariant #9 rests on. A semantics decision, not a sweep — it did not belong here.
  **Note for the collector:** 26 `.claude/skills/*/SKILL.md` files were already deleted in this
  worktree when the session started. They are ESM-provisioned, they are not mine, and I left
  them unstaged — worth checking whether worktree provisioning dropped them.
  **Next session:** SR-8 (`scutemob-60`, protocol versioning) — it is the hard blocker before
  M10, and SR-7 just moved `HASH_SCHEMA_VERSION` again, which is the closest thing the project
  has to a wire-version policy today. Or SR-11 (`scutemob-63`, pin the toolchain) as cheap
  filler.

- 2026-07-10 — SR-6 (scutemob-58) — **done** — Card definitions now compile in isolation
  from the engine. Three crates where there was one: `crates/card-types`
  (`mtg-card-types` — `cards/{card_definition,helpers,registry}.rs` plus the 11 pure-data
  `state/` modules the DSL closes over) sits at the **bottom**; `crates/card-defs`
  (`mtg-card-defs` — 1,749 def files + `build.rs` discovery) depends on card-types **only,
  never on the engine**; `crates/engine` depends on both and re-exports them. Touching
  `crates/engine/src/rules/sba.rs` and running `cargo check -p mtg-engine -v` now reports
  `Fresh mtg-card-defs`; wall clock 7s → 2–3s with `CARGO_INCREMENTAL=0`, 2–3s → 0–1s
  incremental. Control verified: touching `card-types` correctly marks the defs dirty.
  3129 tests pass — identical to baseline, 313 suites. All four gates clean; replay-viewer
  and TUI built explicitly. **Zero content edits to the 1,749 def files, and zero edits to
  `rules/` or `effects/`** — both fall out of module re-exports (`pub use
  mtg_card_types::state::{game_object, …}` in the engine; `pub(crate) use
  mtg_card_types::{cards, state}` in card-defs, so `use crate::cards::helpers::*;` still
  resolves).
  **Hazards discovered:** all seven written up in the SR-6 gotcha above — chiefly
  (a) **the pipeline doc that justified this task specified an impossible layout**
  (`card-defs` depending on `engine`), which would have delivered none of the promised
  isolation; the DSL had to go *below* the engine. Doc corrected. (b) The dependency
  **closure** (11 state modules, all pure data, no `GameState`, no `pub(crate)`, no outside
  inherent impls) is what made the cut viable, and three greps established it before any
  code moved. (c) `all_cards()` is not the only consumer of `defs` — 14 test files reach
  `cards::defs::<card>::card()`, visible only in `tests/`. (d) **`git mv` silently destroyed
  `tools/authoring-report.py`'s provenance**: without `--follow`, all 1,748 cards attributed
  to the SR-6 commit and the activity table read "1,749 added this week." Windowed queries
  now pass `-M` + both pathspecs. **Re-run history-mining tools after committing a move, not
  before.** (e) SR-5's registry site paths were crate-relative and stopped being unique;
  now workspace-relative, scanning two crates, with a per-root non-vacuity guard.
  **Caught by `/review`, not by me:** seven `.claude/agents/` prompts and three
  `.claude/skills/` still directed card authoring at `crates/engine/src/cards/defs/`. These
  are **write** targets — `build.rs` no longer scans that directory, so an authoring agent
  would have created a card that never registers and never compiles, with no error anywhere.
  Fourth consecutive SR task where the review found the gap in the *gate* rather than a bug
  in the code; the pattern this time is that a compiler cannot see a path in a prompt.
  `primitive-impl-runner.md` also still said "Register the card in `defs/mod.rs`" — wrong
  since the W2 split, unrelated to SR-6, and now removed. Deliberately left stale:
  `memory/` PB plans/reviews (137 files), generated-script `generation_notes` (52), and
  closed-issue tables — those are records, not instructions.
  **Next session:** SR-7 (`scutemob-59`, PendingTrigger → TriggerData; read `state/hash.rs`
  header first and bump `HASH_SCHEMA_VERSION`) — note `state/{stack,stubs}.rs` now live in
  `crates/card-types`, so `PendingTrigger` and `TriggerData` are both there while `hash.rs`
  stays in the engine. Or SR-11 (`scutemob-63`, pin the toolchain) as cheap filler; the
  workspace just gained two crates, so a toolchain float now reddens CI across three.

- 2026-07-10 — SR-5 (scutemob-57) — **done** — A new `KeywordAbility` variant can no
  longer be silently inert. New `crates/engine/src/state/keyword_registry.rs`:
  `handling(&KeywordAbility) -> KeywordHandling` is an exhaustive match over all 166
  variants, classifying each as `Handled { sites }` (engine code branches on it, at
  these files) or `Marker { carrier, cr }` (presence marker only; the rules text is
  implemented by `carrier`, per the cited CR). 148 Handled, 18 Marker. Adding a variant
  is a compile error until classified. `crates/engine/tests/keyword_registry.rs` closes
  what a compile error cannot: `all_keywords()` is checked against the enum declaration
  parsed out of `types.rs` via `include_str!`; each `Handled` site set is checked for
  **exact equality** against a comment-stripped scan of the source tree (so a keyword
  losing its last dispatch site, or a `Marker` gaining one, fails); the 18 markers are
  pinned by name. All 18 marker CR citations verified against rule text on the
  mtg-rules MCP server. 3129 tests pass (3123 + 6). All four gates clean.
  **Hazards discovered:** all written up in the SR-5 gotcha above — chiefly
  (a) **the task's premise was a misattribution.** Only 2 of the ~117 `_ => {}` arms in
  the six named files are on `KeywordAbility`; building what the brief literally asked
  for (expand catch-alls into explicit listings) would have added ~2 000 lines and
  caught nothing. (b) The real hazard is `keywords.contains(&KeywordAbility::X)` —
  ~350 reads, no match arm, nothing to grep. (c) **A green test proved nothing until
  the non-vacuity guards existed**: the site-equality test passed while the variant
  parser was returning zero variants, because empty == empty for all 166 keywords.
  (d) `hash.rs` and the replay viewer's `view_model.rs` were already exhaustive, so
  "new variant → compile error" was true before this task; it just wasn't a *useful*
  error. (e) **`/review` found the one escape I left open** — `Handled { sites: &[] }`
  on a keyword nothing reads compares `{} == {}` against the source scan and passes.
  One assertion closed it. Third SR task running where the review caught the gap in the
  gate rather than a bug in the code; the pattern is that the author checks whether the
  gate fires, not whether it can be satisfied *without* firing.
  **Deliberately not closed:** `scutemob-67` (SR-15) — the ~117 catch-alls really do
  exist, on `AbilityDefinition` (20), `ZoneId` (19), `ZoneChangeAction` (17) and
  others. `AbilityDefinition` is a real dispatch table with the exact hazard SR-5 was
  chartered to fix, one enum over. The registry pattern transfers directly. Also
  unchanged: the 13 `filter_map` projection catch-alls, which are correct.
  **Next session:** SR-6 (`scutemob-58`, extract card-defs crate — wide blast radius,
  check the collision rules first) or SR-11 (`scutemob-63`, pin the toolchain) as cheap
  filler. Note the SR-4/SR-5 pairing paid off exactly as the plan predicted: the terrain
  was already mapped and SR-5 spent its budget on the gate rather than on reading.

- 2026-07-10 — SR-4 (scutemob-56) — **done** — The LKI-vs-bug distinction is now a
  property of the code. New `crates/engine/src/state/diagnostics.rs` supplies two
  families that both return `Option`, so converting a site is a one-token change:
  `expect_player[_mut]` / `expect_object[_mut]` / `expect_zone[_mut]` /
  `expect_move_object_to_zone` / `expect_move_object_to_bottom_of_zone` /
  `expect_add_object` / `layers::expect_characteristics` all `debug_assert!` with
  `#[track_caller]`; `lki_object[_mut]` / `lki_move_object_to_zone` return a silent,
  CR-cited `None`. Plus `debug_assert_object_live!` for sites that need a disjoint
  field borrow. All 398 candidate sites in `effects/mod.rs` (219) and `resolution.rs`
  (179) classified — 222 impossible, 69 fizzle, 107 non-swallow — with the per-site
  table in `docs/sr-4-silent-failure-audit.md`, anchored to `ef0d9579`. 3123 tests pass
  with debug assertions live; **no assert fired**, which is the point: these are
  tripwires for the next regression, not a bug hunt. All four gates clean.
  **Hazards discovered:** all six written up in the SR-4 gotcha above — chiefly
  (a) establish the never-removed invariants *first*, they decide ~40 sites for free;
  (b) `move_object_to_zone` has four error variants and `Err(_)` was eating all of them
  (caught by `/review`, not by me); (c) the SR-3 disjoint-borrow problem recurs on any
  accessor migration. Also: `combat.rs`'s two `unwrap_or_default()` sites (the ones the
  task named) would have silently disabled the CR 702.14c landwalk check had they ever
  fired, because a blank `Characteristics` contains no `CardType::Land`.
  **Deliberately not closed:** `scutemob-65` (SR-13) — the engine reads a damage
  source's wither/infect through a *live* lookup, so a source that died with its damage
  ability on the stack silently loses both, contra CR 702.80c / 702.90e / 113.7a. That
  is a real bug and a semantics change, not an assert, so it did not belong in a sweep.
  `scutemob-66` (SR-14) — the same vocabulary over the ~200 unswept
  `calculate_characteristics` sites in the rest of `rules/`; SR-4 was scoped to its two
  named files.
  **Next session:** SR-5 (`scutemob-57`) — it is the natural pair, on the same files,
  and the terrain is now mapped.

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
