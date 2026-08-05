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

## Stage 3 — simulator: the variant, the provider, `params.rs`

* `LegalAction::ChooseDredge { card: Option<ObjectId>, mill: u32 }` added to
  `crates/simulator/src/legal_actions.rs`'s enum, doc citing CR 702.52a and
  the plan §3 Q1/Q2 arguments verbatim.
* `StubProvider::legal_actions`: the emission block appended immediately
  after the `PayRecover` loop and before `is_main_phase`'s computation
  (exactly the plan's named insertion point). Guard is the conjunction
  `state.pending_draws().iter().any(|p| p.player == player)` AND
  `!dredge_options(state, player).is_empty()`; when both hold, pushes
  `ChooseDredge { card: None, mill: 0 }` plus one `Some` entry per
  `dredge_options` result, in that function's own (`ObjectId`-sorted) order.
* `crates/simulator/src/params.rs::action_to_command_with_params`: one arm,
  `LegalAction::ChooseDredge { card, .. } => Ok(Command::ChooseDredge {
  player, card: *card })`. Left OUTSIDE the nine-arm parameterisation
  allowlist (`:271-286`), confirmed by reading — no edit needed there since
  the allowlist is closed over exactly nine named variants and `ChooseDredge`
  is not one of them; any `params` field announced alongside it is refused
  with `ParamError::UnsupportedParam` by the existing pre-match guard.

Gate: `cargo build --workspace` after the three edits pointed at exactly the
sites Stage 5 predicts (see below) — no more, no fewer.

## Stage 4 — bot policy

`crates/simulator/src/heuristic_bot.rs::score_action`: `_player` promoted to
`player` (line `:186`, now read by the new arm). Two arms added at the end
of the match, verbatim per plan §3 Q4:

* `ChooseDredge { card: None, .. } => 2` (the `PayEcho { pay: false }`
  precedent).
* `ChooseDredge { card: Some(_), mill } => if library_count >= 2 * mill { 3 }
  else { 0 }`, `library_count` read via `state.zones().get(&ZoneId::Library(
  player))` (mirrors the provider's own zone-length idiom elsewhere in this
  file).

## Stage 5 — the exhaustive-match sweep

`cargo build --workspace` pointed at exactly three FORCED sites, matching
the plan's table one-for-one (line numbers shifted from the plan's estimate,
as expected — the plan itself said "let `cargo build --workspace` confirm"):

| file | match | action taken |
|---|---|---|
| `tools/play-server/src/view.rs::action_kind` | `"ChooseDredge"` |
| `tools/play-server/src/view.rs::action_object` | `LegalAction::ChooseDredge { card, .. } => *card` |
| `tools/play-server/src/view.rs::action_label` | `format!("Dredge {} (mill {mill})", card(*c))` / `"Decline dredge — draw normally"` |

**One compile-forced site the plan's table did NOT name, found only by
running `cargo clippy --workspace --all-targets`** (a plain `cargo build
--workspace` does not compile test targets by default and did not surface
it): `crates/simulator/tests/local_game_playthrough.rs::kind_of`, the S8
scripted-playthrough helper's own exhaustive `LegalAction` match (a
`#[test]`-only file, so `cargo build` never reaches it). Added
`LegalAction::ChooseDredge { .. } => "ChooseDredge"`, documented as a no-op
for that policy (step 4's `PassPriority` always matches first while a
dredge offer stands, since Q1 made it an ordinary, non-exclusive action).
This is exactly the class of gap `cargo build --workspace` is known to miss
per CLAUDE.md's Milestone Completion Checklist ("catches missed match arms
... that `cargo check` misses") — here the miss was one level further,
inside a `#[cfg(test)]`-gated integration test, and clippy's
`--all-targets` flag is what closed it.

**Confirmed NOT compile-forced, by reading** (matching the plan's second
list exactly, no surprises): `view.rs::action_needs_x`, `action_modes`,
`action_target_requirements`, `target_query_source`, the combat-options
match, `blocking_decision_view` (`_ => None`), `api.rs::
validate_combat_params` / `validate_decision_params` (both catch-all).
`tools/tui/` compiled with zero changes (`cargo build --workspace` never
touched it after the `mtg-simulator` recompile) — confirms `OOS-DX23-3`
(the TUI gets no dredge channel) without needing a new probe.
`crates/view-model` does not depend on `crates/simulator` and was untouched.

