# Primitive Batch Review: PB-DP1 — priority after cast / activate / special action goes to the ACTOR

<!-- last_updated: 2026-07-26 -->

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-149`
**CR Rules verified via mtg-rules MCP**: 116.2 (a/b/c/d/e/f/g/h/i/j/k/m), 116.3, 117.3a-d,
117.4, 508.1, 509.1, 601.2i, 602.2 (a/b only — **602.2e does not exist**), 605.1a, 605.3a/b,
605.5, 606.1, 606.3, 702.24a, 702.30a, 702.59a, 702.167a, 716.2a
**Engine files reviewed**: `rules/casting.rs`, `rules/abilities.rs`, `rules/engine.rs`,
`rules/resolution.rs`, `rules/combat.rs`, `rules/priority.rs`, `rules/turn_structure.rs`,
`rules/lands.rs`, `rules/foretell.rs`, `rules/plot.rs`, `rules/suspend.rs`,
`rules/commander.rs`, `rules/mana.rs`, `crates/simulator/src/{legal_actions,local_game}.rs`
**Card defs reviewed**: **0** — no file under `crates/card-defs/` is touched, as planned.
Coverage must stay 1,139/1,804.

## Verdict: **needs-fix**

The core of the batch is right and the mechanical work is clean. Every one of the 14 Group-A
assignment/event pairs agrees (checked exhaustively, not sampled); Group B is byte-identical
with a ruling I independently confirm against CR 702.30a/702.24a/702.59a; Group C is
untouched behaviourally at every one of the 13 sites; and — the thing I most expected to find
and did not — **no test and no golden script silently re-encodes the old active-player
behaviour**. I enumerated every `assert*(…priority_holder…)` site in `crates/engine/tests`
and every `117.3c` occurrence in the corpus: each reordered pass-list is engine-forced (the
harness records `CommandRejected` on a wrong order, `script_replay.rs:167-193`), every note
names the actual actor, and the one test the task flagged
(`core::resolution::test_608_1_priority_goes_to_active_player_after_resolution`) has its
CR 117.3b assertion intact at `:392` with only the setup sequence at `:387` moved. AC 5513 is
satisfied.

Three MEDIUM findings stand. The largest is Group D-c: granting priority in three handlers
that have **no entry priority guard** is a strict pessimization — an identity write when the
command is legal, and a *priority theft* when it is not. Second, D-c ships with **zero test
coverage**: P9 is vacuous with respect to it and would pass unchanged if commit `7851da58`
were reverted. Third, the PRESERVE'd mana-ability `players_passed` non-reset is a real
CR 117.4 deviation that this batch has now hard-pinned (P7, "Never weaken this test") and
newly made a golden script depend on (`stack/066`), with a note citing "CR 117.3b
parenthetical" as authority for a rule that says nothing about `players_passed`.

No HIGH. Wire confirmed: `PROTOCOL_VERSION == 27` (`rules/protocol.rs:260`),
`HASH_SCHEMA_VERSION == 63` (`state/hash.rs:578`), neither file touched.

---

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `rules/engine.rs:1650`, `:2686`, `:2849` | **Group D-c grants priority in three unguarded handlers.** Identity write when legal; steals priority when not. Strictly worse than pre-fix in the illegal case, zero gain in the legal case. **Fix:** add the CR-mandated entry priority guard to all three, or revert the writes. |
| 2 | **MEDIUM** | `crates/engine/tests/primitives/pb_dp1_actor_priority.rs:545-604` | **D-c has zero test coverage.** P9 seeds `priority_holder = Some(p2)` and asserts `Some(p2)` — passes identically with `7851da58` reverted. No probe at all for loyalty / level-up. **Fix:** add a probe that fails if the write is removed. |
| 3 | **MEDIUM** | `test-data/generated-scripts/stack/066_…json:187`; `rules/mana.rs:628-630`; `pb_dp1_actor_priority.rs:405-409` | **A CR 117.4 deviation is now pinned and cited as CR-blessed.** A mana activation between passes breaks succession per CR 117.4's own parenthetical; the engine keeps `players_passed`. PRESERVE is fine, the citation is not. **Fix:** relabel as a known deviation, cite CR 117.4, file the seed. |
| 4 | LOW | `rules/commander.rs:1022-1026` | **Non-sequitur justification.** "the `:941` sorcery-speed gate forces `player == active_player`, so no write is needed" — being the AP does not imply holding priority. **Fix:** reword. |
| 5 | LOW | `rules/resolution.rs:5175`; `rules/lands.rs:24`, `:419` | Stale in-comment line refs: `7744` (site is `7751`), `:31` (guard is `:32`). **Fix:** correct or drop the numbers. |
| 6 | LOW | `rules/engine.rs:1650` vs `:1652` | `handle_turn_face_up` writes `priority_holder` *before* `check_and_apply_sbas`; `handle_activate_craft` writes *after* (`:1480`/`:1487`). **Fix:** move the write below the SBA call. |
| 7 | LOW | `crates/engine/tests/primitives/pb_dp1_actor_priority.rs` | D-b coverage is 1-of-4 (foretell only). plot / suspend / bring_companion resets untested. **Fix:** add three cheap probes. |
| 8 | LOW | `casting/casting.rs:87`; `rules/abilities.rs:189`; `casting/mana_and_lands.rs:96`, `:118` | Residual pre-fix prose ("Active player retains priority") on assertions that now pass only because actor == AP. **Fix:** reword to CR 117.3c. |
| 9 | LOW | `primitives/pb_ef8_exile_self_from_hand.rs:182-185`, `:209`, `:219` | Still calls a mana ability "a special action (CR 605.5)" — the exact misclassification PB-DP1 corrected in `mana.rs`. **Fix:** apply the same correction. |
| 10 | LOW | `primitives/pb_ef2_create_token_recipient.rs:311-315` | Comment says "same shape as the happy-path test above" (`[p2, p1]`); the call is `pass_all(&[p2, p1, p1, p2])`. **Fix:** correct the comment. |
| 11 | LOW | `docs/audits/decision-point-audit.md:428`, `:560` | Plan step 16 not done: DP-1 row not marked SHIPPED and still lists the five confirmed false positives (`engine.rs:757/958/1072/1759/1805`, `combat.rs:1373`) as DP-1 sites; §8 PB-DP1 row unmarked; seeds OOS-DP1-1/2/3 not filed. **Fix:** run step 16. |

---

## Finding details

### Finding 1 — Group D-c grants priority in three handlers with no entry priority guard

**Severity**: MEDIUM (ruling requested by the task; see below)
**Files**: `crates/engine/src/rules/engine.rs:1650` (`handle_turn_face_up`), `:2686`
(`handle_activate_loyalty_ability`), `:2849` (`handle_level_up_class`) — commit `7851da58`
**CR**: 116.2b "A player can take this action **any time they have priority**"; 606.3 "A
player may activate a loyalty ability … **any time they have priority** and the stack is empty
during a main phase of **their turn**"; 716.2a "Activate only … as a sorcery"; 116.3 / 117.3c.

**Verified in source.** None of the three has an entry priority check, and there is no global
one: `process_command` gates these commands with `validate_player_active`
(`engine.rs:1964-1970`), which only rejects `has_lost || has_conceded`. `handle_turn_face_up`
checks zone / face-down / `face_down_as` / controller (`:1510-1529`) and nothing about
priority. `handle_activate_loyalty_ability` checks main phase + empty stack (`:2495-2505`) —
**not** priority and **not** "their turn", so CR 606.3 is under-enforced in two ways.
`handle_level_up_class` checks controller / battlefield / empty stack / main phase / level
(`:2712-2746`) — same two gaps.

**The ruling.** Decompose by case:

- *Actor held priority* (the legal case): pre-fix `priority_holder` simply stayed on the
  actor; post-fix we write the same value. **Identity write. Zero correctness gain.** I
  checked that nothing between handler entry and the tail write moves `priority_holder` — the
  three tails are the only writes, and `sba::check_and_apply_sbas` does not touch it
  (`sba.rs` has no `priority_holder` assignment).
- *Actor did not hold priority* (the illegal-but-accepted case): pre-fix, the command still
  mutated the board and cleared `players_passed` — already bad — but priority stayed with its
  rightful holder. Post-fix, the illegal actor **also takes priority**. A non-active player
  can now issue `LevelUpClass` or `ActivateLoyaltyAbility` during the AP's main phase with an
  empty stack (nothing checks whose turn it is) and walk away holding priority.

So the change is a no-op when legal and a pessimization when not. That is not a defensible
reading of CR 116.3 / 117.3c: both rules are conditioned on *"if a player has priority
when…"* / *"if a player takes a special action"* — a player who does not have priority cannot
legally take the action, so the rules never license handing them priority afterward. The write
manufactures a state no legal play sequence can reach.

The plan deferred the guards to DP-21 as "new enforcement surface" (§5). That reasoning was
sound *before* this write existed; it is not sound now, because this batch is what converts a
missing guard into a priority-theft vector, and R7 in the plan already flagged that PB-DP1
"makes that gap slightly easier to hit."

**Fix (pick one, in preference order):**
1. Add the entry guard to all three handlers, matching the existing pattern
   (`foretell.rs:48-53`, `plot.rs:55-60`, `suspend.rs:59-64`):
   ```rust
   // CR 116.2b (resp. 606.3 / 716.2a): the actor must have priority.
   if state.turn.priority_holder != Some(player) {
       return Err(GameStateError::NotPriorityHolder {
           expected: state.turn.priority_holder,
           actual: player,
       });
   }
   ```
   With the guard in place the tail write becomes a true identity write and the D-c comments
   become accurate as written. Note this is a *new rejection path* — run the full suite; if it
   moves tests, that is DP-21 debt surfacing, and it should be triaged, not suppressed.
2. If the guard is judged out of scope, **revert the three `priority_holder` writes**
   (keep the corrected comments, keep the `players_passed` resets) and record the whole D-c
   item as blocked on DP-21. The plan's own §5 escape hatch authorises exactly this.

Do **not** leave the current combination (write, no guard) — it is the one option strictly
worse than both alternatives.

### Finding 2 — Group D-c ships with zero test coverage; P9 is vacuous with respect to it

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dp1_actor_priority.rs:545-604` (P9)
**Issue**: P9 builds a state, sets `state.turn_mut().priority_holder = Some(p2)` at `:583`,
issues `TurnFaceUp` by p2, and asserts `priority_holder == Some(p2)` at `:599-603`. Pre-fix
this passed because the handler wrote nothing; post-fix it passes because the handler writes
the value that was already there. **It would pass identically with commit `7851da58`
reverted.** The WIP's own RED capture confirms this — P9 is listed as `ok` in the pre-fix
run. The plan called this out ("Partially"), but the consequence was not closed: the single
largest, deliberately-revertable behavioural step in the batch has no probe that can detect
its removal. There is also no probe at all for `handle_activate_loyalty_ability` (CR 606.1 →
602.2b) or `handle_level_up_class` (CR 716.2a → 602.2b), both of which the plan's §0 C3 added
to the roster specifically because the sweep had missed them.

