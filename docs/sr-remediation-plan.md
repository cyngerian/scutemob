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
| 1 | scutemob-53 | SR-1: Revive CI | S | Do first. Everything after benefits from a working gate. |
| 2 | scutemob-54 | SR-2: Registry gate (invariant #9) | M | Supersedes archived scutemob-48. Do before any further card-authoring waves. |
| 3 | scutemob-55 | SR-3: Seal GameState | M–L | Wide blast radius (tui, replay-viewer, simulator, network, testing). See collision rules below. |
| 4 | scutemob-56 | SR-4: Silent-failure sweep | M–L | Mechanical but large; classification work. Can run any time after SR-1. |
| 5 | scutemob-57 | SR-5: KeywordAbility catch-all audit | M | Pairs naturally with SR-4 (same files). |
| 6 | scutemob-58 | SR-6: Extract card-defs crate | M | Wide blast radius. Coordinate with card authoring (collision rules). |
| 7 | scutemob-59 | SR-7: PendingTrigger → TriggerData cutover | M | Requires HASH_SCHEMA_VERSION bump; read `state/hash.rs` header first. |
| 8 | scutemob-60 | SR-8: Protocol versioning policy | M | Hard blocker before M10's first networked client. Design + implement. |
| 9 | scutemob-61 | SR-9: Test infra consolidation | L | Three sub-items (binaries / equivalence test / script triage). **Split into 2–3 ESM subtasks at dispatch time.** |
| 10 | scutemob-62 | SR-10: Dependency & lint hygiene | S–M | Four independent chores; safe filler work between larger tasks. |

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

- **SR-1:** the two historical CI runs failed at `cargo fmt --check` (fmt is
  clean as of 2026-07-10). Clippy and tests have *never* run in CI — expect
  first-run surprises from the Ubuntu runner (missing system libs in tool
  crates, runtime). Fallback: gate on `cargo test -p mtg-engine` first. The raw
  `actions/cache` of `target/` will blow the 10 GB cache budget — replace with
  `Swatinem/rust-cache@v2` once green.
- **SR-1 scope cap (user direction, 2026-07-10): keep CI cheap.** One Ubuntu
  job, fmt + clippy + tests, nothing more. No OS matrix, no nightly benchmark
  runs, no Tauri builds — long/expensive Actions are worthless this far from
  playable alpha; revisit the full matrix around M10/M11. Related drift to
  reconcile: `.claude/CLAUDE.local.md` describes CI as Ubuntu/Windows/macOS
  with nightly benchmark regression alerts — that is aspirational, not real
  (actual workflow is a single Ubuntu job). Correct that doc to describe the
  minimal CI as the intended current state, noting the matrix as an M10/M11
  follow-up.
- **SR-2:** cross-check marker conventions with `tools/authoring-report.py` so
  the gate and the authoring report share one source of truth (carried over
  from archived scutemob-48).
- **SR-3:** the testing harness (`testing/replay_harness.rs`) constructs state
  directly and is `pub` (shared with the replay viewer) — it will need explicit,
  documented constructors, not raw field access.
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

- (no sessions yet)