## Stage 7 (T4/T5 test rows) — revert matrix, all EXECUTED and watched RED,
rebuild confirmed each time (`Compiling mtg-simulator` / `Compiling
play-server` observed), then restored and re-confirmed GREEN

| test | revert executed | observed failure | restored? |
|---|---|---|---|
| T4.1 `test_dx23_provider_offers_decline_plus_one_per_eligible_card` | dropped the `card: None` push in the provider's dredge block | `dredge actions: [ChooseDredge { card: Some(ObjectId(1)), mill: 6 }]` — decline assertion fails | yes |
| T4.2 `test_dx23_provider_offers_nothing_when_no_dredge_card_is_eligible` | removed the `options.is_empty()` guard | `Offered: [ChooseDredge { card: None, mill: 0 }]` — the exact Q2 re-defer-loop bait reappears | yes |
| T4.3 `test_dx23_provider_is_silent_while_a_blocking_decision_stands` | duplicated the dredge emission block to run BEFORE the `blocking_decision()` early return | `Offered: [ChooseDredge { card: None, mill: 0 }, ChooseDredge { card: Some(ObjectId(1)), mill: 6 }, DiscardToHandSize {...}]` — both dredge options appear alongside the cleanup discard | yes |
| T4.4 `test_dx23_every_offered_action_is_engine_accepted` | added an extra unconditional push offering a real LIBRARY object (not a graveyard/Dredge card) as `Some` | `error=Some(InvalidCommand("dredge card ObjectId(2) is not in PlayerId(4401)'s graveyard (zone: Library(PlayerId(4401)))"))` — engine's own independent validation catches it, proving the assertion is a real discriminator | yes |
| T4.5 `test_dx23_heuristic_bot_declines_rather_than_milling_itself_out` | dropped the `2 *` margin multiplier (`library_count >= mill` instead of `>= 2*mill`) | `Chose: ChooseDredge { player: PlayerId(4501), card: Some(ObjectId(1)) }` — bot mills itself at library == mill == 6 | yes |
| T5.1 `test_dx23_browser_can_answer_a_dredge_offer` | disabled the provider's whole dredge conjunct (`if false && ...`) | drive loop exhausts the game (`GameOver` at turn 86, human lost to Bot-2's commander damage) without the offer ever appearing — `the game ended at step 778 without ever offering a named ChooseDredge option` | yes |
| Stage-0 probe T1.1 (re-confirmed, not re-executed fresh — Stage 3 is what turns it green) | N/A, see above | goes GREEN once Stage 3 lands: `pending_draws_at_halt=0, card_drawn_p1=1, dredged_p1=1, a2_lhs=2, a2_rhs=2, dredge_choice_required_p1=1` — non-vacuous (a real dredge occurred, not just declines) | n/a |

Post-restore verification: `grep -rn "REVERT-" crates/simulator/src
tools/play-server/src` returns 0 hits; every touched production file
matches its pre-revert state (confirmed by re-running the full suite green
after each restore, not just by eyeballing the diff).

## T5.1 — HTTP fixture notes

**Seed had to be swept, exactly as the plan's Q6/budget-note anticipated.**
First draft used an unswept seed (`235_023`) with p1 and p2 both dealt an
identical 99-card `mono-green` deck (98 Forests + Golgari Grave-Troll,
commander `azusa-lost-but-seeking` — {2}{G}, chosen ONLY for CR 903.5c color
identity, +2 land plays as an incidental but harmless side effect since both
players get the identical deck). **That draft never reached a dredge
offer at all** — with the Troll buried deep in a near-homogeneous 98-card
library and no interaction beyond commander-damage combat, the two
identical decks raced to a mutual near-deck-out, and the game ended (Bot-2
lost to `LibraryEmpty` at turn 96 / step 866) with the Troll never drawn by
either seat. Root-caused by inspection of the `game_over` JSON payload
(`graveyard_size: 0` for the human — the Troll was never cast, so never
died, so never dredge-eligible).

**Fix: swept seeds 0..2000 with a throwaway `#[test]` (`session::new_game`
+ direct `state.objects_in_zone(&Hand(p1))` inspection, NOT going through
HTTP), looking for the FIRST seed that deals the Troll straight into p1's
opening 7-card hand** — the same `ui1_deck`/`UI1_SEED` precedent
("`main_deck[0]` lands in the opening hand at the right seed, verified by
sweep, not assumed"). Seed **1** hit on the first attempt (the sweep
printed ~90 hits between 0 and 2000; used the lowest). The throwaway sweep
test was deleted after use — `grep -rn scratch_sweep` in `tools/play-server/
src/main.rs` returns 0 hits.

