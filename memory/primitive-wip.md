# Primitive WIP — PB-DP9 (DP-7 / DP-8 / DP-9: search, scry, surveil player choice)

<!-- last_updated: 2026-07-27 -->

> Previous occupant: **PB-DP8 (DP-6 / OOS-M11-4: triggered-ability target choice) — SHIPPED**
> `scutemob-156`, merge `48353a36`, PROTOCOL 28 → **30**, HASH 65 → **67**, tests **3,878**.

- **PB**: PB-DP9 — **DP-7 / DP-8 / DP-9** (CR **608.2d** / **701.23a** / **701.22a** / **701.25a**).
- **Task**: `scutemob-157`
- **Branch**: `feat/pb-dp9-search-scry-surveil-player-choice-auto-pick-inverts-t`
- **Class**: AGENCY + CORRECTNESS (Tier 1, class **B** ×3 → **A** ×3). Rank 9 of the PB-DP
  suite; the suite's **third** wire change.
- **Phase**: **fix — COMPLETE** (review findings 1-14 dispositioned; **closing-review
  cycle also COMPLETE** — 1 HIGH + 4 LOW dispositioned, see the Closing-review section)
- **Plan**: `memory/primitives/pb-plan-DP9.md`
- **Review file**: `memory/primitives/pb-review-DP9.md`
- **Baseline**: PROTOCOL **30**, HASH **67**, tests **3,878**
- **Shipped**: PROTOCOL **31**, HASH **68**, tests **3,905** → **3,906** after the fix cycle
  → **3,909** after the closing-review cycle (neither cycle changed a wire type, so both
  versions are unmoved throughout)

## What shipped

The engine's **first resolution-time decision channel**:
`GameState.pending_effect_choice` + `GameEvent::EffectChoiceRequired` (discriminant 131) →
`Command::AnswerEffectChoice`, backed by an **abort-and-replay** continuation for
`resolve_top_of_stack` rather than a resumable effect-list cursor.

`resolve_top_of_stack` clones the state at entry. An effect that needs an unanswered CR 608.2d
choice records the *question* and returns without applying anything. The wrapper **restores the
clone wholesale** — the stack object is back, no card has moved, no event has happened — records
the pending entry on that restored state, and returns exactly one event: the question. The
answer is appended to a per-resolution answer bank on `GameState` and `resolve_top_of_stack` is
called **again from the top**; execution is deterministic, so it retraces the identical path and
consumes the banked answer at the choice point.

**One command for all three effects.** CR 608.2d is one rule; CR 701.22a / 701.23a / 701.25a are
three instances of it with identical timing, actor and validity condition. So one admission-gate
entry, one `LegalAction`, one `DecisionKind`, one `BlockingDecision` variant, one harness action
string.

## Roster (SR-36, enumerated from `all_cards()`, recursive `Effect`-tree walk)

| effect | audit claimed | implement phase | **fix cycle (final)** | non-`Complete` |
|---|---:|---:|---:|---:|
| `SearchLibrary` | 74 | ~~69~~ | **73** | +25 |
| `Scry` | 16 | 16 | **16** | +3 |
| `Surveil` | 8 | ~~7~~ | **8** | +1 |

**The implement phase's own numbers were wrong** (review Finding 5). The walk was a
hand-written `match` that never descended into `AbilityDefinition::{Spell,Triggered,Activated}::modes`,
never visited `AbilityDefinition::{SagaChapter,LoyaltyAbility}` or split-card halves at
all, and omitted `Effect::CoinFlip` while claiming to cover it. Ten defs were missing:
Binding the Old Gods, Evolution Charm, Insatiable Avarice, Thirsting Roots,
Connive // Concoct (`Complete`); Tooth and Nail, Urza's Saga, Retreat to Coralhelm,
Wrenn and Seven, Kaito Bane of Nightmares (non-`Complete`). Replaced with a
**structurally complete serde walk** of the serialized `CardDefinition` — every field of
every variant at every depth, by construction, and it cannot rot as the DSL grows.
(The review said all four mode-nested defs were `Complete`; `tooth_and_nail` is `partial`.)

**0 card-def source edits, 0 completeness flips**, as predicted.