Every other probe is sound and load-bearing: P1-P5 and P8 have verbatim RED evidence; P6 is a
declared control; **P7 is genuinely load-bearing** — its three assertions
(`players_passed.contains(&p1)`, `players_passed.len() == 1`, `priority_holder == Some(p2)`)
each fail under a plausible "tidy `mana.rs`" edit, so the PRESERVE pin is not vacuous.

**Fix**: add `test_dp1_loyalty_activation_grants_actor_priority` (and ideally a level-up
twin). Construct it so it is **sensitive to the write**, e.g. seed `priority_holder =
Some(p2)` while p1 (the Class/planeswalker controller and active player) activates, and assert
the post-state. That probe simultaneously answers Finding 1: if the correct expectation is
`Some(p2)` (priority untouched), D-c should be reverted; if it is `Some(p1)`, the guard is
mandatory. Whichever way Finding 1 is dispositioned, this probe must exist and must fail
against the other disposition.

### Finding 3 — the mana-ability CR 117.4 deviation is now pinned and cited as CR-sanctioned

**Severity**: MEDIUM
**Files**: `test-data/generated-scripts/stack/066_krosan_grip_split_second_blocks_counterspell.json:187`;
`crates/engine/src/rules/mana.rs:628-630`; `crates/engine/tests/primitives/pb_dp1_actor_priority.rs:405-409`
**CR 117.4**: "If all players pass in succession (that is, if all players pass **without
taking any actions in between passing**), the spell or ability on top of the stack resolves…"
**CR 117.1 / 605.3a**: activating a mana ability is an action a player takes with priority.