With seed 1, the shipped test completes in **0.17s** wall time — no long
drive was needed once the Troll started in the opening hand (cast on curve,
dies to CR 704.5f as a 0/0, dredge offer arrives on the very next draw
step). `max_steps: 4_000` in the drive loop is generous headroom, not a
measured requirement.

**play-server test count**: 79 (Stage-0-measured pre-batch baseline, itself
a divergence from the plan's stale "78" pin — see the Stage 0 section
above) → **80** (+1, `test_dx23_browser_can_answer_a_dredge_offer`),
confirmed by `cargo test -p play-server` (80 passed / 0 failed).

## Full-suite gates — all EXECUTED after Stage 5 + the T4/T5 tests, output
captured to a file, none piped through `tail`

| gate | result |
|---|---|
| `cargo build --workspace` | clean, 0 warnings |
| `cargo test --workspace --no-fail-fast` (final, post-`cargo fmt`) | **4,412 passed / 0 failed / 5 ignored**. Delta over the Stage-2 collect's **4,405 / 1 failed / 5 ignored**: +6 new tests (T4.1-T4.5 in `crates/simulator/tests/pb_dx23_dredge_answer_channel.rs`, +1 `test_dx23_browser_can_answer_a_dredge_offer` in `tools/play-server/src/main.rs`) and the Stage-0 probe flipping from failing to passing (no count change from that flip alone). Arithmetic note: 4,405 + 6 = 4,411, one short of the measured 4,412; the extra +1 was not traced to a specific PB-DX23 edit (no other test file changed) and is flagged here rather than silently rounded away -- possibly a pre-existing count fluctuation orthogonal to this batch (e.g. a doctest or parallel-enumeration artifact), not re-investigated further since the only load-bearing fact -- **0 failed, residual list empty** -- is independently confirmed twice (`/tmp/pb-dx23-full-suite.txt` and `/tmp/pb-dx23-full-suite-final.txt`, both `grep -c FAILED` == 0). |
| `cargo test -p mtg-engine --test core hash_schema` | 21/21 green; `hash_schema_version_sentinel` confirms **HASH 73 unmoved** |
| `cargo test -p mtg-engine --test core protocol_schema` | 17/17 green; `protocol_schema_fingerprint_is_pinned` confirms **PROTOCOL 35 unmoved** |
| `cargo test -p play-server` | **80 passed / 0 failed** (79 baseline + 1) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean, run twice (pre- and post-`cargo fmt`), both clean |
| `cargo fmt --check` | FAILED once (4 blocks in the new test file needing reformat, all long builder-chain / assert-macro lines) → `cargo fmt` → `cargo fmt --check` clean on re-run |
| `tools/check-defs-fmt.sh` | clean, "1803 defs checked" |
| `python3 tools/authoring-report.py` | ran; **62.8% (1,133/1,803) unmoved** -- body differs only in self-dating fields (generated timestamp, HEAD sha, branch name, rolling 7-day commit-touch count, recent-commits list), confirmed by reading the diff before reverting it; `git checkout -- docs/authoring-status.md docs/authoring-status-missing.txt docs/authoring-status-prev.json` afterward so the working tree carries no unrelated doc noise |
| golden script `replacement/014_golgari_grave_troll_dredge.json` | green -- `run_all_approved_scripts: 1 of 271 discovered scripts ran and passed; 0 retired; 0 skipped silently`, byte-identical outcome to the Stage 0 baseline |
| `git diff --stat -- crates/card-defs/` | **empty** -- 0 card-def lines touched in Stages 3-5 |

## R1 ratchets (fuzz seed drift) — checked explicitly, all UNMOVED

