# PB-DX56 — adversarial bypass attempts (`scutemob-235`)

**Reviewer**: adversarial gate-bypass pass
**Date**: 2026-09-05
**Worktree**: `/home/skydude/projects/scutemob/.worktrees/scutemob-235`

---

## ⛔ READ THIS FIRST — THE MANDATED METHOD COULD NOT BE RUN

**The `Bash` tool is DISABLED in this session** (`Error: No such tool available: Bash. Bash is
disabled for this session, in subagents as well as here.`). That removes every one of:

* `cp <file> <file>.dx56bak` / `cp` back / `cmp` — the plant-and-restore protocol;
* `~/.cargo/bin/cargo test …` — the only way to observe RED vs GREEN;
* any rebuild at all.

So **I planted nothing and executed nothing.** Every verdict below is a **STATIC** verdict:
derived by tracing, from source, exactly which assertion in the workspace would observe the
plant. That is strictly weaker than execution and I am not pretending otherwise — this
project's own history (`OOS-DX55-4`: a revert harness that silently measured the previous
row's binary; `OOS-DX39-8`: an over-wide build detector that turned verdicts into
non-verdicts) is the argument for why a read is not a run.

**Files touched: ZERO.** No `.dx56bak` file exists anywhere in the worktree (verified by
`Glob **/*.dx56bak` → *No files found*), no source file was written, and the only file this
pass creates is this one. There is therefore nothing to restore and no `cmp` to show.

**What a follow-up with a shell must do**: take the 16 rows below verbatim as a plant list.
The nine predicted GREEN are the ones worth executing first — each is a claim that a specific
gate does not exist, and each is falsifiable in one `cargo test` run.

**Method actually used, stated so a later reader can weigh it**: for each proposed plant I
enumerated every call site of the affected function across the whole workspace by `Grep`, read
every assertion that could observe it, and asked whether any of them constrains the value the
plant changes. Where the answer is "no assertion reads it", the GREEN verdict is not an
estimate — it is a structural fact about the assertion set, and the only way it can be wrong is
if I missed a call site. I list the greps I relied on at the end of each target section so that
is checkable.

---

## Headline findings (all STATIC)

1. **`check_attachment_symmetry` has NO front-door test.** Both probes call the private
   function directly. Deleting its line from `check_all` reddens nothing in the workspace.
   `check_stack_consistency` has exactly this gate (`t10_check_all_dispatches_to_this_check`)
   and the new check did not get one.
2. **The `Err(_)` arm of `check_attachment_symmetry` — the dead-`ObjectId` direction, which is
   the entire subject of `OOS-DX22-8` and of that function's own doc comment — is exercised by
   NO test.** Both probe branches drive the `Ok(att)` arm. The test's own comment claims the
   dead-id direction "is covered end-to-end by `pb_dx56_departure_hygiene.rs`"; **that claim is
   false about this check.** That file is in a different crate, the function is private, and
   the file never calls it — it covers the ENGINE fix (F1), not the CHECK's arm.
3. **`check_no_dangling_attachment_at_rest` has NO call-site gate.** Deleting its line from
   `result_snapshot` reddens nothing. Its sibling `check_no_leaked_tokens` has no call-site
   gate either, so the answer to the task's question is: *neither* has one.
4. **`promote_if_it_crossed_a_turn` — the CR 800.4k turn-boundary promotion that is this
   batch's entire justification for calling the departed-active-player class transient — has NO
   test of any kind.** `HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN` appears in zero test files.
   Four independent mutations of that function are all predicted GREEN, including deleting its
   call from `record_violations`.
5. **Adding `HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN` to `is_transient_check` silently makes
   the promotion a no-op** and nothing catches it. The three transient names and two of the
   three hard names are pinned by name in probes; the promoted name is not.
6. **F1's second call site (`move_object_to_bottom_of_zone`) has no probe** — and, separately,
   I could not find a production path that reaches it with an attached permanent, so it is
   defensive code today. Both facts are worth stating; the batch's revert row R-E disabled
   *both* sites together and so cannot distinguish them.