**Issue**: `p2 passes → p1 activates a mana ability → p1 passes` is not "all players pass
without taking any actions in between passing." Per CR 117.4 the round should restart; the
engine keeps `players_passed`. PRESERVE (an explicit task directive) correctly stops this
batch from changing the behaviour — but three artefacts now assert it as *correct*:

- `stack/066:187` note: *"CR 117.3b parenthetical: the mana-ability tap does not disturb the
  priority holder **or reset players_passed**, so p2's earlier pass still stands."* CR 117.3b's
  parenthetical ("other than a mana ability") governs **who receives priority after a
  resolution**. It says nothing about `players_passed`. The script is citing a rule for a
  claim that rule does not make — the exact failure the task names.
- This is also a **new dependency**: the script previously reached the same point via the
  buggy AP hand-back; the repair (`priority_pass` by p2, then a one-element trailing round
  `["p1"]`) makes the corpus structurally reliant on the deviation. `["p1","p2"]` would now
  fail, so the deviation is load-bearing for a golden script.
- `mana.rs:628-630` is the most honest of the three — it says "a deliberate, long-standing
  engine choice" — but does not name CR 117.4 as the rule being deviated from, which is
  conspicuous in a batch whose entire second half was *adding* CR 117.4 resets to
  foretell/plot/suspend/companion.