| gate | file | result |
|---|---|---|
| `test_dx32_sr38_bot_rejection_rate_is_ratcheted` | `pb_dx32_fuzz_output.rs` | green, aggregate 6.909 per mille (pin 40) -- unmoved |
| `test_dx32_random_bot_waste_ratio_is_bounded` | same | green, aggregate 92% (pin 95%) -- unmoved |
| `test_dx32_orphaned_tokens_are_transient_and_the_end_state_is_clean` / `test_dx32_distinct_collapses_checkpoint_weighting` | same | green, seed-2 transient counts unchanged |
| `test_dx32_a_fuzz_run_reaches_at_least_one_served_row` | same | green, reached partition `{"discard_cards","scry","search_library","triggered_targets"}`, never-reached `{"surveil"}` -- identical partition, unmoved |
| `heuristic_pools_emptied_is_pinned` | `sim5_bot_cast_discipline.rs` | green, `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` pin unmoved |
| `test_s8_scripted_human_playthrough_is_clean_on_five_seeds` | `local_game_playthrough.rs` | green, all 5 seeds; **diagnosis**: none of seeds 1/7/42/1234/9001's `kinds` sets contain `"ChooseDredge"` -- no dredge offer arose in any of the five S8 seeds (none deals `golgari_grave_troll` to a reachable graveyard within the drive), so this ratchet correctly did not move. Per R1's rule ("a moved pin is a FINDING first"): here nothing moved, and the diagnosis (checked, not assumed) is that these five recorded seeds' deals simply never touch the new channel. |

**play-server's `DeckSource::Fixed` seeded pins** (`UI1_SEED`/`SIM1_SEED`/
`UI2_SEED`/`UI6_SEED`/`COMBAT_SEED`/`TARGET_SEED`/`UI3_SPLIT_COMBAT_SEED`/
`DISTINCTIVE_SEED`): confirmed **unmoved** -- the full `cargo test -p
play-server` run (80/0) includes every test built on those fixtures and
none reddened; none of those fixed decks contains `golgari-grave-troll`
(mono-black/-white/etc. `UI1_COMMANDER`/`UI6_SEARCHER` decks per those
fixtures' own doc comments), so this is exactly the "must NOT move at all"
case the plan named, and it held.

## Files touched this dispatch (Stages 3-5 + T4/T5 tests, nothing committed)

* `crates/simulator/src/legal_actions.rs` -- `ChooseDredge` variant + doc
  (Stage 3a); `StubProvider` emission block (Stage 3b)
* `crates/simulator/src/params.rs` -- one arm (Stage 3c)
* `crates/simulator/src/heuristic_bot.rs` -- `_player` -> `player`; two
  score arms (Stage 4)
* `tools/play-server/src/view.rs` -- three arms: `action_kind`,
  `action_object`, `action_label` (Stage 5)
* `crates/simulator/tests/local_game_playthrough.rs` -- one arm in the S8
  scripted-playthrough's `kind_of` helper (Stage 5, found by clippy
  `--all-targets`, not in the plan's table)
* `crates/simulator/tests/pb_dx23_dredge_answer_channel.rs` -- 5 new tests
  (T4.1-T4.5) + 3 shared helpers (`pass_all`, `build_single_dredge_offer_state`,
  `drive_to_a_blocking_decision`) appended after the existing T1.1
* `tools/play-server/src/main.rs` -- T5.1 (`test_dx23_browser_can_answer_a_dredge_offer`)
  plus its own deck/install/hand/drive helpers, all prefixed `t5_dx23_`

## Deviations from the plan's literal text

1. **Stage 5's compile-forced-sites table missed one site**: the S8
   scripted-playthrough test file's own exhaustive `LegalAction` match
   (`local_game_playthrough.rs::kind_of`). `cargo build --workspace` does
   not compile test targets, so this was only caught by running `cargo
   clippy --workspace --all-targets -- -D warnings`. No production
   behavior depends on this arm (the policy never chooses `ChooseDredge`,
   documented inline); it exists purely so the match stays exhaustive.
2. **T5.1's seed could not be picked arbitrarily** -- the plan's own §3 Q6
   budget note anticipated this ("if the drive proves impractical..."); a
   sweep (not a guess) found seed 1 puts the Troll in the opening hand,
   avoiding the ~90-turn mutual-deck-out race an unswept seed hit on first
   attempt.
3. ~~**Full-suite count arithmetic has an unreconciled +1**~~ — **RESOLVED at
   the Stage 3-5 collect, and the resolution is worth recording because the
   mistake is an easy one to repeat.** The Stage 2 run reported
   `4,405 passed / 1 failed`; that is **4,406 tests**, not 4,405 — the
   failing Stage 0 probe is a test too, and reading the *passed* column as a
   *total* is what produced the phantom `+1`. The full reconciliation:

   | | tests | running total |
   |---|---|---|
   | pre-edit baseline (measured on this branch at `e490153b`) | 4,398 | 4,398 |
   | Stage 0 — the mandatory probe (committed RED) | +1 | 4,399 |
   | Stages 1-2 — engine probes (T2.1-2.3, T3.1-3.4) | +7 | 4,406 |
   | Stages 3-5 — T4.1-4.5 (simulator) + T5.1 (play-server) | +6 | **4,412** |

   Measured post-edit: **4,412 / 0 / 5**, residual list empty. Exact. There is
   no residual and nothing was rounded away.

Everything else matched the plan's predictions exactly: the three named
`view.rs` sites, the `params.rs` allowlist exclusion, the Q1/Q2 provider
placement and guard, the Q4 bot-policy formula, zero card-def lines, zero
wire changes (HASH 73 / PROTOCOL 35 both gate-executed and confirmed
unmoved), and the TUI's total non-involvement (`OOS-DX23-3` re-confirmed by
observation rather than assumed).

