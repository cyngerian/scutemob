# Primitive Batch Review: PB-DP9 — Search / scry / surveil become player choices (CR 608.2d)

**Date**: 2026-07-27
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-157` · branch `feat/pb-dp9-search-scry-surveil-player-choice-auto-pick-inverts-t`
**Baseline**: `48353a36` (PROTOCOL 30 / HASH 67 / 3,878 tests)
**Shipped**: PROTOCOL **31** / HASH **68** / **3,905** tests, all gates green
**CR Rules verified via MCP**: 608.2 (a–p, esp. **608.2c/d/e/f/m/n**), **701.22a–d**, **701.23a–j**,
**701.25a–d**, 400.7, 401.4, 401.7, 104.3a, 605.1b/605.4a, 726, 104.4b

**Engine files reviewed**
`crates/engine/src/rules/resolution.rs` (the abort wrapper), `crates/engine/src/effects/mod.rs`
(`ask_or_consume_effect_choice`, `handle_answer_effect_choice`, `validate_partition`, the three
effect arms, the four default helpers, `execute_effect_answering`, the recursion guards),
`crates/engine/src/rules/engine.rs` (`BlockingDecision`, `blocking_decision`, the admission gate,
`finish_stack_resolution`, `discharge_departed_effect_choice`, `handle_concede`),
`crates/engine/src/rules/abilities.rs` (`repair_departed_priority_holder`),
`crates/engine/src/rules/mana.rs` (the CR 605.4a gate),
`crates/engine/src/rules/loop_detection.rs`, `crates/engine/src/rules/events.rs`
(`private_to`, `reveals_hidden_info`, discriminant 131), `crates/engine/src/rules/protocol.rs`,
`crates/engine/src/state/{mod.rs,hash.rs}`, `crates/card-types/src/state/{stubs.rs,zone.rs}`,
`crates/engine/src/testing/{replay_harness.rs,script_schema.rs}`,
`crates/simulator/src/{legal_actions.rs,random_bot.rs,heuristic_bot.rs,local_game.rs}`,
`tools/tui/src/play/{input.rs,app.rs,panels/action_menu.rs}`,
`tools/replay-viewer/frontend/src/lib/eventFormat.js`.

**Card defs reviewed**: **0 source edits** — verified. Spot-checked the roster's two partition
classes against oracle text (`demonic_tutor.rs`, `polluted_delta.rs`) plus the four modal defs the
roster test misses (`evolution_charm.rs`, `insatiable_avarice.rs`, `thirsting_roots.rs`,
`tooth_and_nail.rs`) and the 17 `ChooseCreatureType` defs.

---

## Verdict: **needs-fix**

The design is sound and the CR mapping is, with one modelling caveat, right: 701.22a/b/c/d,
701.25a/c/d and 701.23a/b/d are all implemented as written, the fail-to-find predicate
(`*filter == TargetFilter::default()`) genuinely partitions the corpus (I confirmed
`TargetController::default() == Any`, which the plan flagged as the load-bearing unknown, and
checked both classes against real defs), the CR 400.7 same-zone renumber fix is correct,
PB-RS1's top/bottom orientation is preserved *and* still pinned (the `library_ordering` probes
were re-pointed at an explicit bottom-everything answer rather than weakened), and the wire,
hash, SR-3 sealing, `private_to`, exhaustive-match and driving-loop plumbing are all complete.
The abort-and-replay mechanism itself is the right call and the wrapper's index arithmetic
(`consumed = restart_len − remaining_len`, `bank.take(consumed)`) is correct under every
interleaving I could construct — it does **not** reproduce PB-DP8's HIGH-1 mis-binding.

Two HIGH findings. **(1)** The concede-path test that the plan's §11 item 5 singles out as *the*
must-not-be-vacuous test is vacuous: its 2-player fixture ends the game on the concede, so
`discharge_departed_effect_choice` returns at its `is_game_over` early exit and the entire
"drive the rolled-back resolution, do not merely clear it" behaviour — PB-DP8's exact
three-times-shipped bug class — has zero coverage, while the test's doc comment claims the
opposite. **(2)** The answer bank is abandoned only when the *entry's own* player concedes; a
**foreign** concede mutates the board and leaves the bank bound to the pre-concede state, which
drives the replay's question-equality check into `debug_assert!(false, "replay determinism
violation")` on a legal command sequence — a panic in every debug/test/fuzzer build. Both are
concede-exit findings, which is the departure exit this suite keeps missing.

Four MEDIUMs follow the batch's own meta-lessons: the `MAX_EFFECT_CHOICES_PER_RESOLUTION` bound
cannot reach the cycle its doc says it bounds; the nondeterminism audit was scoped to
`target_remaps` and missed two live `HashMap`-iteration-to-outcome sites; the SR-36 roster walk
skips `ModeSelection.modes` and `Effect::CoinFlip`, so the published 69/16/7 are undercounts by
at least four `Complete` defs; and two source comments promise a `debug_assert` the code
deliberately does not have.

---

## Judgement on the 12 falsified plan premises

| # | Runner's correction | Verdict |
|---|---|---|
| 1 | `next_choice_id` renamed `next_effect_choice_id` (a `next_choice_id()` method already existed) | **Correct and important.** The shadowing hazard was real; the separate-counter *reason* is load-bearing and is preserved. |
| 2 | `Effect::Scry` had no CR 701.22b guard; T18 is a real fix, not a regression guard | **Correct.** Verified: the arm now returns before emitting `Scried` for `n == 0`, matching the surveil arm's 701.25c guard. |
| 3 | Destructive `pop_front` + `index = restart_len − remaining_len` instead of a positional cursor | **Correct, and I attacked it hard.** The arithmetic is right on the normal path, the mismatch path and the multi-choice path. One caveat the record does not state: the dead-player and mana-gate early returns bypass the bank entirely, so bank positions map to *live-player* choice points only — which is fine while liveness is constant across a resolution and is exactly what Finding 2 breaks. |
| 4 | `MAX_EFFECT_CHOICES_PER_RESOLUTION` enforced in the answer handler, not at the ask site | **Rejected — see Finding 3.** The stated equivalence ("achieves the same thing… one diagnosable rejection") is false: the wrapper truncates the bank on the mismatch path, so it oscillates between `i` and `i+1` and the bound is unreachable on the only path it was meant to bound. |
| 5 | No `debug_assert!` on the mana gate; obligation discharged by a roster assertion | **Accepted in substance, but not fully delivered.** The roster assertion exists and is a real `all_cards()` walk — but it inherits Finding 5's `CoinFlip` gap, and two comments still advertise the assertion that was removed (Finding 6). |
| 6 | `compute_mandatory_state_hash` made `pub`; T21 re-pointed at the rolled-back-vs-before pair | **Correct and better than the plan.** The construction genuinely isolates the fields and pins both directions. |
| 7 | One new harness action string, not three | **Correct.** Verified in `next_action_answers_the_block`. |
| 8 | One golden script needed the explicit answer, not ~4 | **Correct.** Both touched scripts (`stack/071`, `baseline/009`) make the choice explicit and keep their original assertions; no assertion was weakened. |
| 9 | `etb-triggers/205` unaffected | Accepted. |
| 10 | 25 unit tests across 6 files | Accepted; the two I read in full (`library_ordering.rs`, the roster/mana tests) are CR-justified adaptations, not "make it pass". |
| 11 | `target_remaps` audit CLEAN | **Correct as far as it goes, but the audit was scoped too narrowly — Finding 4.** `target_remaps` genuinely never iterates (3 inserts, 1 get). The replay re-executes the *whole resolution*, and two other `HashMap`-iteration-to-outcome sites exist. |
| 12 | SR-19's delete-a-field demo run; found the struct-only gate gap (OOS-DP9-13) | **Correct, and honestly disclosed** — `hash.rs:3128-3132` says in-source that the two enum impls are held by review, not by the gate. Good practice. |
| — | Benchmarks: no regression, pre-scan not added | Accepted; the numbers are recorded and the escape hatch stayed out of the correctness path, as the plan required. |
| — | Scry/surveil defaults flipped to the identity | **Endorsed.** The CR argument (both are legal under 701.22a/701.25a's "any number"; mill-everything can force a CR 704.5b deck-out) is right, the flip is pinned in both directions by `test_dp9_defaults_reproduce_the_stated_behaviour`, and the search half stayed byte-identical so the zero-churn claim holds. |
| — | OOS-DP9-8 (APNAP vs ascending `PlayerId`) not fixed, pinned instead | **Endorsed.** CR 608.2e / 701.22c / 701.23i do require APNAP; the deviation is pre-existing, far wider than this roster, and `test_dp9_choice_inside_for_each_each_player` pins the actual behaviour with the deviation named in-test. |

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `crates/engine/tests/primitives/pb_dp9_effect_choice.rs:1370` | **The owner-concede test is vacuous on its own central assertion.** 2-player fixture ⇒ game over ⇒ the discharge's resolve-and-drive path never executes and the deadlock assertion is skipped. **Fix:** rebuild on a 3-player fixture. |
| 2 | **HIGH** | `crates/engine/src/rules/engine.rs:2551-2558` | **A foreign concede keeps a stale answer bank.** The board mutates but the bank survives, so the replay's question-equality check fires `debug_assert!(false)` — a panic in every debug/test/fuzzer build on a legal command sequence. **Fix:** drop the entry + bank and re-drive on *any* concede that occurs while a CR 608.2d choice is outstanding. |
| 3 | MEDIUM | `crates/engine/src/effects/mod.rs:482-489`, `crates/engine/src/rules/resolution.rs:82-93` | **`MAX_EFFECT_CHOICES_PER_RESOLUTION` cannot bound the cycle it documents.** The wrapper truncates the bank on mismatch, so it never grows to 64; a genuine determinism violation is an unbounded ask/re-ask livelock in release. **Fix:** bound re-asks per index, or restore the plan's force-default fallback; correct the doc either way. |
| 4 | MEDIUM | `crates/engine/src/effects/mod.rs:4204-4225`; `crates/engine/src/rules/replacement.rs:2089`, `:2128` | **`HashMap` iteration reaches an outcome, falsifying the replay premise (latent).** `ChooseCreatureType` ties are broken by `HashMap` iteration order. Latent only because no `Complete` def co-locates it with the three asking effects. **Fix:** `BTreeMap`, or sort before `max_by_key`; widen OOS-DP9-10 and the wrapper's determinism doc. |
| 5 | MEDIUM | `crates/engine/tests/primitives/pb_dp9_effect_choice.rs:1692-1726` | **SR-36 roster walk misses `ModeSelection.modes` and `Effect::CoinFlip`.** ≥4 `Complete` defs undercounted; the published 69/16/7 went into the audit as fact. **Fix:** walk modes + `CoinFlip`/`Splice`/`on_cast_effect`, re-run, re-write the audit rows. |
| 6 | MEDIUM | `crates/engine/src/effects/mod.rs:199-200`; `crates/engine/src/rules/mana.rs:872` | **Two comments promise a `debug_assert` the gate deliberately does not have.** **Fix:** delete both claims. |
| 7 | MEDIUM | `crates/engine/src/rules/engine.rs:2529-2535` | **The exit-4 coverage claim overstates the code.** The function's only caller is `handle_concede`, so an SBA elimination never reaches it; the plan's prescribed reap was not implemented. **Fix:** implement the reap, or narrow the comment to "unreachable by the admission gate". |
| 8 | LOW | `crates/engine/src/rules/events.rs:1400-1402` | **Stale claim: "`GameEvent::private_to()` … does not exist — seed OOS-DP8-6"**, ~100 lines above the `private_to()` this batch added. **Fix:** rewrite to point at `private_to()` and note OOS-DP8-6's declaration half is closed. |
| 9 | LOW | `crates/card-types/src/state/zone.rs:214-235` | **`reposition_within` silently inserts a phantom id** not already in the zone. **Fix:** `debug_assert!` that every named id is present. |
| 10 | LOW | `crates/engine/src/effects/mod.rs:3992-4025` | **A real loop site got no suspension guard**, and the wip's justification for omitting it is factually wrong. Harmless (total restore) but the derivation record is. **Fix:** add the guard inside the `for pid in payer_ids` loop and correct the record (which also says 5 `resolution.rs` guard sites; there are 4). |
| 11 | LOW | `crates/engine/tests/scripts/script_replay.rs:1285-1325` | **`next_action_answers_the_block`'s unit test was not extended** with an `EffectChoice` case. **Fix:** add case (e) mirroring (c)/(d). |
| 12 | LOW | `crates/engine/src/effects/mod.rs:3602-3605` vs `:3666-3669` | **`Scried` reports the requested count, `Surveilled` the actual count.** Neither is CR-wrong (701.22d/701.25d both fire regardless) but the asymmetry is a trap. **Fix:** comment the asymmetry, or align on `actual_count`. |
| 13 | LOW | `crates/engine/src/rules/engine.rs:2631` | **`discharge_departed_effect_choice`'s `Err` propagates out of `handle_concede`**, so a resolution error during the discharge makes the player permanently unable to concede. **Fix:** clear the entry and swallow-with-diagnostic rather than `?`. |
| 14 | LOW | `crates/engine/src/rules/resolution.rs:31-62` | **CR 608.2m is not named** in the wrapper's long design argument, although the abort puts a resolving object back on the stack. **Fix:** one sentence stating the deviation and why it is unobservable through legal commands. |

---

## Finding Details

### Finding 1 — The owner-concede test never executes the behaviour it exists to guard

**Severity**: HIGH
**File**: `crates/engine/tests/primitives/pb_dp9_effect_choice.rs:1370-1414` (fixture at `:182`)
**Invariant**: PB-DP8 transferable rule (ii) — *a test that constructs a hazardous state and does
not assert against it is worse than no test*; plan §11 item 5 names this exact test as the one
that must not do that.

`fixture()` builds a **two-player** game. `test_dp9_owner_concedes_mid_choice` blocks p1 on a
search question and has p1 concede. Inside `handle_concede`, `p.has_conceded = true` runs first,
so by the time `discharge_departed_effect_choice` (`engine.rs:2536`) is reached, `active_players()`
has length 1 and the function takes its `if is_game_over(state) { return Ok(()); }` early exit at
`:2563` — **before** `resolve_top_of_stack` at `:2574`. The test's remaining assertions
(`pending_effect_choice().is_none()`, bank empty, `blocking_decision().is_none()`) are all
satisfied by the *clear-only* half at `:2559-2560`. The `if !over { … }` block at `:1405-1413`,
which contains every assertion about the hazardous state — that a live seat holds priority, that
it is not the conceded seat, and that `PassPriority` actually succeeds — is skipped, because
`over` is `true` by construction.

So the batch's answer to "PB-DP8 shipped an unrecoverable `priority_holder` deadlock three times"
is a code path with **no test at all**: nothing anywhere in the suite drives
`discharge_departed_effect_choice` past `:2563`. The doc comment at `:1362-1369` asserts the
opposite in so many words ("so this test asserts against the hazardous state … not merely that
the entry is gone").

**Fix**: rebuild `test_dp9_owner_concedes_mid_choice` on a **three-player** fixture (the
`test_dp9_foreign_concede_does_not_step_over_the_block` builder at `:1423-1450` is the model), so
the game survives the concede. Then delete the `if !over` escape hatch and assert
unconditionally: the spell has left the stack, the search applied the default (the
lowest-`ObjectId` candidate is in the destination zone), a **live** seat holds priority, and that
seat's `PassPriority` succeeds.

---

### Finding 2 — A foreign concede leaves the answer bank bound to a pre-concede board

**Severity**: HIGH
**File**: `crates/engine/src/rules/engine.rs:2551-2558`; detector at
`crates/engine/src/effects/mod.rs:459-472`
**CR**: 608.2d (the choice is announced *while applying the effect*, against the state as it then
is); 104.3a / 800.4a (a departing player's objects and choices leave the game)

`discharge_departed_effect_choice` reads the entry's **owner**'s liveness and returns `Ok(())`
untouched if the owner is still alive. Its own doc block (`:2524-2528`) justifies dropping the
bank on the owner path with: *"the concede mutated the board, so a banked answer may answer a
question the replay no longer asks."* That argument does not depend on whose entry it is — a
foreign concede mutates the board too (`has_conceded`, `UntilYourNextTurn` continuous-effect
expiry at `:2636-2648`, `temporary_protection_qualities.clear()`, initiative transfer at `:2651`),
yet leaves the bank intact.

Concrete, fully reachable sequence, on the shape `test_dp9_choice_inside_for_each_each_player`
already builds (`Effect::SearchLibrary { player: PlayerTarget::EachPlayer, … }`), extended to
three seats:

1. p1's question is asked, p1 answers ⇒ `effect_choice_answers = [A(Q_p1)]`.
2. The replay reaches p2's choice ⇒ p2's question is outstanding.
3. **p1 concedes.** `Concede` is explicitly admitted while blocked (`engine.rs:292`).
   `discharge_departed_effect_choice` sees p2 alive ⇒ returns; bank untouched.
4. p2 answers ⇒ bank `[A(Q_p1), A(Q_p2)]`; `resolve_top_of_stack` replays.
5. The replay re-enters the effect. `resolve_player_target_list` (`effects/mod.rs:7492-7503`)
   filters on `has_lost` only, so p1 is **still in the loop**, but
   `ask_or_consume_effect_choice`'s liveness check (`:436-443`) sees `has_conceded` and returns
   the **default without popping the bank**.
6. p2's choice therefore compares its recomputed `Q_p2` against `bank.front() == A(Q_p1)`.
   They differ ⇒ `debug_assert!(false, "CR 608.2d (PB-DP9): replay determinism violation …")`
   at `effects/mod.rs:463-468` **panics** in every `cargo test` / fuzzer / debug build.

In release the mismatch path recovers (the wrapper truncates to `consumed`, the stale head is
dropped, p2 is re-asked with a fresh `choice_id`), so the cost there is one extra round trip and
a `choice_id` churn a client must tolerate. But the classification is wrong either way: this is
not "execution is not a deterministic function of `(GameState, Command)`" — it is the state
having legitimately changed between the ask and the replay, which the design permits by admitting
`Concede`.

The same class reaches a **single-choice** resolution whenever the concede changes the recorded
question's payload (a filtered search whose candidate set depends on a continuous effect the
concede expired). The `EachPlayer` case is the structural one, because a departure shifts the
question *positions*, not just their contents.

`test_dp9_foreign_concede_does_not_step_over_the_block` (`:1422`) exercises only the benign case:
a single-choice resolution with an empty bank and a question the concede cannot change. It
therefore passes while the hazard stands.

**Fix**: in `discharge_departed_effect_choice`, treat **any** concede that happens while
`pending_effect_choice.is_some()` as invalidating: clear both `pending_effect_choice` and
`effect_choice_answers`, then re-drive `resolve_top_of_stack` exactly as the owner path does. The
still-live owner is simply re-asked with a fresh `choice_id` against the post-concede board,
which is what CR 608.2d requires ("the player announces these **while applying the effect**").
Update the doc block at `:2515-2535` to state the widened rule, and extend
`test_dp9_foreign_concede_does_not_step_over_the_block` (or add a sibling) to the 3-player
`EachPlayer` shape with a non-empty bank, asserting no panic and that both surviving seats'
answers are applied.

---

### Finding 3 — `MAX_EFFECT_CHOICES_PER_RESOLUTION` cannot bound the cycle its doc claims

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:482-489` (the constant + doc), `:584-589` (the check);
`crates/engine/src/rules/resolution.rs:82-93` (the truncation)
**Invariant**: PB-DP8 meta-lesson — a comment that justifies skipping work is a reachability
claim; wip premise 4 is the claim.

The constant's doc says: *"Reaching it means the replay is asking a fresh question every pass…
Refusing the answer turns an unbounded ask/re-ask cycle into one diagnosable rejection."*

Trace the mismatch cycle. On a mismatch at bank index `i`, `ask_or_consume_effect_choice` falls
through **without popping** (`:463-472`), so the inner pass consumed exactly `i` answers. The
wrapper computes `consumed = restart_len − remaining_len = (i+1) − 1 = i` and sets
`state.effect_choice_answers = restored_bank.take(i)` (`resolution.rs:82-93`). The bank is now
length `i`. The player answers ⇒ length `i+1`. The next replay mismatches at `i` again ⇒
truncates back to `i`. **The bank oscillates between `i` and `i+1` and never reaches 64**, so
`handle_answer_effect_choice`'s check at `:584` is unreachable on precisely the path it exists
for. Under a genuine per-pass nondeterminism (Finding 4's class) this is an unbounded question /
answer livelock in release; the `debug_assert` is the only thing that stops it in a debug build.
(The harness is protected by `auto_answer_blocking_decisions`' own `MAX_ROUNDS = 256`, and
`execute_effect_answering` by its `panic!`; the *engine* is not.)

The truncation itself is correct and necessary — it is what makes a *transient* mismatch (e.g.
Finding 2's) self-heal in one round trip. The problem is that the bound was moved to a place the
truncation makes unreachable, and the record asserts otherwise.

**Fix**: add a per-entry re-ask counter (`PendingEffectChoice { reasks: u32 }`, hashed like the
rest) incremented when the wrapper records an entry at an `index` it has already asked at, and
reject / force the default past a small bound; **or** restore the plan's §1.4 remedy (a
force-default flag on `EffectContext` for the remainder of the resolution). Either way, rewrite
the constant's doc to describe what it actually bounds — bank *growth* from correctly-answered
distinct choices — and say plainly that the mismatch cycle is bounded by something else.

---

### Finding 4 — The nondeterminism audit was scoped to `target_remaps`; two outcome-reaching `HashMap` iterations exist

**Severity**: MEDIUM
**Files**: `crates/engine/src/effects/mod.rs:4204-4225` (`Effect::ChooseCreatureType`);
`crates/engine/src/rules/replacement.rs:2089`, `:2128` (the ETB-replacement twins)
**Invariant**: SR-9b, now load-bearing at runtime (`resolution.rs:57-62` says so explicitly);
plan §1.4's determinism fact list; wip premise 11.

```rust
let mut type_counts: std::collections::HashMap<SubType, usize> = HashMap::new();
…
type_counts.into_iter().max_by_key(|(_, count)| *count).map(|(st, _)| st)
```

`max_by_key` over an unordered iterator returns the **last** maximum in iteration order, so every
tie is broken by `HashMap` iteration order. Rust's `RandomState` derives a fresh key per
`HashMap::new()` from a per-thread counter, so two structurally identical maps built at different
moments in the *same process* iterate differently — this is not merely a cross-process hazard.
Ties are the common case (a single Elf Warrior yields `Elf: 1, Warrior: 1`).

Any such site that executes **before** the last choice point of a resolution runs on every replay
pass and can diverge, which either changes the banked question (⇒ Finding 3's cycle) or silently
produces a different game state than the one the player was shown. The plan's fact list is scoped
to *"the three effects' candidate derivation"* — narrowly true — but the replay re-executes the
**whole resolution**, and wip premise 11 records the audit as "CLEAN" after checking only
`target_remaps` (which I independently confirmed: three `insert`, one `get`, nothing iterates).

I enumerated `HashMap<`/`HashSet<` across `crates/engine/src` and checked each. Everything else
is `contains`/`get`-only or feeds a set that is later sorted; these three are the outcome-reaching
ones. I also confirmed **no** current `Complete` def puts `ChooseCreatureType` in the same
`CardDefinition` as `SearchLibrary`/`Scry`/`Surveil` (all 17 `ChooseCreatureType` defs checked),
so the hazard is **latent, not live** — which is why this is MEDIUM and not HIGH. It becomes live
the first time the authoring campaign writes such a card, with no gate to catch it.

**Fix**: switch all three to `BTreeMap` (or collect and sort by `(count, key)` before
`max_by_key`). Widen **OOS-DP9-10** from "`target_remaps` is a `HashMap`" to "the replay's
determinism premise covers every statement of a resolution; these are the sites that break it",
and amend `resolve_top_of_stack`'s doc (`resolution.rs:57-62`) so the premise is stated at
resolution scope rather than effect scope.

---

### Finding 5 — The SR-36 roster walk misses `ModeSelection.modes` and `Effect::CoinFlip`

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dp9_effect_choice.rs:1692-1726`
**Invariant**: SR-36 — *enumerate `all_cards()` for rosters, never grep source*.

`roster::ability_effects` (`:1719-1726`) returns only the single `effect` field of
`AbilityDefinition::{Spell, Triggered, Activated}`. It never walks
`AbilityDefinition::Spell { modes: Option<ModeSelection> }`, whose `modes: Vec<Effect>` is where
every modal card's real effects live. `roster::effect_contains` (`:1692-1716`) covers `Sequence`,
`Conditional`, `ForEach`, `Repeat`, `Choose`, `MayPayOrElse`, `MayPayThenEffect` and `RollDice`,
but **not** `Effect::CoinFlip { on_win, on_lose }`, `AbilityDefinition::Splice { effect }` or
`CardCastPermission { on_cast_effect }` — while its own doc comment at `:1690-1691` claims *"the
coin-flip arms and so on"* are covered.

Confirmed undercounts, all `Complete` (`Completeness::default() == Complete`, verified at
`card_definition.rs:197-200`) and all carrying `Effect::SearchLibrary` inside `modes:`:

| def | line |
|---|---|
| `evolution_charm.rs` | `:36` (mode 0, "Search your library for a basic land card") |
| `insatiable_avarice.rs` | `:47` |
| `thirsting_roots.rs` | `:31` |
| `tooth_and_nail.rs` | `:35` |

So the search roster is at least 69 → 73, and the printed numbers that wip §"Roster" and the
audit's §5 DP-7/8/9 rows now record as **fact** are wrong. This is the defect SR-36 exists to
prevent, wearing an enumeration's authority instead of a grep's. It also weakens
`test_dp9_mana_ability_gate`'s obligation discharge (Finding 6's sibling): a mana trigger nesting
a scry inside a `CoinFlip` would not be found.

The engine itself is unaffected — modal effects execute through `resolve_top_of_stack`'s modal
`execute_effect` sites (`resolution.rs:596/607/617`), which are guarded and inside the restartable
unit.

**Fix**: extend `ability_effects` to also yield `modes.modes.iter()` (and `mode_costs`-adjacent
effects if any carry one), extend `effect_contains` with `CoinFlip`/`Splice`/`on_cast_effect`, and
delete the "coin-flip arms" claim if any variant is deliberately left out. Re-run
`test_dp9_roster_enumeration`, and re-write the three numbers in `memory/primitive-wip.md` and in
`docs/audits/decision-point-audit.md` §5.

---

### Finding 6 — Two comments promise a `debug_assert` the code deliberately removed

**Severity**: MEDIUM
**Files**: `crates/engine/src/effects/mod.rs:199-200`; `crates/engine/src/rules/mana.rs:872`
**Invariant**: PB-DP8 meta-lesson (hit three times in one batch) — every comment that justifies
skipping work is a claim to be checked against the code.

`EffectContext::effect_choice_gate_closed`'s doc says *"a `debug_assert` records if it ever
actually fires"*; `mana.rs`'s CR 605.4a branch says *"(with a `debug_assert` recording that it
happened)"*. The gate at `effects/mod.rs:419-431` has **no** assertion and explains at length why
not (*"Not an assertion, deliberately"*) — which is the position I agree with (CR 605.4a leaves no
room for an announcement, so the default *is* the defined behaviour, not a swallowed failure).
Both stale comments point a future reader at a diagnostic that does not exist.

**Fix**: rewrite both to name the actual discharge — `test_dp9_mana_ability_gate`'s roster
assertion — and delete the `debug_assert` claim.

---

### Finding 7 — `discharge_departed_effect_choice`'s exit-4 coverage claim overstates the code

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/engine.rs:2529-2535`, called only from `:2631`
**Plan**: §1.5 exit 4 prescribes *"a **reap** at the top of `resolve_top_of_stack` and in
`blocking_decision`'s liveness filter"*.

The doc claims the function *"also covers exit 4 (elimination by an SBA rather than a concede)"*
because it is written against the liveness predicate. The predicate form is good, but the
function's **only** call site is `handle_concede`, so an SBA elimination never reaches it. Only
half the plan's prescription shipped: `blocking_decision` (`:251-260`) *filters* a dead owner's
entry out — which **unblocks** the game — but nothing clears the field.

The resulting state (`pending_effect_choice.is_some()` with a dead owner) is a trap:
`blocking_decision` returns `None`, so `PassPriority` is admitted; `resolve_top_of_stack`'s entry
`debug_assert!` at `:64-68` fires; and because `ask_or_consume_effect_choice`'s "an earlier effect
in this same pass already suspended" guard (`:447-449`) short-circuits on the *pre-existing*
entry, the inner pass applies nothing and the wrapper rolls back and re-emits the stale question
forever. The resolution can never complete.

I could not construct a live path to it: while blocked, the admission gate rejects everything
that could run an SBA, and `Concede` routes through the discharge. So this is a latent hazard
guarded by an argument that lives in a *different* function, not the coverage the comment claims.

**Fix**: either implement the reap (clear + re-drive at the top of `resolve_top_of_stack` when
the entry's owner is not alive, mirroring `drop_departed_trigger_flush`'s placement), or narrow
the comment to state that exit 4 is unreachable **because the admission gate prevents any SBA
from running while blocked**, and that `blocking_decision`'s filter is a defence in depth that
does not itself clear the field.

---

## Verification of the replay's soundness (the review's centre of gravity)

Recorded because the brief asked for the attack, not just the verdict.

| Attack | Result |
|---|---|
| Non-`Ord` iteration reaching an outcome | **Two sites found** — Finding 4. `state.objects`/`zones`/`players` are `imbl::OrdMap`/`Vector`; the search candidate list is built by `retain` over `objects` and `debug_assert!`-checked ascending (`effects/mod.rs:3463-3467`), which is a good pin. |
| `SystemTime` / `Instant` / entropy RNG in the engine | **None.** Workspace grep: `Instant::now` only in `fuzzer.rs` and `snapshot_perf.rs`; no `from_entropy`/`thread_rng`/`rand::random` anywhere in `crates/engine/src`. All randomness is `StdRng::seed_from_u64(state.timestamp_counter)`. |
| Statics / thread-locals / interior mutability escaping the clone | **None found.** No `static mut`, `thread_local!`, `OnceLock`, `lazy_static` or atomics in the engine crate. |
| A counter incremented outside `GameState` | **None.** `timestamp_counter` is a `GameState` field and is rolled back; `next_effect_choice_id` is deliberately separate and minted **on the restored state** (`resolution.rs:99`) — this is the plan's "single subtlest trap" and it is implemented exactly right. |
| An event pushed into a caller-owned `Vec` before the abort | **No.** `resolve_top_of_stack_inner` owns its `events` vec and the wrapper discards it wholesale on the abort path; the two resume sites extend only from the wrapper's return. `test_dp9_two_choices_in_one_resolution` pins "exactly two questions, exactly one `Scried`" across all three command returns. |
| Is `*state = restart_point` genuinely total? | **Yes** for everything I could reach. `EffectContext` is rebuilt per pass inside `resolve_top_of_stack_inner`; `execute_effect_answering` additionally restores the ctx for the direct-call path (`effects/mod.rs:658-665`). |
| Bank mis-binding under interleaving (PB-DP8 HIGH-1) | **Sound.** `consumed = restart_len − remaining_len` and `take(consumed)` are correct on the normal, mismatch and multi-choice paths; question-equality is checked before every pop; `choice_id` is a fresh moment guard per ask and `test_dp9_stale_choice_id_rejected` pins the previous-choice case. Cross-player collision by question equality is impossible (`ObjectId`s are globally unique, and an empty candidate list never asks). |
| `Concede` between abort and replay | **Broken on the foreign path** — Finding 2. The owner path drops the bank correctly but is untested — Finding 1. |
| Mismatch path reachable & safe? | Reachable (Finding 2). Safe in release for a *transient* divergence (self-heals in one round trip). Unbounded under a *persistent* per-pass divergence — Finding 3. |

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 608.2d | Yes | Yes | The unifying rule; one command, one `BlockingDecision`, one action string. The CR argument for unification is sound. |
| 608.2c (order written) | Yes | Yes | `test_dp9_two_choices_in_one_resolution` asserts search-then-scry order. |
| 608.2e / 701.22c / 701.23i (APNAP) | **No — deviation** | Yes (pins actual behaviour) | Ascending `PlayerId`, not APNAP. Pre-existing in `resolve_player_target_list`; correctly seeded as OOS-DP9-8 and named in the test. |
| 608.2f (relative order) | Yes | Partly | Surveil's graveyard order follows the announcement. |
| 608.2m | **Deviation, unstated** | No | The object goes back on the stack after resolution has begun. Unobservable through legal commands. Finding 14. |
| 701.22a | Yes | Yes | Partition, both halves, in-any-order; `test_dp9_scry_keeps_cards_on_top_in_a_chosen_order`. |
| 701.22b | Yes — **new fix** | Yes | Scry 0 no longer emits `Scried`; wip premise 2 confirmed against source. |
| 701.22d | Yes | Yes | Event fires even with a short library. |
| 701.23a | Yes | Yes | Candidate list = filter matches, ascending `ObjectId`, `debug_assert`-pinned. |
| 701.23b | Yes | Yes | `may_fail_to_find` true for a stated-quality filter; `test_dp9_legal_fail_to_find`. |
| 701.23c | No | No | Undefined-quality search unmodelled; pre-existing, out of scope. |
| 701.23d | Yes | Yes | Quantity-only ⇒ finding mandatory; the one-candidate narrowing is CR-derived and right. Predicate verified against `demonic_tutor` / `polluted_delta`; `TargetController::default() == Any` confirmed. |
| 701.23e | No (pre-existing) | — | `reveal` still `_`-destructured; OOS-DP9-9. |
| 701.23h | No (pre-existing) | — | Two searches before a shuffle are two searches; now *visible* as two questions. Seeded. |
| 701.25a | Yes | Yes | Graveyard/top partition; `test_dp9_surveil_keeps_cards_on_top`. |
| 701.25c/d | Yes | Yes | Surveil 0 guard pre-existing; event fires on a short library. |
| 400.7 | Yes — **new fix** | Yes | `Zone::reposition_within` permutes in place; the graveyard half of surveil correctly keeps `move_object_to_zone` and does renumber. Fallout was one test, repaired on CR 401.4 grounds. |
| 401.4 / 401.7 | Yes | Yes | Bottom = `push_front`/index 0, top = last element; PB-RS1's pins re-pointed, not weakened. |
| 605.1b / 605.4a | Yes (gate) | Partly | Gate correct; obligation discharged by a roster assertion that has Finding 5's gap, and advertised by two stale comments (Finding 6). |
| 104.3a / 800.4j | Partly | **No** (Finding 1) | Owner-concede discharge exists; its drive half is untested and its foreign-concede sibling is wrong (Finding 2). |
| 104.4b | Yes | — | `reset_loop_detection` on each answer, matching the other announcement commands. |
| 726 | Yes (exclusion) | Yes | The three fields are out of the mandatory fingerprint and in `public_state_hash`; `test_dp9_loop_detection_fingerprint_excludes_the_choice_state` pins both directions. The argument is right, and `next_effect_choice_id`'s monotonicity makes it *necessary*, not merely defensible. |

---

## SR-gate check

| Gate | Result |
|---|---|
| **SR-3** (`GameState` sealed) | **Pass.** All three fields `pub(crate)`; read-only accessors at `state/mod.rs:530/536`; no `_mut`. |
| **SR-4** (silent failures classified) | **Pass with one mis-classification.** The mana gate picks the "defined behaviour" side explicitly. The replay mismatch is classified as an engine bug, which is wrong for the concede-induced case — Finding 2. |
| **SR-8** (wire closure) | **Pass.** PROTOCOL 30→31 with a `- 31:` history line, appended `ProtocolEpoch`, gate-computed fingerprint; HASH 67→68 with an appended epoch; no existing history row edited (gates green, and I read the surrounding rows). |
| **SR-19** (`HashInto` coverage) | **Pass, with the gap honestly disclosed.** Both new structs use bare type names and hash every field; `NOT_HASHED` stays `&[]`; the two new *enum* impls are outside the struct-only gate and `hash.rs:3128-3132` says so in-source (OOS-DP9-13). |
| **SR-9a** | **Pass.** New file registered under `tests/primitives/`; no top-level `tests/*.rs`. |
| **SR-9b/9c** | **Pass** for the harness (pump extended, one new action string, both golden scripts made explicit rather than edited around). SR-9b's runtime premise is where Finding 4 bites. |
| **SR-29 / trust boundary** | **Pass.** Every id is re-checked against the engine's own recorded question; `validate_partition` enforces the multiset while leaving order as player payload; the direct-handler `entry.player != player` hole PB-DP7 found is closed and tested. |
| **SR-36** | **Fail — Finding 5.** Real `all_cards()` walk, incomplete tree. |
| **SR-38** (never offer an action the engine rejects) | **Pass.** `StubProvider` offers the engine's own default; the harness pump `debug_assert`s on rejection; `answer_pending_effect_choice` `expect`s acceptance. |
| Exhaustive matches | **Pass.** `LocalGame`'s `BlockingDecision`→`DecisionKind` match, `random_bot`, `heuristic_bot`, `StubProvider`, `BlockingDecision::{player,Display}`, `hash.rs`'s no-`_`-arm `GameEvent` match (131), TUI `stack_view`/replay-viewer `view_model` unmoved. |
| Driving loops | **Pass.** TUI auto-pass + `acting_player` read `blocking_decision()`; `LocalGame::advance` compile-forced; `GameDriver`/fuzzer inherit; harness pump extended (`replay_harness.rs:396-408`, `MAX_ROUNDS = 256`); TUI `'r'` key added at `input.rs:110` with the not-compile-forced hazard stated in-source. |
| Hidden information (Invariant 7) | **Pass.** `private_to()` added and returns `Some(player)` for `EffectChoiceRequired` and `CleanupDiscardChoiceRequired`; `reveals_hidden_info()` extended; the leak probe is non-vacuous (serializes the event and asserts no card name appears). I checked the companion paths: `Scried`/`Surveilled` carry only `(player, count)`, the search emits nothing about the candidate set, `StubProvider` gives every other seat `[]`, and the TUI formatter deliberately prints neither ids nor counts. No public event leaks what the private one protects. `Scried`/`Surveilled` still returning `false` from `reveals_hidden_info()` is correctly seeded as OOS-DP9-6. |

---

## Card Def Summary

`git`-visible card-def edits: **0**, as predicted. No completeness flips. The behavioural change to
`Complete` defs is delivered entirely by the default flip, which is the batch's stated design.

| Card (sampled) | Oracle Match | TODOs | Game State Correct | Notes |
|---|---|---|---|---|
| `demonic_tutor.rs` | Yes | 0 | Yes | `TargetFilter::default()` ⇒ CR 701.23d, finding mandatory. Correct. |
| `polluted_delta.rs` | Yes | 0 | Yes | Basic-land-with-subtype filter ⇒ CR 701.23b, may fail to find. Correct, and matches the real fetchland ruling. |
| `evolution_charm.rs` | Yes | 0 | Yes | Def is fine; **missing from the SR-36 roster** (mode-nested search) — Finding 5. |
| `insatiable_avarice.rs`, `thirsting_roots.rs`, `tooth_and_nail.rs` | Yes | (pre-existing) | Yes | Same roster gap. `tooth_and_nail` carries the pre-existing "finds one card" residual (OOS-DP9-3). |
| 17 `ChooseCreatureType` defs | Yes | 0 | Yes | None co-locates a search/scry/surveil, so Finding 4 stays latent. |

---

## Recommended fix order

1. **Finding 2** (engine, HIGH) — widen the concede discharge; it is ~6 lines and it unblocks 1.
2. **Finding 1** (test, HIGH) — 3-player fixture; assert the drive path unconditionally.
3. **Finding 5** (test, MEDIUM) — roster walk; re-publish the three numbers before the audit rows
   harden.
4. **Findings 6, 7, 8, 14** (comments, MEDIUM/LOW) — one pass; all are reachability claims.
5. **Finding 4** (engine, MEDIUM) — `BTreeMap` swap + widened seed.
6. **Finding 3** (engine, MEDIUM) — re-ask bound or force-default; correct the constant's doc.
7. **Findings 9–13** (LOW) — opportunistic.

## Seeds to file / amend

- **OOS-DP9-10** — widen from "`target_remaps` is a `HashMap`" to the resolution-scope premise,
  naming `effects/mod.rs:4204` and `replacement.rs:2089/2128` (Finding 4).
- **New** — "`Zone::reposition_within` has an unchecked membership precondition" (Finding 9).
- **New** — "`handle_concede` cannot fail safely if the discharge's resolution errors"
  (Finding 13).
- **New** — "the CR 608.2m model deviation: a suspended object is put back on the stack"
  (Finding 14).

---

## Fix-cycle disposition (2026-07-27)

Applied on branch `feat/pb-dp9-search-scry-surveil-player-choice-auto-pick-inverts-t`.
**13 of 14 findings FIXED; 1 (Finding 12) deliberately documented-and-seeded rather than
changed.** No wire type changed, so **PROTOCOL 31 / HASH 68 are unmoved**. Tests
3,905 → **3,906**; build / clippy / fmt / `check-defs-fmt.sh` all clean; fuzzer unchanged
(0 errors, 1,993 pre-existing violations, identical to the pre-fix-cycle run).

| # | Sev | Disposition |
|---|---|---|
| 1 | HIGH | **FIXED.** New `fixture_3p` + `cast_and_resolve_3p`; `test_dp9_owner_concedes_mid_choice` rebuilt on three seats, `if !over { … }` deleted, every assertion unconditional, plus a sanity assertion that two seats survive so it cannot go vacuous again. It now asserts the drive half directly: the stack is empty, the default candidate is in hand, a live seat holds priority and its `PassPriority` succeeds. Doc comment rewritten to state the fixture requirement and why. |
| 2 | HIGH | **FIXED, and the rule re-derived rather than patched.** Reproduced first (panic at `effects/mod.rs:463`, exactly as described). `discharge_departed_effect_choice` → **`discharge_effect_choice_on_concede`**: **any** concede while an entry is outstanding drops the entry and the whole bank and re-drives, because the replay's soundness premise is "the board has not changed since the questions were asked" and `Concede` is the only other admitted command, which always changes it. **The call site also moved** — from before `PlayerConceded` to after `check_game_over` — because the re-drive records a fresh question and doing so before the CR 611.2b expiry / CR 725.4 initiative transfer reproduces the same defect one step out. New probe `test_dp9_foreign_concede_invalidates_a_non_empty_bank` (3 seats, `EachPlayer`, non-empty bank, fail-before verified); `test_dp9_foreign_concede_does_not_step_over_the_block` updated to assert a fresh `choice_id` and a re-emitted question. |
| 3 | MEDIUM | **FIXED, by bounding the thing that grows.** Added a **strict-progress** check to `handle_answer_effect_choice`: banking an answer for index `i` must leave the replay finished or suspended at `index > i`; anything else is the mismatch path, i.e. an engine bug, and is rejected on its FIRST occurrence. No counter, no new `GameState` field, no HASH bump. The constant is retained for what it really bounds (bank growth from distinct choice points; `execute_effect_answering`'s loop) and its doc now says so explicitly, including *why* it cannot bound the mismatch cycle. `memory/primitive-wip.md` premise 4 corrected. |
| 4 | MEDIUM | **FIXED, and the audit widened beyond the review's list.** `Effect::ChooseCreatureType` and its `replacement.rs` ETB twin are `BTreeMap`s. **Correction to the finding: `replacement.rs:2128` (`ChooseColor`) was already deterministic** — `max_count` from `.values().max()` and a unique-max-discriminant tie-break; converted anyway. The re-run workspace audit found **three more** the review did not: `abilities.rs`'s `AnyCreatureYouControlBatchCombatDamage` map and `turn_actions.rs`'s CR 603.7b delayed-trigger map both **queued triggers in map iteration order**, i.e. CR 603.3b stack order; and `replacement.rs:1281` built `PendingZoneChange.already_applied` from a `HashSet` **without the sort its own sibling in `pending_draws` documents as "load-bearing, not cosmetic"** — and that `Vec` is fed element-by-element into the state hash. `combat.rs:1250`'s menace check made deterministic too (it only affected which `ObjectId` an error message named). `resolve_top_of_stack`'s doc now states the premise at **resolution** scope. OOS-DP9-10 widened and re-classed rankable. |
| 5 | MEDIUM | **FIXED, more broadly than prescribed.** Rather than adding `modes` + `CoinFlip` + `Splice` + `on_cast_effect` arms to a walk that would rot again, the roster is now a **structurally complete serde walk** of the serialized `CardDefinition` (no `#[serde(skip)]`/`skip_serializing*` anywhere in `card_definition.rs`; externally-tagged enums make a variant an object key; no name collisions for the three variants — all checked). The old walk missed **more than the review found**: not just `modes` and `CoinFlip` but `AbilityDefinition::SagaChapter`, `AbilityDefinition::LoyaltyAbility` and split-card halves entirely — **ten** defs, not four. New numbers **73 / 16 / 8 `Complete` (+25 / +3 / +1)**. **Correction to the finding: `tooth_and_nail` is `partial`, not `Complete`** (its own note, OOS-DP9-3), so the by-name regression guard asserts roster membership for all four and `Complete` membership for the other three. `test_dp9_mana_ability_gate` re-pointed at the same walk. Numbers corrected in `memory/primitive-wip.md` and in the audit's §4.9, §5 (DP-7/8/9 rows) and §8. |
| 6 | MEDIUM | **FIXED.** Both comments (`effects/mod.rs:199-200`, `mana.rs:872`) rewritten to say there is deliberately no assertion, give the CR 605.4a reason, and name `test_dp9_mana_ability_gate`'s roster assertion as where the skipped obligation is discharged. |
| 7 | MEDIUM | **FIXED by making the comment true.** The exit-4 coverage claim is gone. The doc now states that SBA-elimination-while-blocked is **unreachable** — no SBA runs while the block stands, because `process_command`'s admission gate admits only the answer and `Concede`, and neither reaches `check_state_based_actions` without first clearing the entry — records that `blocking_decision`'s liveness filter is defence in depth that does **not** clear the field, seeds the residual trap state as **OOS-DP9-14**, and states that widening the admission gate obliges a second caller. |
| 8 | LOW | **FIXED.** Rewritten to point at `private_to()`, which returns `None` here, and to record OOS-DP8-6's *declaration* half closed while its consumer half (no M10 filter exists) stays open. |
| 9 | LOW | **FIXED.** `Zone::reposition_within` `debug_assert!`s that every named id is already in the zone, classified SR-4 engine-bug (both engine callers partition a list the engine itself produced). |
| 10 | LOW | **FIXED.** Guard added inside `MayPayThenEffect`'s `for pid in payer_ids` loop. The record is corrected in `memory/primitive-wip.md`: 4 `resolution.rs` guard sites (not 5) and now 5 in `effects/mod.rs`; the original justification for omitting this one was simply wrong about the site. |
| 11 | LOW | **FIXED.** Case (e) added, covering `answer_effect_choice` ↔ `EffectChoice` positively and against both other decision kinds in both directions. |
| 12 | LOW | **DELIBERATELY NOT CHANGED — documented and seeded as OOS-DP9-15.** Both arms now carry the asymmetry in-source with the argument: neither is CR-wrong (701.22d / 701.25d fire regardless of how many cards were seen, and no corpus trigger reads the count), and reporting the requested N keeps `Scry 3` on an empty library distinguishable from `Scry 0`, which emits nothing at all (701.22b). Unifying one side in isolation is the trap the finding identifies; the seed carries the unification. |
| 13 | LOW | **FIXED.** The re-drive runs on a clone that is committed only on success; on error the concede stands, the entry and bank stay cleared, a `debug_assert` records it, and the unresolved stack object is picked up by the next ordinary priority round. Conceding is the one action CR 104.3a always allows and must not be gated on a resolution succeeding. |
| 14 | LOW | **FIXED.** CR 608.2m named in `resolve_top_of_stack`'s doc, with the deviation stated (the CR has no notion of a resolving object being put *back* on the stack) and the argument for why it is unobservable through legal commands. |

### What the review got wrong

1. **`replacement.rs:2128` was already deterministic** (Finding 4). `max_count` comes from
   `.values().max()` and the tie-break picks the unique highest colour discriminant, so
   iteration order never reached the outcome. Converted anyway; noted in-source.
2. **`tooth_and_nail` is `partial`, not `Complete`** (Finding 5). All four mode-nested defs
   were genuinely missing from the roster, but only three are `Complete`.
3. **Finding 5's list of gaps was too short.** The walk also skipped
   `AbilityDefinition::SagaChapter`, `AbilityDefinition::LoyaltyAbility` and split-card
   halves — ten missing defs across the three rosters, not four.
4. **Finding 4's site list was too short in the other direction.** Three further
   outcome-reaching unordered iterations exist (`abilities.rs`'s combat-damage batch map,
   `turn_actions.rs`'s CR 603.7b map, `replacement.rs:1281`'s unsorted hashed `Vec`), and
   two of them are *live* CR 603.3b ordering hazards rather than latent ones.
5. **Finding 3's prescribed remedies were both heavier than necessary.** Neither a
   `reasks: u32` field (a HASH bump) nor a force-default flag is needed: `index` already
   encodes progress, so a strict-progress comparison detects the cycle on its first turn.

Everything else in the review held up under implementation, including both HIGH
reproductions.
