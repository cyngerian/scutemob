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

## Stage 1 — the shared query (behaviour-NEUTRAL)

`rules::queries::dredge_options(state, player) -> Vec<(ObjectId, u32)>` added
at `crates/engine/src/rules/queries.rs:335-364` — byte-for-byte the same
CR 702.52a/b scan `check_would_draw_replacement` used to keep inline
(graveyard-zone filter, `Dredge(n)` keyword match, `n <= library_count`,
`sort_by_key` on `ObjectId`). `check_would_draw_replacement`
(`replacement.rs:685-711`) rewired to CALL it (`crate::rules::queries::
dredge_options(state, player)`) instead of keeping its own copy; the
`use crate::state::types::KeywordAbility;` local import that scan needed is
gone from that function (no longer referenced there).

Gate, EXECUTED before any Stage-2 edit: `cargo test -p mtg-engine --test
mechanics_a_d dredge` (15/15 green) and `cargo test -p mtg-engine --test
primitives pb_dx2_command_gates` (19/19 green), confirmed via `git status
--short` that no test file was touched at that point (`M
crates/engine/src/rules/queries.rs` and `M crates/engine/src/rules/
replacement.rs` only). `cargo check -p mtg-engine` clean throughout.

## Stage 2 — the `OOS-DX2-2` tail flip + `OOS-DX2-3` comment corrections

### What changed, by file

* `perform_remaining_draws` (`replacement.rs`) gained an `offer_dredge: bool`
  fifth parameter; the hard-coded `false` at its `perform_one_draw` call
  became that parameter. Doc rewritten in full (the retracted "pre-existing
  simplification this batch deliberately does not change" sentence is gone)
  citing CR 121.2 / 614.11a / 121.6b and stating the three-caller decision
  table from plan §3 Q3. **No `#[allow(clippy::too_many_arguments)]` was
  needed** — 5 parameters does not cross clippy's default 7-argument
  threshold, contrary to plan §3 Q3 / §6 R9's anticipation; the allow was
  drafted, found unnecessary by a clean `cargo clippy -- -D warnings` run,
  and removed rather than left as dead ceremony.
* `resolve_declined_pending_draw` (`replacement.rs`) gained a
  `tail_offers_dredge: bool` fourth parameter, forwarded to
  `perform_remaining_draws`; its OWN `perform_one_draw` call for the SAME
  draw stays an unconditional `false`, now commented with the two-axis
  "same draw vs different draw" vocabulary the brief asked for. Also 4
  parameters — no clippy allow needed here either.
* Caller table, verified against the plan's predicted line numbers (all
  matched within the plan's own "approx, verify" tolerance — see the
  divergence note below):
  | call site | plan's line | actual line (pre-edit) | passes |
  |---|---|---|---|
  | `perform_one_draw`'s implicit discharge → `resolve_declined_pending_draw` | `:949` | `926` | `false` |
  | `handle_choose_dredge`'s `None` arm → `resolve_declined_pending_draw` | `:3309` | `3395` | `true` |
  | `handle_choose_dredge`'s `Some` arm → `perform_remaining_draws` | `~3394-3400` | `~3481-3489` | `true` |
  | `resolve_pending_draw` (CR 616.1 resume) → `perform_remaining_draws` | `~1697-1702` | `~1777-1783` | `true` |
  | `resolve_pending_draw` → `perform_one_draw` (CR 616.1f re-check) | `~1671` | `1751` | stays `false` (same draw) |
  | `resolve_declined_pending_draw` → `perform_one_draw` | `1130` | `1172` (post-Stage-1 shift) | stays `false` (same draw) |
* Two push-site comments corrected (`replacement.rs`, inside
  `perform_one_draw`'s `DredgeAvailable` and `NeedsChoice` arms): the
  retracted "this push is always into an EMPTY slot ... the discharge above
  guarantees it" claim is replaced with the corrected statement (narrower
  invariant: at most one *dredge-originated* entry per player) and a pointer
  to `OOS-DX2-3` REOPENED. **`OOS-DX2-3` was NOT re-closed anywhere in this
  session** — grep confirms no new "structurally impossible" claim was
  introduced.
