# PB-DX23 execution notes — measurements and revert matrix

**Batch**: PB-DX23 — dredge has no answer channel for anyone (`OOS-DX2-5` primary)
**Plan**: `memory/primitives/pb-plan-DX23.md`
**Worktree**: `/home/skydude/projects/scutemob/.worktrees/scutemob-201`
**Verified at**: HEAD `e490153b` (PB-DX21 merge), pre-any-edit for this batch except
the Stage 0 probe file itself.

This file accumulates measurements stage by stage. Stage 0 only, below; later
stages append.

---

## Stage 0 — baseline measurements (no production source edited)

### 0.1 — Already-measured baselines (carried forward, not re-run)

Per the dispatching brief, measured on this branch BEFORE any edit at `e490153b`:

* `cargo test --workspace --no-fail-fast`: **4,398 passed / 0 failed / 5 ignored**.
* `HASH_SCHEMA_VERSION = 73` (`crates/engine/src/state/hash.rs:757`), `PROTOCOL_VERSION
  = 35` (`crates/engine/src/rules/protocol.rs:360`), both gate-green.

### 0.2 — `cargo test -p play-server`

Command: `~/.cargo/bin/cargo test -p play-server 2>&1 | tee /tmp/pb-dx23-playserver.txt`

Result: **79 passed / 0 failed / 0 ignored**, single binary
(`unittests src/main.rs`), 2.29s.