- P7's doc comment (`:405-409`) frames the non-reset purely as CR 605.3a/b + the 117.3b
  parenthetical and instructs "Never weaken this test," turning the deviation into a
  ratchet.

**Fix**: keep the behaviour (PRESERVE holds).
1. `stack/066:187` — replace the citation: the round completes because the engine does **not**
   restart the pass-round on a mana activation, a known deviation from CR 117.4's "without
   taking any actions in between passing"; do not attribute it to CR 117.3b.
2. `mana.rs:628-630` — add one sentence naming CR 117.4 as the deviated rule and pointing at
   the seed.
3. P7 — rename the intent in the doc comment from "CR 117.3b parenthetical" to "pins a known
   CR 117.4 deviation; PRESERVE directive, see OOS-DP1-4."
4. File **OOS-DP1-4**: "mana-ability activation does not restart the pass-round (CR 117.4);
   engine choice, pinned by P7 and depended on by `stack/066`; decide deliberately."

### Finding 4 — `handle_bring_companion`'s justification does not follow

**Severity**: LOW
**File**: `crates/engine/src/rules/commander.rs:1022-1026`
**Issue**: the comment argues that because the `:941` sorcery-speed gate forces
`player == active_player`, "no write is needed here." Being the active player does not imply
holding priority — the AP can pass and remain the AP. The handler has no priority guard (the
comment says so, and tracks it as OOS-DP1-2), so the newly-added unconditional
`players_passed = OrdSet::new()` at `:1027` can wipe a legitimate pass-set when the AP issues
`BringCompanion` out of priority. Same shape as Finding 1, one notch milder (round reset, not
priority theft), and it *is* a behaviour delta versus pre-fix.
**Fix**: reword to state the truth — the reset is unconditional and correct only when the
actor held priority; the missing guard is OOS-DP1-2. If Finding 1 is fixed by adding guards,
add one here too and the comment becomes accurate.

