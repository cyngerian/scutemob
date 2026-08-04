# Primitive Batch Plan: PB-DX32 — make the fuzzer's *output* mean something

**Generated**: 2026-08-03
**Task / branch**: `scutemob-197` ·
`feat/pb-dx32-make-the-fuzzers-output-mean-something-oos-sim3-2-oo`
**Primitive**: **none — this is a test-instrumentation batch in `crates/simulator`.**
No `Effect`/`Condition`/`TargetFilter`/`ContinuousRestriction` variant, no `Command`, no
`GameEvent`, **no card-def edits**, no CR-701-style rules work.
**Seeds**: `OOS-SIM3-2` + `OOS-SIM3-3` + `OOS-SIM3-4` + `OOS-CARDS2-3`
**Queue row**: `memory/primitives/seed-rerank-2026-08-02.md` §4 rank **19**
(promoted per `docs/mtg-engine-feedback-engineering.md` §2.3, user-approved 2026-08-03)
**Brief**: `docs/mtg-engine-feedback-engineering.md` §2.3 (+ §1.2, §1.3, §1.5)
**Dependencies**: PB-DX22 (`scutemob-196`, `95f53b78`) — the fuzzer must actually cast
spells before its output is worth instrumenting. SHIPPED.
**Wire**: **none.** PROTOCOL **35** / HASH **72** must be gate-EXECUTED and unmoved.
**Coverage**: **1,133/1,803 = 62.8%**, must be unmoved and proven by regenerating
`tools/authoring-report.py` to a byte-identical body.
**Card-def edits**: **zero.** `git diff main..HEAD -- crates/card-defs/` must be EMPTY.

---

## 0. Stage-0 measurements ALREADY TAKEN AT HEAD (`45dacc7c`) — use these, do not re-derive

Committed raw evidence:

* `memory/primitives/pb-dx32-measurement-head-fuzzer.txt`
* `memory/primitives/pb-dx32-measurement-head-harness.txt`

**Do not quote any pre-2026-08-03 fuzz figure anywhere in this batch.** `OOS-DX22-13`
(`docs/audits/decision-point-audit.md:1141`) files the reason: before the PB-DX22 fix
cycle the binary's only by-`check` output was a **five-offending-game sample**, so every
historical "check X fired N times / never fired" claim is a sample unless it says
otherwise. That applies in particular to `OOS-SIM3-4`'s "929 of 938 are
`no_orphaned_tokens`" and SIM-3's 90.3% `stack_consistency` share.

### 0.1 Workspace baseline

**4,358 passed / 0 failed / 5 ignored**, residual list empty, measured on this branch
before any edit.

### 0.2 `./target/fuzz/mtg-fuzzer --games 20 --seed 1 --max-turns 200 --threads 1`

20 wins · 0 draws · 0 errors · avg **103.4** turns · **426** total violations ·
**16/20** games have ≥1.

By `check`, from `print_violation_histogram` (**ALL 20 games**, not a sample):

| check | violations | games |
|---|---|---|
| `no_orphaned_tokens` | **301** (70.7%) | 15 |
| `player_consistency` | **114** (26.8%) | 5 |
| `attachment_validity` | **11** (2.6%) | 3 |

**Two corrections this batch must carry, and state in its own artefacts:**

1. `OOS-SIM3-4`'s "929 of 938 are `no_orphaned_tokens`" is **stale AND was a sample**. At
   HEAD orphaned tokens are **70.7%** of the run's violations, not ~99%.
2. **`player_consistency` is a second, larger-than-recorded class** — 26.8% of the run,
   and it appears in *no* prior noise-floor account. It is **not** in scope to classify
   (§7), but it is in scope to file and to measure.

### 0.3 Five fuzz-shaped games (seeds 1-5, `build_fuzz_state` + `RandomBot` + `StubProvider`, `max_turns` 200, journal ON)

| seed | turns | cmds | rejections | taps (runs) | wasted taps (runs) | poolsEmptied | violations raw/dedup | orphan raw/dedup | leaked tokens |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 50 | 2190 | 79 | 155 (90) | 119 (72) | 77 | 9 / 2 | 9 / 2 | 0 |
| 2 | 123 | 5518 | 27 | 673 (263) | 560 (219) | 190 | 22 / 5 | 21 / 4 | 0 |
| 3 | 90 | 4812 | 43 | 459 (234) | 317 (163) | 150 | 14 / 3 | 14 / 3 | 0 |
| 4 | 119 | 5902 | 157 | 727 (353) | 506 (260) | 232 | 16 / 4 | 16 / 4 | 0 |
| 5 | 112 | 5191 | 236 | 627 (318) | 484 (254) | 236 | 33 / 6 | 30 / 5 | 0 |
| **tot** | | **23,613** | **542 = 22.953‰** | **2,641 (1,258)** | **1,986 (968)** | **885** | **94 / 20** | **90 / 18** | **0** |

Derived, and used as pins below:

* rejection rate **22.953 per mille** aggregate; per-seed 36.1 / 4.9 / 8.9 / 26.6 / 45.5‰.
* wasted taps / total taps = 1,986/2,641 = **75.20%**; wasted runs / tap runs =
  968/1,258 = **76.95%**.
* dedupe by `(check, description)` collapses **94 → 20** (**4.7×**) — this is
  `OOS-SIM3-3`'s checkpoint weighting, measured at HEAD for the first time.
* **`leaked_tokens` = 0 on every seed.** The strictly-stronger end-state property
  already holds, which is exactly what makes component (c) safe.

### 0.4 Distinct rejection shapes at HEAD — the classes an SR-38 threshold must tolerate

All known-open, all filed:

| shape | seed |
|---|---|
| `DeclareBlockers` → `CrossPlayerBlock{..}`, `AlreadyDeclaredBlockers(..)`, `"The attacking player cannot declare blockers"`, `"cannot block .. (attacker has flying / Can't be blocked ..)"` — the **largest family** | `OOS-SIM5-3` |
| `ActivateAbility`/`CastSpell` → `InvalidTarget("expected 1..=1 target(s) but got 0")`, `InvalidTarget("modal spell with per-mode targets requires exactly 1 target(s) for ..")` | `OOS-SIM5-5` |
| `InsufficientMana` on `ActivateAbility` and on `TapForMana` (auto-tap covers `CastSpell` alone) | `OOS-SIM6-3` |
| `CastSpell` → `InvalidCommand("Aura spells require exactly one target (CR 303.4a)")` | `OOS-CARDS2-4` |
| `ActivateAbility` → `"graveyard-activated ability can only be activated from the graveyard"`, `"a spell with split second is on the stack.."`; `DeclareAttackers` → `"Object .. is not a creature"` | `OOS-SIM4-2` / `OOS-SIM1-3` family |

**This is why the SR-38 threshold is a ratchet and never zero.** Five named open seeds
sit under it; a hard zero would be red on arrival and would be deleted within a week.

### 0.5 Deck pool at HEAD

`all_cards()` = **1,803** · `Complete` = **1,133** · commander pool
(`Complete` ∧ `Legendary` ∧ `Creature`, i.e. `crates/simulator/src/deck.rs:40-47`) = **90**.

---

## 1. What this batch delivers, in one paragraph

`mtg-fuzzer` currently prints a violation count that is 70% known-transient noise,
counted per checkpoint rather than per condition, and it prints **nothing at all** about
the millions of bot commands the engine *refused* — the property SR-38 is named for.
PB-DX32 makes four measurements first-class on `GameResult`: **rejections** (SR-38's own
invariant, with a measured ratchet), **waste** (the SIM-5 test-only tap/pool instrument,
promoted and made always-on), **a noise floor** (the transient token class split out and
answered by the strictly stronger end-state property, plus `(check, description)`
dedupe), and **decision-point runtime coverage** (which of `decision_site_walk.rs`'s
`ROWS` a fuzz run actually reaches). Plus one gate that makes `OOS-CARDS2-3`'s
corpus→seed coupling **announce** itself instead of being discovered by watching eight
seeded fixtures go red.

---

## 2. Code facts established by research — cite these, do not re-discover