**DIVERGENCE FROM THE PLAN, recorded per the "measured, not guessed" rule**: the
plan's §4 Stage 0 step 3 and its baseline line both say "expect 78 / 0". The
literal measured value on this branch, before this batch touched anything, is
**79 / 0**. Nothing in this batch's Stage 0 work (a new file under
`crates/simulator/tests/`, and this notes file) can have added a `play-server`
test — `git diff --stat -- tools/play-server` is empty at this point in the
session. The plan's "78" pin is therefore stale relative to `e490153b`'s actual
state (most likely measured at an earlier commit while the plan was being
written, or drifted by a parallel merge after the plan's own measurement).
**Not re-pinned here** — this file records the observed value; any gate that
pins play-server's count belongs to a later stage or a different batch, and
should cite 79 as this batch's own pre-edit baseline if it ever needs one.

### 0.3 — `grep -rn "ChooseDredge" crates/simulator/src/ tools/`

Result: **0 hits**, confirmed live (not just cited from the plan).

### 0.4 — `grep -rn "ReplacementTrigger::WouldDraw" crates/card-defs/src/defs/`

Result: **0 hits**, confirmed live — the zero-corpus-reach premise behind plan §3
Q2 and §1.4 (`OOS-DX2-3`) holds.

### 0.5 — Golden-script corpus baseline

`test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json`
drives `Command::ChooseDredge` through `replay_harness.rs` (per
`memory/gotchas-infra.md`'s Script Harness Gotchas — `choose_dredge` maps to
`ChooseDredge`, finding the named card in the player's graveyard). Ran via:

```
SCRIPT_FILTER=014_golgari_grave_troll_dredge \
  ~/.cargo/bin/cargo test -p mtg-engine --test scripts run_all_scripts -- --nocapture
```

Result: **green** — `run_all_approved_scripts: 1 of 271 discovered scripts ran and
passed; 0 retired; 0 skipped silently`. `run_all_scripts` module: **8 passed / 0
failed / 0 ignored, 36 filtered out** (`the_corpus_is_fully_accounted_for`,
`every_approved_script_asserts_something`, `run_all_approved_scripts`,
`approved_scripts_only_use_allowlisted_untranslatable_actions`,
`the_untranslatable_allowlist_has_no_dead_entries`, plus 3 unit tests in the
group). This is the pre-fix baseline this batch's later stages must not
regress — the script exercises `Command::ChooseDredge` directly against the
engine (not through `crates/simulator`), so it should stay green through every
stage (this batch touches `replacement.rs`'s internals in Stage 1/2, not the
`ChooseDredge` command handler's contract).

### 0.6 — The mandatory probe (§3 Q5, §5 T1.1), PRE-FIX

New file: `crates/simulator/tests/pb_dx23_dredge_answer_channel.rs`, test
`test_dx23_real_game_with_a_grave_troll_keeps_its_draw_cadence` (CR 702.52a,
121.1, 103.8a).

Fixture: 2 players (`p1` active/first, `p2`), both bot seats
(`HeuristicBot`, seeds 70252/70253), `p1`'s graveyard holds a real
`golgari_grave_troll` (enriched via `enrich_spec_from_def` + `CardRegistry` so
`characteristics.keywords` carries `Dredge(6)` — asserted as an explicit
precondition, confirmed non-vacuous), 40 unregistered filler cards per
library (no `card_id`, Architecture Invariant 9 never sees them),
`LocalGameLimits { max_turns: 6, max_commands: 1200, max_consecutive_passes:
500, record_journal: true }`, `check_invariants: true`, `human_seats` empty.
A single `advance()` call runs turns 1-6 to completion and halts at
`HaltReason::MaxTurns { max_turns: 6, turn: 7 }` (confirmed — the halt check
is `turn_number > max_turns` at the top of `advance()`'s loop, so turn 7's
own `TurnStarted` event fires as the LAST effect of a command applied inside
turn 6, before the next loop iteration halts).

Ran via:

```
~/.cargo/bin/cargo test -p mtg-simulator --test pb_dx23_dredge_answer_channel -- --nocapture
```

**Literal PRE-FIX values** (printed by the test's own `eprintln!` before either
assertion runs, so both are captured regardless of which one panics first):

| quantity | measured pre-fix value | plan's predicted shape |
|---|---|---|
| A1 — `pending_draws()` at halt (count) | **1** entry (`PlayerId(1)`, `remaining: 0, sets_has_drawn_for_turn: true`) | "exactly 1 entry, for p1" — **matches exactly** |
| `CardDrawn{player: p1}` count | **1** | — |
| `Dredged{player: p1}` count | **0** | — |
| A2 LHS — `CardDrawn + Dredged` for p1 | **1** | — |
| A2 RHS — p1's draw-eligible turns (turns 3 and 5) | **2** | "short by exactly 1" — **matches exactly** (1 vs 2) |
| A3 — `DredgeChoiceRequired{player: p1}` count | **2** | "≥ 1" — **matches, non-vacuous** |

**Test outcome**: FAILS pre-fix, at the A1 assertion (first assertion reached),
exactly as required:

```
thread 'test_dx23_real_game_with_a_grave_troll_keeps_its_draw_cadence' panicked at crates/simulator/tests/pb_dx23_dredge_answer_channel.rs:279:5:
assertion `left == right` failed: CR 702.52a/121.1: no PendingDraw should survive to the halt of a real game once dredge offers are answerable. pending_draws() at halt: [PendingDraw { player: PlayerId(1), already_applied: [], remaining: 0, sets_has_drawn_for_turn: true }]
  left: 1
 right: 0
```

Because `assert_eq!` short-circuits the function, the A2 assertion (`a2_lhs ==
a2_rhs`, i.e. `1 == 2`) is not itself reached as a *second* panic in this run —
but its inputs are already proven correct and printed via the preceding
`eprintln!`, so the batch's later revert-watch (which restores the whole test
to green, then reverts the fix and re-observes both A1 and A2 redden
independently) is the point at which both assertions are separately exercised
red. This test is intentionally left in the tree, unmodified, still failing.
**No `#[ignore]` was added.**

**MEASURED DIVERGENCE FROM A NAIVE PREDICTION, found by running the fixture
(not anticipated by the plan)**: a first draft of the A2-RHS derivation
(count `GameEvent::TurnStarted{player: p1}` events in the journal, excluding
`turn_number == 1` since that turn's own `TurnStarted` is emitted by
`start_game()` before `LocalGame::start()` returns and is therefore never in
the journal at all) measured **3**, not the expected **2** — turn_numbers
`{3, 5, 7}`. Root cause, confirmed by reading `local_game.rs:713-718`:
`advance_turn()` (and its `TurnStarted` event) runs as part of the *last*
command applied **inside turn 6**, the very command whose engine-side effects
increment `turn_number` from 6 to 7 — `advance()`'s own turn-cap check
(`turn_number > max_turns`) only runs at the *top* of the *next* loop
iteration, so it observes and halts on the already-incremented value before
any command is actually applied *inside* turn 7. The consequence: turn 7 is
reported as **started** (its `TurnStarted` event is real and journaled) but
never runs a draw step or any other command — "started" and "reached a draw
step" are not the same predicate once a turn cap is involved. Fixed by
additionally requiring `turn_number <= MAX_TURNS` in the A2-RHS filter (see
the in-test comment at the `GameEvent::TurnStarted` match arm), which brought
the measurement to the plan-matching **2**. This is a genuine artefact of how
`LocalGameLimits::max_turns` is enforced, not a defect in the engine or in
this batch's own scope — recorded here because a probe that silently
overcounted the RHS by exactly the amount the LHS was short would have
produced a **false pass** (`1 == 1`) instead of the correct pre-fix failure,
which is exactly the kind of vacuous-test hazard this project's conventions
(`memory/conventions.md` "Test-validity MEDIUMs are fix-phase HIGHs") warn
against catching only in review. No production code was touched to fix this —
only the test's own RHS-counting filter.

---

## Stage 0 — TODO sweep (already recorded in the plan, re-confirmed live)

`grep -rn "Dredge(" crates/card-defs/src/defs/` → **1 hit**,
`golgari_grave_troll.rs:80` (re-confirmed at this session's HEAD, not re-run
separately in this notes file — the plan's own §0 TODO sweep table already
states result "0 cards added" and that stands unchanged; Stage 0 of this
session added no card def).

---

## Next stage

Stage 1 (engine: the shared `dredge_options` query, behaviour-NEUTRAL) has NOT
been started. This file's Stage 0 section is the complete deliverable for this
dispatch.