---

## Fix cycle — review `memory/primitives/pb-review-DX23.md` (0 HIGH / 4 MEDIUM /
9 LOW, all 13 taken)

**Task**: `scutemob-201` fix cycle, same worktree. **Baseline going in**:
4,412 / 0 / 5, PROTOCOL 35 / HASH 73 gate-executed and unmoved (both
re-confirmed, not re-derived, below).

### S1 (MEDIUM) — the real defect: the Q2 guard was graveyard-keyed, the
entry it answers is FIFO-keyed

**Taken as the third option** (a real third conjunct on the provider's
guard), NOT the reviewer's rejected first suggestion (a `RepeatKey::
ChooseDredge` bot-side cap) — that shape is exactly what PB-DX21
(`scutemob-200`) deleted for the once-per-combat attack loop, and the
durable lesson from that batch (SR-38: suppress a bad offer at the offer
layer, not with a bot-side damper) applies here without modification.

**What changed**: `crates/simulator/src/legal_actions.rs`'s dredge-offer
block, previously `if let Some(_) = pending_draws().find(...) { if
!options.is_empty() { ...push... } }`, now also computes, before pushing:

```rust
let fifo_already_applied: std::collections::HashSet<_> =
    fifo.already_applied.iter().copied().collect();
let fifo_decline_would_redefer = matches!(
    mtg_engine::rules::replacement::check_would_draw_replacement(
        state, player, &fifo_already_applied, false,
    ),
    mtg_engine::rules::replacement::DrawAction::NeedsChoice(_)
);
if !options.is_empty() && !fifo_decline_would_redefer { ...push... }
```

`fifo` is the FIRST `PendingDraw` matching `player` (`state.pending_draws()
.iter().find(...)`) — the SAME entry `handle_choose_dredge`'s `None` arm
will actually answer (its own `position()` call is FIFO-oldest, identical
selection). The check re-derives, ON CURRENT STATE, what declining that
entry would do; if it would re-raise `NeedsChoice`, the whole offer
(including the `Some` entries) is withheld.

**Honesty requirement, followed**: the in-source comment states this is a
CONSERVATIVE APPROXIMATION, not an equality — the real resume runs after any
EARLIER queued entry's own discharge has already recursed and mutated
state, which this snapshot-check cannot see. It is exact only when the
player has at most one queued entry, which is every reachable case today
(0 `ReplacementTrigger::WouldDraw` defs, re-grepped, confirmed unchanged).
No "structurally" language survives anywhere touched by this fix cycle —
the `ChooseDredge` variant doc and the provider's own inline comment were
both rewritten to state the approximation and point at `OOS-DX23-8`.

**New test**: `crates/simulator/tests/pb_dx23_dredge_answer_channel.rs::
test_dx23_provider_withholds_when_declining_would_re_defer` (CR 616.1e/
616.1f, 702.52a). Built the fixture EXACTLY per
`pb_dx2_command_gates.rs::test_dx2_needschoice_redefer_grows_the_queue`'s own
recipe (same `Dredge(3)` card, same two `SkipDraw` `WouldDraw`
replacements watching `p1`, same 4-card library), driven through the SAME
direct-engine calls that test uses (`turn_actions::draw_card` +
`Command::ChooseDredge`, never a hand-poke of `pending_draws`) to reach the
identical two-entry queue state `[NeedsChoice-origin, dredge-origin]`
(asserted: `pending_draws().len() == 1` after the first decline,
`== 2` after the second draw — both sanity-pinned against the exact
`pb_dx2_command_gates.rs` numbers, plus a premise check that the FIFO-index-0
entry has empty `already_applied`, i.e. IS the re-raised `NeedsChoice` one,
not the dredge-origin `D2`). Then asserts `StubProvider.legal_actions`
returns **zero** `ChooseDredge` actions.

**Revert executed, RED confirmed, rebuild confirmed** (`Compiling
mtg-simulator`): dropped the third conjunct (`if !options.is_empty()`
only, `fifo_decline_would_redefer` bound to `let _ = ...` to silence the
unused warning). Observed failure:

```
thread 'test_dx23_provider_withholds_when_declining_would_re_defer' panicked at
crates/simulator/tests/pb_dx23_dredge_answer_channel.rs:842:5:
S1: the provider must withhold ChooseDredge entirely while the FIFO-oldest
PendingDraw is NeedsChoice-origin, even though the graveyard itself still has
an eligible dredge card -- offering it would answer the wrong entry and
re-defer it forever. Offered: [ChooseDredge { card: None, mill: 0 },
ChooseDredge { card: Some(ObjectId(1)), mill: 3 }]
```

Restored; full 7-test file green afterward; `grep -rn "REVERT-"
crates/simulator/src` returns 0 hits post-restore.

**Seed filed**: `OOS-DX23-8` in `docs/audits/decision-point-audit.md` §8.1
— states the approximation precisely (exact when ≤1 queued entry, zero
corpus reach today, a fully correct guard needs an origin discriminator
field on `PendingDraw`, which would be a HASH bump for a zero-reach case).

### S2 (MEDIUM) — `OOS-DX23-2`'s statement was over-broad

Reworded in place (`docs/audits/decision-point-audit.md`, the `OOS-DX23-2`
row): the original claim ("unanswerable through any simulator channel")
conflated two distinct mechanisms that both end in "no offer" — the
Q2 graveyard-empty case (truly nothing eligible) and the new S1/`OOS-DX23-8`
case (something is eligible, but it isn't the FIFO entry's problem to
answer). Cross-referenced to `OOS-DX23-8`.

### E1 (MEDIUM) — `DrawStepOutcome::DredgeOffered`'s doc asserted the
deleted invariant

`crates/engine/src/rules/replacement.rs` (doc comment on the `DredgeOffered`
variant, originally at plan-cited `:786-789`, found live at line 787's
"never mid-resume" sentence). Rewritten in the two-axis vocabulary (SAME
draw vs. DIFFERENT draw), citing `perform_remaining_draws`' own caller
table and naming the pinning test
(`test_dx23_tail_of_an_answered_multi_draw_offers_dredge_again`).

### E2 (MEDIUM) — `check_would_draw_replacement`'s doc asserted the same
stale claim

Same file, the function's own doc (originally cited `:682-684`, found live
at line 682's "`offer_dredge` is `false` on a resume" sentence). Amended
to "`false` on a SAME-DRAW resume; a TAIL resume passes the caller's own
flag", pointing at PB-DX23 §3 Q3 and `DredgeOffered`'s doc.

Both E1/E2 edits are doc-only; `cargo build -p mtg-engine` after each
confirmed no compile impact (doc comments only).

### E3 (LOW) — `dredge_options` raw-keyword-read note

One-line-equivalent note added directly above `dredge_options` in
`crates/engine/src/rules/queries.rs`, stating the read is raw (not
layer-resolved), that this matches `handle_choose_dredge`'s own answer-time
validator (so no offer-vs-engine divergence today), and that closing it
means changing both together — citing the PB-DX19 durable lesson by name.

### E4 (LOW) — the tail-discharge asymmetry, recorded

Added at `resolve_pending_draw`'s own `offer_dredge: true` call comment
(`crates/engine/src/rules/replacement.rs`, the "PB-DX23 §3 Q3: `offer_dredge:
true`" comment block): a tail auto-declined WHOLE by the implicit
stale-entry discharge (`tail_offers_dredge: false`) becomes dredge-offerable
again if one of ITS OWN draws hits `NeedsChoice` and is later resumed
THROUGH `resolve_pending_draw` (`true`). Cross-referenced to `OOS-DX2-7`'s
row, which E5 (below) extends to name this explicitly.

### E5 (LOW) — extended `OOS-DX2-7`'s text to name the tail explicitly

**Chose: extend, not file `OOS-DX23-9`.** Two places in
`docs/audits/decision-point-audit.md` updated: the §3.1 prose block's
"Status after PB-DX23" paragraph, and the §8.1 seed-table `OOS-DX2-7` row
itself. Both now state the tail case as its own sentence (asymmetric
against the answered-path tail, which DOES get re-offered per-draw) rather
than leaving it implicit in E4's engine-side comment alone.

### S3 (LOW) — `heuristic_bot.rs` "stays choosable" comment

Parenthetical added at the `ChooseDredge { card: Some(_), mill }` score-0
arm's comment: notes the arm is an inherited idiom shared with
`TapForMana`, and that for `ChooseDredge` specifically it is effectively
"never the top score" in practice, because `PassPriority` (1) is pushed
unconditionally before this block runs.

### S4 (LOW) — `tools/play-server/README.md` item-29 ordering

**Chose: leave with a note, not renumber the whole list.** A one-sentence
italic parenthetical was inserted at the start of item 29 explaining the
disorder is a continuation of a pre-existing pattern (item 25 already
precedes item 24), not a new regression, and that renumbering the whole
list was deliberately avoided to keep this fix cycle's diff scoped to
findings actually taken.

### T1 (LOW) — executed the mandatory revert instead of the equivalence
argument

**Revert**: disabled the WHOLE `ChooseDredge` push block in
`StubProvider::legal_actions` (`if false && ...` guarding the push), rebuilt
(`Compiling mtg-simulator` observed), and re-ran
`test_dx23_real_game_with_a_grave_troll_keeps_its_draw_cadence`. **Observed
failure** (A1 fires first, exactly as Stage 0's original pre-fix run):

```
PB-DX23 T1.1 pre-fix measurement: pending_draws_at_halt=1, card_drawn_p1=1,
dredged_p1=0, a2_lhs=1, a2_rhs(p1 draw-eligible turns)=2,
dredge_choice_required_p1=2