| # | fact | location |
|---|---|---|
| F1 | `GameResult { seed, winner, turn_count, total_commands, violations, error }`, derives `Clone, Debug` only. | `crates/simulator/src/report.rs:32-40` |
| F2 | `GameResult` is constructed at **four** sites: `local_game.rs:487` (GameOver), `driver.rs:120` (start failure), `driver.rs:138` (Halted), `fuzzer.rs:332` (build failure) — **and a fifth outside the crate**, `tools/play-server/src/main.rs:3326`, in a `#[cfg(test)]` module. | grep `GameResult\s*\{` |
| F3 | `GameResult` is **read** at `tools/play-server/src/view.rs:2488` (`game_over_view`) by field access, not by exhaustive destructuring — so new fields do not break the reader. | `view.rs:2491-2504` |
| F4 | `rejection_count` is `saturating_add`ed **unconditionally**; only the *record* is gated on `record_journal` and capped at `MAX_RETAINED_REJECTIONS = 256`. So (a)'s count is already free; only the bounded **sample** needs a decision. | `local_game.rs:1025-1035`, `:357` |
| F5 | The single `record_rejection` call site is `local_game.rs:717`, inside `advance()`'s bot arm. The human path (`submit` → `apply_sequence`) never records — asserted by `tools/play-server/src/main.rs:3202-3205`. | — |
| F6 | `GameDriver` sets `record_journal: false` **on purpose** (thousands of long games in parallel). | `driver.rs:92-99` |
| F7 | `MechanicsTally` is the precedent to copy: a constant-size counter set, `Copy`, folded from events already in hand, always on, needing no journal, surfaced by `GameDriver::run_game_with_mechanics` and printed by `print_mechanics_summary`. | `local_game.rs:180-286`, `:414-439`; `driver.rs:79-154`; `fuzzer.rs:443-547` |
| F8 | The fold sites are exactly two, and together they see **precisely the journal's command stream in order**: `apply_sequence`'s commit loop (`local_game.rs:957-961`, folded only on commit because the sequence is atomic) and `apply_command` (`local_game.rs:997-998`). A rejected sequence journals nothing and folds nothing. | — |
| F9 | `invariants::check_all(state, prev_turn) -> Vec<InvariantViolation>`; `InvariantViolation { check, description, turn_number }` (derives `Clone, Debug, Serialize, Deserialize` — **no `PartialEq`/`Hash`**). `check_no_orphaned_tokens` at `:466`. Called **only** from `local_game.rs:964` and `:990`. | `invariants.rs:26-43`, `:463-481` |
| F10 | **Every one of the nine live checks is a pure function of a single `GameState`.** Rejections, tap runs and pool emptying are properties of the **command stream**, which `check_all`'s signature cannot express. See §3.0. | `invariants.rs:26-43` |
| F11 | The transient/end-state treatment to copy: split by `v.check == "no_orphaned_tokens"` into `transient_token_violations`, then assert the strictly stronger end-state property (no token anywhere but the battlefield at game end). | `local_game_playthrough.rs:455-472` |
| F12 | `bin/fuzzer.rs`: `print_violation_histogram` `:376` (ALL games), `print_mechanics_summary` `:443`, the first-5-offending-games detail loop `:266-285`, `--stop-on-error` `:190-209`, the crash-report writer `:292-310` (writes on `violations.first()`). | — |
| F13 | `deck.rs:30-157 random_deck`; the commander filter is `:40-47`. | — |
| F14 | `state.blocking_decision() -> Option<BlockingDecision>` with exactly three variants (`CleanupDiscard`, `TriggerTargets`, `EffectChoice`), deliberately **not** `#[non_exhaustive]`; `EffectChoice` carries `choice_id`/`source` but **not** the question. `state.pending_effect_choice() -> Option<&PendingEffectChoice>` carries `question: EffectChoiceQuestion`. | `crates/engine/src/rules/engine.rs:146-176`; `crates/engine/src/state/mod.rs:538,558`; `crates/card-types/src/state/stubs.rs:1022-1041` |
| F15 | `EffectChoiceQuestion` has exactly **four** variants: `SearchLibrary`, `Scry`, `Surveil`, `Discard`. | `stubs.rs:930-965` |
| F16 | `ROWS` is a `pub static` of **22** `Row { id, cr, site, class, predicate }` in the engine **integration-test module** `crates/engine/tests/core/decision_site_walk.rs:287-514`. Exactly **five** rows are `DecisionClass::Served`: `triggered_targets`, `search_library`, `discard_cards`, `scry`, `surveil`. | — |
| F17 | `decision_gate.rs` already has the source-gate idiom: `workspace_root()` `:1111`, `read_ct(rel)` `:1119`, `strip_line_comments` `:1124`, and `named_residual_seed_ids_still_exist_in_the_audit` `:1413` reads `docs/audits/decision-point-audit.md` from the engine test binary. **The extension point exists; nothing new has to be invented.** | — |
| F18 | The exact-ratchet message idiom (fails ABOVE *and* BELOW, and tells the reader to move the constant in the same commit): `auto_chosen_complete_union_is_ratcheted` `:796-845`. | — |
| F19 | The fuzzer is **not** run in CI (`grep -rn fuzz .github` → no matches), so a non-zero exit on threshold breach cannot redden the pipeline. | — |
| F20 | `crates/simulator/tests/` is a flat directory of integration targets; adding one is the convention and SR-9a does **not** apply there (it is scoped to `CARGO_MANIFEST_DIR = crates/engine`). | `pb_dx22_fuzz_instrument.rs:13-15` |

---

## 3. Design decisions, with the reasoning stated

### 3.0 The counters do NOT go in `invariants.rs`, and the brief's wording is loose

`docs/mtg-engine-feedback-engineering.md` §2.3(b) says "promoting it into
`invariants.rs`/`GameResult`". **`invariants.rs` cannot hold it** (F10): `check_all` takes
`(&GameState, Option<u32>)` and every check is a pure function of one state. A rejection
count, a tap run and a `ManaPoolsEmptied` event are properties of the *command stream*,
not of any state. Putting them behind `check_all` would mean changing that function's
contract for four callers and nine checks that do not want it.

**Decision**: the fold lives in `crates/simulator/src/local_game.rs` beside
`MechanicsTally` (F7 — the precedent, same mechanism, same always-on/no-journal
property); the results surface on `GameResult`; the thresholds live as `pub const`s in
`crates/simulator/src/report.rs` so the binary and the tests read **one** definition.
`invariants.rs` gains exactly one new function, and it *is* a pure state function:
the end-state leaked-token check (§3.3).

Record this reasoning in the plan-divergence section of `memory/primitive-wip.md` so the
reviewer does not read it as a missed requirement.

### 3.1 `GameResult` gains five fields, and `tools/play-server` is touched by exactly one line

Criterion (a) says verbatim that **`GameResult` carries** `rejection_count` and a bounded
rejections sample. PB-DX22's precedent (`driver.rs:76-78`) deliberately avoided a
`GameResult` field *because* of the out-of-crate construction site — that route is
closed here by the criterion's own wording.

**Decision, stated deliberately**: add `#[derive(Default)]` to `GameResult` and append
`..Default::default()` to the literal at `tools/play-server/src/main.rs:3326`. That is
**one inserted line in `tools/`**, in a `#[cfg(test)]` module, and it is the whole of
this batch's footprint outside `crates/simulator` + `crates/engine/tests/`.

* Every field is `Default`-able (`u64`, `Option<PlayerId>`, `u32`, `usize`, `Vec<_>`,
  `Option<GameDriverError>`), so the derive compiles.
* The other four in-crate construction sites are handled in Stage 1.
* **Invariant 7 hazard, load-bearing**: do **not** surface any new field in
  `GameOverView`. `game_over_view` (`view.rs:2491-2504`) is a *seat* payload, and review
  MR-M11-08 exists precisely because `InvariantViolation.description` interpolates
  `obj.characteristics.name`. A `RejectedCommand` carries a whole `Command` and its
  `Debug` string; a `transient_violations` entry carries a token's name. **Neither may
  enter `GameOverView`.** The un-redacted channel for both already exists and is
  deliberate: `GET /api/game/report` (`view.rs:748-764`, `:2564`).
* Acceptance for this: `git diff main..HEAD --numstat -- tools/` shows exactly one file
  and `+1 -0`. Anything else is a scope breach.

### 3.2 The bounded rejection sample: one store, two caps

The existing retention is gated on `record_journal` and capped at 256 — so the fuzzer
(F6) retains **nothing**, and a `GameResult.rejections` field would be empty in the exact
place it is needed.

**Decision**: keep one vector, make the cap depend on the flag —

```
record_journal == true   -> MAX_RETAINED_REJECTIONS  = 256   (unchanged: play-server, SIM-5/SIM-6 fixtures)
record_journal == false  -> MAX_SAMPLED_REJECTIONS   = 8     (new: the fuzzer)
```

Rationale for 8, not 256, on the fuzz path: `results` retains **every** game's
`GameResult` for the whole run, so at `--games 1000` a 256-cap would retain up to 256,000
cloned `Command`s. Eight per game is a diagnosis sample, which is all the fuzzer's summary
needs; `rejection_count` is uncapped and ungated (F4) so truncation stays visible.

**Verified safe**: no test anywhere asserts `rejections()` is empty under `record_journal:
false` (grep over `rejections()` / `rejection_count` — 8 call sites, all listed in §5
Stage 2). The doc comment at `local_game.rs:443-445` currently says *"Empty when
`LocalGameLimits::record_journal` is off"* — that becomes an aspirationally-wrong comment
the moment this ships and **must** be rewritten (`memory/conventions.md`).

### 3.3 The noise floor: split, don't delete; and answer with the stronger property

Copy `local_game_playthrough.rs:455-472` (F11) into `LocalGame` itself:

