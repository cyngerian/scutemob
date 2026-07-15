# SR Remediation Track — Operations Guide

<!-- last_updated: 2026-07-11 -->

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
| 9 | scutemob-61 | SR-9: Test infra consolidation | L | **DONE 2026-07-10** — all three sub-items landed (SR-9a/9b/9c). Umbrella closed. |
| 9a | scutemob-69 | SR-9a: Consolidate 291 integration-test binaries | M | **DONE 2026-07-10.** 297 binaries → 9 targets; warm test-build 34.2s→11.1s; `target/` 19GB→2.2GB. |
| 9b | scutemob-70 | SR-9b: Harness-vs-direct equivalence property test | M | **DONE 2026-07-10.** `tests/scripts/harness_equivalence.rs`. Four harness divergences found and fixed/pinned; `build_initial_state` was **nondeterministic**. |
| 9c | scutemob-71 | SR-9c: Golden-script corpus triage | M | **DONE 2026-07-10.** 94→**210 approved**, **61 retired** (each with a recorded reason), **0 pending**. Six scripts didn't even deserialize; the replay checker passed 244 unimplemented-path + 583 `zones.stack` assertions vacuously — all closed. Gate: `run_all_scripts.rs`. |
| 10 | scutemob-62 | SR-10: Dependency & lint hygiene | S–M | **DONE 2026-07-10.** im→imbl migrated (704/705 refs ordered collections), rand 0.9, `[workspace.lints]` (non-vacuous), CastSpell boxed (`PROTOCOL_VERSION` 1→2). Chore C bakes toolchain-float into local builds → makes SR-11 more urgent. |
| 11 | scutemob-63 | SR-11: Pin the Rust toolchain | S | **DONE 2026-07-10.** `rust-toolchain.toml` pins exact stable `1.95.0` (single source of truth); CI reads `channel` from it and verifies the installed rustc matches. Local `clippy -D warnings` is now an authoritative CI preview. |
| 12 | scutemob-64 | SR-12: Unbypassable invariant-9 gate + marker anti-rot | M | **DONE 2026-07-10.** `start_game` now runs a completeness pre-game check (the choke point every assembly path shares); explicit opt-out `start_game_allowing_incomplete`. Deviation-language source scan guards Partial/KnownWrong; 6-entry reviewed allowlist. Fuzzer `random_deck` filters to Complete. |
| 13 | scutemob-65 | SR-13: Damage-source characteristics must use LKI | M | **DONE 2026-07-10.** `GameState.lki_objects` layer-resolved snapshot captured in `move_object_to_zone`; infect/wither/deathtouch/lifelink now apply from a dead source (CR 702.80c/702.90e). `HASH_SCHEMA_VERSION` 37 → 38. |
| 14 | scutemob-66 | SR-14: Extend the SR-4 diagnostics vocabulary to the rest of `rules/` | M | **DONE 2026-07-10.** ~360 sites across the ten named `rules/` files classified impossible vs fizzle; two uncertain IMPOSSIBLE calls demoted to FIZZLE by the debug-assert suite. Record: `docs/sr-14-silent-failure-audit-rules.md`. |
| 15 | scutemob-67 | SR-15: Catch-all audit for the *other* dispatch enums | M | **DONE 2026-07-10.** `state::ability_definition_registry` compile gate (68 variants: 64 Handled / 4 Marker) + trial-variant demo; `ZoneChangeAction` proven already compile-gated by construction. All ~26 `AbilityDefinition` catch-alls are benign projections. |
| 16 | scutemob-68 | SR-16: `PendingTrigger` serde round-trip drops `kind`/`data`/`embedded_effect` | S–M | **DONE 2026-07-10.** Option (a): the three `#[serde(skip)]` fields are now serialized; `PendingTriggerKind` gained the derive. `HASH_SCHEMA_VERSION` 38 → 39 (serde shape change; hash stream unchanged). Round-trip gate `pending_trigger_serde_roundtrip`. `PROTOCOL_VERSION` untouched — `PendingTrigger` is inside `GameState`. **Closes the SR remediation track.** |

### Re-audit batch (2026-07-11) — SR-17 … SR-32

Filed by the full re-audit of the remediated baseline (see the 2026-07-11 session-log
entry for method and verified-clean claims). Numbering continues the track; `/remedy`
picks these up unchanged.

| Order | ESM ID | Task | Size | Notes |
|-------|--------|------|------|-------|
| 17 | scutemob-72 | SR-17: `HASH_SCHEMA_VERSION` fingerprint gate | M | **HIGH.** The state-hash analogue of SR-8's cure; sentinels force noticing a bump, never making one. Do before M10 replay/rewind work. |
| 18 | scutemob-73 | SR-18: `proptest-regressions` is an ungoverned auto-built test target | S | **HIGH, cheap.** Demonstrated: stray `.rs` there never compiles, gate 5/5 green; `#![cfg(any())]` in a group module file deleted 14 tests green. |
| 19 | scutemob-74 | SR-19: HashInto-vs-struct coverage gate; `embedded_effect` hashes zero bits | M | False "Effect has no HashInto impl" comment at `hash.rs:2970`. Pairs naturally with SR-17. |
| 20 | scutemob-75 | SR-20: Registry-scan alias/glob bypass + simulator scan root | S–M | Demonstrated bypass on **both** registries; `crates/simulator/src/legal_actions.rs` dispatches on 5 keywords outside `SCAN_ROOTS`. |
| 21 | scutemob-76 | SR-21: Completeness gate bypassed by the script/replay path | M | replay-viewer + `build_initial_state` run games with no `start_game`; SR-12's "no silent bypass" claim is false for this path. |
| 22 | scutemob-77 | SR-22: Script schema strictness (unknown keys, discovery asymmetry, dead init fields, PROPTEST_CASES) | M | Live evidence: `stack/135` carries a silently-ignored stray top-level `review_status`. |
| 23 | scutemob-78 | SR-23: `lki_object` / `lki_object_snapshot` API collision + misdirecting assert text | S | SR-4×SR-13 interaction; assert text sends authors to the live getter at LKI sites. |
| 24 | scutemob-79 | SR-24: Bound `capture_lki_snapshot` cost | S–M | Full layer eval per battlefield departure, unmeasured; measure before gating. Same axis as MR-M1-18/MR-M6-14. |
| 25 | scutemob-80 | SR-25: Diagnostics ratchet + sweep `layers.rs` (45 bare lookups) / `commander.rs` (6) | M | SR-4/14 discipline has no anti-regression scan; two files were never swept. |
| 26 | scutemob-81 | SR-26: `authoring-report.py` anti-rot + stale `project-status.md` Card Health | S–M | `Completeness::Partial(…)` direct spelling buckets as "clean"; zero tests on the tool that owns the campaign headline number. |
| 27 | scutemob-82 | SR-27: Protocol version-bump enforcement + guard token-anchoring | S | Re-pin-without-bump passes today; `contains("Serialize")` matched by "Deserialize". |
| 28 | scutemob-83 | SR-28: Tap-and-sacrifice mana sources read a dead object (CR 106.12a/b) | M | **Rules bug**, two sites (trigger filter + production replacement); the SR-14 left-open seed, now precise. Fix shape = SR-13 snapshot. |
| 29 | scutemob-84 | SR-29: CR 616.1 batch — wrong chooser (owner≠controller), no 616.1f fixed point, `OrderReplacements` applies unvalidated ids | M–L | **Rules + M10 trust boundary.** Chooser fix and applicability check are shippable now; the rest lands with interactive choice. |
| 30 | scutemob-85 | SR-30: Layers hygiene — re-attach doesn't re-timestamp effects (CR 613.7a); vacuous 613.8b test over dead code | S–M | Plus one decisions.md line for the 613.8 static-approximation scope. |
| 31 | scutemob-86 | SR-31: Equivalence coverage ratchet (6 of 60+ command shapes) | S–M | Known-open from SR-9b, now tracked. |
| 32 | scutemob-87 | SR-32: Hygiene batch — assert-blind release fuzzing, doc drift (PROTOCOL_VERSION 1→2 etc.), stack-edge flake, floating action refs | S | Bundle of confirmed LOWs; each named with file:line in the task. |

Re-audit sequencing: SR-17 and SR-18 first (HIGH; SR-18 is an hour). SR-28 and the
shippable half of SR-29 are the only *rules-correctness* bugs — do them before the
authoring campaign leans further on mana triggers / replacement ordering. SR-21+SR-22
touch the same script-harness files — do adjacently. SR-23 before any new `rules/`
sweep (SR-25) so the sweep uses the corrected vocabulary.

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
- **SR-9(a): DONE (2026-07-10).** `crates/engine/tests/*.rs` is now
  `crates/engine/tests/<group>/{main.rs, *.rs}` — 9 targets, not 297 binaries. A former
  per-file binary is a **module**: `--test run_all_scripts` → `--test scripts run_all_scripts`,
  `--test layers` → `--test rules layers::` (keep the `::`; a bare `layers` substring-matches
  `layer_correctness`). Never add a top-level `tests/*.rs` —
  `tests/no_stray_test_binaries.rs` fails the suite. Layout:
  `docs/sr-9a-test-consolidation.md`. Also: `/tmp` on this box is a 16 GB **tmpfs** — never
  point a cargo `target/` at it.