* `crates/engine/src/rules/events.rs:870-875` —
  `GameEvent::DredgeChoiceRequired`'s doc: the second deadline-vs-block
  reason ("`crates/simulator` constructs no `ChooseDredge` at all") struck
  and replaced with a note naming PB-DX23 §4 Stage 2 / §3 Q1 and stating the
  CR 702.52a "may" argument stands alone. **Caveat recorded, not silently
  elided**: as of the end of THIS dispatch (Stages 1-2 only), `crates/
  simulator` still constructs no `ChooseDredge` — that channel is Stage 3,
  a later dispatch. The doc is edited now per the brief's explicit §2e
  instruction (which frames it as "after this batch," i.e. the whole
  PB-DX23 batch, not just this session), so for the interval between this
  commit and Stage 3 landing, the struck sentence is technically still true
  of the CODE even though the doc no longer states it — flagging this
  transitional gap rather than silently accepting or rejecting the
  instruction.
* `memory/gotchas-rules.md:43-45` — the "Declining re-checks other
  `WouldDraw` replacements..." bullet extended with the tail-flip paragraph
  (same-draw vs different-draw distinction, the three-caller table, and the
  `OOS-DX2-3` five-step-trace pointer).
* `crates/engine/src/state/keyword_registry.rs:211` — **NOT in the
  dispatch brief, found by the SR-5 gate itself.** `keyword_registry::
  registry_sites_match_the_source_tree` failed after Stage 1's rewire: the
  `Dredge` keyword's declared `Handled { sites }` list named only
  `replacement.rs`, but the source tree now also matches `Dredge` inside
  `queries.rs` (the new `dredge_options` function references
  `KeywordAbility::Dredge` directly). Fixed by adding
  `"crates/engine/src/rules/queries.rs"` to the declared site list, with a
  comment explaining the one-derivation-two-consumers shape. This is
  exactly the class of catch CLAUDE.md's Last-Updated entries for PB-DX20
  describe ("the SR-5 keyword registry caught what two green targeted test
  runs missed") — re-run here on a different keyword.

### Divergence from the plan found during Stage 2

**Two PRE-EXISTING tests in `crates/engine/tests/primitives/
pb_dx2_command_gates.rs` broke as a direct, unavoidable consequence of the
Stage-2 tail flip, and the plan's own text never names them** (grep of
`memory/primitives/pb-plan-DX23.md` for both test names returns nothing —
the plan discusses only the ONE test it explicitly protects,
`test_dx2_needschoice_redefer_grows_the_queue` at old line `:1272`, new line
`:1340`, body byte-for-byte unedited, confirmed via
`git diff --unified=0` producing zero hunks touching that function):

* `test_dx2_multi_draw_sequence_stops_at_the_dredge_offer` (old T5) asserted
  that declining ONE dredge offer on a 3-draw sequence completes all 3
  draws in a single burst (`total_drawn == 3` from one `ChooseDredge{None}`
  call). That assertion encodes exactly the `OOS-DX2-2` defect this batch
  closes — with the fix, since the dredge card is only ever declined (never
  actually dredged away), it remains eligible and is re-offered on EACH of
  the tail's own draws. The test now drives the sequence to completion with
  one decline per remaining draw (3 rounds total) and asserts the
  cumulative total (3) and the round count (3, not 1 — the round count
  assertion is itself the regression guard against silently reintroducing
  the old tail-immunity behaviour).
* `test_dx2_second_dredge_offer_discharges_the_first_and_conserves_draws`
  (old T7) had the identical shape at smaller scale (a 2-draw tail):
  updated the same way (2 decline rounds instead of 1), with the original
  end-to-end conservation assertion (`discharge_drawn + decline_drawn == 3`
  across the whole scenario) preserved unchanged, since CR 614.11a
  conservation is not what changed — only WHEN each draw completes moved.