* `check_all`'s output is split at the point of collection — `v.check ==
  "no_orphaned_tokens"` goes to a new `transient_violations` vector, everything else to
  `violations`.
* At game end (both terminal paths), run a new pure-state check
  `invariants::check_no_leaked_tokens(state)` emitting `check: "leaked_tokens"` into
  **`violations`** (the hard bucket). This is the strictly stronger end-state property,
  and §0.3 measures it at **0 on all five seeds** — so this is safe *and* it is the thing
  that keeps the split honest.
* `print_violation_histogram` prints **both** buckets, with distinct counts alongside raw
  counts (`OOS-SIM3-3`'s own prescription: "report distinct conditions alongside the raw
  count"). Nothing is hidden; it is reclassified and still printed.
* `--stop-on-error` and the crash-report writer key on `violations` only, so a transient
  token no longer halts a smoke run and no longer writes `crash_<seed>.json`.

**Dedupe** is by `(check, description)` — first occurrence wins, order preserved. Neither
field carries a turn number (the turn is a separate field), which is why the collapse
works: §0.3 measures 94 → 20. `InvariantViolation` has no `PartialEq`/`Hash` (F9); dedupe
with a `BTreeSet<(String, String)>` rather than deriving anything (a derive on a
serialized type is a change nobody needs here, and `InvariantViolation` is simulator-only
so it is outside both wire closures either way).

**Honest limit, and the plan must say it in the same breath as the claim (§7 R1):** after
this change a 20-game `--stop-on-error` run at HEAD **still halts** — on
`player_consistency` (114 violations across 5 games) and `attachment_validity` (11 across
3). Criterion (c) is satisfied in its literal wording ("without halting on a
**known-transient** class"), not in the colloquial sense of "the fuzzer is now silent".

### 3.4 The waste tally: a promotion, plus an equivalence gate against its origin

`Metrics` in `crates/simulator/tests/sim5_bot_cast_discipline.rs:40-58` is computed by
`metrics_of` (`:144-200`) by walking `game.journal()`. Promote the counters to a
`WasteTally` on `LocalGame`, folded at the two sites in F8, which see exactly the journal's
command stream in order — so the streaming fold and the journal walk are provably the same
measurement.

* The run cursor (`Option<(PlayerId, u32)>`) lives on `LocalGame`, **not** on
  `WasteTally`, so `WasteTally` is a plain public `Copy` counter set with no private
  field (mirrors `MechanicsTally`).
* `metrics_of`'s trailing `if run.take().is_some() { tap_runs += 1 }` (`:196-198`) closes
  an open run at the end of the walk. The accessor `LocalGame::waste()` must do the same
  on a **copy** — exactly the shape `LocalGame::mechanics()` already uses for its
  CR 903.10a final-state read (`local_game.rs:424-439`). Miss this and the streaming
  tally is off by one run in every game that ends mid-tap-run.
* **Do not delete `metrics_of`.** Keep it and add an equivalence probe (§5 Stage 3, T3.2)
  asserting `metrics_of(&game)` and `game.waste()` agree field-for-field on a journal-ON
  game. That probe is what stops the promoted copy from drifting from its origin, and it
  is cheap.

**The threshold caveat, and this is the part a careless pin gets wrong.** SIM-5's A/B
reported wasted taps 20/15/10 → **0/0/0**, but that was **`HeuristicBot`, 25 turns, via
`setup::build_initial_state`**. The fuzzer runs **`RandomBot`** for 200 turns, and
`RandomBot` picks `TapForMana` uniformly with no plan, so **75.20% of its taps are wasted
by design of the bot, not by a defect** (§0.3). A threshold copied from the SIM-5 numbers
would be red on arrival and would immediately be deleted.

**Decision: two thresholds, one per bot, each with a stated meaning:**

| const (in `report.rs`) | value | measured at HEAD | what a RISE means | what it does NOT mean |
|---|---|---|---|---|
| `MAX_RANDOM_BOT_WASTED_TAP_PCT: u32` | **85** | 75 (1,986/2,641, §0.3) | `RandomBot` is tapping *even more* blindly than uniform choice explains — i.e. the auto-tap or the atomic-sequence rollback regressed | **nothing about engine correctness.** `RandomBot` has no plan; a value near 75 is its ordinary behaviour and must never be treated as a defect |
| `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED: usize` | **re-measure at Stage 0** (SIM-5 recorded 0/1/0 on seeds 0/7/42) | — | a `HeuristicBot` funded a cast it then did not make, or the greedy solver's slack widened | it can never be **0**: `OOS-SIM2-1` (greedy-solver slack on a cast that **succeeded**) is open and is documented **at the pin** |

Why SIM-5's HeuristicBot numbers may be re-used as a *starting point* while its fuzz
numbers may not: SIM-5 measured through `setup::build_initial_state`, and PB-DX22 changed
only `fuzz_setup.rs` (`git diff` over `crates/simulator/src/setup.rs` across PB-DX22 was
**comment lines only**). The `setup.rs` path did not move. **Re-measure anyway at Stage 0
and pin from that measurement** — the SIM-5 figure is a prior, not evidence.

### 3.5 Decision-point runtime coverage — the hard design problem, solved

**Constraint.** `ROWS` lives in an engine *integration-test module* (F16).
`crates/simulator` cannot import it; the engine cannot dev-depend on the simulator
(dependency-policy violation in `memory/conventions.md`, and a cycle in spirit even where
Cargo tolerates dev-dep cycles). **Inventing a parallel `ROWS` table in the simulator is
the failure mode the criterion names.**

**Where the counter lives**: simulator side, in a new
`crates/simulator/src/decision_coverage.rs`. Folded in `LocalGame::advance()` at the
`state.blocking_decision()` branch (`local_game.rs:544-560`) — the branch every path goes
through, human or bot, *before* the human-return.

**How the roster stays single-source**: a **source gate**, the project's own idiom (F17).
`decision_gate.rs` gains one new test that reads
`crates/simulator/src/decision_coverage.rs` as text and asserts the two tables agree. That
is "extending the static gate", which the criterion demands, and it is why the simulator's
list is a list of **ids only** — it carries no predicates, no CR cites, no classification
logic. There is nothing to drift *except* the id set and the observable/unobservable
split, and both are gated.

**What "reached" means, honestly — and the limit is stated in the same sentence as the
claim, everywhere it appears.**

* **Five rows are observable at runtime, and they are exactly the five `Served` rows.**
  The mapping is total and one-to-one:

  | `BlockingDecision` / `EffectChoiceQuestion` | `ROWS` id | CR |
  |---|---|---|
  | `TriggerTargets` | `triggered_targets` | 603.3d |
  | `EffectChoice` + `SearchLibrary` | `search_library` | 701.23a |
  | `EffectChoice` + `Scry` | `scry` | 701.22a |
  | `EffectChoice` + `Surveil` | `surveil` | 701.25a |
  | `EffectChoice` + `Discard` | `discard_cards` | 701.9b |
  | `CleanupDiscard` | **no `ROWS` row** (CR 514.1 is not a §3.1 effect row) | 514.1 |

  For these, **"reached" means the engine raised that question at least once during the
  run** — proof the decision point was *exercised*, not merely *recorded*. It does **not**
  prove the answer was non-default, and it does not prove the row's whole card population
  is exercised: one card lights the row.

* **Seventeen rows are UNOBSERVABLE, and they are exactly the rows a reader most wants
  covered.** Fourteen `AutoChosen` + two `Gated` + one `NoDecision`. **This must be said
  loudly, and the reason is not an oversight — it is the definition:** an `AutoChosen` row
  is one where *the engine takes the choice inline and leaves no artefact*. The absence of
  an artefact is the same property that makes it a defect. There is no hook to count
  because there is no hook at all; that is the finding, not a gap in the instrument.

* **Three alternatives were considered for the `AutoChosen` rows and all three are
  rejected, with reasons** (record these so a successor does not re-litigate):
  1. *"a card def hitting row R was cast/resolved"* — needs `row_hits`, i.e. the
     predicate table, simulator-side. That is the forbidden second copy.
  2. *"the corresponding `Effect` variant executed"* — needs an engine instrumentation
     hook in `effects/mod.rs`. This batch's footprint is `crates/simulator` +
     `crates/engine/tests/`; an engine source hook is out of scope and would be a
     different batch.
  3. *"infer from events"* — e.g. `Proliferate` → `CounterAdded`. Rejected: the mapping is
     a judgement call, it is not injective (a sacrifice **cost** emits the same event as
     `SacrificePermanents`), and it is a second drift surface with no gate.

  A successor that wants `AutoChosen` runtime coverage should serve the row (which turns
  it into a `Served` row and makes it observable for free) — which is the right incentive.

* **Counts are re-observation-weighted, exactly like §0.3's violation counts.** The
  `blocking_decision()` branch is re-entered on every `advance()` loop iteration until the
  decision is answered, and a bot whose answer is refused (F5) falls through to
  `PassPriority` and re-observes. **The report's primary output is therefore the boolean
  reached / never-reached partition**; the counts are secondary and carry the caveat in
  the printed header.

**Two forcing functions, one on each side:**

* Simulator side: the `match` on `BlockingDecision` and on `EffectChoiceQuestion` is
  **exhaustive with no wildcard**. A new variant of either is a compile error until
  someone classifies it — the same SR-5 pattern `stack_card_of` uses
  (`invariants.rs:127-130`) and that `local_game.rs:554-559` already applies to
  `BlockingDecision`.
* Engine-test side: the source gate. A row that becomes `Served` and is not wired into
  `OBSERVABLE_ROW_IDS` reddens `decision_gate.rs`.

**Explicitly NOT taken**: the static gate is **not** rebuilt, and the authoring-time blind
spot (`decision_gate.rs:19-28` — "you may X, if you do Y" authored as a bare
`Effect::Sequence` with the `may` dropped) is **not** touched. That rides **PB-DX8**
(v3 rank 10).

### 3.6 The corpus→seed gate (`OOS-CARDS2-3`)

An exact three-value pin, in the `MAX_AUTO_CHOSEN_COMPLETE_UNION` idiom (F18) — fails
above **and** below, and its failure message tells the reader what to do:

```
CORPUS_DEFS      = 1803
CORPUS_COMPLETE  = 1133
COMMANDER_POOL   = 90
```

`COMMANDER_POOL` is recomputed in the test by the **same filter as
`deck.rs:40-47`**, not by a hard-coded predicate written from memory — a copy of the
filter would let the pin stay green while `random_deck`'s own filter changed underneath
it. Read the filter's three clauses off `deck.rs` and mirror them with a comment naming
the line range.

The failure message must say, in these terms: *"the fuzz deck pool changed. Every seeded
fixture in the workspace now deals a different game (`OOS-CARDS2-3`). Update these three
constants in the SAME commit as the card-def change, and expect the seeded pins listed in
`memory/workstream-state.md` (CARDS-2 handoff, item 1) to move."*

---

## 4. CR rules touched

This batch implements no new rules behaviour. The CR cites that must appear in the new
code and tests, each already implemented and merely being *observed*:

| CR | what is being observed |
|---|---|
| **CR 704.3** | SBAs are checked on step entry and at resolution, not on every priority grant — the mechanism `OOS-M11-7` records and the reason the orphaned-token class is transient rather than wrong. |
| **CR 704.5m / 704.5n** | The Aura (graveyard) vs Equipment/Fortification (unattach, stays) dispositions — cited by `attachment_validity`; corrected once already by PB-DX22's fix cycle. Do **not** re-invert them. |
| **CR 800.4a** | A player who leaves the game — the hypothesised mechanism behind `player_consistency`'s 114 checkpoint-weighted reports (§7 R1). Hypothesis only; this batch does not act on it. |
| **CR 500.4** | Mana pools empty at the end of each step and phase — what `GameEvent::ManaPoolsEmptied` reports, and the oracle behind the waste instrument. |
| **CR 601.2c** | Targets are announced as part of casting — the `OOS-SIM5-5` rejection family. |
| **CR 603.3d / 608.2d / 701.9b / 701.22a / 701.23a / 701.25a** | The five `Served` decision rows, observed at runtime (§3.5). |
| **CR 514.1** | Cleanup discard — a `BlockingDecision` variant with no `ROWS` row, recorded as a non-row observation so the exhaustive match still forces classification. |

Every new test cites its rule (Architecture Invariant 8).

---

## 5. Implementation stages

Each stage: files, exact symbols, the gating test, **the revert that proves each test
red**, and the threshold with its measurement and its open seed.

**Standing rules for every stage** (`memory/gotchas-infra.md:577-585`):
* A revert-and-rerun proves nothing unless the **rebuild succeeded** — `-D warnings`
  turns an unused import or an unused `mut` into a build failure and `cargo test` then
  runs the **stale** binary and reports a pass. Look for a `Compiling mtg-simulator`
  line, and write the revert as a no-op that still consumes its bindings (or add a
  scoped `#[allow(..)]`) rather than deleting a line.