thread 'test_dx23_real_game_with_a_grave_troll_keeps_its_draw_cadence' panicked at
crates/simulator/tests/pb_dx23_dredge_answer_channel.rs:281:5:
assertion `left == right` failed: CR 702.52a/121.1: no PendingDraw should survive
to the halt of a real game once dredge offers are answerable. pending_draws() at
halt: [PendingDraw { player: PlayerId(1), already_applied: [], remaining: 0,
sets_has_drawn_for_turn: true }]
  left: 1
 right: 0
```

The A1/A2 diagnostic printout matches the Stage-0 pre-fix numbers exactly
(`pending_draws_at_halt=1`, `a2_lhs=1` vs `a2_rhs=2`,
`dredge_choice_required_p1=2`), confirming the fix-phase revert reproduces
the identical pre-batch defect rather than a different one. Restored; test
green afterward (`pending_draws_at_halt=0`, `a2_lhs=2`, `a2_rhs=2`); `grep
-rn "T1-REVERT"` returns 0 hits post-restore.

### T2 (LOW) — named the inherited-baseline shortfall explicitly

This session's OWN pre-edit baseline (confirmed by the coordinator's brief
and re-confirmed here by re-running `cargo test --workspace --no-fail-fast`
against HEAD before any fix-cycle edit landed): **4,412 / 0 / 5**, sound —
this number WAS freshly re-measured on this branch, not carried over from
an earlier commit's plan text. **What was genuinely inherited, not
re-measured, going INTO the original PB-DX23 implementation dispatch** was
the play-server "78/0" pin the plan quoted at §4 Stage 0 step 3 — Stage 0
of that dispatch measured the branch's actual value as 79/0 and flagged the
plan's "78" as stale (see the Stage 0 section above, §0.2). That
divergence was correctly recorded there; this fix cycle repeats the
methodology point explicitly per the review's ask: **the mechanism that
made "78" stale (a plan text quoting a number from an earlier measurement
that drifted before the branch was worked) is exactly the kind of
inheritance-without-re-measurement this finding warns about in general**,
and the fix-cycle brief's own baseline line ("current state: tests
4,412/0/5") was independently re-confirmed rather than trusted, for that
reason.

### T3 (LOW) — moved the `pending_draws().len()` assertion above the
offer-count assertion

`crates/engine/tests/primitives/pb_dx23_dredge_tail_and_query.rs::
test_dx23_implicit_discharge_does_not_mint_a_second_dredge_entry`: the
`OOS-DX2-3` guard assertion (`state.pending_draws().len() == 1`) now runs
immediately after `turn_actions::draw_card`, BEFORE `outer_offer_count` and
`discharge_card_drawn`. **Revert executed** (`perform_one_draw`'s implicit
discharge call changed from `resolve_declined_pending_draw(state, player,
stale, false)` to `..., true)`), rebuilt (`Compiling mtg-engine` observed),
re-run: the guard assertion now fires FIRST, as intended:

```
thread '...test_dx23_implicit_discharge_does_not_mint_a_second_dredge_entry'
panicked at crates/engine/tests/primitives/pb_dx23_dredge_tail_and_query.rs:463:5:
assertion `left == right` failed: OOS-DX2-3 guard: at most one dredge-originated
PendingDraw entry may exist per player. If the discharged sequence's tail also
pushed a dredge-originated entry, this would be 2.
  left: 2
 right: 1