7. **F2's liveness predicate's `has_conceded` half is untested**, and **the `while` loop's
   multi-entry behaviour is untested** — both F2 probes queue exactly one extra turn and use
   `has_lost`.

---

## Attempt table

`Observed` is **not** observed. It is **PREDICTED (static)**. Read the "why" column as the
evidence.

### Target 1 — `check_attachment_symmetry`

| # | Exact plant | Would run | Verdict (STATIC) | Why / narrowest fix |
|---|---|---|---|---|
| A1 | `invariants.rs:176` — delete `check_attachment_symmetry(state, &mut violations);` from `check_all` | `-p mtg-simulator --lib`, then `--workspace` | **GREEN (bypass succeeds)** | The only two callers of this private fn in the whole workspace are `check_all` and the probe, and **the probe calls it directly** (`invariants.rs:1764`, `:1780`). No test anywhere drives `check_all` over an asymmetric fixture. The execution notes measure `attachment_symmetry` at **0 raw** in the 20-game fuzz run at HEAD (§ "after" table), so no fuzz-based pin moves either. **Fix:** add the `t10_check_all_dispatches_to_this_check` treatment — a probe that plants an asymmetry and asserts `check_all(&state, None).iter().filter(\|v\| v.check == HARD_ATTACHMENT_SYMMETRY).count() == 1`, and asserts 0 on the healthy state. |
| A2 | `invariants.rs:849-864` — replace the `Err(_) => violations.push(…)` arm with `Err(_) => {}` | same | **GREEN (bypass succeeds)** | Both probe branches set `attached_to = Some(bearer)` then `Some(other)` on a **live** object, so both take `Ok(att)`. Nothing constructs a host listing a **dead** id. This is the direction the function's doc calls "the direction that never heals" and is the whole of `OOS-DX22-8`. **Fix:** the probe cannot plant a dead id through the public API in that crate — but it can: mint `ObjectId(999_999)` into `bearer.attachments` exactly the way `t_dangling_attachment_at_rest_is_a_hard_violation` mints it into `attached_to` (`invariants.rs:1824`). One `push_back(ObjectId(999_999))` and one `assert_eq!(vs.len(), 1)` closes it. |
| A3 | `invariants.rs:846` — narrow the host loop: `for host in state.objects_in_zone(&ZoneId::Battlefield).into_iter().filter(\|h\| h.characteristics.card_types.contains(&CardType::Creature))` | same | **GREEN (bypass succeeds)** | The probe's host is `ObjectSpec::creature(p(1), "Bearer", 2, 2)` — a creature — so the narrowing is invisible to it. It would blind the check to every Aura on a **land** or **planeswalker** and every **Fortification** (CR 301.5c / 702.65a), which are exactly the cases CR 704.5m/704.5n distinguish. **Fix:** add a second host to the probe that is NOT a creature (an enchanted land) and assert the check still fires — i.e. make the probe's host set non-homogeneous. |
| A4 | `invariants.rs:845` — prepend `if !state.stack_objects().is_empty() { return; }` with a plausible comment ("CR 704.3 window: suppress mid-resolution noise") | same | **GREEN (bypass succeeds)** | The probe's `GameStateBuilder` fixture has an empty stack, so it never reaches the guard. This is the realistic shape of a future "silence the noise" edit, and it is precisely wrong: the function's own doc argues there is **no** CR 704.3 window for this class, so a stack-conditioned excuse contradicts the doc while passing every test. **Fix:** assert the negative — the probe should push one `StackObject` onto the asymmetric fixture and assert the violation is still reported, pinning "there is no CR 704.3 window here" as a *test* rather than as a *sentence*. |

**Attempts: 4. Predicted successful bypasses: 4.**

Greps this rests on: `check_attachment_symmetry` appears at `invariants.rs:73, 176, 845, 1764,
1780` and nowhere else in the worktree; `HARD_ATTACHMENT_SYMMETRY` at `:81, 853, 871, 1787,
1789` and nowhere else.

### Target 2 — `check_no_dangling_attachment_at_rest` (and its sibling's call site)