## Plan premises FALSIFIED or corrected on the branch

Recorded because the plan's own §9 exists for exactly this.

1. **`GameState::next_choice_id()` ALREADY EXISTS** and draws from `timestamp_counter` (PB-DP8's,
   `state/mod.rs:972`). The plan's §1.1 asks for a *new* field named `next_choice_id`, which
   would have shadowed it in the most error-prone way possible (`self.next_choice_id` vs
   `self.next_choice_id()`, different namespaces, no compile error). **Renamed the new field and
   minter to `next_effect_choice_id`.** The plan's *reason* for a separate counter is exactly
   right and load-bearing — `timestamp_counter` seeds every shuffle and feeds `next_object_id`,
   so bumping it between an abort and its replay would change what the replay executes.
2. **`Effect::Scry` had NO CR 701.22b guard.** The plan's T18 says "passes today; regression
   guard". It did not: the scry arm emitted `Scried { count: 0 }` for a `Scry 0`, so a
   "whenever you scry" trigger could have fired off one. (The *surveil* arm did have the
   mirror-image CR 701.25c guard, which is presumably where the assumption came from.) Fixed
   in scope; T18 is a real fix on the scry half.
3. **The answer bank is consumed DESTRUCTIVELY (`pop_front`), not by a positional cursor.** The
   plan's §1.4 describes indexing `bank[i]`, which needs a per-pass cursor field on
   `GameState`. Popping is equivalent and needs no fourth field: the roll-back restores the
   full bank, so nothing is lost on the abort path, and the wrapper recovers this choice's
   `index` as `restart_bank_len - remaining_bank_len`. That arithmetic also handles the
   question-mismatch case exactly as §1.4 prescribes — the arm suspends *without* consuming, so
   the wrapper truncates the restored bank to the good prefix and the stale tail is dropped.
4. **`MAX_EFFECT_CHOICES_PER_RESOLUTION` is enforced in the ANSWER HANDLER, not at the ask
   site.** §1.4 says "exceeding it applies defaults for the remainder", which the ask site
   cannot do without a fifth `GameState` field to carry a force-default flag.
   **The claimed equivalence was FALSE and the review (Finding 3) was right to reject it.**
   Bounding the bank's *growth* does NOT bound the ask/re-ask cycle: on a question-equality
   mismatch at index `i` the arm suspends without consuming, the wrapper computes
   `consumed == i` and truncates the restored bank back to `i`, so the bank oscillates
   between `i` and `i+1` and can never reach 64 on precisely the path the ceiling was
   written for. Fixed in the fix cycle by a **strict-progress** check instead: banking an
   answer for index `i` must make the replay reach `i+1` or finish, so a re-ask at
   `index <= i` is rejected on its FIRST occurrence — no counter, no new `GameState` field,
   no second round trip. The constant is retained for what it actually bounds (bank growth
   from distinct choice points, plus `execute_effect_answering`'s loop) and its doc now
   says so.
5. **The mana-ability gate does NOT `debug_assert!`.** §1.3 prescribes one. CR 605.4a leaves no
   room for an announcement inside a mana ability, so applying the default *is* the defined
   behaviour rather than a swallowed failure — and an assertion there makes the branch
   untestable in every `cargo test` build, which would have left the guard's claim unverified.
   Instead the branch **names where its skipped obligation is discharged**:
   `test_dp9_mana_ability_gate`'s roster assertion, which proves no `Complete` def puts one of
   the three asking effects inside a mana ability. (Plan §11 item 9 / PB-DP8's transferable
   rule (i).)
6. **`rules::loop_detection::compute_mandatory_state_hash` was private.** T21 as specified
   ("two states differing only in the choice fields") is not constructible from an external
   test, because the fields are `pub(crate)`. Made the function `pub` and re-pointed the test
   at the one construction that DOES isolate them: a rolled-back blocked resolution has a board
   byte-identical to the moment before the resolving pass, so it must have the **same**
   mandatory-state fingerprint and a **different** `public_state_hash`. Both directions pinned.
7. **OOS-DP8-14 predicted three new harness action strings.** One: `"answer_effect_choice"`.
   Correction recorded in the audit's §8.1 row.
8. **The golden-script fallout was ONE script, not the ~4 §8.1 listed.** 210 of 211 approved
   scripts were absorbed by the pump extension unchanged. `stack/071_consider_surveil_then_draw`
   needed the explicit answer (the plan called this one exactly);
   `baseline/009_read_the_bones_scry_draw` did **not** fail but was given an explicit answer
   anyway per §8.1, and its library list turned out to be **top-first**, not bottom-first —
   worth knowing for any future script that reasons about library order.
9. **`etb-triggers/205_nadaar_ventures_on_etb` was unaffected**, as §8.1 allowed for.
10. **The unit-test fallout was 25 tests across 6 files**, against §8.1's list of 9 candidate
    files. Two of the surprises are worth naming: `pb_os4b_face_aware_abilities` (its fixture
    casts the real `Opt`, i.e. a scry) and `pb_ac6_card_integration` (Land Tax — the block
    made a later `PassPriority` return `BlockedByPendingDecision`, which is the loud failure
    mode, not a silent one).
11. **`EffectContext.target_remaps` audit (OOS-DP9-10): CLEAN — but the AUDIT WAS
    MIS-SCOPED.** The `target_remaps` half is right: exactly three `insert(idx, new_id)`
    sites (`effects/mod.rs:1975`, `:2020`, `:2847`) and one `get(&idx)` (`:6699`), nothing
    iterates it. **The scope was the error** (review Finding 4): the replay re-executes the
    *whole resolution*, not the asking effect's candidate derivation, so the premise is
    resolution-scoped and every statement in it matters. The widened workspace audit ran in
    the fix cycle and fixed five sites — `Effect::ChooseCreatureType` + its ETB twin
    (`max_by_key` over a `HashMap`, ties are the common case), `abilities.rs`'s combat-damage
    batch map and `turn_actions.rs`'s CR 603.7b delayed-trigger map (**both queued triggers
    in map order, i.e. CR 603.3b stack order**), and `replacement.rs:1281`'s
    `PendingZoneChange.already_applied`, built from a `HashSet` without the sort its own
    sibling site documents as "load-bearing, not cosmetic" — and that field is hashed.
    OOS-DP9-10 is now rankable, not hygiene.