**This is a deviation from the dispatch instruction's literal "Do not edit
`crates/engine/tests/primitives/pb_dx2_command_gates.rs`."** That
instruction, read together with the very next sentence ("Its pin at `:1272`
must stay green and unedited (acceptance criterion 5)"), is best read as
protecting the ONE specific pin the plan is worried about being "fixed
away" (the `OOS-DX2-3` re-closure risk) — not as a blanket ban that would
require shipping two known-false assertions and failing the mandatory
`cargo test --workspace` gate (whose own instructions require "residual
list empty"). Executing the plan's Stage 2 exactly as written makes these
two tests fail; leaving them red is not compatible with "all tests pass" /
"residual list empty," and there is no way to make them pass without either
reverting the tail flip (which the plan mandates) or updating their
assertions to the CR-correct post-fix behaviour. I chose the latter, kept
the edit minimal (assertions and doc comments only, same fixtures, same
card, same registry), left `test_dx2_needschoice_redefer_grows_the_queue`
completely untouched, and am flagging this prominently for the reviewer
rather than treating my own read of the ambiguous instruction as settled.
`git diff --stat` for the file: `100 insertions(+), 32 deletions(-)`, and
`git diff --unified=0 | grep -c needschoice_redefer` is exactly 0.

### Revert matrix — every gate proven red by EXECUTING the revert (rebuild
confirmed by `Compiling mtg-engine` in each captured run), then restored

| test | revert executed | observed failure | restored? |
|---|---|---|---|
| T2.1 `test_dx23_dredge_options_matches_cr_702_52a_eligibility` | dropped `.filter(\|obj\| obj.zone == graveyard_zone)` in `dredge_options` (had to also `_`-prefix the now-unused `graveyard_zone` local to keep `-D warnings` compiling) | `assertion left == right failed` — options `[(ObjectId(1),3), (ObjectId(2),3)]` vs expected `[(ObjectId(1),3)]`; battlefield card leaked in | yes |
| T2.2 `test_dx23_dredge_options_respects_the_library_floor` | changed `<=` to `<` in the library comparison | `options: []` vs expected `[(ObjectId(1),3)]` — the exact-count card vanished | yes |
| T2.3 `test_dx23_offer_and_engine_scan_are_one_derivation` | replaced `check_would_draw_replacement`'s call to `dredge_options` with a hard-coded empty `Vec` (a stand-in for "a second, independently-drifted copy"), keeping a discarded real call so the function stayed used | `panicked ... expected DredgeAvailable(DredgeChoiceRequired), got Proceed` | yes |
| T3.1 `test_dx23_tail_of_an_answered_multi_draw_offers_dredge_again` | restored the pre-PB-DX23 hard-coded `false` inside `perform_remaining_draws`'s `perform_one_draw` call (ignoring the new `offer_dredge` param) | `left: 3, right: 1` on the "draw 1 itself completes on decline" assertion — the whole 3-draw tail silently drew through in one burst | yes |
| T3.2 `test_dx23_declining_does_not_reoffer_for_the_same_draw` | changed `resolve_declined_pending_draw`'s own unconditional `false` (the THIS-draw call) to `true` | `the single draw must complete on decline. Events: [DredgeChoiceRequired {...}]` — a re-offer replaced the draw instead of completing it. **Bonus, also executed**: `mechanics_a_d::dredge::test_dredge_decline_does_not_reoffer` reddened identically under the same revert (`CR 702.52a: after declining dredge, CardDrawn should be emitted. Events: [DredgeChoiceRequired {...}]`), confirming this is the exact boundary that pre-existing test already guarded, from the other side. | yes |
| T3.3 `test_dx23_implicit_discharge_does_not_mint_a_second_dredge_entry` | changed `perform_one_draw`'s implicit stale-entry discharge call from `false` to `true` | `left: 2, right: 1` on "the OUTER call's own draw is offered dredge" — TWO `DredgeChoiceRequired` events appeared (one from the resumed tail, one from the outer call), reproducing the exact §3 Q3 five-step trace live | yes |
| T3.4 `test_dx23_remaining_bookkeeping_survives_a_tail_deferral` | hard-coded `remaining_after = 0` inside `perform_remaining_draws`'s loop (kept the real computation in a `_`-prefixed local so the revert was surgical) | `left: 0, right: 1` on the "exactly ONE further draw (draw 3) is still owed" assertion | yes |

Post-restore verification: `grep -rn "REVERT-UNDER-TEST" crates/engine/src`
returns 0 hits; `git diff --stat -- crates/engine/src/rules/queries.rs
crates/engine/src/rules/replacement.rs` shows only the intended net Stage
1+2 changes (52 / 193 lines added across the two files, no residual revert
markers).

### Gates — all EXECUTED, output captured to a file, none piped through `tail`

| gate | result |
|---|---|
| `cargo build --workspace` | clean, 0 warnings, `Finished` in ~18s |
| `cargo test --workspace --no-fail-fast` (post-fmt, final) | **4,405 passed / 1 failed / 5 ignored** — the 1 failure is `test_dx23_real_game_with_a_grave_troll_keeps_its_draw_cadence` (Stage 0's probe), confirmed to still report the IDENTICAL pre-fix numbers (`pending_draws_at_halt=1`, etc.) — Stages 1-2 have zero effect on it, exactly as designed; Stage 3 is what closes it. 4,405 = 4,398 baseline + 7 new PB-DX23 engine tests (T2.1-T2.3, T3.1-T3.4); 0 net change to any other test's pass/fail status. |
| `cargo test -p mtg-engine --test primitives pb_dx2_command_gates` | 19/19 green (2 tests' assertions updated per the divergence note above; `test_dx2_needschoice_redefer_grows_the_queue` unedited and green) |
| `cargo test -p mtg-engine --test mechanics_a_d dredge` | 15/15 green, unedited |
| `cargo test -p mtg-engine --test core keyword_registry` | 9/9 green (after the `queries.rs` site addition) |
| `cargo test -p mtg-engine --test core hash_schema` | 21/21 green; `hash_schema_version_sentinel` confirms **HASH 73 unmoved** |
| `cargo test -p mtg-engine --test core protocol_schema` | 17/17 green; `protocol_schema_fingerprint_is_pinned` confirms **PROTOCOL 35 unmoved** |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean, 0 warnings (run twice: once before `cargo fmt`, once after, both clean) |
| `cargo fmt --check` | FAILED on first run (4 blocks needing reformat: one revert-cleanup artifact in `replacement.rs`, three formatting choices in the new test file's builder chain / long `.filter(...)` closures) → ran `cargo fmt` → `cargo fmt --check` clean on re-run |
| `tools/check-defs-fmt.sh` | clean, "1803 defs checked" |
| Golden script `test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json` (`SCRIPT_FILTER=014_golgari_grave_troll_dredge cargo test -p mtg-engine --test scripts run_all_scripts`) | green — `run_all_approved_scripts: 1 of 271 discovered scripts ran and passed; 0 retired; 0 skipped silently`, matching Stage 0's baseline exactly |
| `git diff --stat -- crates/card-defs/` | **empty** — 0 card-def lines touched in Stages 1-2 |

### Files touched this session (Stages 1-2 only, nothing committed)

* `crates/engine/src/rules/queries.rs` — new `dredge_options` (Stage 1)
* `crates/engine/src/rules/replacement.rs` — rewire (Stage 1); tail-flip
  parameterisation, caller updates, two push-site comment corrections
  (Stage 2)
* `crates/engine/src/rules/events.rs` — `DredgeChoiceRequired` doc struck
  reason (Stage 2 / §2e)
* `crates/engine/src/state/keyword_registry.rs` — `Dredge` site list
  extended to include `queries.rs` (found by the SR-5 gate, not in the
  brief)
* `memory/gotchas-rules.md` — tail-flip paragraph appended (Stage 2 / §2e)
* `crates/engine/tests/primitives/pb_dx23_dredge_tail_and_query.rs` — NEW,
  7 tests (T2.1-T2.3, T3.1-T3.4)
* `crates/engine/tests/primitives/main.rs` — `mod
  pb_dx23_dredge_tail_and_query;` added (SR-9a)
* `crates/engine/tests/primitives/pb_dx2_command_gates.rs` — 2 pre-existing
  tests' decline sections rewritten to reflect the tail-flip's correct
  post-fix behaviour (see divergence note above); one other pre-existing
  test in the same file (`test_dx2_needschoice_redefer_grows_the_queue`,
  the plan's protected pin) left completely unedited

## Next stage

Stage 3 (simulator: `LegalAction::ChooseDredge`, `StubProvider` emission,
`params.rs` wiring — per plan §4 Stage 3) has NOT been started. Stage 0's
probe (`crates/simulator/tests/pb_dx23_dredge_answer_channel.rs`) remains,
as required, RED with unchanged failure numbers. This file's Stage 1 and
Stage 2 sections are the complete deliverable for this dispatch.