### Finding 5 — stale in-comment line references

**Severity**: LOW
**Files**: `rules/resolution.rs:5175` cites "resolution.rs:7744"; the site is now `:7751`.
`rules/lands.rs:24` and `:419-420` cite "the `:31` guard"; `:31` is the comment, the guard is
`:32`. **Fix**: correct both, or name the function instead of the line.

### Finding 6 — inconsistent write-vs-SBA ordering

**Severity**: LOW
**File**: `rules/engine.rs:1650` (write) vs `:1652` (`check_and_apply_sbas`)
**Issue**: `handle_activate_craft` writes `priority_holder` *after* its SBA pass
(`:1480` SBA, `:1487` write); `handle_turn_face_up` writes *before* (`:1650` write, `:1652`
SBA). The pre-SBA position is the one that can leave `priority_holder` on a player the SBA
pass just marked lost — INV-PI-02 (`core/invariants.rs:490`) is the detector, and no code in
`sba.rs` reassigns `priority_holder`. Marginal reachability, but free to fix.
**Fix**: move the `:1649-1650` block below the SBA call at `:1652`, matching craft.

---

## Exhaustive Group-A pairing check (task item 2)

Requested as a mechanical, non-sampled check. All 14 assignment/companion-event pairs read in
source; function boundaries taken from `^pub fn handle_` in `abilities.rs`; every event lies
inside its own handler and before the next assignment.

| handler | assign | event | both `player`? |
|---|---|---|---|
| `casting.rs::handle_cast_spell` | `4719` | `4911` | yes |
| `abilities.rs::handle_activate_ability` | `1388` | `1405` | yes |
| `abilities.rs::handle_cycle_card` | `1553` | `1559` | yes |
| `abilities.rs::handle_activate_forecast` | `1756` | `1762` | yes |
| `abilities.rs::handle_activate_bloodrush` | `1971` | `1986` | yes |
| `abilities.rs::handle_unearth_card` | `2108` | `2114` | yes |
| `abilities.rs::handle_ninjutsu` | `2349` | `2355` | yes |
| `abilities.rs::handle_embalm_card` | `2514` | `2520` | yes |
| `abilities.rs::handle_eternalize_card` | `2693` | `2699` | yes |
| `abilities.rs::handle_encore_card` | `2871` | `2877` | yes |
| `abilities.rs::handle_crew_vehicle` | `8807` | `8813` | yes |
| `abilities.rs::handle_saddle_mount` | `9019` | `9025` | yes |
| `abilities.rs::handle_scavenge_card` | `9223` | `9229` | yes |
| `engine.rs::handle_activate_craft` | `1487` | *(none — correctly not added)* | n/a |

`rg "PriorityGiven \{ player: active \}" crates/engine/src` → 4 hits, all Group C and all
correct: `priority.rs:88` (`grant_initial_priority`, CR 117.3a), `resolution.rs:115` (fizzle),
`:7752` (after resolution), `:7991` (after countering) — all CR 117.3b.
`rg "let active" crates/engine/src/rules/abilities.rs` → 1 hit, `:8453` in `apnap_order`,
unrelated. **No mismatched pair. Item 2 clean.**

## Group B verification (task item 4)

`engine.rs:763-765` (echo), `:972-974` (cumulative upkeep), `:1094-1096` (recover) all read
`players_passed = imbl::OrdSet::new(); let active = state.turn.active_player;
priority_holder = Some(active);` — logic byte-identical, only the preceding comment block
changed. The ruling is independently correct: CR 702.30a, 702.24a and 702.59a each define the
mechanic as a **triggered ability**, so the pay/decline choice happens during that ability's
resolution, when no player holds priority — CR 117.3c's antecedent is false and there is no
actor to hand priority to. The comments claim exactly that and nothing more; the CR 117.3b
framing for "re-establish a clean round" is apt. **Item 4 clean.**

## Group C verification (task item 5)