12. **SR-19's delete-a-field demonstration was RUN, and it found a gate gap (`OOS-DP9-13`).**
    Deleting `PendingEffectChoice.index` or `AnsweredEffectChoice.answer` from their `HashInto`
    impls fails `every_hashed_struct_field_is_hashed_or_allowlisted` **by name**, as designed.
    But the gate covers **structs only**: rewriting the `EffectChoiceQuestion::SearchLibrary`
    arm as `{ candidates, .. }` and dropping the `may_fail_to_find` feed passes **every gate in
    the suite green**. Same family as OOS-DP7-11. `NOT_HASHED` is `&[]` and stays empty.
13. **The benchmark obligation is discharged with no regression.** Measured on `48353a36` in a
    throwaway worktree vs this branch: `full_turn_4p` **253.10 µs → 229.31 µs**,
    `priority_cycle_4p` **25.68 → 25.34 µs**, `sba_check` **17.80 → 15.53 µs**. The
    unconditional `GameState::clone()` per resolution costs nothing measurable — `imbl`
    persistent structures make a clone a handful of refcount bumps — so the `effect_may_ask`
    pre-scan escape hatch was **not** added.

## Deliberate deviations (argued in source, not silent)

- **Scry and surveil defaults are the IDENTITY**, not the pre-PB-DP9 bottom-everything /
  mill-everything. Search keeps its lowest-`ObjectId` default byte-for-byte (zero churn).
  Argued in the helpers' docs and pinned in **both directions** by
  `test_dp9_defaults_reproduce_the_stated_behaviour`.
- **The three new fields are excluded from `loop_detection.rs`'s mandatory-state fingerprint**,
  deviating from PB-DP7 and PB-DP8, because they grow between successive replays of one
  resolution and could mask a CR 726 mandatory loop. Recorded as **obligation (7)** on the
  `BlockingDecision` doc block — the first evidence PB-DP8's six-obligation list generalises
  (PB-DP9 discharged 6/6 plus the new one).