| # | Exact plant | Would run | Verdict (STATIC) | Why / narrowest fix |
|---|---|---|---|---|
| B1 | `local_game.rs:547` — delete `violations.extend(invariants::check_no_dangling_attachment_at_rest(&self.state));` | `-p mtg-simulator --lib` and `-p mtg-simulator` (all integration targets) | **GREEN (bypass succeeds)** | The probe `t_dangling_attachment_at_rest_is_a_hard_violation` calls the `pub fn` **directly** (`invariants.rs:1806`, `:1825`); it never builds a `GameResult`. No test in `crates/simulator/tests/` names `dangling_attachment_at_rest` at all (grepped). The execution notes measure this class at **0** in the fuzz run, so no fuzz pin moves. **Fix:** one assertion in `pb_dx32_fuzz_output.rs` that plants a dangling `attached_to` into a real `LocalGame`'s state and asserts `result_snapshot(None, None).violations` carries the class — the `T1.1`/`T2.3` idiom, which exists in that file precisely for this shape. |
| B2 | `local_game.rs:541` — delete `violations.extend(invariants::check_no_leaked_tokens(&self.state));` (the *sibling*, to answer the task's explicit question) | same | **GREEN (bypass succeeds)** | Same shape. `T4.2` calls `invariants::check_no_leaked_tokens` directly (`pb_dx32_fuzz_output.rs:763`, `:778`); `T4.1` calls it directly on `game.state()` (`:756`); `local_game_playthrough.rs:431` **re-derives the token scan by hand** rather than reading the `GameResult`. So **the answer to the task's question is that neither has a call-site gate** — this is a PB-DX32 hole PB-DX56 inherited and copied. **Fix:** one gate covering both, asserting `result_snapshot` output contains both end-state classes on a planted state. |
| B3 | `local_game.rs:547` — keep the call but route it to the transient bucket: change `violations.extend(…)` to `let mut t = self.transient_violations.clone(); t.extend(invariants::check_no_dangling_attachment_at_rest(&self.state));` and use `t` for the `transient_violations` field | same | **GREEN (bypass succeeds)** | The probe's `assert!(!is_transient_check(HARD_DANGLING_ATTACHMENT_AT_REST))` checks the **predicate**, not where `result_snapshot` puts the result — so classifying it correctly and then *filing* it in the wrong bucket passes. This is the "whitewash" the probe's own message says it exists to prevent, achieved without touching `is_transient_check`. **Fix:** same as B1 — the gate must read the bucket, not the predicate. |
| B4 | `invariants.rs:909` — narrow to `for obj in state.objects_in_zone(&ZoneId::Battlefield).into_iter().filter(\|o\| o.attachments.is_empty())` | same | **GREEN (bypass succeeds)** | The probe's `Bearer` has an empty `attachments`, so the filter is invisible. It would blind the check to any permanent that is both a host and an attacher — a Fortification on a land that itself carries an Aura, or Equipment attached to a creature that carries an Equipment (legal via animation). Narrow, but real, and free to catch. **Fix:** give the probe's dangling subject one live attachment of its own. |

**Attempts: 4. Predicted successful bypasses: 4.**

Greps this rests on: `check_no_dangling_attachment_at_rest` appears at `invariants.rs:72, 82,
907, 1806, 1825` and `local_game.rs:547` and nowhere else; `HARD_DANGLING_ATTACHMENT_AT_REST`
at `invariants.rs:85, 916, 1827, 1829` only — no occurrence in any `tests/` directory.

### Target 3 — `promote_if_it_crossed_a_turn` / `is_transient_check`

| # | Exact plant | Would run | Verdict (STATIC) | Why / narrowest fix |
|---|---|---|---|---|
| C1 | `invariants.rs:100-104` — add a fourth name: `TRANSIENT_ORPHANED_TOKENS \| TRANSIENT_DEPARTED_ACTIVE_PLAYER \| TRANSIENT_ATTACHMENT_VALIDITY \| HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN` | `-p mtg-simulator --lib`, `-p mtg-simulator`, `--workspace` | **GREEN (bypass succeeds) — and this is the sharpest one** | `HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN` occurs in **exactly two places in the workspace**: its own `const` (`invariants.rs:51`) and the assignment in `promote_if_it_crossed_a_turn` (`local_game.rs:717`). **Zero test files mention it.** `record_violations` promotes and *then* asks `is_transient_check`, so this plant makes the CR 800.4k promotion — the entire "strictly stronger property" that licenses calling the CR 800.4j class transient — route straight back into the transient bucket. The batch's headline honesty claim becomes false with a one-token edit. Nor does `T4.1`'s `violations().all(\|v\| !is_transient_check(&v.check))` help: that assertion is **tautological** given `record_violations`' shape (it can only be violated by a *second, drifted* copy of the split), and widening the transient set makes it *more* true. **Fix:** an exhaustive-by-construction gate — assert `is_transient_check` returns `true` for exactly the three `TRANSIENT_*` consts and `false` for **each** of the three `HARD_*` consts by name, in one test. Two of the three false-cases already exist in separate probes; the promoted one is the missing third and is one line. |
| C2 | `local_game.rs:699` — `if v.turn_number <= first` → `if v.turn_number >= first` (or `< first`) | same | **GREEN (bypass succeeds)** | Nothing anywhere asserts a promotion happens or does not happen. With `>=`, *every* departed-active report on the first turn it is seen is promoted to HARD — which would flip 189 fuzz reports from transient to hard and blow up the execution notes' "HARD total 0" claim **if anything asserted it**, but no committed test does. `local_game_playthrough.rs:466` asserts `run.violations.is_empty()` on 5 scripted seeds — a real gate, but only if one of those 5 seeds actually produces a departed active player, which the batch's own §"HARD total 0" table does not establish for that fixture. I flag this row as the one most likely to surprise a shell-equipped reviewer in *either* direction, and it should be run first. **Fix:** a unit test of the promotion itself (below). |
| C3 | `local_game.rs:688-694` — replace the `arm_player=` extraction with a constant key: `let seat = "any".to_string();` | same | **GREEN (bypass succeeds)** | This re-creates, verbatim, the false-promotion defect the batch narrates finding on fuzz seed 5 (`PlayerId(4)`'s turn-154 report keyed against `PlayerId(1)`'s turn-133 one). `t_arm_player_key_is_not_shadowed_by_state_context` is a probe on **`player_consistency_evidence`'s output format**, not on the consumer — it asserts the evidence *carries* `arm_player=` and that `player=` would be ambiguous. It never checks that `promote_if_it_crossed_a_turn` reads it. So the batch's own recorded incident is re-openable with the gate green. **Fix:** the missing unit test is small and would close C2, C3 and C4 at once: build a `LocalGame` (or extract the promotion into a pure `fn(&mut BTreeMap<String,u32>, InvariantViolation) -> InvariantViolation` and test that), feed it two departed-seat violations for **different** seats at turns 10 and 11, assert **neither** promotes; then feed the **same** seat at 10 and 11 and assert the second promotes and carries `first_seen_on_turn=10` / `turns_crossed=1`. |
| C4 | `local_game.rs:641` — delete `let v = self.promote_if_it_crossed_a_turn(v);` (leave the fn, add `#[allow(dead_code)]`) | same | **GREEN (bypass succeeds)** | Same reason as C1/C3: no assertion reads the promotion. Note this is *also* the shape that makes `departed_active_first_turn` a dead field, and nothing gates that either. **Fix:** as C3. |

**Attempts: 4. Predicted successful bypasses: 4 (C2 flagged as the one that might redden a
scripted-playthrough seed; run it first).**

Greps this rests on: `promote_if_it_crossed_a_turn` at `invariants.rs:42, 649, 1607` (all doc
prose) and `local_game.rs:641, 672` only. `HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN` at
`invariants.rs:44, 49, 51` and `local_game.rs:717` only. `crossed_a_turn` returns **zero** hits
under `crates/simulator/tests/`.

### Target 4 — the three engine fixes

| # | Exact plant | Would run | Verdict (STATIC) | Why / narrowest fix |
|---|---|---|---|---|
| D1 | `state/mod.rs:2308` — delete `self.detach_from_host_on_departure(object_id, &old_object);` from **`move_object_to_bottom_of_zone` only**, leaving `:1798` intact | `-p mtg-engine --test primitives`, then `--workspace` | **GREEN (bypass succeeds)** | `pb_dx56_departure_hygiene.rs`'s `destroy()` helper routes `Effect::DestroyPermanent` → graveyard, i.e. `move_object_to_zone` (`:1798`). No test in the workspace attaches something and then bottoms it. The batch's revert row **R-E disabled both sites together** and therefore cannot distinguish them — that is the coverage hole stated precisely. **Second, more useful finding, from tracing the callers:** I could not find a production path that reaches `:2308` with an *attached battlefield permanent*. Its four production callers are `resolution.rs:6592/6671` (cascade/hideaway remainder), `copy.rs:602/831` (exile→library), `commander.rs:996` (mulligan hand→library) and `effects/mod.rs:6594/6838` (`RevealAndRoute` unmatched / `LookAtTopThenPlace` rest) — all library/hand/exile cards. The general `Effect::MoveZone` to `LibraryPosition::Bottom` (the Condemn shape, which *would* reach it with an enchanted permanent) does **not** route here — `effects/mod.rs:6560-6564` and `:6797-6801` record that gap explicitly as `OOS-RS1-1`. So `:2308` is correct defensive duplication with no live blast radius, and the honest statement is "no probe **and** no reachable path", not "untested defect". **Fix (cheap):** add a `t1b` using `state::test_util::move_object_to_bottom_of_zone` (already `pub`, already used by `pb_dx15a_same_zone_identity_roster.rs:432`) on an attached permanent, asserting the host's list is cleaned — one fixture, no new API. |
| D2 | `turn_structure.rs:158` — `while let Some(candidate) = turn.extra_turns.pop_back()` → `if let Some(candidate) = turn.extra_turns.pop_back()` (i.e. break on the first dead entry instead of continuing) | `-p mtg-engine --test primitives`, `--test mechanics_e_l`, `--workspace` | **GREEN (bypass succeeds)** | **Both** F2 probes queue exactly ONE extra turn: `pb_dx56_departure_hygiene.rs:285` (`extra_turns.push_back(p1)`) and `extra_turns.rs:613` (`push_back(PlayerId(2))`). With one entry, `if` and `while` are identical — pop it, find it dead, drop it, fall through to normal turn order. `t4`'s three assertions all still hold; `test_extra_turn_eliminated_player_skipped`'s three all still hold; `t5` and `test_multiple_extra_turns_stack` queue only **live** players, so the first pop succeeds and the loop never iterates. The behaviour this loses is real and is a fresh CR violation: with `[live_p3, dead_p2]` queued (LIFO, `p2` popped first), the `if` version discards `p2`'s entry **and** silently abandons `p3`'s live extra turn — CR 500.7 says `p3` takes it. **Fix:** one probe with a two-entry queue, dead on top, asserting the *live* entry underneath is honoured and the queue is empty afterwards. Currently the `while` is untested as a `while`. |
| D3 | `turn_structure.rs:161` — `.map(\|p\| !p.has_lost && !p.has_conceded)` → `.map(\|p\| !p.has_lost)` | same | **GREEN (bypass succeeds)** | Both F2 probes set `has_lost = true` (`pb_dx56_departure_hygiene.rs:287`, `extra_turns.rs:616`). No test in the workspace queues an extra turn for a player who **conceded**. CR 800.4k does not distinguish — CR 104.3a concession and CR 704.5a loss both make a player "leave the game" — and this batch's own `player_consistency_evidence` doc argues at length that the two must be reported separately, which makes the untested half here the more conspicuous. **Fix:** flip one probe (or add a third) to `has_conceded = true`. One character of fixture, and it also gives F2 the CR 104.3a leg the batch's own reasoning says it owes. |
| D4 | `engine.rs:2731` — replace `priority::grant_priority_to_active_player(state, &mut events);` with the pre-fix line `state.turn.priority_holder = Some(state.turn.active_player);` | `-p mtg-engine --test primitives`, `--workspace` | **RED (gate holds)** — *predicted, and this is the one row I expect to hold* | `t6` drives real `PassPriority` commands to `handle_all_passed` → `enter_step`, forces the cleanup SBA round with a 0-toughness creature (CR 704.5f), and asserts `priority_holder != Some(p1)` with `p1.has_lost = true`. The plant writes `Some(p1)` on that exact path. `t6` also asserts its own non-vacuity (the creature really died), which is the discipline that makes this row trustworthy. **But**: the `has_conceded` leg is untested here too — narrowing `grant_priority_to_active_player`'s predicate to `!p.has_lost` alone (a D3-shaped plant one file over) leaves `t6` GREEN, and `priority.rs:168-169`'s own doc already concedes *"The classification above is a snapshot of that `grep`, not a machine-checked invariant: a new unconditional grant added tomorrow will not fail any gate."* That is an honest NO-GATE admission and it is the residual worth recording rather than the F3 fix itself. |

**Attempts: 4. Predicted successful bypasses: 3; 1 predicted to be caught.**

---

## Per-target summary

| Target | Attempts | Predicted GREEN (bypass succeeds) | Predicted RED |
|---|---|---|---|
| 1 — `check_attachment_symmetry` | 4 | **4** | 0 |
| 2 — `check_no_dangling_attachment_at_rest` (+ sibling call site) | 4 | **4** | 0 |
| 3 — promotion / `is_transient_check` | 4 | **4** | 0 |
| 4 — the three engine fixes | 4 | **3** | 1 (D4, F3) |
| **total** | **16** | **15** | **1** |

Fifteen predicted bypasses out of sixteen is a much worse ratio than this project's usual
3-5 per batch, and I want to be careful about how it is read: **a static prediction is cheap
and a static prediction of GREEN is the cheapest kind.** Every row above is a claim that *no
assertion in the workspace reads the value the plant changes*, and that claim can only fail if
I missed a call site. I listed the greps so that is checkable. What it is **not** is evidence
that any of these plants compiles, or that some unrelated gate does not incidentally trip.

---

## Vacuous / mis-described probes

* **`t_attachment_symmetry_catches_both_asymmetries` does not catch both asymmetries.** Its
  name and its docstring both say it does ("A host listing a dead `ObjectId` is a hard
  violation, **and so is** a host listing a live attacher that points somewhere else"), and it
  only ever plants the second. Its own comment (`invariants.rs:1770-1773`) explains the
  omission by pointing at `pb_dx56_departure_hygiene.rs` — a file in another crate that cannot
  call this private function and never does. **The name overclaims and the excuse is wrong.**
  This is the batch's own recurring lesson (`OOS-DX54-6`: a self-test written by the same
  author from the same mental model exercises the inputs that author already thought of) landing
  on the flagship new check. Not vacuous — the half it tests is real — but mis-named.
* **`T4.1`'s third assertion is a tautology.**
  `game.violations().all(|v| !is_transient_check(&v.check))` cannot fail while
  `record_violations` is the only splitter, because that function routes by exactly this
  predicate. Its message ("the split must be exhaustive in both directions") describes a
  property the code enforces by construction. It is a legitimate *anti-drift* gate against a
  second copy of the split appearing, and it should say so; as written it reads as a check on
  the split's correctness, which it is not, and it is blind to C1.
* **`t_arm_player_key_is_not_shadowed_by_state_context` tests the producer, not the consumer.**
  It proves `player_consistency_evidence` emits `arm_player=` and that `player=` is ambiguous.
  It does not prove `promote_if_it_crossed_a_turn` reads `arm_player=`. C3 defeats it.

## Checks with NO call-site gate

* `invariants::check_attachment_symmetry` — not gated into `check_all` (A1).
* `invariants::check_no_dangling_attachment_at_rest` — not gated into `result_snapshot` (B1).
* `invariants::check_no_leaked_tokens` — not gated into `result_snapshot` either (B2);
  **inherited from PB-DX32, not introduced here.**
* `GameState::detach_from_host_on_departure` — the `move_object_to_bottom_of_zone` site has no
  probe (D1). No source gate anywhere counts its call sites, unlike
  `pb_dx15a_same_zone_identity_roster.rs:614`, which counts `next_object_id()` per function and
  is the exact idiom that would have covered this.
* `priority::grant_priority_to_active_player` — its own doc says so in as many words.

## Claims of this batch that look wrong to me

1. **`invariants.rs:1770-1773` — "the dead-id direction … is covered end-to-end by
   `crates/engine/tests/primitives/pb_dx56_departure_hygiene.rs`."** Verified by reading: that
   file never calls `check_attachment_symmetry` (grep: the symbol does not occur outside
   `invariants.rs`), and could not — the function is private to the simulator crate. What that
   file covers is F1, the engine-side supply. The **check's** dead-id arm is uncovered. This
   sentence is a coverage claim that a `grep` refutes, in a batch whose own subject is
   diagnosability.
2. **`invariants.rs:41-45` / `local_game.rs:650-671` — the CR 800.4k promotion is presented as
   "the strictly stronger property that keeps this split honest."** The *design* is right and
   the CR reading checks out (I verified CR 800.4a's last sentence, 800.4j and 800.4k verbatim
   against the rules server — all three are quoted correctly). But the property is **enforced by
   an untested function**, and C1/C3/C4 each disable it silently. A split is only as honest as
   the assertion behind the stronger property, and there is no assertion.
3. **Module header `invariants.rs:3` still says "Ten checks exist; nine of them fire from
   `check_all`".** At HEAD `check_all` dispatches **ten** (`check_zone_integrity`,
   `check_id_uniqueness`, `check_mana_non_negative`, `check_stack_consistency`,
   `check_player_consistency`, `check_turn_order`, `check_object_zone_agreement`,
   `check_attachment_validity`, `check_attachment_symmetry`, `check_no_orphaned_tokens`, plus
   conditionally `check_game_progression`) and there are **twelve** functions, two of them
   end-state-only. PB-DX56 added two checks and did not re-take this count — PB-DX32's own
   `/review` findings M3/M4 were about exactly this sentence. Low severity, but it is the same
   "a comment is a claim" failure the batch's neighbours keep filing.