| site | state | verdict |
|---|---|---|
| `resolution.rs:114` fizzle | `Some(active)` | unchanged, CR 117.3b |
| `resolution.rs:5178` cipher free-cast | `Some(active)` | unchanged; new comment correctly derives it from CR 601.2i's false antecedent |
| `resolution.rs:5842` suspend free-cast | reset only, no holder write | unchanged, comment correct |
| `resolution.rs:7751` after resolution | `Some(active)` | unchanged, CR 117.3b |
| `resolution.rs:7990` after countering | `Some(active)` | unchanged, CR 117.3b |
| `combat.rs:682` declare attackers | `Some(player)`, AP-gated at `:46` | unchanged; CR 508.1 turn-based action, CR 117.3a — comment now correct |
| `combat.rs:1377` + event `:1378-1380` declare blockers | `Some(state.turn.active_player)` | unchanged; CR 509.1 verbatim confirms it is a turn-based action, so the *defending* player must not get it. Audit false positive confirmed |
| `engine.rs:1788`, `:1834` `enter_step` | `Some(active)` | unchanged, CR 117.3a |
| `engine.rs:1665`, `:1670` `handle_pass_priority` | `Some(next)` / `None` | unchanged, CR 117.3d |
| `engine.rs:1841/1844/1912/1922` concede | unchanged | correct |
| `turn_structure.rs:103`, `:140` | `None` | unchanged |

`core::resolution::test_608_1_priority_goes_to_active_player_after_resolution`: the runner's
claim is **verified directly**. `:387` changed from `[p1,p2,p3,p4]` to `[p2,p3,p4,p1]`
(setup only, because the setup casts as p2); the assertion at `:392`
(`priority_holder == Some(p1)` after resolution, CR 117.3b) and `:394`
(`players_passed.is_empty()`) are intact and unmoved. Sibling
`test_608_1_instant_resolves_to_graveyard` (`:172`) got the identical setup-only change.
**Item 5 clean.**

## Citation audit (task item 6)

Every replacement checked against the live CR:

| claim | verdict |
|---|---|
| CR 601.2i actually reads "If the spell's controller had priority before casting it, they get priority" | **confirmed** — the old `casting.rs` comment's "Then the active player receives priority" was a fabrication; `:4713-4717` now quotes it correctly |
| CR 602.2 has only 602.2a/602.2b; **602.2e does not exist** | **confirmed** |
| CR 602.2b is the right replacement (601.2b-i apply to activation) | **confirmed** |
| CR 116.3 has **no subrules**; "116.3b" does not exist and maps to today's CR 117.3b | **confirmed** |
| CR 117.3a-d and 117.4 text as quoted in the plan | **confirmed verbatim** |
| CR 116.2a/b/f/g/h/k as quoted | **confirmed** (CR 116.2 has twelve special actions; 116.2m unlock cost also exists — not relevant here) |
| CR 508.1 / 509.1 "turn-based action" framing on the combat sites | **confirmed verbatim** |
| CR 702.167a craft is an activated ability, sorcery-only | **confirmed** |
| CR 702.30a / 702.24a / 702.59a are triggered abilities | **confirmed** |
| CR 605.1a mana ability is an activated ability, not a CR 116.2 special action (`mana.rs` correction) | **confirmed** |
| CR 606.1 / 716.2a as the loyalty / level-up chains | **confirmed**; note CR 606.3 additionally requires "their turn", which the engine does not enforce (Finding 1) |

`rg "602\.2e|116\.3[abcd]" crates/engine/src` → 3 hits, all the intended does-not-exist prose
at `abilities.rs:129`, `:8803`, `:9014`. As designed; not a miss.

Residual mis-citations *outside* the swept scope, filed as LOW 9: `pb_ef8_exile_self_from_hand.rs`
still calls a mana ability "a special action (CR 605.5)" at `:182-185`, `:209`, `:219` — CR
605.5 actually reads "Abilities that don't meet the criteria specified in 605.1a-b and spells
aren't mana abilities," which says nothing of the sort. This is the same misclassification
PB-DP1 corrected in `mana.rs`; leaving it in a test that pins the same behaviour is an
inconsistency worth closing.