- **CR 400.7**: scry no longer renumbers. New `Zone::reposition_within` permutes in place.
  Fallout was one test (`library_ordering`'s two-to-bottom probe), whose bottomed pair now
  orders by the player's announcement (CR 401.4) rather than by an `ObjectId` sort artefact.
- **OOS-DP9-8 not fixed**: multi-player resolution-time choices are asked in ascending
  `PlayerId` order, not APNAP (CR 608.2e / 701.22c / 701.23i). Pre-existing in
  `resolve_player_target_list` and far wider than this roster; PB-DP9 makes it *observable* for
  the first time and pins the engine's actual behaviour in `test_dp9_choice_inside_for_each_each_player`.

## Seeds filed

**OOS-DP9-1..12**, in `docs/audits/decision-point-audit.md` §8.1 (the durable inventory — this
file is rewritten wholesale by the next `/implement-primitive` run). The two to rank first:

- **OOS-DP9-3** — `Effect::SearchLibrary` finds exactly ONE card; CR 701.23 searches for "one or
  more". ~7 `partial` defs say so in their own source. On PB-DP9's machinery this is a
  `count: EffectAmount` + `found: Vec<ObjectId>` with **zero** new plumbing. **The largest
  card-yield item adjacent to this batch.**
- **OOS-DP9-11** — the same-zone CR 400.7 renumber sweep across every other caller of
  `move_object_to_zone` / `move_object_to_bottom_of_zone`.

## Mechanical verification (plan §3 / §12), recorded

```
grep -rn 'execute_effect('  crates/*/src   # 19 hits
grep -rn 'resolve_top_of_stack' crates/*/src
```

- **`execute_effect(`: 19 hits = 17 production callers + the definition + one new
  test/tool helper.** All **15** in `resolution.rs` are between `resolve_top_of_stack_inner`
  (`:131`) and the next `fn` (`execute_gift_effect`, `:7945`) — i.e. inside one function, as
  the plan's §1.3 claimed. The other two production callers are `mana.rs:876` (the CR 605.4a
  triggered mana ability — **gated**) and `replacement.rs:1964` (a literal `Effect::CreateToken`
  built two lines above — **provably unreachable**). The 19th is
  `effects/mod.rs:666`, inside `execute_effect_answering`, the new test/tool helper.