* Never `cargo test | tail`. Redirect to a file and sum `^test result` with awk.
* Whole-game gates run on a **64 MiB** spawned thread (`local_game_playthrough.rs:58,
  485-487`): they play the full 1,803-def pool and deep resolution chains have exhausted
  the default 2 MiB test stack before (`OOS-DP3-9`/`OOS-M11-3`).

New test target: **`crates/simulator/tests/pb_dx32_fuzz_output.rs`** (SR-9a does not apply
here — F20; copy the header note from `pb_dx22_fuzz_instrument.rs:13-15`).
New engine test: a module added to the **existing** `core` group — see Stage 6.

---

### Stage 0 — baseline, no source edits

1. Full workspace, `--workspace --no-fail-fast`, redirected to a file, summed with awk.
   **Expected 4,358 / 0 / 5**, residual list empty. If it differs, **stop and report** —
   §0.1 was measured on this branch.
2. Wire sentinels **executed, not predicted**:
   `cargo test -p mtg-engine --test core hash_schema` and `--test core protocol_schema`.
   Record `HASH_SCHEMA_VERSION` (`state/hash.rs:743`) and `PROTOCOL_VERSION`
   (`rules/protocol.rs:360`) by reading the constants. **Expected HASH 72 / PROTOCOL 35.**
3. `cargo run --profile fuzz --bin mtg-fuzzer -- --games 20 --seed 1 --max-turns 200
   --threads 1 --verbose` → capture to a file. **Expected to reproduce §0.2 exactly**
   (20/0/0, 426, 103.4, and the three-row histogram). This is the "before" side of (c)'s
   required A/B; commit the file.
4. **Measure the two thresholds the plan defers**:
   * `cargo test -p mtg-simulator --test sim5_bot_cast_discipline -- --nocapture` and read
     `mana_pools_emptied` per seed off the printed `Metrics` (seeds 0/7/42, HeuristicBot,
     25 turns) → pins `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` (§3.4).
   * Run the Stage-2 gate's **own** configuration once (3 seeds × `max_turns: 25`,
     `RandomBot`, `build_fuzz_state`, `record_journal: false`) and record its aggregate
     rejection-per-mille → pins `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG`.
     **Do not reuse §0.3's 22.953‰ for the test gate**: that number is a 200-turn
     measurement and the gate is a 25-turn one. §0.3's number pins the *binary's*
     threshold (Stage 2), not the test's.
5. Tree hygiene: `git status --short` clean except the plan file;
   `git check-ignore -v crash-reports` confirms the run's artefacts are untracked.

---

### Stage 1 — `GameResult` construction collapses to one place (behaviour-NEUTRAL)

**Files**: `crates/simulator/src/report.rs`, `crates/simulator/src/local_game.rs`,
`crates/simulator/src/driver.rs`, `crates/simulator/src/bin/fuzzer.rs`,
`tools/play-server/src/main.rs` (one line).

1. `report.rs` — add `Default` to `GameResult`'s derive list. Document at the type why:
   the struct is constructed at five sites, one of them outside the crate, and this batch
   adds five fields to it.
2. `local_game.rs` — add
   ```rust
   pub fn result_snapshot(
       &self,
       winner: Option<PlayerId>,
       error: Option<GameDriverError>,
   ) -> GameResult
   ```
   returning the full instrumented `GameResult` from `self`. Rewire **both** real
   construction sites onto it: `local_game.rs:487` (GameOver) and `driver.rs:136-145`
   (Halted). Leave `driver.rs:120` and `fuzzer.rs:332` as literals with
   `..Default::default()` — they are error paths where no game ran.
   **Why one function**: after Stage 2-6 the two real sites must populate five new fields
   identically; two hand-maintained literals is a divergence class that produces a
   Halted-game report silently missing its instrumentation.
3. `tools/play-server/src/main.rs:3326` — append `..Default::default()` to the literal.
   Nothing else in `tools/` changes.

**Gate — T1.1** `test_dx32_halted_and_game_over_results_carry_the_same_instrumentation`
(in the new test file): drive one seeded game to `GameOver` and one to a
`Halted(MaxTurns)` (set `max_turns` low), and assert both `GameResult`s report the same
*shape* — for now, that `turn_count`/`total_commands` match the game's accessors on both
paths. This test is written at Stage 1 and **strengthened at each later stage** to cover
each new field.

**Revert proof (EXECUTE)**: in `driver.rs`'s Halted arm, replace `game.result_snapshot(..)`
with a literal that sets `total_commands: 0`. Rebuild (confirm `Compiling mtg-simulator`),
run → T1.1 reddens on the Halted half with `left: 0`.

**NEUTRALITY EVIDENCE (mandatory, PB-DX22 Stage-1 precedent)**: re-run Stage 0 step 3
verbatim and `diff` against the committed baseline with cargo's build chatter filtered.
**Expect exactly one differing line, and it must be wall time.** Any per-game line moving
at this stage means the refactor is not neutral — stop.

**Stage gates**: `cargo build --workspace` · `cargo test -p mtg-simulator` ·
`cargo test -p play-server` (expect **78 / 0**) · `cargo clippy --workspace --all-targets
-- -D warnings` · `cargo fmt --check` · `tools/check-defs-fmt.sh` (1803 clean, SR-35).

---

### Stage 2 — (a) SR-38: the rejection channel becomes a run-level invariant

**Files**: `crates/simulator/src/local_game.rs`, `src/report.rs`,
`src/bin/fuzzer.rs`, `src/lib.rs`, `tests/pb_dx32_fuzz_output.rs`.