- **SR-9(b): DONE (2026-07-10).** The gotcha held: all three divergences were the harness's.
  `tests/scripts/harness_equivalence.rs` drives one scenario through both regimes and requires the
  same fingerprint (public hash **+ every player's private hash**, so hidden-zone bugs are visible)
  after **every** step. **`build_initial_state` was nondeterministic** — `InitialState`'s zone and
  player maps are `std::collections::HashMap`, whose per-instance `RandomState` seed made two builds
  of the same script hand out different `ObjectId`s: 40 builds, 2 distinct hashes. Nothing that
  hashes a harness-built state could have worked. Fixed by `sorted_zone_entries`, which every loop
  over a script-supplied map must now go through. Also: `init.turn_number` was declared and never
  read, so every script ran on turn 1 whatever it claimed. **New landmine for anyone adding a
  proptest under `tests/`**: on its first failure `proptest` writes `tests/proptest-regressions/`,
  which SR-9a's `every_expected_group_exists_and_has_a_module_root` reads as a stray test group — one
  red test becomes two, and the second buries the first. `NON_GROUP_DIRS` exempts it.
  Two more, both from the review: `resolve_targets` used to **drop** unresolvable targets
  (`filter_map`), turning a `cast_spell` at an absent permanent into a targeted spell cast with no
  target (CR 601.2c) — it now returns `None`. And a determinism fixture with a **one-owner** zone map
  tests nothing: a one-key `HashMap` has exactly one iteration order, so the gate silently becomes a
  no-op. `determinism_fixture_has_two_owners_in_every_zone_map` guards that.
- **SR-9(c): DONE (2026-07-10).** The corpus was 271 scripts, **94 approved and 175 silently skipped**
  (`run_all_scripts` filtered to `review_status == Approved` and dropped the rest without a count).
  Triaged to **210 approved / 61 retired / 0 pending**. The skip was not the only silent hole — the
  *checker itself* was largely vacuous:
  - **An unrecognized assertion path returned "no mismatch."** 244 assertions across the corpus were
    spelled in paths `check_assertions` never implemented (`zones.battlefield.<p>.count`, `zones.exile`,
    `zones.hand.<p>`, `permanent.<c>.power`, …) and were simply *not checked*. An unknown path is now a
    hard `AssertionMismatch`; every path the corpus uses is implemented (power/toughness read through
    `calculate_characteristics`, never the pre-layers printed value).
  - **`zones.stack` was checked against a hardcoded empty list.** All **583** `"zones.stack": {"is_empty":
    true}` assertions passed unconditionally — `names.is_empty()` on an always-empty `&[]`. It now reads
    the real stack length. This alone turned three previously-"approved" scripts red (they asserted an
    empty stack where a dies/ETB trigger legitimately sits, CR 603.3).
  - **`includes` + `excludes` on one assertion dropped the `excludes` half** (early `return`); a malformed
    list entry became the empty card name `""` and silently "wasn't found." Both fixed.
  - **A `player_action` the harness could not translate ran as a no-op with no record.** Nine such action
    names were live in approved/pending scripts (`assign_damage`, `choose_option`, `cast_spell_from_command_zone`,
    `transform`, `activate_craft`, `cast_spell_disturb`, `order_replacements`, `sacrifice`, `mulligan_decision`).
    `translate_player_action` now emits `ReplayResult::ActionNotTranslated`; `transform` was wired to its
    existing `Command::Transform`; the genuinely-informational ones (`assign_damage`, `choose_option`,
    `sacrifice`, `search_library`) are an **allowlist with a dead-entry guard**.
  - **Six scripts never deserialized at all** — two carried `review_status: draft` (not a variant) and four
    had `disputes[]` entries missing the required `raised_by`. They had been invisible since written;
    `discover_scripts` used to swallow the `Err`. It now panics naming the file and the serde error.
  The gate (`tests/scripts/run_all_scripts.rs`) **partitions** the discovered set: `no_script_is_awaiting_triage`
  (no `pending_review`/`disputed`/`corrected` may persist), `retired_scripts_carry_a_reason`,
  `every_approved_script_asserts_something`, `the_corpus_is_fully_accounted_for` (`approved + retired ==
  discovered`, printing each retirement reason), and `approved_scripts_only_use_allowlisted_untranslatable_actions`.
  Retirement is a first-class status: `ReviewStatus::Retired` + a required `retirement_reason`. The 61
  retirements are all blocked on real, out-of-scope gaps — missing `CardDefinition`s (invariant #9),
  un-translated alt-cost/combat-damage Commands (SR-9b: only 6 of 60 command shapes cross-validated), the
  empty-target ETB DSL gap, and the informational `stack_resolve` that leaves triggers unresolved — each
  named in the script's own `retirement_reason`. **Adversarial demo** (`adversarial_demo.sh`, 7 attacks,
  each asserted to change a file first): pending script, undeserializable file, reason-less retirement,
  vacuous approved script, un-allowlisted untranslatable action, unimplemented assertion path, and a
  reverted `zones.stack` fix — all seven reddened exactly their intended gate. **Only fix, not retire, one
  currently-approved failure**: `stack/050` correctly re-asserted as `zones.stack.count == 1` (Solemn
  Simulacrum's dies trigger, CR 603.3). `stack/170` (tribute) and `cc31` (commander damage) were retired
  rather than "fixed to match the engine," because matching a possibly-wrong engine would bless a bug and
  `cc31` is inexpressible (it fakes combat damage via `stack_resolve` and pre-seeds
  `commander_damage_received`, an init field the harness has never read).
- **SR-10: DONE (2026-07-10).** Four independent chores; each committed separately so any
  one is revertable. Findings:
  1. **im → imbl: DECISION = MIGRATE (done).** The brief's stated risk ("im→imbl behaviour
     differences") did not materialize, and the reason is measurable up front, not by faith: of
     **705** `im::` references, **704** are *ordered* collections — `im::OrdSet` (350),
     `im::Vector` (148), `im::OrdMap` (136), plus their macros. imbl is a fork of im 15.1 with the
     **same** B-tree (`OrdMap`/`OrdSet`) and RRB-vector (`Vector`) internals, so iteration order —
     the only property the engine's determinism and state-hashing depend on — is byte-identical.
     The **one** `im::HashMap` reference is a *comment* at `rules/replacement.rs` explaining why the
     engine never uses a hash-ordered im collection; there is zero real `HashMap`/`HashSet` usage to
     worry about. The swap was mechanical: 4 manifest lines (`im = "15"` → `imbl = "7"`, `serde`
     feature retained) and a `\bim:: → imbl::` rename across the tree; **no API changes were
     needed** and the compiler was clean first try. Gates green under imbl: `state_hashing` (19),
     `zone_integrity` shuffle determinism (19), core (347) + rules (535) + casting (147),
     `clippy --all-targets`. **The SR-8 `PROTOCOL_SCHEMA_FINGERPRINT` did not move** even though the
     rename touched declaration files — because struct fields name the collections *unqualified*
     (`Vector<AbilityInstance>`, imported via `use`), and the fingerprint scanner reads field
     declarations, not `use` lines. So the wire is provably unchanged; no `PROTOCOL_VERSION` bump.
     No `HASH_SCHEMA_VERSION` bump either (runtime hashes are identical because iteration order is).
     Had even a handful of real `im::HashMap`/`HashSet` uses existed on a hashed or iterated path,
     the correct call would have been *defer* — the migration is clean precisely because the codebase
     had already disciplined itself onto ordered types (the `replacement.rs` comment is that
     discipline, written down).
  2. **rand 0.8 → 0.9: clean.** Mechanical API renames (`gen_range`→`random_range`,
     `gen_bool`→`random_bool`, `.gen()`→`.random()`, `thread_rng()`→`rng()`,
     `StdRng::from_entropy()`→`StdRng::from_os_rng()`); `SeedableRng::seed_from_u64` and
     `SliceRandom::shuffle` are unchanged, so the Fisher-Yates library shuffle and every seeded
     `StdRng` kept the same call shape. **The determinism canary was a non-event because the tests
     are built right**: every `state_hashing` and `zone_integrity` assertion is an *instance-equality*
     check (two instances agree / differ), never a comparison to a hardcoded digest, so a
     hypothetical change in rand 0.9's sampling algorithm would not have reddened them anyway — only
     a *non-determinism* would. Confirmed the shuffle seed is still `timestamp_counter`-derived
     (deterministic), so replay correctness holds. If a future rand bump ever *did* change
     `random_range`'s output for a fixed seed, the thing that would catch it is a stored replay-log
     fixture, which the corpus does not yet have — worth noting for whoever adds one.
  3. **`[workspace.lints]`: `warnings = "deny"` + `[lints] workspace = true` on all 11 members.**
     Source-encodes CI's `clippy --all-targets -- -D warnings` so a plain `cargo build`/`cargo clippy`
     enforces the same bar. **Proved non-vacuous adversarially** (this track's rule): injecting an
     `unused_variables` (rustc) *and* a `bool_comparison` (clippy) into a member both error with **no
     `-D` flag** — clippy lints are promoted by the `warnings` group too, so one setting covers both.
     Interaction with SR-11: this bakes the toolchain-float hazard into *local* builds as well as CI
     (a newer stable's new lints now redden `cargo build` here), which is exactly the motivation to
     pin the toolchain next (`scutemob-63`).
  4. **Box `Command::CastSpell`: done; `PROTOCOL_VERSION` 1 → 2.** The ~16-field variant was extracted
     into a new `pub struct CastSpellData` and the variant became `CastSpell(Box<CastSpellData>)`;
     the `#[allow(clippy::large_enum_variant)]` came off `command.rs` and `clippy --all-targets`
     confirms the lint stays quiet. **739 construction sites** were wrapped by a tokenizer-aware
     brace-matcher (skips string/char/comment content while counting `{}`), and the **3** pattern
     sites — the `process_command` dispatch arm plus two `if let` sites in tui/simulator — were
     hand-converted *first* so the script's `Command::CastSpell {` needle matched only constructions.
     The dispatch arm destructures `*cast` into the same local names, leaving the handler body
     byte-for-byte unchanged. **The wire bytes do not change** — a boxed newtype variant wrapping a
     struct is serde-identical to the former struct variant — **but the SR-8 shape digest does**,
     because the type closure grew 90 → 91 and the variant's declared form changed. Per the gate's
     own instruction (and its explicit "any non-reorder digest move bumps the version" policy),
     bumped `PROTOCOL_VERSION` 1→2, re-pinned `PROTOCOL_SCHEMA_FINGERPRINT`, and updated the
     `protocol_version_sentinel`. **No `HASH_SCHEMA_VERSION` bump** — `Command` is not reachable from
     `GameState`'s hash closure (that boundary is exactly why SR-8 keeps the two versions separate).
     `cargo test --all` stayed at **3185 passed / 0 failed**: same tests, just wrapped — the count not
     moving is the evidence the box is behaviour-neutral.

- **SR-11: DONE (2026-07-10).** The brief's recommendation (option a — pin exact stable
  in `rust-toolchain.toml` + components, single source of truth) was correct and held up.
  What mattered:
  1. **A `rust-toolchain.toml` already existed and was itself the float.** It said
     `channel = "stable"`, tracked since M0. `stable` floats to the newest release at
     resolution time, so it gave the *appearance* of a pin while providing none — CI's
     `dtolnay/rust-toolchain@stable` and a dev box's last-`rustup-update`d stable resolve
     to different versions from the same file. Fixed to `channel = "1.95.0"` (the dev-box
     version), + `components = ["rustfmt", "clippy"]` + `profile = "minimal"`. **Check
     whether the lever already exists before adding one; here it existed and was lying.**
  2. **`dtolnay/rust-toolchain` does not read `rust-toolchain.toml`.** Its `@rev` (`@stable`,
     `@1.95.0`) *is* the toolchain selector; there is no auto-detect. So a single source of
     truth requires *extracting* the version and passing it to the action, not hoping the
     action honors the file. CI now has a "Read pinned toolchain" step
     (`grep -oP '^\s*channel\s*=\s*"\K[^"]+' rust-toolchain.toml` → `$GITHUB_OUTPUT`) feeding
     `dtolnay/rust-toolchain@master`'s `toolchain:` input. The `^\s*channel` anchor is load-
     bearing: the file's explanatory comment contains the word "channel" in prose, and an
     unanchored grep matches it — verified the extraction yields exactly one line, `1.95.0`.
  3. **The pin needs its own machine gate, or it is just another process guarantee.** A
     "Verify toolchain matches the pin" step asserts `rustc --version` equals the extracted
     channel and fails the run otherwise — catching a rustup resolution surprise or a future
     edit that re-floats the version. Without it, "CI installs the pin" is a claim nobody
     checks, exactly the shape this track exists to delete.
  4. **This closes the loop SR-10 chore C opened.** SR-10's `[workspace.lints] warnings =
     "deny"` baked `-D warnings` into local `cargo build`, so a newer stable's new lints
     could redden the *local* build too — not just CI. With the toolchain pinned, local and
     CI share one lint set by construction, and `cargo clippy -- -D warnings` locally is an
     authoritative preview of the CI clippy gate. Documented in `.claude/CLAUDE.local.md`
     (Project-Scoped Version Pinning + CI sections): the local gate is only as authoritative
     as the pin, so bump `channel` and `rustup install` deliberately, then re-run the full
     gate to surface new lints.
- **SR-12: DONE (2026-07-10).** Both halves of the brief held up. What mattered:
  1. **`start_game` is the only choke point, so the gate goes there — not on the builder.**
     `GameStateBuilder` is a test utility that assembles objects field-by-field; there is no
     single "assemble a game" method on it to guard. Every path that actually *runs* a game —
     the simulator's `driver.rs`, the fuzzer, and any future production caller — funnels through
     `start_game(state)`. So the completeness check lives in `start_game`, which scans
     `state.objects` and refuses any object whose `card_id` resolves to a **known but
     non-`Complete`** registry def (`GameStateError::IncompleteCardsInGame`). The explicit
     opt-out is a sibling fn, `start_game_allowing_incomplete`, not a flag — a distinct symbol is
     greppable and cannot be defaulted-through.
  2. **Scope the gate to "known but non-Complete", not "unknown".** An object whose `card_id` is
     *absent* from the registry is a different axis (`UnknownCard`) and, critically, hundreds of
     existing tests place naked objects or set a `card_id` against an **empty** registry. Gating
     unknown ids here would redden all of them. `registry.get(cid)?` returning `None` is a pass,
     by construction — the gate only fires on a def that exists and is marked. This is the same
     precision `validate_deck` already has (it reports `UnknownCard` and `IncompleteCard`
     separately).
  3. **The gate found exactly one legitimate opt-out in the whole suite, and it is instructive.**
     `test_leyline_opening_hand` places Leyline of the Void — whose def is **`known-wrong`**
     ("begin the game on the battlefield" is not modelled on the def; the test drives the
     engine's pre-game placement instead) — and calls `start_game`. That is precisely the case
     the opt-out exists for; switched to `start_game_allowing_incomplete`. **A card marked
     non-Complete for reason X can still be the subject of a test about behaviour Y**, and that
     test is not wrong — it just has to say "yes, incomplete, on purpose."
  4. **The fuzzer was silently building games out of 742 non-Complete cards.** `random_deck`
     drew commanders and deck cards straight from `all_cards()` with no completeness filter, so
     most fuzz games included inert/partial/known-wrong cards — the exact ungated inventory this
     task exists to close. With the gate in place those games would now `IncompleteCardsInGame`
     at `run_game`; filtering `random_deck` to `is_complete()` up front keeps the fuzzer
     exercising real play. The gate *forced* the fuzzer to become correct — a process guarantee
     converted to a machine one, on the nose.
  5. **The Partial/KnownWrong anti-rot guard has to be a source scan, because the deviation is a
     comment.** The Inert class is checkable at runtime (`abilities.is_empty() && oracle_text
     non-empty`), but "this clause is Simplified / an approximation / modeled as X" lives only in
     a `//` comment that never reaches the compiled `CardDefinition`. So the guard
     (`tests/core/completeness_deviation_scan.rs`) reads the def source and requires any file
     matching a deviation needle (`simplif|modeled/modelled as|deviation|approximat`, lower-cased)
     to either carry a non-`Complete` marker fragment or be on a **reviewed 6-entry allowlist**.
     Same technique as SR-5's keyword registry and SR-8's fingerprint.
  6. **The allowlist is all false positives, and every one is "modeled as" / "approximation" used
     to describe *faithful* modeling.** 136 files match the needles; 130 already carry a marker.
     The 6 that don't are Complete cards whose comment says "Modeled as two separate triggers"
     (a faithful decomposition), "not an overbroad generic-creature approximation" (explicitly
     *denying* a deviation), or "fixes the previous approximation" (describing a since-fixed one).
     Reviewed and documented in-test with a per-entry reason; the value of keeping the needle set
     broad (`modeled as` catches real "we modeled it as X but the card does Y" deviations) is worth
     the six-entry allowlist. Per SR policy the allowlist is guarded:
     `every_allowlist_entry_is_live_and_necessary` fails if an entry stops matching a needle
     (dead weight) or gains a marker (redundant), so it cannot silently mask a future real
     deviation.
  7. **Every derived set is denominator-guarded** (SR track rule): the scan asserts it reaches
     >1500 files, the deviation detector asserts ≥50 hits, and the marker detector asserts ≥700 —
     without these an absence-shaped gate ("no offenders") passes vacuously if the path is wrong
     or a needle typo stops matching.
- **SR-14: DONE (2026-07-10).** Extended SR-4's `state::diagnostics` vocabulary to the ten
  named `rules/` files; ~360 sites. Full record: `docs/sr-14-silent-failure-audit-rules.md`.
  Two lessons for any future sweep of this shape:
  1. **Bias-to-fizzle + a real suite is not optional, it is the mechanism.** Both of the two
     verdicts the classifiers flagged as uncertain-IMPOSSIBLE fired debug asserts and were
     correct as FIZZLE (`queue_carddef_etb_triggers` new_id; `has_split_second_on_stack`
     Spell source outliving its object). A function *parameter* named like a live id (`new_id`)
     is not automatically live — check whether the function itself already tolerates absence
     (it had three `unwrap_or` fallbacks), which is a louder signal than the name.
  2. **The disjoint-borrow hazard recurs and self-review misses it.** Parallel classifiers
     were forbidden to run cargo (shared target dir), so the *only* place a
     `state.expect_object_mut(id)`-vs-`card_registry`-borrow conflict shows up is the
     coordinator's central `cargo build --workspace`. One did slip through
     (`activate_loyalty_ability`, E0502) and was fixed to `debug_assert_object_live!` + raw
     `get_mut`. Budget a compile-and-fix pass after any fan-out that edits `_mut` sites.
- **SR-16: DONE (2026-07-10).** The brief was right and the fix was small; what mattered was
  getting the *decision* right rather than the diff. What mattered:
  1. **Option (a) was the only one that survives `GameState: Serialize`.** Once a type derives
     `Serialize`, any caller can serialize it anywhere — there is no single boundary to guard.
     So (b) "assert `pending_triggers.is_empty()` at the serde boundary" and (c) "only serialize
     at a priority boundary" are process guarantees wearing a machine costume: nothing forces the
     serialization to go through the one function that checks. (a) — just serialize the fields —
     removes the constraint instead of policing it, and it is what M10 mid-turn state sync and
     rewind/replay snapshots actually need (a state serialized while a trigger is pending must not
     silently lose it). This is the same "delete the process guarantee, don't automate it" move the
     whole track is about.
  2. **The fix is three `#[serde(skip)]` removals + one derive, because SR-7 had already done the
     hard part.** `data`/`embedded_effect` are `Option<TriggerData>`/`Option<Effect>`, both already
     `Serialize`; only `PendingTriggerKind` lacked the derive, and every variant it reaches
     (`KeywordAbility`, `TriggerData`) was already serializable. No new plumbing.
  3. **`HASH_SCHEMA_VERSION` bumps but the hash stream does not change.** `kind` and `data` were
     *already* fed to `HashInto` (that is why SR-7's note said "the state hash catches divergence
     at runtime even though serde does not"), and `embedded_effect` still is not. So a given state
     hashes to the same fingerprint as before — the bump (38 → 39) is purely because the *serde*
     wire shape of `GameState` gained three fields, and a `ReplayLog` written by lossy older code
     must be rejected. **The hash-version checklist is about the serialized shape, not only the
     hashed bytes; the two can move independently.**
  4. **No `PROTOCOL_VERSION` bump, and the reason is load-bearing.** `PendingTrigger` lives inside
     `GameState`, which SR-8 deliberately excluded from the protocol closure
     (`CLOSURE_MUST_NOT_CONTAIN`). That exclusion is exactly what lets `HASH_SCHEMA_VERSION` and
     `PROTOCOL_VERSION` move independently here. Verified by `tests/core/protocol_schema.rs` staying
     green (the fingerprint did not move). The comment there that called this "an open bug" was
     updated.
  5. **The round-trip gate is adversarial per SR-5's lesson.** `pending_trigger_serde_roundtrip`
     first asserts the payload-bearing fields appear in the JSON (non-vacuity: re-adding
     `#[serde(skip)]` to any one fails a *named* assertion), then asserts the decoded `kind`/`data`/
     `embedded_effect` equal the originals and that `kind != Normal`. An equality-only test would
     pass vacuously if the encoder silently dropped a field on both sides.

## Session Log

_One entry per session, newest first. Format:_
`- YYYY-MM-DD — SR-<N> (scutemob-<id>) — <status: done / in progress / blocked> — <one-line outcome + hazards + pointer for next session>`

- 2026-07-14 — SR-19 (scutemob-74) — **done** (collected, merge `cd1801a1`) — The HashInto/struct
  drift mechanism is closed: `every_hashed_struct_field_is_hashed_or_allowlisted`
  (tests/core/hash_schema.rs) parses each named-field struct with a `HashInto` impl and asserts every
  field is read as `self.<field>` or sits in a per-type `NOT_HASHED` allowlist with a dead-entry
  guard. The `embedded_effect` asymmetry is gone — `PendingTrigger`'s impl now feeds
  `embedded_effect.is_some()`, matching `StackObjectKind` — which changes the hash **stream**, so
  `HASH_SCHEMA_VERSION` 39 → 40 per the checklist: sentinels updated, a v40 row appended to
  `HASH_SCHEMA_HISTORY`, and both fingerprints re-pinned — **SR-17's gate forced exactly this
  procedure on its first real bump, one session after landing.** Both false "Effect has no HashInto
  impl" comments corrected. Hash gate suites verified green on main post-merge (40 tests).
  **Next:** SR-21+SR-22 (scutemob-76/77) still in flight.

- 2026-07-14 — SR-29 (scutemob-84) — **done** (collected, merge `b622ced4`) — CR 616.1 batch, all
  four parts. (1) **Chooser is now the controller** (owner fallback per CR 616.1) in
  `check_zone_change_replacement`, propagated to every interception site — with the correct
  subtlety that *destination* zones (graveyard/hand/library go to the **owner's** zone, CR 400.3)
  keep using owner; pinned by a control-change + two-competing-replacements test and an
  owner-scoped-destination test added at review. (2) **CR 616.1f fixed point**: single-applicable
  redirects now loop (already_applied threaded per CR 614.5); two-hop Graveyard→Exile→Library chain
  test settles correctly. (3) **`OrderReplacements` trust boundary closed**: sender must be the
  affected chooser of a pending event AND every id must be `find_applicable` to *that* event; the
  no-pending trust-hole fallback removed; rejection tests added — this was the M10 server's
  validation surface. (4) Deferred player-choice items registered as **MR-SR29-01/02/03** in the
  milestone-reviews ledger (legend rule 704.5j, regen-vs-umbra ordering, interactive replacement
  ordering — M10 scope); stale sba.rs comments re-pointed. `/review` ran (one LOW, fixed).
  Replacement suite verified green on main post-merge (97 tests). **Next:** SR-19 (scutemob-74) and
  SR-21+SR-22 (scutemob-76/77) still in flight this session.

- 2026-07-14 — SR-18 + SR-20 (scutemob-73/75, paired in one worktree) — **done** (collected, merge
  `ff8b7dfdbc18`) — Both demonstrated bypasses from the re-audit are closed, each with its attack
  re-run and caught. **SR-18**: `exempt_dirs_contain_no_rust_files` recurses `NON_GROUP_DIRS` and
  fails on any `.rs`; `auto_built_targets_match_expected` models Cargo's autotests rule (top-level
  `*.rs` + subdirs with `main.rs`) and asserts the auto-built set == `EXPECTED_GROUPS` +
  `ALLOWED_TOP_LEVEL` with **no exemptions** — the ungoverned `proptest-regressions` target hole is
  gone; `no_module_level_cfg_in_group_files` forbids module-level `#![cfg` with a comment/whitespace-
  aware detector (the `#![cfg(any())]` attack plus two obfuscated forms all caught; 5/5 in
  `sr18_adversarial_demo.sh`). **SR-20**: `use_imports_do_not_bypass_the_scanner` +
  `type_aliases_do_not_bypass_the_scanner` in **both** registry suites flag alias (`as`), glob
  (`::*`), grouped (`::{}`), and single-variant use-imports and `type` aliases of the dispatch enums
  outside `EXCLUDED` (the worker went beyond brief: type aliases too); `crates/simulator/src` is now
  a scan root with per-root non-vacuity anchors, and the 5 `legal_actions.rs` keyword sites are
  declared. Gates verified green on main post-merge (`no_stray` 5→9 tests; both registry suites 9
  each). **This closes the dispatched re-audit cut (SR-17, SR-18, SR-20, SR-28 — all done
  2026-07-14).** **Next:** remaining re-audit backlog per the inventory table — SR-19 pairs with the
  now-landed SR-17 machinery; SR-21+SR-22 adjacently (same script-harness files); shippable half of
  SR-29 when a rules session is next.

- 2026-07-14 — SR-17 (scutemob-72) — **done** (collected, merge `b4736f3e`) — `HASH_SCHEMA_VERSION`
  is now machine-enforced on **two axes**, closing the disease SR-8 named. (1) A declaration
  fingerprint (blake3 of the GameState serde type closure — **118 types**, rooted at `GameState`,
  skip-aware so the `#[serde(skip)]` `card_registry` correctly keeps `CardRegistry`/`CardDefinition`
  out, and asserted **disjoint from the protocol closure** from this side too). (2) A **stream
  fingerprint** (blake3 of the actual hash byte-stream — public + all four private hashes — over a
  canonical builder-only fixture), catching `HashInto` edits the declaration digest cannot see.
  `HASH_SCHEMA_HISTORY` is an append-only (version, decl-fp, stream-fp) table: ascending unique
  versions, tail == current constants, frozen v39 baseline, and a frozen-prefix digest so superseded
  rows cannot be quietly rewritten — a re-pin without a bump now fails. Demonstrated adversarially
  (shape change / stream change / re-pin — each reddens exactly its gate); `/review` ran and its
  findings (fixture too narrow, a wrong coverage comment, missing frozen-prefix pin) were fixed in a
  follow-up commit. Version stays **39** — gates only, no shape change. 18 new tests
  (`tests/core/hash_schema*`), verified green on main post-merge. **Next:** SR-18+SR-20
  (scutemob-73/75) still in flight.

- 2026-07-14 — SR-28 (scutemob-83) — **done** (collected, merge `b77d4210`) — Tap-and-sacrifice
  mana sources: both filters now read a pre-cost snapshot instead of a dead ObjectId. The source's
  layer-resolved characteristics are captured in `handle_tap_for_mana` **before** the sacrifice cost
  is paid (SR-13 shape, but a local snapshot threaded through the function — `GameState` did not
  grow, so no `HASH_SCHEMA_VERSION` movement), and threaded into `apply_mana_production_replacements`
  (AnyLand arm, CR 106.12b — Caged Sun now applies to a sacrificed land) and `mana_source_matches`
  (Land/LandSubtype/Creature/AnyPermanent arms, CR 106.12a — `WhenTappedForMana` now fires for
  Crystal Vein / Dwarven Ruins / Treasure-class sources). Both new tests adversarially confirmed to
  fail pre-fix. The trigger-side controller-scope gap (`fire_mana_triggered_abilities` scopes to
  `o.controller == player`; Vernal Bloom / Mana Flare class will mis-scope) recorded in
  `memory/gotchas-rules.md` beside the PB-Q3 replacement note per criterion 3. **Gates:** worker
  attested full suite + clippy + fmt + build --workspace; CI green on the merge is the confirming
  gate. **Next:** SR-17 (scutemob-72) and SR-18+SR-20 (scutemob-73/75) still in flight this session.

- 2026-07-11 — **SR re-audit** (review only, no fixes) — **done** — Full senior-review pass against
  the remediated baseline, same style as the 2026-07-10 review that created this track. **Method:**
  five parallel agents — two adversarial gate-perturbation agents in isolated worktrees (engine gates;
  test-infra gates), three read-only (remaining-process-guarantee hunt; 19-merge interaction review;
  CR spot-audit of five rules subsystems via the mtg-rules MCP) — plus direct measurement. Every
  perturbation asserted a non-empty diff before trusting a "survived" (SR-9a's lesson, applied).
  **Outcome: 16 new tasks filed, SR-17..SR-32 (`scutemob-72`..`87`)** — see the re-audit inventory
  table above. Highest-severity: `HASH_SCHEMA_VERSION` still has no fingerprint gate (SR-17, the
  disease SR-8 named and cured only for the protocol); `tests/proptest-regressions/` is an ungoverned
  auto-built test target where a stray `.rs` silently never compiles (SR-18, demonstrated); two
  rules bugs found the SR-13 way — tap-and-sacrifice mana sources read a dead object at **two** sites
  (SR-28, CR 106.12a/b; Zendikar Resurgent + Crystal Vein yields nothing, Caged Sun + Dwarven Ruins
  misses +1, Treasures are the ubiquitous carrier; no test can pass today) and the CR 616.1
  replacement-ordering chooser is the **owner**, not the controller, with no 616.1f fixed point on the
  single-applicable Redirect path and `Command::OrderReplacements` applying unvalidated ids (SR-29 —
  the M10 trust boundary). Demonstrated gate bypasses: registry scanners on **both** enums pass an
  `use … as KA` / glob-import dispatch site green (SR-20); the invariant-9 completeness gate never
  sees the replay/script path — replay-viewer runs whole games without `start_game` (SR-21); the
  invariant-9 *script* gate walks two directory levels while `discover_scripts` recurses fully
  (SR-22). In the wild: `stack/135_prototype_blitz_automaton.json` carries a silently-ignored stray
  top-level `review_status: approved` beside the real `metadata.review_status: retired` — no
  `deny_unknown_fields` anywhere in the schema (SR-22). Cross-merge interactions no solo review saw:
  the SR-4 `lki_object()` (live read) vs SR-13 `lki_object_snapshot()` (real LKI) collision, with
  `expect_object`'s assert text directing authors to the wrong one (SR-23); `capture_lki_snapshot`
  runs a full layer evaluation on every battlefield departure, unmeasured (SR-24). Also: `rules/layers.rs`
  has 45 unswept bare lookups and the diagnostics discipline has no ratchet (SR-25); `authoring-report.py`
  buckets a `Completeness::Partial(…)` direct spelling as "clean" and has zero tests (SR-26);
  re-pinning the protocol fingerprint without bumping `PROTOCOL_VERSION` passes (SR-27).
  **Measurements (all SR-9a/SR-1 claims verified):** CI 12/12 green on 2026-07-10, 3m07s–4m59s per
  run; cold `cargo test --all --no-run` 27 s / warm 12–13 s (`CARGO_INCREMENTAL=0`; claimed 24 s /
  11.1 s); `target/` 2.2 GB exactly as claimed (the main tree carried 28 GB of *stale pre-SR-9a*
  binaries, never rebuilt since — now cleaned); suite **3208 passed / 0 failed / 4 ignored**, 29
  suites, ~8 s execution (note: 4 of those are the known `include!` double-count).
  **Verified clean — do not re-falsify:** the GameState seal **holds** (no pub `&mut` surface outside
  the cfg-gated block, no test-util enabler on any production edge, no Deref/interior-mutability
  escape; live mutation probes in `crates/network` fail `build --workspace` *and* `check --workspace`;
  the `--all-targets` unification caveat is real, as documented). The protocol fingerprint **holds**
  against cfg_attr-wrapped serde attrs, `serde(default)`, `serde(with)`, and type-alias retargets;
  doc comments correctly ignored; `Envelope` field pinning real. All five PendingTrigger attacks
  (new field, hand literal, `Self` literal, deleted `resolution.rs` consumer, re-added
  `#[serde(skip)]`) fail named assertions. SR-12's core gate is real (library cards scanned,
  simulator/fuzzer/TUI all pass through `start_game`, opt-out has zero production callers). The
  script partition gate **executes** approved scripts (falsified assertions redden the suite;
  flipping a retired script to approved reddens two gates); all three `sorted_zone_entries` sites are
  individually caught, and the private-hash half of `Fingerprint` is demonstrably load-bearing. All
  11 workspace members carry workspace lints; CI has every claimed step incl. the toolchain
  pin-verify. The 739-site CastSpell boxing is 100% clean (zero clones added, no double-boxing);
  SR-9c's mass diff edited exactly one script's content (`stack/050`), as claimed; `lki_objects` is
  bounded, hashed, and serialized (no SR-16-style resume hole); imbl/rand swaps clean; all
  claimed allowlist dead-entry guards exist and bite. CR-verified correct: combat first/regular-strike
  eligibility snapshots (702.4c/d), lifelink controller-at-damage-time (702.15a), trample-past-dead
  -blockers (702.19d), deathtouch lethal=1 (702.2c), SBA fixed-point batch + 704.5f/g regeneration
  split + 704.5q/u, self-replacements-first (614.15/616.1a), ETB pass loops to a true fixed point,
  mana-ability CR 605 plumbing (Stony Silence reach, triggered-mana immediacy, Nyxbloom ruling).
  **Next:** `/remedy` dispatches SR-17/SR-18 first, then SR-28 + the shippable half of SR-29 (rules
  bugs) — full sequencing note under the re-audit inventory table.

- 2026-07-10 — SR-16 (scutemob-68) — **done** (collected, merge `c93db34f`) — **Closes the SR
  remediation track (final task).** `PendingTrigger.{kind, data, embedded_effect}` were
  `#[serde(skip)]`, so a serialized-then-deserialized `GameState` coerced every pending keyword
  trigger to an anonymous `Normal` with no payload — silently. Chose **option (a)**: serialize the
  fields (`PendingTriggerKind` gained the `Serialize`/`Deserialize` derive; the other two types
  were already serializable). (b)/(c) — "assert empty at the boundary" / "only serialize at a
  priority boundary" — were rejected because once `GameState` derives `Serialize` there is no
  single boundary to police; they are process guarantees, which is what this track deletes.
  `HASH_SCHEMA_VERSION` 38 → 39 (serde shape of `GameState` grew three fields; the **hash stream is
  unchanged** — `kind`/`data` were already hashed, `embedded_effect` still isn't — so states hash
  identically and the bump only rejects a `ReplayLog` from lossy older code; 29 live sentinels
  updated). **No `PROTOCOL_VERSION` bump**: `PendingTrigger` is inside `GameState`, which SR-8's
  `CLOSURE_MUST_NOT_CONTAIN` keeps off the wire — `protocol_schema.rs` fingerprint did not move.
  New adversarial gate `pending_trigger_serde_roundtrip` (tests/core/pending_trigger_shape.rs).
  **Gates:** `cargo test --all` 0 failed, `clippy --all-targets -D warnings`, `fmt --all --check`,
  `build --workspace` — all green. **Next:** none — SR inventory is fully DONE (SR-1..16).

- 2026-07-10 — SR-14 (scutemob-66) — **done** (collected, merge `c93db34f`) — Extended the
  SR-4 `state::diagnostics` vocabulary to the ten named `rules/` files (abilities, casting,
  combat, sba, replacement, turn_actions, mana, copy, engine, lands). ~360 state-lookup /
  fallible-mutation sites classified impossible-absence (`expect_*`, asserts) vs
  expected-fizzle (`lki_*` / kept-`calculate_characteristics`, CR-cited). Method reused
  verbatim: lands.rs done by hand as a template + compile check, then nine parallel
  classifier-editors (one per file), then one central compile + full suite. **Two things
  worth carrying forward.** (1) *The suite is the adjudicator, and it adjudicated.* Two
  IMPOSSIBLE verdicts both classifiers had explicitly flagged as uncertain fired debug
  asserts: `replacement.rs::queue_carddef_etb_triggers` (a caller-supplied `new_id` the
  function already tolerates absent — 3 sites → `lki_object`, CR 400.7) and
  `casting.rs::has_split_second_on_stack` (a `Spell` stack entry outlives its source object
  on a free/plotted cast — CR 400.7/113.7a). Bias-to-fizzle plus a real suite is the whole
  safety mechanism; do not skip either. (2) *The disjoint-borrow hazard recurs and a
  classifier can miss it:* `engine.rs::activate_loyalty_ability` needed
  `debug_assert_object_live!` + raw `get_mut` because `def`/`effect` hold a `card_registry`
  borrow — the agent wrote `expect_object_mut` and it surfaced as an E0502 on the first
  `cargo build --workspace`, not in any agent's self-review (agents were forbidden to run
  cargo to avoid target-dir contention, so the central compile is the only place borrow
  errors show up). Also retired a stale `MR-M4-01` "player may have been removed" comment in
  `sba.rs` — ground truth 1 (no `players.remove` anywhere) says otherwise. **Gates:**
  `cargo test --all` 3201 passed / 0 failed (debug asserts ON), `clippy --all-targets -D
  warnings`, `fmt --all --check`, `build --workspace` — all green on 1.95.0. Record:
  `docs/sr-14-silent-failure-audit-rules.md`. **Left open (noted, not widened):** a mana.rs
  SR-13-style LKI *semantics* gap (`mana_source_matches` on a tap-and-sacrifice source) —
  file a new SR task if wanted. **Next:** SR-15 (`scutemob-67`, catch-all audit for the
  other dispatch enums) or SR-16 (`scutemob-68`, PendingTrigger serde round-trip).

- 2026-07-10 — SR-12 (scutemob-64) — **done** (collected, merge `c93db34f`) — Made the
  invariant-9 marker gate unbypassable and added anti-rot for the Partial/KnownWrong classes.
  (a) `start_game` — the choke point the simulator, fuzzer, and any production caller all share
  (the builder is a field-by-field test utility with no single "assemble" method to guard) — now
  scans `state.objects` and refuses any object whose `card_id` resolves to a **known but
  non-Complete** registry def (`GameStateError::IncompleteCardsInGame`). Scope is deliberately
  narrow: an unknown `card_id` (empty/absent registry) passes, so the hundreds of naked-object
  tests are untouched. Explicit opt-out `start_game_allowing_incomplete` (a distinct symbol, not
  a flag). The suite surfaced **exactly one** legitimate opt-out: `test_leyline_opening_hand`
  drives Leyline (marked `known-wrong`) through pre-game placement on purpose — switched to the
  opt-out. The fuzzer's `random_deck` was silently drawing from all 742 non-Complete cards; now
  filtered to `is_complete()`, so the gate forced the fuzzer to become correct.
  (b) `tests/core/completeness_deviation_scan.rs` scans def source for deviation language
  (`simplif|modeled/modelled as|deviation|approximat`) and requires a non-Complete marker or a
  reviewed 6-entry allowlist. Of 136 matches, 130 already carry markers; the 6 exempt are all
  false positives ("modeled as" for a faithful decomposition, "not an approximation", "fixes the
  previous approximation") — documented per entry, and guarded so a stale/redundant entry fails.
  Denominator guards on the scan (>1500 files) and both detectors (≥50 / ≥700). **Gates:**
  `cargo test --all` (3195 passed / 0 failed, +10), `clippy --all-targets -D warnings`,
  `fmt --all --check`, `build --workspace` all green on 1.95.0. **Next:** SR-13 (`scutemob-65`,
  damage-source characteristics via LKI), then SR-14+.

- 2026-07-10 — SR-11 (scutemob-63) — **done** (collected, merge `c93db34f`) — Pinned the Rust
  toolchain. The repo already had a tracked `rust-toolchain.toml` that said `channel = "stable"`
  — a pin in appearance only, since `stable` floats to the newest release, so CI (fresh fetch)
  and the dev box (last `rustup update`) diverge from the same file. Changed to `channel =
  "1.95.0"` (dev-box version) + `components = ["rustfmt", "clippy"]` + `profile = "minimal"`,
  which becomes the single source of truth: rustup honors it locally and auto-installs on demand.
  `dtolnay/rust-toolchain` does **not** read the file, so CI now greps `channel` out of it
  (`^\s*channel` anchored — the comment says "channel" in prose) and feeds it to
  `@master`'s `toolchain:` input, then a verify step fails the run if `rustc --version` ≠ the pin.
  That verify step is the machine gate that keeps the pin honest. Closes the loop SR-10 chore C
  opened (`[workspace.lints] warnings="deny"` had baked toolchain-float into local `cargo build`
  too); local `clippy -D warnings` is now an authoritative CI preview. Docs updated:
  `.claude/CLAUDE.local.md` (version-pinning + CI sections), this file's gotcha + inventory row.
  **Gates:** `cargo test --all`, `clippy --all-targets -D warnings`, `fmt --all --check`,
  `build --workspace` all green on 1.95.0. **Next:** SR-12 (`scutemob-64`, unbypassable invariant-9
  gate + marker anti-rot), then SR-13+.

- 2026-07-10 — SR-10 (scutemob-62) — **done** (collected, merge `c93db34f`) — Dependency & lint
  hygiene, four independent chores, each its own revertable commit. (A) **im 15.1 → imbl 7.0**:
  migrated, not deferred — 704/705 `im::` refs are ordered collections (OrdSet/Vector/OrdMap) with
  imbl-identical internals, the lone `im::HashMap` is a comment; mechanical dep+path swap, zero API
  changes, wire fingerprint unmoved (fields name collections unqualified). (B) **rand 0.8 → 0.9**:
  mechanical API renames; determinism canary (state_hashing 19, zone_integrity 19) is instance-equality
  so it never depended on rand's sampling staying fixed, only on determinism, which holds. (C)
  **`[workspace.lints]`**: `warnings="deny"` + `workspace=true` on 11 members, source-encoding the CI
  `-D warnings`; proved non-vacuous (an injected rustc lint *and* a clippy lint both error with no flag).
  (D) **box `Command::CastSpell`**: extracted `CastSpellData`, `CastSpell(Box<CastSpellData>)`, removed
  the `large_enum_variant` allow; 739 construction sites wrapped by a comment/string-aware brace matcher,
  3 pattern sites hand-converted first. Wire bytes unchanged but the SR-8 digest moved (closure 90→91),
  so `PROTOCOL_VERSION` 1→2 + fingerprint re-pin + sentinel bump per the gate's policy; no
  `HASH_SCHEMA_VERSION` bump (Command ∉ GameState hash closure). **Gates:** `cargo test --all` **3185
  passed / 0 failed** (count unchanged — the box is behaviour-neutral), `clippy --all-targets -D warnings`,
  `fmt --check`, `build --workspace` all clean. **Hazard handed to SR-11 (`scutemob-63`):** chore C now
  bakes toolchain-float into *local* `cargo build`, not just CI — pinning the toolchain is the pairing
  the inventory table already recommends. **Next:** SR-11 (pin toolchain), then SR-12+.

- 2026-07-10 — SR-9c (scutemob-71) — **done** — Golden-script corpus triaged: 94→**210 approved**,
  **61 retired** (each with a recorded `retirement_reason`), **0 pending**. This closes SR-9 (the
  umbrella `scutemob-61`). The headline is not the triage — it is that the corpus's green was *fiction*.
  `run_all_scripts` filtered to `Approved` and silently dropped the other 175; **six** scripts never even
  deserialized (`review_status: draft`, `disputes[]` missing `raised_by`) and had been invisible since
  written; and the replay checker itself passed **244** assertions against unimplemented paths and **583**
  `zones.stack: is_empty` assertions **vacuously** (checked against a hardcoded empty `&[]`, so
  `names.is_empty()` was always true). All closed: unknown assertion path is now a hard mismatch, every
  path the corpus uses is implemented (power/toughness through `calculate_characteristics`), `zones.stack`
  reads the real depth, `includes`+`excludes` both fire, and an untranslatable `player_action` emits
  `ReplayResult::ActionNotTranslated` instead of a silent no-op. New `ReviewStatus::Retired` + required
  `retirement_reason`; new gate `tests/scripts/run_all_scripts.rs` **partitions** the corpus
  (`approved + retired == discovered`) and fails on any pending / undeserializable / vacuous / reason-less
  script, or an approved script using an un-allowlisted untranslatable action (allowlist has a dead-entry
  guard). **Consistent with the seven-SR-in-a-row pattern, the sharpest finding was a hole in a *checker*,
  not a bug in engine code** — the vacuous `zones.stack` and unknown-path passes meant a large fraction of
  the "passing" corpus asserted nothing. **Only one currently-approved failure was fixed rather than
  retired**: `stack/050` now asserts `zones.stack.count == 1` (Solemn Simulacrum's dies trigger, CR 603.3),
  because that trigger *belongs* on the stack; `stack/170` (tribute) and `cc31` (commander damage) were
  retired precisely so a possibly-wrong engine behaviour is investigated, not blessed by editing the
  script to match it. The 61 retirements are all blocked on out-of-SR-9c-scope gaps: missing
  `CardDefinition`s (invariant #9), un-translated alt-cost/combat-damage Commands (SR-9b flagged only 6 of
  60 command shapes cross-validated — combat-damage assignment, mulligan, commander-zone casts, craft,
  disturb, order-replacements all still un-wired), the empty-target ETB DSL gap, and the informational
  `stack_resolve` that leaves a spell's follow-on trigger unresolved. **Adversarial demo**
  (`crates/engine/tests/scripts/adversarial_demo.sh`, seven attacks, each asserted to change a file first):
  all seven reddened exactly their gate. **`/review` (Opus) then found an eighth hole — again in the gate,
  not the code, tenth consecutive SR task with that shape:** `every_approved_script_asserts_something`
  counted `assert_state` *checkpoints*, so an approved script whose only checkpoint carried an empty
  `assertions: {}` map (zero mismatches, unconditionally green) would have satisfied it. No approved script
  did this today, but a future one could. Fixed to count assertion *entries*, with an eighth attack added to
  the demo. **First demo run "survived" five attacks — because the runner used
  `cargo test <bare-name> -- --exact` and the tests are namespaced `run_all_scripts::<name>`, so the filter
  matched 0 tests and cargo exited 0.** SR-9a/9b's lesson restated: an attack that runs nothing reads as
  "survived," so the runner now treats "0 tests ran" as a distinct `NO TARGET` error. **Gates:** 3178 →
  **3185 tests** (+7, the new gate's own), 0 failed; clippy `--all-targets -D warnings`, `fmt --check`,
  `build --workspace` all clean. **Next session:** SR-9 is closed; the SR track continues at SR-10
  (`scutemob-62`, lint/dependency hygiene) and SR-11 (`scutemob-63`, pin the toolchain). Worth someone's
  time inside the card-authoring campaign, not the SR track: the 61 retirements are a ready-made worklist —
  each names the one missing card, primitive, or harness command that would un-retire it.

- 2026-07-10 — SR-9b (scutemob-70) — **done** — The JSON-script regime and the hand-written
  `Command` regime now cross-validate. `crates/engine/tests/scripts/harness_equivalence.rs` (8 tests,
  one of them a `proptest`) expresses a scenario twice — once as a JSON `initial_state` + action
  strings, once as `GameStateBuilder` + `Command` literals — and requires an identical **Fingerprint**
  (`public_state_hash` **plus every player's `private_state_hash`**) after every step, not just the
  last. The public hash alone omits hand and library *contents*; a harness that dealt the right
  number of cards in the wrong order would have passed a public-hash-only check.
  **The gotcha held — all four divergences were the harness's; the engine needed no change:**
  (1) **`build_initial_state` was not deterministic.** `InitialState`'s zone and player maps are
  `std::collections::HashMap`. Object `ObjectId`s are handed out in insertion order, and `RandomState`
  seeds each map instance separately, so two deserializations of *the same JSON in the same process*
  iterated in different orders and produced different states. Measured: 40 builds of one two-player
  script → **2 distinct hashes**. This is upstream of everything — no hash comparison against a
  harness-built state could have meant anything, and it is why this task had to be done before any
  future work hashes a script. Fixed by `sorted_zone_entries`; every loop over a script-supplied map
  goes through it.
  (2) **`initial_state.turn_number` was declared by the schema and never read.** Every script ran on
  turn 1 regardless of what it said. `turn.turn_number` is hashed, and `entered_turn` and every "this
  turn" comparison read it. Threaded into `GameStateBuilder::turn_number`; all 95 approved scripts
  stayed green, so nothing was leaning on the bug.
  (3) **A script may name a card with no `CardDefinition`** and `enrich_spec_from_def` returns the
  bare `ObjectSpec` — the object enters the game typeless, costless, abilityless, silently. That is
  architecture invariant #9 being bypassed at the front door. **Found by the non-vacuity test, not by
  the equivalence test**: `equivalence_equip` was green because *both* regimes rejected the equip
  identically, Grizzly Bears having no definition and therefore not being a creature. Two mutual
  rejections are equivalent, and worthless. Pinned as a shrinking allowlist
  (`UNDEFINED_CARDS_IN_APPROVED_SCRIPTS`, one entry) with a denominator guard that fails when an
  entry stops being referenced — that guard immediately caught two bogus entries I had put in it
  from a bad `grep`.
  (4) **`resolve_targets` silently dropped unresolvable targets** (`filter_map`), so a `cast_spell`
  naming a permanent that is not on the battlefield became a `CastSpell` with an **empty `targets`
  vec** — a targeted spell cast with no target, CR 601.2c — and the script went green. Found only
  because the review asked why a documented asymmetry had no test. Now returns `None` if any target
  fails; all 95 approved scripts stayed green.
  **What is and is not cross-validated.** Both regimes call `enrich_spec_from_def` (the direct regime
  already imports it — `cost_primitives.rs`, `combat_harness.rs`, `golgari_grave_troll.rs` all do), so
  the equivalence test does **not** prove enrich's *inference* correct; there is no second source of
  truth for that short of re-implementing it. What it does prove is that everything wrapped *around*
  enrich agrees: player-id assignment, insertion order, life/mana/land-play patching, turn, step, and
  — the sharp end — that `translate_player_action` builds the same `Command` literal a hand-written
  test would. `Command` derives `PartialEq`, so the test asserts command equality *and* the hash;
  command inequality is the diagnostic you actually want.
  **Demonstrated adversarially, six attacks**, each asserted to have changed the file before the
  suite ran. All six fire. Two rows carry the argument:
  `play_land` silently falling back to `find_on_battlefield` is caught by **only the property test** —
  it needs the *sequence* `[PlayLand Forest, PlayLand Forest]` before the regimes disagree, and no
  fixed scenario expresses a sequence; and `equivalence_equip` **survives** reverting
  `sorted_zone_entries`, because only one player has permanents in it, so map order cannot matter. A
  scenario proves nothing about a bug it cannot express. That is the case for the proptest and against
  trusting a green fixed-scenario suite.
  **Hazard for the next person who adds a `proptest` under `tests/`:** on its first failure `proptest`
  writes `tests/proptest-regressions/`, and SR-9a's `every_expected_group_exists_and_has_a_module_root`
  reads every directory under `tests/` as a test group. So one property-test failure produced **two**
  red tests and the second buried the first. `tests/core/` has carried four proptest files since
  before that gate existed, so this was live, not introduced here. Fixed with `NON_GROUP_DIRS`.
  **Also destroyed my own work once**: the attack script's `git checkout -- <file>` restore step
  reverted the *uncommitted* fixes it was supposed to be attacking, so the first five attacks
  "changed nothing" and the sixth reported catastrophe. Commit before running a destructive demo, and
  keep SR-9a's rule — assert the attack changed something — because it is what caught it.
  **Handed to SR-9c (`scutemob-71`):** these `initial_state` fields are declared by `script_schema.rs`
  and **never read** by `build_initial_state`, so a script can describe a board the harness will not
  build — `priority`, `step` (only `phase` is parsed), `continuous_effects`, `zones.command_zone`
  (so `find_in_command_zone` can never hit), `PermanentInitState.summoning_sick`,
  `PermanentInitState.attached`, `PlayerInitState.commander_damage_received`. Also `parse_step` has no
  `"combat"` arm although scripts use it as their `phase`; it falls through to the default. And
  `replay_script` silently skips any action `translate_player_action` cannot translate — the combat
  corpus is full of `turn_based_action` entries that dispatch nothing at all.
  **`/review` (Opus) found two perturbations that survived the gate — the eighth consecutive SR task
  where the review's real findings were holes in the *gate*, not bugs in the code, and both were the
  named shape: the author checks that the gate fires on the thing he was thinking about and never
  enumerates what it is not pointed at.** (a) The determinism gate exercised only the **battlefield**
  map: the three scenarios have one-owner hands and populate no graveyard or library at all, so
  reverting `sorted_zone_entries` on those three loops left the entire scripts suite green. Fixed with
  a two-owners-in-every-zone fixture — plus a guard asserting the fixture *still* has two owners in
  each map, because a one-owner `HashMap` has exactly one iteration order and the gate would silently
  become a no-op. (b) `card_names` never read `players.<p>.commander` / `.partner_commander` — the
  canonical place a commander is named, and the one card that can legally live only in the command
  zone; an approved script naming an undefined commander passed.
  **And exercising a documented asymmetry turned it into divergence (4).** The file had a comment
  saying the harness's `resolve_targets` used `filter_map` while a direct test aborts, and that no
  scenario exercised the difference. Writing that scenario failed immediately: `filter_map` silently
  **dropped** an unresolvable target, so a `cast_spell` naming a permanent that is not on the
  battlefield produced a `CastSpell` with an **empty `targets` vec** — a targeted spell cast with no
  target, CR 601.2c — and the script went green. Now returns `None` if any target fails. All 95
  approved scripts stayed green, so nothing was leaning on it. **A documented hazard that nothing
  executes is a hazard, not a note.**
  All three previously-surviving perturbations now fail exactly one test each, verified by re-running
  them.
  **Gates:** 3167 → **3178 tests** (+11, all this file's), 0 failed; clippy `--all-targets -D warnings`,
  `fmt --check`, `build --workspace` all clean.
  **Next session:** SR-9c (`scutemob-71`, golden-script triage) — it now has a concrete worklist above.
  Also worth someone's time: only **6 of `translate_player_action`'s 60+ `Command` shapes** are
  cross-validated (`pass_priority`, `play_land`, `tap_for_mana`, single-target `cast_spell`,
  `activate_ability`, `declare_attackers`). None of the alt-cost translations that give the function
  its 40+ parameters — convoke, delve, escape, kicker, bargain, casualty, splice, escalate, modal,
  mutate, ninjutsu — is covered. Adding a scenario is cheap: a JSON blob, a `direct` fn, a `Move`
  variant.

- 2026-07-10 — SR-9a (scutemob-69) — **done** — The 297 top-level `crates/engine/tests/*.rs`
  files are now **9 test targets** (`core`, `rules`, `combat`, `casting`, `primitives`, `scripts`,
  `mechanics_{a_d,e_l,m_z}`), each a `tests/<group>/main.rs` module root. Every file moved
  **verbatim** — no test body was edited. Layout, the rule for where a new test file goes, and all
  measurements: `docs/sr-9a-test-consolidation.md`.
  **Numbers** (`CARGO_INCREMENTAL=0`, before-tree = a worktree of `abe14f76` with its own
  cold-built target, so the comparison is apples-to-apples): warm rebuild after touching
  `engine/src/lib.rs` **34.2 s → 11.1 s** (median of 3); cold `cargo test --all --no-run`
  **39.8 s → 24.0 s**; `target/` **19 GB → 2.2 GB**. 3162 → 3167 tests (the +5 are the new gate's
  own), 0 failed, 4 ignored, 316 suites → 29. All four verification gates clean.
  **Read the shape of the before-column, not the median**: 22.6 / 34.2 / 38.2 s — each successive
  warm rebuild was *slower*, reproducibly, across two independent measurement passes. It is not
  thermals. Relinking 297 test binaries rewrites ~18 GB of executables per rebuild and writeback
  never catches up. After: 15.0 / 11.0 / 11.1 s, the normal page-cache warm-up shape. The 8.6×
  disk reduction is the same fact as the speedup, and it is the fact CI cares about — SR-1's
  `Bus error` was this, at 68 GB, on a 89 GB runner.
  **New gate:** `tests/no_stray_test_binaries.rs` (5 tests, the only top-level test file, and it
  exists to stay the only one). **Demonstrated adversarially, eight attacks** — three of which are
  the reason the gate is not decoration, because in each the ordinary suite reports success while
  coverage evaporates: with `combat/melee_stub.rs` present but not `mod`-declared,
  `cargo test --test combat` prints `ok. 75 passed; 0 failed` while an `assert!(false)` inside it is
  never compiled; deleting `mod combat_harness;` prints `ok. 69 passed; 0 failed` while **six real
  tests silently cease to exist**; and `#[cfg(feature = "never")] mod combat_harness;` does the same
  while leaving the `mod` line right there to read. A `mod` line is one easily-lost token and losing
  it converts a test file into a text file. All now fail loudly, naming the file. (`mod ghost;` with
  no file is caught by `rustc` anyway — recorded as such, because a demo that stops at a compile
  error measures the compiler, not the gate.)
  **Hazards discovered:**
  (a) **`include_str!` is file-relative, not manifest-relative.** Three tests
  (`keyword_registry`, `pending_trigger_shape`, `pb_ac9_wheel_and_misc`) reach into sibling crates'
  sources and needed one more `../`. `env!("CARGO_MANIFEST_DIR")` paths and `run_all_scripts.rs`'s
  cwd-relative `Path::new("../../test-data/…")` did **not** move. rustc catches the former; nothing
  would have caught a silent mis-resolution.
  (b) **A crate root and a module differ for `dead_code`.** `pub` items unused at a test crate's
  root are reachable; the same items one level down inside `mod` are dead. Surfaced exactly once, on
  `AssertionMismatch` — whose fields are read only through the `include!` copy that
  `run_all_scripts` makes of `script_replay.rs`. That `include!` is now pointless duplication inside
  one binary (it compiles `script_replay.rs` twice and runs its 4 unit tests twice); removing it
  would drop 4 tests from the count, and the acceptance criterion is that the count not move, so it
  is written up as a wart rather than fixed.
  (c) **Do not put a cargo `target/` under `/tmp` on this box** — `/tmp` is a **16 GB tmpfs**. A
  throwaway comparison worktree there filled it, the cold build died at 31.6 s with ENOSPC and
  `rc=0` (output was piped to `/dev/null`), and every subsequent `Bash` tool call failed silently
  because the harness could not write its own output file. Nearly banked a fabricated "31.6 s cold
  build" number. Use `/home/skydude/…` for scratch worktrees.
  **`/review` (Opus) returned 3/3 PASS, 0 HIGH, 0 MEDIUM, 4 LOW — and for the seventh consecutive
  SR task every substantive finding was a hole in the *gate*, not a bug in the code.** The
  declaration check was textual, so three distinct ways to satisfy it while still deleting coverage
  survived: `#[cfg(feature = "never")] mod foo;` (declared, compiled out — verified: `--test combat`
  → `ok. 69 passed`), `#[path = "elsewhere.rs"] mod foo;`, and a nested `<group>/sub/foo.rs` that a
  one-level directory read never sees. A fourth, `pub mod foo;`, failed the *wrong* test with a
  misleading message. Fixed not by teaching the parser each attack but by shrinking the grammar:
  `group_main_rs_declares_modules_and_nothing_else` forbids everything that is not a bare `mod x;`,
  and `group_dirs_are_flat` forbids the nesting. Each fix was verified by re-running its attack.
  **And the pattern generalized one level further: the hole was also in the *demonstration*.**
  Attack 3 as first written deleted `mod melee;` from `combat/main.rs`, where no such module exists
  (melee is in `mechanics_m_z`); `sed` matched nothing, exited 0, and the gate "passed" the attack.
  A demo that passes because it attacked nothing is indistinguishable from a gate that works. Next
  SR author: after writing the attack, assert that the attack *changed something* before you trust
  that surviving it means anything.
  **Next session:** SR-9b (`scutemob-70`, harness-vs-direct equivalence) or SR-9c (`scutemob-71`,
  golden-script triage). SR-9b now has an obvious home — `tests/scripts/` — and should note that
  `run_all_scripts`'s `include!` of `script_replay.rs` is the shadow-implementation seam it is
  chartered to cross-validate.

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
