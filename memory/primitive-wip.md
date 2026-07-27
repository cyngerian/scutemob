# Primitive WIP — PB-DP9 (DP-7 / DP-8 / DP-9: search, scry, surveil player choice)

<!-- last_updated: 2026-07-27 -->

> Previous occupant: **PB-DP8 (DP-6 / OOS-M11-4: triggered-ability target choice) — SHIPPED**
> `scutemob-156`, merge `48353a36`, PROTOCOL 28 → **30**, HASH 65 → **67**, tests **3,878**.

- **PB**: PB-DP9 — **DP-7 / DP-8 / DP-9** (CR **608.2d** / **701.23a** / **701.22a** / **701.25a**).
- **Task**: `scutemob-157`
- **Branch**: `feat/pb-dp9-search-scry-surveil-player-choice-auto-pick-inverts-t`
- **Class**: AGENCY + CORRECTNESS (Tier 1, class **B** ×3 → **A** ×3). Rank 9 of the PB-DP
  suite; the suite's **third** wire change.
- **Phase**: **implement — COMPLETE** (awaiting review)
- **Plan**: `memory/primitives/pb-plan-DP9.md`
- **Review file**: `memory/primitives/pb-review-DP9.md`
- **Baseline**: PROTOCOL **30**, HASH **67**, tests **3,878**
- **Shipped**: PROTOCOL **31**, HASH **68**, tests **3,905**

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

| effect | audit claimed | **enumerated** | non-`Complete` |
|---|---:|---:|---:|
| `SearchLibrary` | 74 | **69** | +23 |
| `Scry` | 16 | **16** | +1 |
| `Surveil` | 8 | **7** | +0 |

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
   cannot do without a fifth `GameState` field to carry a force-default flag. Bounding the
   *bank's growth* in `handle_answer_effect_choice` achieves the same thing (it can only be
   reached if the engine is replaying nondeterministically, i.e. an engine bug) and turns an
   unbounded ask/re-ask cycle into one diagnosable rejection.
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
11. **`EffectContext.target_remaps` audit (OOS-DP9-10): CLEAN.** The plan required every read
    to be checked for outcome-affecting iteration. Workspace-wide there are exactly three
    `insert(idx, new_id)` sites (`effects/mod.rs:1975`, `:2020`, `:2847`) and one `get(&idx)`
    (`:6699`). **Nothing iterates it.** SR-9b is safe; the seed stays hygiene-class.
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

## Phase log

- 2026-07-27 — plan phase opened.
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