1. `local_game.rs`
   * add `pub const MAX_SAMPLED_REJECTIONS: usize = 8;` next to
     `MAX_RETAINED_REJECTIONS` (`:357`), documented per §3.2 (why 8 and not 256: the run
     retains every game's `GameResult`).
   * `record_rejection` (`:1025-1035`): the cap becomes
     `if self.limits.record_journal { MAX_RETAINED_REJECTIONS } else { MAX_SAMPLED_REJECTIONS }`
     and the `record_journal &&` conjunct is **dropped**. The count line is unchanged.
   * **Rewrite** the now-false doc comments at `:338-341`, `:443-445` and `:1017-1024`
     (`memory/conventions.md`, aspirationally-wrong comments) — each keeps its original
     account and says what changed and why.
2. `report.rs`
   * `GameResult` gains `pub rejection_count: u32` and `pub rejections:
     Vec<RejectedCommand>`.
   * add the pinned constant, with the ratchet instruction in its doc:
     ```rust
     /// SR-38 at run scale. Measured at HEAD (2026-08-03) over 5 fuzz-shaped games,
     /// 23,613 commands, 542 rejections = 22.953 per mille.
     /// Pinned with headroom, NOT at zero: OOS-SIM5-3 (blocker refusals, the largest
     /// family), OOS-SIM5-5 (modal per-mode target slices), OOS-SIM6-3 (auto-tap covers
     /// CastSpell alone), OOS-CARDS2-4 (Aura offers refused by CR 303.4a) and
     /// OOS-SIM4-2 are all open. Ratchet DOWNWARD as each closes; never raise it to
     /// fit a measurement without naming the seed that justifies the rise.
     pub const MAX_BOT_REJECTION_PER_MILLE: u32 = 30;
     ```
3. `bin/fuzzer.rs` — new `print_sr38_summary(results: &[GameResult])`, called beside
   `print_violation_histogram` in **both** the run path (`:256`) and the `--replay` path
   (`:168`). It prints: total rejections, total commands, aggregate per-mille, the
   per-seed band, and the **top rejection classes by error-string prefix** (truncate the
   string at the first `(` so `InvalidTarget("expected 1..=1 ...")` groups). Sort rows
   descending by count then by name so the output is stable across runs — it gets
   committed as evidence (mirrors `print_violation_histogram:403`).
   On breach: print a loud `SR-38 THRESHOLD EXCEEDED: N per mille > MAX_BOT_REJECTION_PER_MILLE`
   line and `std::process::exit(1)` at the end of `main`. Safe: F19 — the fuzzer is not in
   CI.
4. `lib.rs` — re-export `RejectedCommand`, `MAX_RETAINED_REJECTIONS`,
   `MAX_SAMPLED_REJECTIONS` (additive; `RejectedCommand` is not currently re-exported).

**Gates** (all in `tests/pb_dx32_fuzz_output.rs`):

| test | asserts | threshold / floor |
|---|---|---|
| **T2.1** `test_dx32_rejections_are_sampled_without_the_journal` | a fuzz-shaped game with `record_journal: false` returns a **non-empty** `rejections()` capped at `MAX_SAMPLED_REJECTIONS`, and `rejection_count() >= rejections().len()` | non-vacuity: the seed must actually produce ≥1 rejection |
| **T2.2** `test_dx32_sr38_bot_rejection_rate_is_ratcheted` | 3 seeds × 25 turns × `RandomBot` × `build_fuzz_state`, `record_journal: false`: aggregate per-mille `<= MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG` | **value measured at Stage 0 step 4**, pinned with ~30% headroom. Never 0 — §0.4's five seeds. Floors: `total_commands >= <measured × 0.8>` and `rejections > 0`, so a game that stops early or a bot that stops acting cannot pass trivially |
| **T2.3** `test_dx32_game_result_carries_the_rejection_channel` | `GameResult.rejection_count` equals `LocalGame::rejection_count()` on **both** the GameOver and the Halted path | — |

**Revert proofs (EXECUTE all three)**:

| test | revert | expected red |
|---|---|---|
| T2.1 | restore `self.limits.record_journal &&` in `record_rejection`'s guard | `rejections()` empty; T2.1 fails on the non-empty assertion |
| T2.2 | set `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG` to `measured - 1` | fails, naming the measured rate — proves the comparison is live (a threshold test whose bound is never approached proves nothing) |
| T2.3 | in `result_snapshot`, hard-code `rejection_count: 0` | fails on both paths |

---

### Stage 3 — (b) the waste instrument, promoted and thresholded

**Files**: `crates/simulator/src/local_game.rs`, `src/report.rs`, `src/bin/fuzzer.rs`,
`src/lib.rs`, `tests/pb_dx32_fuzz_output.rs`,
`tests/sim5_bot_cast_discipline.rs` (equivalence probe + a pin comment only).

1. `local_game.rs` — new `pub struct WasteTally` beside `MechanicsTally`:
   `tap_runs`, `wasted_tap_runs`, `wasted_taps`, `total_taps`, `mana_pools_emptied`,
   `casts`, `targeted_casts` — all `u32`, `#[derive(Clone, Copy, Debug, Default,
   PartialEq, Eq)]`. Doc it as the promotion of
   `sim5_bot_cast_discipline.rs:40-58`'s `Metrics`, naming that file.
   * `LocalGame` gains `waste: WasteTally` and `waste_run: Option<(PlayerId, u32)>`
     (the cursor, private — §3.4).
   * `fn fold_waste(&mut self, command: &Command, events: &[GameEvent])` — a literal
     transcription of `metrics_of`'s per-record body (`:154-195`), with the same
     three-arm `match` and the same "a different player interleaved closes the old run
     unclassified" behaviour.
   * called at the two fold sites in F8, **immediately beside the existing
     `self.mechanics.record(..)` calls**, in the same order, so the two tallies cannot
     disagree about what they saw.
   * `pub fn waste(&self) -> WasteTally` — returns a **copy** with the open run closed
     (`if self.waste_run.is_some() { tally.tap_runs += 1 }`), documented as the mirror of
     `mechanics()`'s final-state read and of `metrics_of:196-198`.
2. `report.rs` — `GameResult` gains `pub waste: WasteTally`, plus:
   ```rust
   /// Measured at HEAD (2026-08-03), RandomBot, 5 fuzz-shaped games, 200 turns:
   /// 1,986 wasted of 2,641 taps = 75%. Pinned at 85 with headroom.
   /// **RandomBot picks TapForMana uniformly with no plan, so most of its taps are
   /// wasted BY DESIGN OF THE BOT.** A value near 75 is ordinary behaviour and is not a
   /// defect; a rise past 85 means the auto-tap or the atomic-sequence rollback
   /// regressed. This can only be ratcheted toward zero by a PLANNING bot
   /// (OOS-DX32-<n>), never by an engine fix.
   pub const MAX_RANDOM_BOT_WASTED_TAP_PCT: u32 = 85;

   /// Measured at Stage 0 on the SIM-5 A/B seeds (0/7/42, HeuristicBot, 25 turns).
   /// NOT zero: OOS-SIM2-1 — the greedy solver leaves slack on casts that SUCCEED, so a
   /// destroyed pool is not necessarily a wasted one. That seed is the reason this pin
   /// exists at all; closing it is what lowers this to 0.
   pub const MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED: usize = <measured>;
   ```
3. `bin/fuzzer.rs` — `print_waste_summary(results)` beside the other summaries, printing
   totals, the wasted-tap percentage, `ManaPoolsEmptied`, and **one sentence naming the
   bot**: a `RandomBot` number and a `HeuristicBot` number are not comparable.

**Gates**:

| test | file | asserts |
|---|---|---|
| **T3.1** `test_dx32_random_bot_waste_ratio_is_bounded` | new | 3 seeds × 25 turns × `RandomBot`, `record_journal: **false**`: `wasted_taps * 100 / total_taps <= MAX_RANDOM_BOT_WASTED_TAP_PCT`; floor `total_taps > 0` |
| **T3.2** `test_dx32_streaming_waste_tally_equals_the_sim5_journal_walk` | `sim5_bot_cast_discipline.rs` | on a journal-**ON** game, `metrics_of(&game)` and `game.waste()` agree on all seven counters. **This is the anti-drift gate for the promotion** |
| **T3.3** `heuristic_pools_emptied_is_pinned` | `sim5_bot_cast_discipline.rs` (extend `seeded_four_bot_game_wastes_no_taps`) | per seed, `m.mana_pools_emptied <= MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED`, with `OOS-SIM2-1` named in the assertion message **at the pin** (criterion (b)'s literal requirement) |

**Revert proofs (EXECUTE all three)**:

| test | revert | expected red |
|---|---|---|
| T3.1 | set `MAX_RANDOM_BOT_WASTED_TAP_PCT` to `measured - 1` | fails naming the measured ratio |
| T3.2 | drop the trailing open-run close in `waste()` (`tap_runs += 1`) | fails with `tap_runs` off by one on at least one seed — **and if it does NOT fail, the fixture never ends mid-run; pick a seed that does, or the probe is not discriminating** |
| T3.3 | set `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` to `measured - 1` | fails on the seed that produced the measured value |

---

### Stage 4 — (c) the noise floor

**Files**: `crates/simulator/src/invariants.rs`, `src/local_game.rs`, `src/report.rs`,
`src/bin/fuzzer.rs`, `tests/pb_dx32_fuzz_output.rs`.

1. `invariants.rs` — new pure-state check:
   ```rust
   /// CR 704.3 / OOS-M11-7: `check_no_orphaned_tokens` reports a token in a
   /// non-battlefield zone at EVERY checkpoint until the next SBA sweep clears it, and
   /// this engine sweeps on step entry and at resolution rather than on every priority
   /// grant. That makes the report transient by construction. The strictly stronger
   /// property — and the one that would actually be a bug — is that no token is
   /// anywhere but the battlefield when the game is OVER. Measured 0 on all five HEAD
   /// seeds. Mirrors `local_game_playthrough.rs:464-472`.
   pub fn check_no_leaked_tokens(state: &GameState) -> Vec<InvariantViolation>
   ```
   emitting `check: "leaked_tokens"`. **Not** added to `check_all` — it is an end-of-game
   check, and `check_all` runs per command.
   Also add `pub fn distinct(violations: &[InvariantViolation]) -> Vec<InvariantViolation>`
   — first occurrence per `(check, description)`, order preserved, `BTreeSet<(String,
   String)>` backed (F9: no `PartialEq`/`Hash` derive needed or wanted).
2. `local_game.rs` — `violations: Vec<..>` gains a sibling `transient_violations:
   Vec<..>`. Both `check_all` call sites (`:964`, `:990`) route through one new helper
   `fn record_violations(&mut self, new: Vec<InvariantViolation>)` that splits on
   `v.check == "no_orphaned_tokens"` — the literal `local_game_playthrough.rs:457-463`
   treatment, with that file named in the comment. Add `pub fn transient_violations(&self)`.
   At **both** terminal paths (the `is_game_over` return `:487` and — via
   `result_snapshot` — the Halted path), run `check_no_leaked_tokens(&self.state)` and
   extend `violations` with the result.
3. `report.rs` — `GameResult` gains `pub transient_violations: Vec<InvariantViolation>`.
4. `bin/fuzzer.rs` —
   * `print_violation_histogram` prints **two** blocks (hard / transient) and, for each,
     a **raw and a distinct** count, with a header line stating that raw counts are
     checkpoint-weighted (`OOS-SIM3-3`) and distinct counts are the defect-shaped number.
   * `--stop-on-error` (`:204-209`) and the crash-report writer (`:294-309`) key on
     `result.violations` only — which, post-split, no longer contains the token class.
   * The `Total violations:` summary line (`:253`) prints hard + transient separately.
     **Do not silently change what that one number means**; a successor comparing it to a
     pre-merge run must be able to see the redefinition (`OOS-DX22-13`'s lesson).

**Gates**:

| test | asserts |
|---|---|
| **T4.1** `test_dx32_orphaned_tokens_are_transient_and_the_end_state_is_clean` | on a seed measured to produce them: `transient_violations()` non-empty, every entry `check == "no_orphaned_tokens"`, `violations()` contains **no** `no_orphaned_tokens`, and the final state has **no** token outside the battlefield (CR 704.3 / `OOS-M11-7`) |
| **T4.2** `test_dx32_leaked_token_at_game_end_is_a_hard_violation` | hand-build a terminal state with a token in a graveyard; `check_no_leaked_tokens` returns exactly one `leaked_tokens` violation. **Both directions**: a clean state returns empty (`invariants.rs`'s own paired-probe convention, `:483-501`) |
| **T4.3** `test_dx32_distinct_collapses_checkpoint_weighting` | `distinct()` over a hand-built vector with three identical `(check, description)` at three turn numbers returns one entry, preserving the FIRST; and over a real seeded game, `distinct(violations).len() < violations.len()` on a seed measured to repeat (§0.3: 94 → 20) |

**Revert proofs (EXECUTE)**:

| test | revert | expected red |
|---|---|---|
| T4.1 | change the split predicate to `v.check == "zone_integrity"` | token violations land in the hard bucket; T4.1 fails on the "no `no_orphaned_tokens` in `violations()`" half |
| T4.2 | make `check_no_leaked_tokens` return `Vec::new()` unconditionally | fails on the broken-state half while the clean half stays green — which is what proves the probe is paired, not one-sided |
| T4.3 | make `distinct` return its input unchanged | fails on the hand-built half |

**A/B, MANDATORY (criterion (c) requires before/after counts recorded)**: re-run Stage 0
step 3 verbatim into a second committed file and table it against the baseline:

| metric | before (§0.2) | after |
|---|---|---|
| `Total violations` (hard) | 426 | **expect 125** = 114 `player_consistency` + 11 `attachment_validity` |
| transient (reported, not halting) | — | **expect 301** `no_orphaned_tokens` |
| distinct hard / distinct transient | — | measure |
| games with ≥1 **hard** violation | 16 / 20 | **expect ≤ 8** |
| crash reports written | 16 files | **expect ≤ 8** |
| `--stop-on-error` halts on `no_orphaned_tokens` | yes | **must be NO** |

Then run `--games 20 --seed 1 --max-turns 200 --stop-on-error` and record **what it
halts on**. §7 R1: expect it still to halt, on `player_consistency`. Record it; do not
suppress it.

---

### Stage 5 — (d) the corpus→seed gate (`OOS-CARDS2-3`)

**File**: `crates/simulator/tests/pb_dx32_fuzz_output.rs`.

**T5.1** `test_dx32_fuzz_deck_pool_size_is_pinned`, three exact-equality assertions
(fail above AND below, F18) against `CORPUS_DEFS = 1803`, `CORPUS_COMPLETE = 1133`,
`COMMANDER_POOL = 90`, the last recomputed with the filter mirrored from `deck.rs:40-47`
(§3.6), with the prescribed failure message.

**T5.2** `test_dx32_commander_pool_filter_mirrors_deck_rs` — non-vacuity + anti-drift:
assert `COMMANDER_POOL < CORPUS_COMPLETE < CORPUS_DEFS` and that `random_deck` on a fixed
seed returns a commander that **is** in the recomputed pool. Without this, T5.1's mirrored
filter can diverge from `deck.rs` while both pins stay green.

**Revert proofs (EXECUTE)**: (a) change `CORPUS_COMPLETE` to 1132 → T5.1 reddens naming
the direction and `OOS-CARDS2-3`. (b) drop the `CardType::Creature` clause from the
mirrored filter → the pool grows, T5.1 reddens **and** T5.2's membership half must stay
green (proving T5.2 tests something T5.1 does not — if T5.2 also reddens here, restate it
so it discriminates independently).

---

### Stage 6 — (e) decision-point runtime coverage

**Files**: new `crates/simulator/src/decision_coverage.rs`; `src/local_game.rs`,
`src/report.rs`, `src/lib.rs`, `src/bin/fuzzer.rs`;
**new engine test module** `crates/engine/tests/core/decision_runtime_gate.rs` **or** an
appended section in the existing `crates/engine/tests/core/decision_gate.rs`.
**Prefer appending to `decision_gate.rs`** — the criterion says *extend*, and `read_ct` /
`strip_line_comments` / `MIN_ROWS` are already there (F17). If a new module is used
instead, its `mod` line **must** be added to `crates/engine/tests/core/main.rs` (SR-9a: a
dropped `mod` line silently deletes coverage).

1. `decision_coverage.rs`
   ```rust
   /// Row ids that CAN be observed at runtime. These are exactly the five
   /// `DecisionClass::Served` rows of `crates/engine/tests/core/decision_site_walk.rs`,
   /// and the correspondence is machine-checked by `decision_gate.rs`'s
   /// `runtime_decision_coverage_roster_matches_rows`.
   pub const OBSERVABLE_ROW_IDS: &[&str] = &[
       "triggered_targets", "search_library", "scry", "surveil", "discard_cards",
   ];

   /// Row ids with NO runtime hook, and why — one entry per row. An `AutoChosen` row is
   /// one where the engine takes the choice INLINE and leaves no artefact; the absence
   /// of an artefact is the same property that makes it a defect, so these counters can
   /// never move and are pinned unobservable rather than silently reported as zero.
   pub const UNOBSERVABLE_ROW_IDS: &[(&str, &str)] = &[ /* 17 entries */ ];

   pub const ROW_COUNT: usize = OBSERVABLE_ROW_IDS.len() + UNOBSERVABLE_ROW_IDS.len();

   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub struct DecisionCoverage { observations: [u32; ROW_COUNT] }
   impl Default for DecisionCoverage { /* hand-written: [0; ROW_COUNT] */ }
   ```
   plus `row_id_at(usize)`, `index_of(&str)`, `observations(&str) -> u32`,
   `reached() -> Vec<&'static str>`, `never_reached() -> Vec<&'static str>`, and
   ```rust
   /// CR 603.3d / 608.2d / 701.9b / 701.22a / 701.23a / 701.25a / 514.1.
   /// EXHAUSTIVE with no wildcard on BOTH enums (the SR-5 forcing pattern): a new
   /// `BlockingDecision` or `EffectChoiceQuestion` variant is a compile error here until
   /// someone decides which row it observes. `None` means "a real decision with no ROWS
   /// row" — CR 514.1 cleanup discard is the only one today.
   pub fn row_id_for(state: &GameState, decision: &BlockingDecision) -> Option<&'static str>
   ```
   **Write `Default` by hand**, not derived: `[T; N]: Default` is macro-provided for
   `N <= 32` only, and a hand-written impl is immune to that boundary moving.
2. `local_game.rs` — `LocalGame` gains `decisions: DecisionCoverage`; at the
   `state.blocking_decision()` branch (`:544-560`), call
   `decision_coverage::row_id_for(&self.state, &decision)` and increment. Add
   `pub fn decision_coverage(&self) -> DecisionCoverage`. The existing exhaustive
   `BlockingDecision` match at `:555-559` stays where it is (it maps to `DecisionKind`,
   a different axis) — do **not** merge them; say so in a comment, or the next reader
   will.
3. `report.rs` — `GameResult` gains `pub decision_coverage: DecisionCoverage`
   (`Copy`, `ROW_COUNT * 4` = 88 bytes; retained per game for the whole run, which is
   why it is a fixed array and not a map).
4. `bin/fuzzer.rs` — `print_decision_coverage(results)`: prints **reached** and
   **NEVER REACHED** as two lists over `OBSERVABLE_ROW_IDS`, then the
   `UNOBSERVABLE_ROW_IDS` list under a header that says, in one sentence, that these
   have no runtime hook **by construction** and that a zero there means nothing. The
   counts print with the caveat that they are re-observation-weighted (§3.5).

**Gates**:

| test | file | asserts |
|---|---|---|
| **T6.1** `runtime_decision_coverage_roster_matches_rows` | `decision_gate.rs` (**the source gate**) | reads `crates/simulator/src/decision_coverage.rs` via `read_ct`, **`strip_line_comments` first**, extracts the string literals inside the `OBSERVABLE_ROW_IDS` and `UNOBSERVABLE_ROW_IDS` const blocks (locate the const name, read to the terminating `];` — block-scoped, so `rustfmt` re-wrapping cannot defeat it), and asserts: (1) union == `ROWS` ids, both directions, **naming the offenders**; (2) `OBSERVABLE` == `{r.id : r.class is Served}`; (3) floors `total >= MIN_ROWS (22)`, `observable >= 5`, `unobservable >= 1` |
| **T6.2** `test_dx32_row_id_for_covers_every_observable_row` | `pb_dx32_fuzz_output.rs` | constructs one `BlockingDecision` (+ `pending_effect_choice`) per observable row and asserts `row_id_for` returns exactly that id; and that the set of ids it can ever return equals `OBSERVABLE_ROW_IDS`. **This is the non-vacuity partner of T6.1**: T6.1 proves the list matches `ROWS`, T6.2 proves the list is reachable from real code rather than a decorative constant |
| **T6.3** `test_dx32_a_fuzz_run_reaches_at_least_one_served_row` | `pb_dx32_fuzz_output.rs` | a seeded fuzz-shaped game reaches ≥1 observable row (measure at implementation time which; if **none** of the five is reached at 25 turns, raise the turn cap or the seed count until one is, and **record the number** — "no fuzz game ever reaches a served decision row" is itself the finding the criterion exists to surface, and it must be reported, not hidden by deleting the test) |

**Revert proofs (EXECUTE all three)**:

| test | revert | expected red |
|---|---|---|
| T6.1 | move `"surveil"` from `OBSERVABLE_ROW_IDS` to `UNOBSERVABLE_ROW_IDS` | reddens naming `surveil` and the class mismatch |
| T6.1 (second, **mandatory**) | comment out one `UNOBSERVABLE_ROW_IDS` entry with `//` | must **still** redden — this proves `strip_line_comments` is applied. PB-DX22's review cycle 2 found a comment-satisfiable source gate in this exact family; do not ship one again |
| T6.2 | make `row_id_for` return `None` for `EffectChoiceQuestion::Scry` | reddens naming `scry` |
| T6.3 | make `DecisionCoverage::observe` a no-op | reddens with an empty `reached()` |

---

### Stage 7 — close-out (f)

1. **Comment corrections** (`memory/conventions.md`, aspirationally-wrong rule) — each
   past-tenses the record and carries the measured replacement:
   * `local_game.rs:338-341`, `:443-445`, `:1017-1024` — the `record_journal` gating
     claim (Stage 2).
   * `invariants.rs:1-11` header — "nine checks can fire" becomes ten (the new end-state
     check is not in `check_all`; say exactly that), and the SIM-3 A/B block is re-dated
     with §0.2's **complete** histogram, naming `OOS-DX22-13`.
   * `invariants.rs:463-466` `check_no_orphaned_tokens` — record that it is now split out
     as transient by `LocalGame` and answered by `check_no_leaked_tokens`.
   * `sim5_bot_cast_discipline.rs:39-58` and `:96-106` — `Metrics`/`emptied_pool_context`
     note that the counters are now shipped on `GameResult` and that `metrics_of` is
     retained as the equivalence oracle (T3.2).
   * `bin/fuzzer.rs:29-61` — append a fourth boundary-event row: PB-DX32 does **not**
     move any seed (it adds no RNG draw and no provider action), and say why that is
     checkable — `git diff` over `deck.rs`/`fuzz_setup.rs`/`legal_actions.rs`/the bot
     files is empty.
   * `docs/mtg-engine-simulator.md` — its twelve-check list is `OOS-SIM3-2`'s second
     half. Mark #10 (legal-action soundness) **served at run scope by PB-DX32's SR-38
     invariant, not by a `check_all` function**, and say plainly that #11 (SBA
     idempotency) is **still** unwritten. Do not claim `OOS-SIM3-2` is fully closed.
   * `docs/mtg-engine-feedback-engineering.md` §2.3 — mark row 3 SHIPPED with the
     measured outcomes.
2. **Seed dispositions** in `docs/audits/decision-point-audit.md` §8.1 — append in-row,
   never rewrite:
   * **`OOS-SIM3-3` — CLOSED** (dedupe shipped; the 4.7× collapse measured at HEAD).
   * **`OOS-SIM3-4` — CLOSED, with its number CORRECTED**: the "929 of 938" figure was a
     sample and is stale; at HEAD orphaned tokens are 70.7% of 426, and the fuzzer no
     longer halts on them.
   * **`OOS-CARDS2-3` — CLOSED** (gate shipped; the three pinned values recorded in-row).
   * **`OOS-SIM3-2` — PARTIALLY closed, and say which part.** #10 is now asserted at run
     scope; #11 SBA idempotency is untouched. **Do not mark it closed.**
3. **New seeds `OOS-DX32-1..n`** — at minimum:
   * `player_consistency` is 26.8% of the run's violations (114 / 5 games), appears in no
     prior noise-floor account, and its check (`invariants.rs:367-390`) fires while the
     *active or priority* player has lost/conceded — the shape of a CR 800.4a transient
     with heavy checkpoint weighting. **Not classified by this batch, deliberately**: a
     26.8% class nobody has diagnosed must be diagnosed, not suppressed.
   * the `AutoChosen` runtime blind spot: 17 of 22 rows have no runtime hook by
     construction, and the only way to cover one is to **serve** it (§3.5).
   * `RandomBot`'s 75% waste ratio is a bot-design property; only a planning bot ratchets
     `MAX_RANDOM_BOT_WASTED_TAP_PCT` toward zero (`OOS-SIM2-1`/`OOS-SIM6-3` adjacent).
   * whatever Stage 6 T6.3 measures about which served rows a fuzz run never reaches.
   * `--replay <SEED>` still carries no command history (`fuzzer.rs:302`,
     `command_history: Vec::new()`) — untouched by this batch, and now more visible
     because the summary is richer. That is `docs/mtg-engine-feedback-engineering.md`
     §2.2, not this row.
4. **Bookkeeping**: lean `CLAUDE.md` Current-State delta + a new
   `Tests (delta 2026-08-03, PB-DX32)` pin; full handoff at the head of
   `memory/workstream-state.md`.
   **`memory/primitives/seed-rerank-2026-08-02.md` is UNTOUCHED — the §4 banner and row
   19 are the coordinator's at collect** (PB-DX22 precedent).
5. **Final gates, all EXECUTED not predicted**:
   * `cargo test --workspace --no-fail-fast` to a file, summed with awk; residual list
     empty. Expected **4,358 + <new probe count>**.
   * `cargo test -p mtg-engine --test core hash_schema` and `--test core protocol_schema`
     → **HASH 72 / PROTOCOL 35 unmoved**, read off the constants.
   * `cargo test -p play-server` → **78 / 0**.
   * `clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
     `tools/check-defs-fmt.sh` (1803 defs).
   * `git diff main..HEAD --numstat -- crates/engine/src/ crates/card-defs/
     crates/card-types/ crates/view-model/` → **EMPTY**.
   * `git diff main..HEAD --numstat -- tools/` → **exactly one file, `+1 -0`** (§3.1).
   * `tools/authoring-report.py` regenerated; body byte-identical except the git-sha stamp
     line; **1,133 / 1,803 = 62.8%** unmoved; revert the regeneration churn.

---

## 6. Traceability: criteria → stages

| criterion | stage(s) | the artefact that proves it |
|---|---|---|
| **(a)** SR-38 runtime invariant: `GameResult` carries `rejection_count` + bounded sample; fuzzer reports/fails above a pinned measured-at-HEAD threshold, ratcheted downward, **not zero** | 1, 2 | `report.rs::MAX_BOT_REJECTION_PER_MILLE = 30` (measured 22.953‰, §0.3) with five named open seeds in its doc (§0.4); `print_sr38_summary` + exit 1; T2.1/T2.2/T2.3 + three executed reverts |
| **(b)** waste thresholds: the `sim5_bot_cast_discipline.rs` instrument promoted into `GameResult` with thresholds, `OOS-SIM2-1` documented **at the pin** | 3 | `WasteTally` on `GameResult`; `MAX_RANDOM_BOT_WASTED_TAP_PCT = 85` (measured 75); `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` (Stage-0 measured) with `OOS-SIM2-1` in the assertion message; T3.2 equivalence gate against `metrics_of` |
| **(c)** noise floor: `check_no_orphaned_tokens` gets `local_game_playthrough.rs`'s treatment; dedupe by `(check, description)`; smoke run completes without halting on a known-transient class; before/after recorded | 4 | `check_no_leaked_tokens` + the split in `record_violations` + `invariants::distinct`; T4.1/T4.2/T4.3; the committed A/B table (426 → ~125 hard + 301 transient reported) and the recorded `--stop-on-error` outcome |
| **(d)** `OOS-CARDS2-3` closed: a gate pins the fuzz deck pool size so a corpus flip **announces** the re-roll | 5 | T5.1 (1803 / 1133 / 90, exact both directions) + T5.2 (filter mirrored from `deck.rs:40-47`, not re-invented) |
| **(e)** decision-point runtime coverage keyed by `decision_site_walk.rs` `ROWS` ids; **the static gate is extended, not rebuilt** | 6 | `decision_coverage.rs` (ids only, no predicates); the fold at `local_game.rs:544`; `print_decision_coverage`; **T6.1 appended to `decision_gate.rs`** + T6.2 + T6.3, with the comment-stripping revert executed |
| **(f)** close-out | 7 | §5 Stage 7's five sub-lists, each item individually checkable |

---

## 7. Risks, edge cases, and places I am flagging uncertainty rather than guessing

**R1 — criterion (c) is satisfiable in its literal wording only, and I am saying so up
front.** After the split, `--stop-on-error` at HEAD will **still halt**, on
`player_consistency` (114 / 5 games) and `attachment_validity` (11 / 3 games). Neither is
a *known-transient* class today, so the criterion's words are met; the colloquial reading
("the fuzzer is now a clean smoke test") is **not**. The right response is Stage 7's seed
filing, not a widened split. **Do not classify `player_consistency` as transient in this
batch** — it is 26.8% of the run's violations, nobody has diagnosed it, and suppressing an
undiagnosed quarter of the signal is the exact defect SIM-3's own withdrawal was about.

**R2 — `player_consistency` might be a second false-positive family.**
`check_player_consistency` (`invariants.rs:367-390`) fires whenever the active player or
the priority holder `has_lost || has_conceded`. Under CR 800.4a that is a legitimate
transient window (the player leaves; the turn ends), and the 94 → 20 dedupe collapse is
consistent with one condition reported over a whole turn of commands. **This is a
hypothesis, not a finding.** File it with the measurement; do not act on it.

**R3 — the 200-turn measurement cannot be reused as a test threshold.** §0.3's numbers
come from `--profile fuzz` (release speed + assertions). A `cargo test` build is debug and
23,613 commands of `im-rs` state cloning there is minutes, not seconds. Stage 0 step 4
therefore measures the gate's **own** configuration. If a 3-seed × 25-turn gate still runs
over ~60 s, drop to 2 seeds and re-measure — **do not** relax the threshold to make a
slower run fit.

**R4 — run-to-run non-determinism (`OOS-M11-3` / `OOS-DP3-9`).** `driver.rs:12-19` says
the fuzzer is not run-to-run deterministic for very long games. Every new threshold is
therefore a **ceiling with a floor**, never an exact equality — the exact-ratchet idiom
(F18) is used **only** for the corpus pins (Stage 5), which are pure functions of the
committed def corpus and cannot drift.

**R5 — the two `GameResult` real-construction sites.** Stage 1 exists precisely to
collapse them; if a later stage adds a field to one site and not the other, a *halted*
game reports missing instrumentation while a *completed* one reports it, and no existing
test looks at a halted game's new fields. T1.1 must be extended at every subsequent stage.

**R6 — Architecture Invariant 7.** Five new `GameResult` fields, two of which carry
free-form strings derived from `Command` `Debug` and from `obj.characteristics.name`.
`GameOverView` is a **seat** payload and must not gain any of them (§3.1). The
un-redacted channel is `GET /api/game/report`, and it already carries the obligation
written at `view.rs:748-764` (re-scope at M10a). Confirm with the one-line `tools/` diff.

**R7 — `-D warnings` and the revert protocol.** Hit three times in PB-DX22 alone. Every
revert in §5 must be written as a **no-op that still consumes its bindings** (or carry a
scoped `#[allow]`), and the rebuild line must be observed before the red result is
trusted. A revert whose build failed proves nothing.

**R8 — the `waste()` open-run close (T3.2's revert).** If the chosen fixture never ends
mid-tap-run, that revert stays green and the probe proves nothing. Pick a seed that does,
or construct a fixture that ends on a tap. **If neither is achievable, say so in the
handoff rather than shipping an undiscriminating probe** (`memory/conventions.md`:
test-validity findings are fix-phase HIGHs).

**R9 — uncertain, flagged: which served rows a fuzz run actually reaches.** I have not
measured T6.3, and I am not going to guess. `scry`/`surveil`/`search_library` require a
`Complete` def with that effect to resolve; the HEAD first-cast band is turn 5-29
(`OOS-DX22-12`), so a 25-turn gate may reach **none of the five**. Stage 6 T6.3's
instruction is therefore: measure, widen the configuration until one is reached, and
**record the number** — including the possibility that the honest answer is "0 of 5 at
this depth", which is a finding worth more than the test.

**R10 — uncertain, flagged: the exact hard-violation count after the Stage-4 split.** I
predict 125 (114 + 11) by subtraction from §0.2, but the split also adds
`check_no_leaked_tokens` at two terminal paths, and §0.3 measured `leaked_tokens = 0` on
five seeds — not on the twenty the A/B uses. If the 20-game run produces a non-zero
`leaked_tokens`, that is a **real find** (a token genuinely leaked at game end) and must
be filed, not tuned away.

**R11 — scope creep into `invariants.rs`'s missing check #11.** `OOS-SIM3-2` names two
missing checks. This batch writes **#10 at run scope only** (and not as a `check_all`
function — §3.0). **SBA idempotency (#11) is not in scope.** Resist the pull; note it in
the handoff.

---

## 8. What this batch explicitly does NOT do

* **No card definitions.** `git diff main..HEAD -- crates/card-defs/` must be EMPTY, and
  coverage must be provably unmoved at 1,133/1,803.
* **No engine source changes.** `git diff main..HEAD -- crates/engine/src/` must be
  EMPTY. The only engine-side edit is a **test**: the source gate appended to
  `crates/engine/tests/core/decision_gate.rs`.
* **No wire.** No `Command`, `GameEvent`, `Effect`, `TargetFilter`, `AbilityDefinition` or
  `Characteristics` change. PROTOCOL 35 / HASH 72, gate-executed.
* **Does not rebuild the static decision gate.** `decision_gate.rs`'s 1,435 lines,
  `BASELINE`, and `MAX_AUTO_CHOSEN_COMPLETE_UNION = 80` are untouched except for one
  appended test.
* **Does not take the authoring-time blind spot** (bare `Effect::Sequence` with the `may`
  dropped, `decision_gate.rs:19-28`). That is **PB-DX8**, v3 rank 10.
* **Does not classify `player_consistency`** as transient (R1/R2) and does not touch
  `attachment_validity` (`OOS-DX22-8`).
* **Does not write SBA idempotency**, `OOS-SIM3-2`'s check #11 (R11).
* **Does not fix any rejection class it measures** — `OOS-SIM5-3`, `OOS-SIM5-5`,
  `OOS-SIM6-3`, `OOS-CARDS2-4` all stay open, and the SR-38 threshold exists **because**
  they do.
* **Does not build the crash→seed→replay pipeline** (`command_history` is still
  unconditionally empty, `fuzzer.rs:302`). That is
  `docs/mtg-engine-feedback-engineering.md` §2.2.
* **Does not build HTTP-FUZZ** (§2.4) or touch the browser.
* **Does not add per-seat RNG streams, change any deck build, or move any recorded seed.**
  Nothing in this batch draws from an RNG or appends to a provider's action list
  (`memory/gotchas-infra.md:566-570`). If any seeded fixture moves, **stop** — something
  in the batch is not what it claims to be.
* **Does not touch `memory/primitives/seed-rerank-2026-08-02.md`.** The §4 banner and
  row 19 are the coordinator's at collect.

---

## 9. Verification checklist

- [ ] Stage 0 baseline recorded: 4,358 / 0 / 5; HASH 72 / PROTOCOL 35 executed; the
      20-game fuzz "before" file committed; the two deferred thresholds measured
- [ ] Stage 1 neutrality proved by fuzz-run diff (exactly one differing line, wall time)
- [ ] `GameResult` has five new fields, built through `result_snapshot` at both real sites
- [ ] `git diff -- tools/` is exactly one file, `+1 -0`
- [ ] Every threshold constant carries: the measurement it came from, its date, the named
      open seed(s) keeping it above zero, and a ratchet instruction
- [ ] Every new test proven red by an **executed** revert whose **rebuild succeeded**
- [ ] T6.1's comment-stripping revert executed (the comment-satisfiable-gate class)
- [ ] The (c) A/B table committed, including what `--stop-on-error` still halts on
- [ ] `cargo test --workspace --no-fail-fast` to a file; residual list empty
- [ ] HASH 72 / PROTOCOL 35 re-executed and unmoved
- [ ] `cargo test -p play-server` 78 / 0
- [ ] clippy `-D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1803) clean
- [ ] `git diff -- crates/engine/src/ crates/card-defs/ crates/card-types/
      crates/view-model/` EMPTY
- [ ] Coverage 1,133/1,803 = 62.8% proven by a byte-identical report regeneration
- [ ] `OOS-SIM3-3`, `OOS-SIM3-4`, `OOS-CARDS2-3` closed; `OOS-SIM3-2` marked **partial**
- [ ] `OOS-DX32-*` filed, including the `player_consistency` finding
- [ ] Aspirationally-wrong comments corrected at all seven sites in Stage 7 item 1
- [ ] `seed-rerank-2026-08-02.md` untouched