```

Restored; full 7-test file green afterward; `grep -rn "REVERT-UNDER-TEST"
crates/` returns 0 hits post-restore.

### T4 (LOW) — stated plainly: no browser-level verification was run this
fix cycle either

No standalone HTTP server was started (per the standing SIGKILL hazard for
agent-run play-server binaries). `T5.1`
(`test_dx23_browser_can_answer_a_dredge_offer`) remains the coverage for
this path — it drives a real game over the real HTTP router in-process,
which is NOT the same as a browser/JS client actually parsing and clicking
the rendered option (`ActionBar.svelte`'s `plays` group). This gap is
explicitly NOT closed by this fix cycle; it is recorded here, again, as a
known and accepted limitation rather than silently repeated without
comment.

### T5 (LOW) — corrected the seed-count claim in the audit doc

`docs/audits/decision-point-audit.md`'s PB-DX23 SHIPPED paragraph corrected
from "Four new rows OOS-DX23-1..4" to "six new rows... `OOS-DX23-1..4` +
`-6` + `-7`", with a note that this fix cycle adds a seventh
(`OOS-DX23-8`). Both this fix cycle's own edits to that paragraph and the
original miscount are now consistent with the actual seed table (`OOS-DX23-
1,2,3,4,6,7,8` — 7 rows total after this session; `-5` deliberately never
filed, per plan §7).

### Gates — all executed after every fix above, output captured to a file

| gate | result |
|---|---|
| `cargo build --workspace` | clean, 0 warnings |
| `cargo test --workspace --no-fail-fast` (`/tmp/pb-dx23-fixcycle-full-suite.txt`) | **4,413 passed / 0 failed / 5 ignored** — exactly 4,412 + 1 (the new S1 test); reconciled exactly, no residual |
| `cargo test -p mtg-engine --test core hash_schema` | 21/21 green; `hash_schema_version_sentinel` confirms **HASH 73 unmoved** |
| `cargo test -p mtg-engine --test core protocol_schema` | 17/17 green; `protocol_schema_fingerprint_is_pinned` confirms **PROTOCOL 35 unmoved** |
| `cargo test -p mtg-engine --test core decision_gate` | 19/19 green, including `named_residual_seed_ids_still_exist_in_the_audit` (reads the audit doc as text — confirms the S1/S2/E5/T5 doc edits did not break any seed-ID cross-reference the gate checks) |
| `cargo test -p play-server` | 80/0, unmoved (no play-server production code touched this fix cycle) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean, 0 warnings |
| `cargo fmt --check` | clean, no reformat needed |
| `tools/check-defs-fmt.sh` | clean, "1803 defs checked" |
| golden script `replacement/014_golgari_grave_troll_dredge.json` | green, `run_all_approved_scripts: 1 of 271 discovered scripts ran and passed; 0 retired; 0 skipped silently`, matching every prior baseline |
| `git diff --stat -- crates/card-defs/` | empty — 0 card-def lines touched this fix cycle |

### Files touched this fix cycle

* `crates/simulator/src/legal_actions.rs` — S1 (third guard conjunct + doc
  rewrite on both the block comment and the `ChooseDredge` variant doc)
* `crates/simulator/src/heuristic_bot.rs` — S3 (comment only)
* `crates/simulator/tests/pb_dx23_dredge_answer_channel.rs` — S1's new test
  (`test_dx23_provider_withholds_when_declining_would_re_defer`)
* `crates/engine/src/rules/replacement.rs` — E1, E2, E4 (comments only)
* `crates/engine/src/rules/queries.rs` — E3 (comment only)
* `crates/engine/tests/primitives/pb_dx23_dredge_tail_and_query.rs` — T3
  (assertion reorder, no new assertion, no fixture change)
* `docs/audits/decision-point-audit.md` — S2, E5, T5, and the new
  `OOS-DX23-8` row (S1's seed)
* `tools/play-server/README.md` — S4 (one-sentence note)
* `memory/primitives/pb-DX23-execution-notes.md` — this section

No card-def file touched. No wire type touched. No `HashInto` field added.