## Anti-inversion audit — AC 5513 (task item 1)

**Method**: (a) enumerated *every* `assert*(…priority_holder…)` site in `crates/engine/tests`
and classified each by whether the last actor was the active player; (b) read all 15 changed
script notes plus their `players` arrays; (c) verified the harness rejects a wrong order;
(d) verified no `pass_all` helper swallows errors.

- **No golden script asserts `priority_holder` anywhere in the corpus** (`rg` over
  `test-data/generated-scripts` → 0 hits outside a prose note). The only signal a script
  carries is the pass order, and `script_replay.rs:167-193` records `CommandRejected` on a
  failed `PassPriority`, so a reorder is load-bearing, not cosmetic. A "fix by reordering that
  keeps the old assertion" is structurally impossible here.
- All 15 changed notes name the **actual** actor and the reason, and are not boilerplate: e.g.
  `pb_ac5_alt_costs.rs:2569` correctly says *p1* is the caster and *p2* the active player,
  inverted from the other 14. `baseline/019:162`, `stack/002:118`, `010:157`, `015:174`,
  `030:172`, `043:171`, `044:173`, `045:162`, `006:195`, `062:280`, `165:278`, `198:191`,
  `066:158`, `layers/081:238`, `tokens/001:156` — each apt.
  (`stack/055` already contained "117.3c" pre-batch; p2 is the AP there, so it needed no
  change. 16 files match the string, 15 were changed.)
- Every `assert*(priority_holder)` site that resolves to the active player does so because the
  **actor was the active player**: `casting/casting.rs:88`, `rules/abilities.rs:190`,
  `casting/mana_and_lands.rs:119/495/514`, `rules/split_second.rs:597/677` (p1 casts on p1's
  turn in both). `core/resolution.rs:392` and `:649` are post-resolution / post-counter
  (CR 117.3b). `core/priority.rs`, `core/six_player.rs`, `core/concede.rs`,
  `core/invariants.rs`, `core/turn_invariants.rs`, `core/state_foundation.rs` only pass
  priority and never cast — all expected green, and per the plan's §9.3 rule a failure there
  would have been a real regression. None failed.
- The two assertions that genuinely encoded the bug were correctly flipped to `Some(p2)`:
  `casting/casting.rs:200` and `:733`, the latter in a test renamed from
  `test_cast_spell_priority_resets_to_active_player` to
  `…_priority_retained_by_actor_after_casting`. Good catch by the runner — the old name
  described the bug as the spec.
- All `pass_all` helpers `panic!` on `Err` (checked `cumulative_upkeep`, `collect_evidence`,
  `afterlife`, `cycling`, `bargain`, `buyback`, `cleave`, `cascade`, `devour` — identical
  shape across 189 files); no `.ok()` / `if let Ok` swallow pattern exists in the tree.
- The `[actor, other, other, actor]` 4-pass shape is engine-forced, not chosen: after the
  first resolution empties the stack, CR 117.3b hands priority back to the AP, so
  `[p2,p1,p2,p1]` would be rejected. The comments say exactly this
  (`aftermath.rs:621-627`, `buyback.rs:556-562`, `flashback.rs:529-535`, `jump_start.rs:561-566`,
  `pb_ef2:281-287`, `retrace.rs:525-529`, `pbt_up_to_n_targets.rs:1100-1104` + `:1116-1119`,
  `pb_ac4:550-555` + `:625-627`, `madness.rs:1049-1051`, `pb_ac5:914-917`, `:2569-2571`).

**Conclusion: AC 5513 passes. No test and no script silently encodes the old
active-player behaviour.** The single blemish is a comment/code mismatch (LOW 10).

## PRESERVE audit (task item 7)