- **`resolve_top_of_stack` now has THREE production callers, not the plan's "exactly 2".** All
  three are PB-DP9's own, and each is documented at its site: `engine.rs:2243`
  (`handle_all_passed`, the pre-existing one), `effects/mod.rs:599`
  (`handle_answer_effect_choice`, the resume) and `engine.rs:2574`
  (`discharge_departed_effect_choice`, §1.5's exit-2/4 discharge). The wrapper is still the
  only suspension-aware site, and `handle_all_passed`'s two post-statements carry the argued
  no-guard comment (factored into `finish_stack_resolution`, which both resume sites call).
- **Guards added (§3, robustness only):** ~~5~~ **4** loop sites in `resolution.rs` (3 modal,
  1 `effects_to_run`; the "splice" one was miscounted — grep confirms 4) and **5** in
  `effects/mod.rs` (`Sequence`, `Repeat`, both `ForEach` arms, and `MayPayThenEffect`'s
  `for pid in payer_ids` loop, added in the fix cycle per review Finding 10 — the original
  justification for omitting it, "nothing loops after them", was simply wrong about that
  site). The single-call recursion sites (`Conditional` branches, `Choose`, `MayPayOrElse`,
  coin-flip, dice) still get none: nothing loops after them.
- **No new wire sentinel** was added in the PB-DP9 test file (OOS-DP7-8 is a standing complaint
  about exactly that growth).
- **Golden corpus**: 211 approved ran and passed, 60 retired, **0 skipped silently** (SR-9c).
- **`git diff --stat -- crates/card-defs/`**: empty. 0 source edits, 0 completeness flips.
- **`NOT_HASHED`**: `&[]`, unchanged.

## Phase log

- 2026-07-27 — plan phase opened.
- 2026-07-27 — fix phase complete (see the Fix cycle section above).
- 2026-07-27 — implement phase complete. Three commits:
  `75ee3b92` engine, `bfb8916b` tests + plumbing + roster, `f4696e09` audit + seeds + script.
  Gates: `cargo build --workspace`, `cargo test --all` (3,905 / 0 failing),
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
  `tools/check-defs-fmt.sh` (1,804 defs) — all clean.
  Fuzzer run for crash surface only (3 games × 4 players × 200 turns, seed 20260727,
  single-threaded): 0 errors, 3 wins, on **both** this branch and `48353a36`. The
  `stack_consistency` violations in very long games are pre-existing and were measured on both
  (2,080 base vs 1,993 here) — OOS-DP3-9 / OOS-M11-3, not chased. **No A/B-vs-`main` trace
  comparison was run or presented as an oracle** (PB-DP8 established that an extra `Command`
  shifts `RandomBot`'s RNG stream, so divergence there is structural).

## Fix cycle (review `pb-review-DP9.md`, 14 findings)

**All 2 HIGH + 5 MEDIUM + 7 LOW dispositioned; 13 fixed, 1 documented-and-seeded.**
No wire type changed, so **PROTOCOL 31 / HASH 68 are unmoved**; tests 3,905 → **3,906**
(one net new test; two existing concede tests rebuilt rather than added to).

### The two HIGHs, both on the concede exit

- **F1 (test vacuity)** — `test_dp9_owner_concedes_mid_choice` ran on the 2-player
  `fixture()`, so the concede ended the game and the discharge returned at its
  `is_game_over` early exit: the "drive the rolled-back resolution, do not merely clear it"
  behaviour had **zero** coverage while the test's doc comment claimed the opposite.
  Rebuilt on a new `fixture_3p` + `cast_and_resolve_3p`, with the `if !over { … }` escape
  hatch deleted and every assertion unconditional — including a sanity assertion that two
  seats survive, so the test cannot silently go vacuous again.
- **F2 (stale answer bank on a foreign concede)** — reproduced first: the shipped code
  panicked at `effects/mod.rs:463` (`replay determinism violation -- banked question
  SearchLibrary { candidates: [2, 3] } but the replay recomputed [4, 5]`) on a legal
  three-command sequence. **The rule was re-derived, not patched.** The abort-and-replay is
  sound only while the board the questions were asked against is the board the replay
  re-executes; the admission gate admits exactly two commands while blocked — the answer
  (the mechanism) and `Concede` — and a concede always mutates the board. So the
  invalidation condition is "a concede happened", by anybody, and
  `discharge_departed_effect_choice` → **`discharge_effect_choice_on_concede`** drops the
  entry AND the bank unconditionally and re-drives. Its call site also **moved**, from
  before `PlayerConceded` to after `check_game_over`: the re-drive records a fresh question,
  and recording it before the CR 611.2b expiry / CR 725.4 initiative transfer would have
  reproduced the same defect one step out.
  Side effect worth keeping: the mismatch `debug_assert!`'s SR-4 engine-bug classification
  is now **honest** — no legal command sequence can reach it.

### MEDIUMs

- **F3** — strict-progress check in `handle_answer_effect_choice` (see falsified premise 4).
- **F4** — 5 determinism sites fixed, audit re-run workspace-wide (see premise 11).
  Two review claims corrected: `replacement.rs:2128` (`ChooseColor`) was **already**
  deterministic, and the audit found three sites the review did not.
- **F5** — roster walk replaced with a serde walk (see the Roster table).
- **F6** — both stale "a `debug_assert` records if it fires" comments rewritten to name the
  real discharge (`test_dp9_mana_ability_gate`'s roster assertion).
- **F7** — exit-4 claim removed. The doc now states plainly that SBA-elimination-while-
  blocked is **unreachable** (the admission gate admits nothing that runs an SBA) rather
  than covered, names `blocking_decision`'s filter as defence-in-depth that does not clear
  the field, and seeds the residual trap state as **OOS-DP9-14**.

### LOWs

- **F8** fixed — `events.rs`'s "`private_to()` … does not exist" rewritten; OOS-DP8-6's
  declaration half recorded closed, its consumer half still open.
- **F9** fixed — `Zone::reposition_within` `debug_assert!`s its membership precondition.
- **F10** fixed — guard added to `MayPayThenEffect`'s payer loop; the record corrected
  (4 `resolution.rs` guard sites, not 5).
- **F11** fixed — `next_action_answers_the_block` case (e) covers `EffectChoice` in both
  directions against both other kinds.
- **F12** **documented, not changed**, and seeded as **OOS-DP9-15**: `Scried.count` is the
  requested N, `Surveilled.count` the actual. Neither is CR-wrong (701.22d/701.25d fire
  regardless), and reporting the requested N keeps `Scry 3` on an empty library
  distinguishable from `Scry 0`. Both arms now say so; changing one side in isolation is
  the trap.
- **F13** fixed — the re-drive runs on a clone committed only on success, so a resolution
  error can no longer make a player permanently unable to concede.
- **F14** fixed — CR 608.2m named in `resolve_top_of_stack`'s doc, with the argument for why
  the deviation is unobservable through legal commands.

### New seeds

**OOS-DP9-14** (dead-owner entry is a latent trap; exit 4 unreachable-not-handled),
**OOS-DP9-15** (`Scried`/`Surveilled` count asymmetry),
**OOS-DP9-16** (CR 603.7b delayed triggers keyed by `target_object` COLLAPSE when two share
a target — found while making that map's iteration order deterministic; ordering fixed,
collapse deliberately not).

### Gates after the fix cycle

`cargo build --workspace`, `cargo test --all` (**3,906 / 0 failing**),
`cargo clippy --all-targets --workspace -- -D warnings`, `cargo fmt --check`,
`tools/check-defs-fmt.sh` (1,804 defs) — all clean. Fuzzer re-run (3 games x 4 players x
200 turns, seed 20260727, single-threaded): **0 errors, 3 wins, 1,993 violations** —
byte-identical to the pre-fix-cycle measurement on this branch, i.e. no regression from the
concede rewrite (the violations are the pre-existing OOS-DP3-9 / OOS-M11-3 class).

## Closing-review cycle (`pb-review-DP9.md` → "Closing review", 1 HIGH + 4 LOW)

**All 5 dispositioned; 3 fixed in-engine, 1 fixed in-repo, 1 seeded-not-changed.**
No wire type changed, so **PROTOCOL 31 / HASH 68 stay put**; tests 3,906 → **3,909**
(3 new probes, all confirmed fail-before).

### HIGH-1 — the priority strand, fixed at the grant and not at the answer

The fourth appearance of the suite's recurring class. Reproduced before fixing: an
**active-player** concede under a *foreign* seat's CR 608.2d block left
`priority_holder` naming the departed seat once the block cleared, and
`blocking_decision` was `None` by then, so `PassPriority` was *admitted* and answered
`PlayerEliminated` from the conceder and `NotPriorityHolder` from everyone else.

The review offered two candidate fixes. **Neither `repair_departed_priority_holder`
at `Command::AnswerEffectChoice`'s tail nor any other repair call was added**; the
CR **800.4j** liveness test was put on the *grant* instead
(`resolution::grant_priority_after_resolution`, used at **both** unconditional grant
sites in `resolve_top_of_stack_inner` — the CR 117.3b tail and the CR 608.2b fizzle
path).

**The deciding evidence is a third probe with no CR 608.2d choice on it at all.**
`resolve_top_of_stack_inner` runs `check_and_apply_sbas` a few lines above the grant,
so a resolution that kills the active player (here: its own `LoseLife 99`) reaches the
grant with `has_lost` already true and hands priority back to a dead seat. That path
existed on `main`; **the bug is pre-existing and PB-DP9 only made it reachable by a
legal three-command sequence.** No repair call at the answer arm could have covered
it. Third reason: `enter_step`'s two grants and `handle_all_passed`'s forced-payment
grant have carried this exact liveness test all along — the two fixed sites were the
engine's only unconditional ones.

**PB-DP8's transferable rule (i), discharged.** `handle_concede`'s
`blocking_decision(state).is_none()` gate skips exactly two things under this block,
and its comment now names both: the priority advance (**a no-op by construction** —
`priority_holder` is `None` while the entry stands, now `debug_assert`ed at
`repair_departed_priority_holder`'s early return) and `advance_turn` for the
conceder's own turn (**not owed** — CR 800.4j says the turn "continues to its
completion without an active player"; the immediate `advance_turn` on the ordinary
concede path is a shortcut the CR does not require). The probe drives a whole step
boundary past the concede to evidence the second claim rather than assert it.

`repair_departed_priority_holder`'s doc block carried the false reachability claim
that `resolve_top_of_stack`'s own grant would "pick this up"; rewritten.

### The other four

- **LOW-2 fixed** — `crash-reports/crash_2026072{7,8,9}.json` (fuzzer output committed
  by `f4696e09`) removed and the directory `.gitignore`d. Rest of the branch's
  added-file set checked: three legitimate files.
- **LOW-3 seeded, behaviour unchanged** — **OOS-DP9-17**. The CR 726.1 argument for
  resetting loop detection on an answer is sound (a player choice is not a mandatory
  action); what is new is that the identity scry/surveil default lets a default-
  answering client repeat a genuine no-op forever, so the loop now runs to
  `MaxTurnsReached` instead of a draw. Ranks with OOS-DP9-1 (same root cause, bot end).
- **LOW-4 seeded + comment corrected** — **OOS-DP9-18**. `ask_or_consume_effect_choice`
  reads `has_lost || has_conceded`; `resolve_player_target_list` reads `has_lost` only,
  so a conceded player's library is still searched (CR 800.4a says it should have left
  the game). The pinning assertion's failure message now says plainly that it records a
  known deviation. Wider than the finding framed it: the engine has **no** CR 800.4a
  object sweep at all.
- **LOW-5 NARROWED, not merely seeded** — `may_fail_to_find` was "any non-default
  `TargetFilter` field is a CR 701.23b stated quality". Six runtime **board-property**
  fields are now subtracted first (`controller`, `exclude_self`, `is_token`,
  `is_nontoken`, and — on the identical argument, beyond the review's four —
  `is_attacking`, `is_blocking`); all six are documented at their declarations as
  invisible to `matches_filter`, so each narrowed nothing while buying a
  CR 701.23d-forbidden decline over the whole library. `is_tapped` / `is_untapped` /
  `has_counter_type` are deliberately **not** subtracted: those three *are* checked
  against library cards and empty the candidate list instead. The predicate stays a
  subtraction so a future field defaults to *allowing* the decline (the safe
  direction); that residual keeps **OOS-DP9-5** open, whose text now carries both axes.

### New tests (3, all fail-before verified)

- `test_dp9_active_player_concedes_under_a_foreign_block` — HIGH-1's dedicated probe,
  on an **empty** bank, plus a step boundary driven past the concede.
- `test_dp9_resolution_grant_skips_an_active_player_killed_by_an_sba` — the same defect
  with no CR 608.2d choice anywhere; the evidence for choosing the grant fix.
- `test_dp9_may_fail_to_find_ignores_non_quality_filter_axes` — both CR 701.23b and
  CR 701.23d directions on otherwise-identical filters.

`test_dp9_foreign_concede_invalidates_a_non_empty_bank` — which built HIGH-1's exact
state and asserted nothing about it — gained the three recoverability assertions,
factored into `assert_recoverable` and shared with `test_dp9_owner_concedes_mid_choice`
so the two cannot drift.

### New seeds

**OOS-DP9-17** (loop-detection reset × identity default), **OOS-DP9-18**
(`has_conceded` vs `has_lost`, CR 800.4a), **OOS-DP9-19** (four further priority-grant
sites that do not answer CR 800.4j — `enter_step`'s cleanup-SBA-round grant is still
unconditional, and three early returns in `resolve_top_of_stack_inner` grant nothing at
all; **reachability not proven**, left for their own probes). **OOS-DP9-5** widened.

### Gates after the closing-review cycle

`cargo build --workspace`, `cargo test --all` (**3,909 / 0 failing**),
`cargo clippy --all-targets --workspace -- -D warnings`, `cargo fmt --check`,
`tools/check-defs-fmt.sh` (1,804 defs) — all clean.