## Claims of this batch that I checked and found SOUND

* **CR grounding.** CR 800.4a ("If the player who left the game had priority at the time they
  left, priority passes to the next player in turn order who's still in the game" —
  unconditional), CR 800.4j ("that turn continues to its completion without an active player"),
  CR 800.4k ("If a player who has left the game would begin a turn, that turn doesn't begin")
  are all quoted **verbatim and correctly** (checked against the rules server, not against the
  batch's own transcription). The transient/hard asymmetry between the two
  `check_player_consistency` arms follows from those two rules and is right.
* **F1 is CR-safe on same-zone moves.** Both zone-move helpers return early through
  `reposition_within_own_zone` when `from == to` (`state/mod.rs:1640`, `:2163`), *before* the
  `detach_from_host_on_departure` call, so a battlefield→battlefield reposition does not strip
  a live attachment. That is the obvious way F1 could have broken equip/aura state and it does
  not.
* **F1's one-directionality is right and the wrong-way-round pin (`t3`) is the right shape.**
  CR 704.5m and CR 704.5n really do prescribe opposite dispositions, and doing either inside a
  zone-move helper would be outside an SBA sweep.
* **"One arithmetic" for the transient split holds.** `is_transient_check` is the only
  classifier: grepping the three literal class-name strings across `crates/` finds them only in
  the `const` declarations, in test fixtures, and in prose — `bin/fuzzer.rs` reads the two
  already-split vectors and does no string comparison of its own.
* **`t6` (F3) is non-vacuous by construction** — it asserts the zero-toughness creature actually
  died, which is what proves the branch under test was reached, and its 3-player fixture reason
  (a 2-player game would be `GameAlreadyOver`) is correct.

## Restoration

**Nothing to restore.** No file in the worktree was written or copied by this pass; `Bash` was
unavailable, so no plant was made and no backup was created. `Glob **/*.dx56bak` → no files
found. The only file created is this one.