`rules/mana.rs` has **zero executable-line changes** — the only edits are the doc block at
`:35-41` and the inline block at `:622-630`, both `///`/`//`. The `:52` priority guard, the
absence of any `priority_holder` write, and the untouched `players_passed` are intact.
P7 (`pb_dp1_actor_priority.rs:410-455`) is **load-bearing, not vacuous**: it seeds
`players_passed = {p1}` and `priority_holder = Some(p2)`, then asserts all three of
`contains(&p1)`, `len() == 1`, and `Some(p2)`. Any of the three plausible regressions (adding
a reset, adding an AP hand-back, adding the actor to the passed set) trips it. See Finding 3
for the one thing wrong with it — what it *claims* to pin, not whether it pins.

## Simulator / M11-local (task item, plan step 14)

Verified independently, not taken on report: `crates/simulator/src/legal_actions.rs:193`
returns early unless `priority_holder == Some(player)` with no active-player special-casing,
and `local_game.rs:306-311` resolves the acting seat as commander-zone-choice → `priority_holder`
→ structural AP pass. Both follow the fix mechanically. No simulator change was needed and
none was made.

## CR coverage check

| CR rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 117.3c (cast) | Yes — `casting.rs:4719` | Yes — P1, P2 | headline flip |
| 117.3c (activate, generic) | Yes — `abilities.rs:1388` | Yes — P3 | |
| 117.3c (cycling / crew / bloodrush / ninjutsu) | Yes | Partial — P4 (cycling), P5 (crew) | bloodrush + ninjutsu untested but AP-effective |
| 117.3c (8 AP-gated keyword sites) | Yes (identity writes) | Indirect | gate line named at each site |
| 117.3c (craft) | Yes — `engine.rs:1487` (identity, AP-gated `:1272`) | Indirect | |
| 117.3c (loyalty, level-up) | Yes — `engine.rs:2686`, `:2849` | **No** | Finding 2 |
| 116.3 (turn face up) | Yes — `engine.rs:1650` | **Vacuously** — P9 | Findings 1, 2 |
| 116.3 (play land) | Already correct by construction | Yes (pre-existing) | comment-only |
| 117.4 (cast / activate) | Yes, pre-existing | Yes — `casting.rs:734` | |
| 117.4 (foretell) | Yes — `foretell.rs:122` | Yes — P8 | |
| 117.4 (plot / suspend / companion) | Yes — `plot.rs:151`, `suspend.rs:182`, `commander.rs:1027` | **No** | LOW 7 |
| 117.4 (mana ability) | **Deviates, deliberately** | Pinned as correct | Finding 3 |
| 117.3a (step start, declare attackers/blockers) | Unchanged | Yes (pre-existing) | Group C |
| 117.3b (resolution / counter / fizzle) | Unchanged | Yes — `core/resolution.rs:392`, `:649` | Group C |
| 601.2i (during-resolution free casts) | Unchanged | — | cipher / suspend; comments now correct |

## Wire / gate confirmation

- `PROTOCOL_VERSION == 27` — `crates/engine/src/rules/protocol.rs:260`, file untouched.
- `HASH_SCHEMA_VERSION == 63` — `crates/engine/src/state/hash.rs:578`, file untouched.
- No enum variant, struct field, `Command` field, `Effect` or `GameEvent` variant added —
  consistent with the plan's §8 "no exhaustive-match sites" and with `cargo build --workspace`
  reported clean.
- No file under `crates/card-defs/`; **0 coverage flips**, as planned.

## Outstanding plan steps

Plan step 16 (close-out) is not done and the WIP scopes it to a later session. For the
record so it is not lost: `docs/audits/decision-point-audit.md:428` still lists
`engine.rs:757`, `:958`, `:1072`, `:1759`, `:1805` and `combat.rs:1373` as DP-1 sites — all
six are now confirmed **not** DP-1 (three are the Group-B resolution-time ruling, three are
CR 117.3a) — and neither the §5 DP-1 row nor the §8 PB-DP1 row (`:560`) is marked shipped.
Seeds OOS-DP1-1/2/3 are described in the plan but not visibly filed; add **OOS-DP1-4** per
Finding 3.
