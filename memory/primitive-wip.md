# Primitive WIP — PB-DP8 (DP-6 / OOS-M11-4: triggered-ability target choice) · SHIPPED (second fix cycle complete)

<!-- last_updated: 2026-07-26 -->

> Previous occupant: **PB-DP7 (DP-3: cleanup discard has no `Command`) — SHIPPED**
> `scutemob-155`, merge `8f890611`, PROTOCOL 27 → **28**, HASH 64 → **65**, tests **3,837**.

- **PB**: PB-DP8 — **DP-6 / OOS-M11-4** (CR **603.3d** / **601.2c** / **603.3b**).
- **Task**: `scutemob-156`
- **Branch**: `feat/pb-dp8-triggered-ability-target-choice-surface-the-84-def-ag`
- **Class**: AGENCY (Tier 1 top, class **B**). Rank 8 of the PB-DP suite; the suite's **second**
  wire change.
- **Phase**: **implement → review → fix → closing review → SECOND FIX COMPLETE**
- **Plan**: `memory/primitives/pb-plan-DP8.md`
- **Review file**: `memory/primitives/pb-review-DP8.md`
- **Baseline**: PROTOCOL **28**, HASH **65**, tests **3,837**
- **Shipped (implement)**: PROTOCOL **29**, HASH **66**, tests **3,858**
- **Shipped (after fix cycle)**: PROTOCOL **30**, HASH **67**, tests **3,871**, still
  **0 card-def edits, 0 completeness flips**
- **Shipped (after CLOSING-review fix cycle)**: PROTOCOL **30**, HASH **67** (unmoved —
  nothing changed a wire type), tests **3,875**, still **0 card-def edits, 0 completeness
  flips**

## What shipped

`GameState.pending_trigger_targets: Option<PendingTriggerTargets>` +
`GameEvent::TriggerTargetChoiceRequired` (discriminant 130) → `Command::ChooseTriggerTargets`.
`abilities::flush_pending_triggers` split into a public entry (drain + APNAP sort) and a private
`flush_sorted(state, sorted, head_targets)` that suspends mid-batch and is resumed by
`resume_trigger_flush`. The compliant CR 603.3d fallback is preserved verbatim as the exported
pure helper `abilities::default_trigger_targets`, which the **caller** submits as a real
`Command` — the engine never auto-picks on a decision path and still knows nothing about seat
kind (Architecture Invariant 1).

## Roster (SR-36, enumerated not grepped)

**77** effectively-`Complete` defs carry a targeted triggered ability (+21 non-`Complete`).
The audit said 84; the planner's grep said 74. Neither reproduced. The number is printed by
`pb_dp8_trigger_target_choice::test_dp8_roster_enumeration` and pinned `>= 60`.

## Divergences from the plan (all deliberate, all recorded)

1. **§5.2's `UpToN` premise is FALSIFIED.** The plan says the old code "contributed 0 targets"
   for a permanent-inner `UpToN`. It returned `None`, and the caller treats `None` as "no legal
   target" and removes the WHOLE TRIGGER. Sword of Sinew and Steel and Elder Deep-Fiend (both
   `Complete`) never once put their trigger on the stack. Fixed — CR 601.2c makes zero targets a
   legal announcement, so CR 603.3d's removal clause does not apply. This is the batch's only
   behaviour flip beyond agency, and it is what golden script 138's edit records.
2. **§4.1 never says who grants the priority the four guards were about to grant.** Added
   `PendingTriggerTargets.grant_priority_on_resume`, set by the guards via
   `mark_flush_owes_priority`, discharged (or inherited by a re-suspension) in
   `finish_resumed_flush`. Without it a resumed game has nobody holding priority. It cannot be
   inferred at resume time — the fifth call site (`check_and_flush_triggers`) owes nothing,
   because PB-DP1 moved priority assignment into the handlers ahead of the flush.
3. **`PendingTriggerTargets` does not derive `PartialEq`/`Eq`** as §2.1 declares: `PendingTrigger`
   derives neither and is SR-7-gated. Nothing compares the entry structurally.
4. **The TUI key is `'n'` (a[n]nounce), not §7.6's `'t'`** — `'t'` is tap-for-mana, `'g'` is the
   graveyard browser.
5. **§6.3's sentinel enumeration was one short**: 53 found exactly as listed, plus
   `pb_dp5_pending_draw_choice.rs`, which spells the constant `mtg_engine::HASH_SCHEMA_VERSION`
   and escaped the regex. 54 re-pinned.
6. **§10's fuzzer A/B oracle ("any winner or turn-count change is a bug") is falsified by §10's
   own body.** An extra `Command` per non-forced trigger shifts `RandomBot`'s RNG stream, so
   trace divergence is structural. Measured over 8 fixed seeds vs `main` at `--max-turns 200`:
   2 byte-identical, 4 same winner / different turn count, 1 same error class, 1 flipped from
   `Winner: P2` to the `EngineError(PlayerEliminated)` shape `main` already produces on other
   seeds. No new violation class.

## Verified, not asserted

- `§4.2`'s claim: all **30** `check_and_flush_triggers` sites in `process_command` are followed
  by exactly `all_events.extend(events);` and the end of the arm. 30 sites, 0 guards.
- `§13` item 5: the TUI's auto-pass loop and `acting_player` already read `blocking_decision()`
  and generalise for free. Read, not assumed. Only the key, the menu hint and the event
  formatter needed arms.
- **Fail-before probes actually run.** Disabling the suspension makes **14 of 18** DP8 tests
  fail; the 4 that pass (T4, T5, T10b, T19) are exactly the regression guards the plan predicted
  would pass. Restoring the pre-PB `UpToN` semantics makes T14b alone fail.
- **SR-19 delete-a-field demonstration run** (OOS-DP7-11): removing one `hash_into` line from
  each new impl makes `every_hashed_struct_field_is_hashed_or_allowlisted` fail **by name**,
  printing `TriggerTargetOption.candidates` and `PendingTriggerTargets.remaining`.

## Seeds filed

`OOS-DP8-1..10` in `docs/audits/decision-point-audit.md` §8.1. New relative to the plan's list:
**OOS-DP8-9** (`handle_concede` advances priority/turn under another player's outstanding
announcement — deliberately ungated; gating risks a hang) and **OOS-DP8-10** (a flush suspended
in `enter_step`'s Cleanup branch skips that branch's `cleanup_sba_rounds` ratchet and CR 726
check).

## Gates

`cargo build --workspace`, `cargo test --all` (**3,858 / 0**), `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1,804 defs) — all
clean. 210 approved golden scripts, **0 new skips** (SR-9c); one script (`138_emerge_elder_deep_fiend`)
corrected with its CR justification recorded in the script's own metadata.

---

## Fix cycle (review verdict `needs-fix`: 2 HIGH, 5 MEDIUM, 3 LOW engine + 2 HIGH, 2 LOW card)

**All 12 findings dispositioned; 12 fixed, 0 deferred.** Per-finding detail, including the
fail-before probe that was actually run for each, is in `memory/primitives/pb-review-DP8.md`
under "Fix-cycle dispositions".

### What changed

1. **Finding 1 (HIGH) — the answer could land on the wrong trigger.** `flush_sorted` binds
   `head_targets` **positionally** (`next_index == 1`) at the top of the loop body. Verified
   sufficient, and strictly stronger than the review's minimum: it also covers the CR 603.2c
   once-per-turn `continue`, which the lazy `take()` leaked through as well.
2. **Finding 2 (HIGH) — `UpToN` was capped at one.** `TriggerTargetOption.max: u32`, a
   `<= max` bound, and a per-slot CR 601.2c duplicate check (latent behind the old cap).
   Elder Deep-Fiend's "up to four" and Cloud of Faeries' "up to two" work.
3. **Finding 3 (MEDIUM) — the 31st `check_and_flush_triggers` site**
   (`handle_all_passed`'s overdue-payment branch) is guarded, and the plan's §16 grep is
   corrected with a statement of why it could not have found it.
4. **Finding 4 (MEDIUM) + OOS-DP8-10 — CR 726 and the cleanup ratchet.**
   `grant_priority_on_resume: bool` → `resume_site: FlushResumeSite`, so
   `finish_resumed_flush` reproduces each site's own obligation, not just the priority grant.
5. **Finding 5 (MEDIUM) + OOS-DP8-9 — a foreign concede stepped over the block.**
   `handle_concede`'s priority/turn advance is gated on `blocking_decision().is_none()`.
   The "gating risks a hang" argument is refuted in the source comment.
6. **Finding 6 (MEDIUM) — positional index shift.** `flatten_slot_answers` keeps each slot at
   its declared width. **Deviation:** `Vec<SpellTarget>` cannot hold a hole, so interior holes
   carry the documented `SpellTarget::unchosen_slot()` placeholder; trailing holes are omitted
   so an all-empty answer cannot trip CR 608.2b.
7. **Finding 7 (MEDIUM) — all six guard sites now have a fail-before-run test**, plus the
   dead-active-player fallback.
8. **Findings 8, 9, 10 (LOW)** — empty optional slot is forced; a departed owner's entry is
   reaped inside `flush_pending_triggers` (`drop_conceded_trigger_flush` renamed
   `drop_departed_trigger_flush`); `players_passed` resets on the suspend return via a
   `placed_any` flag.
9. **Card findings** — 1/2/3 are engine-side (no def edit); script `138_emerge_elder_deep_fiend`
   had both stale notes rewritten and its dispute **resolved**.

### Wire

**PROTOCOL 29 → 30, HASH 66 → 67.** Both forced by the gates, all three fingerprints and both
`FROZEN_HISTORY_PREFIX_DIGEST`s taken from failure texts, both `*_HISTORY` arrays **appended**
to with no row edited. The sentinel re-pin tax came back: **54** copies across **45** files
(the plan's §6.3 list of 53 plus `pb_dp5_pending_draw_choice.rs`), all re-pinned; no new
sentinel added (OOS-DP7-8).

### Seeds

**OOS-DP8-9 and OOS-DP8-10 CLOSED** in `docs/audits/decision-point-audit.md` §8.1, each row
rewritten to record why the deferral was wrong. Two new rows filed: **OOS-DP8-11** (the same
index shift survives on the spell path in `casting.rs`) and **OOS-DP8-12** (the padding
sentinel has no display arm).

### Gates

`cargo build --workspace`, `cargo test --all` (**3,871 / 0**), `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1,804 defs) —
all clean. 210 approved golden scripts, 0 new skips (SR-9c).

---

## Second fix cycle — closing `/review` (1 HIGH, 1 MEDIUM, 6 LOW)

**All 8 findings dispositioned; 8 fixed, 0 deferred.** Per-finding detail, including the
fail-before probe run for each, is in `memory/primitives/pb-review-DP8.md` under
"Closing-review dispositions".

**The HIGH was a game-ending deadlock introduced by the FIRST fix cycle**, and the branch's own
test built exactly that state and asserted the wrong thing about it. That is the lesson of the
batch, not the trigger-target machinery.

### What changed

1. **HIGH — a concede under a suspended batch stranded priority on the conceded player.**
   The first cycle's Finding-5 gate (`handle_concede`'s priority/turn advance gated on
   `blocking_decision().is_none()`) is *kept* — it is what stops a concede stepping over a
   half-placed CR 603.3b batch — but its justifying comment was false for
   `FlushResumeSite::None`, the resume site of all 30 in-match `check_and_flush_triggers`
   calls. The skipped debt is now discharged by `abilities::repair_departed_priority_holder`
   at the end of `resume_trigger_flush`, the earliest moment CR 603.3b permits a grant.
   The reviewer's "grant at the concede site" prescription was **evaluated and not applied**:
   it emits `PriorityGiven` while the batch is still incomplete, and `next_priority_player`
   can return `None`. `grant_priority_after_batch` factored out so the resume tail and the
   repair cannot drift.
2. **MEDIUM — the reap discharged the priority debt inside the caller's own flush.**
   `flush_pending_triggers` zeroes the reaped entry's `resume_site` first. Principle:
   *the debt belongs to a call site whose moment has passed.* Residual (the ratchet and the
   CR 726 check that the same `FlushResumeSite` carried) filed as **OOS-DP8-13**.
3. **LOW ×6** — the engine could refuse its own default answer for two mutually-distinct
   slots (fixed in code *and* the doc guarantee corrected, OOS-DP8-4 narrowed); the golden
   script driver's pump-skip is now kind-aware and cross-step (**OOS-DP8-14**); script 138's
   two backwards prose notes rewritten; the PB-DP5 sentinel comment now records both PB-DP8
   bumps; the ESM criterion-5545 test was **vacuous** and now drives
   `handle_choose_trigger_targets(&mut state, ..)` once per rejection class, proven
   non-vacuous by a mutate-before-validate probe; audit §8's "four guards" → **six** and
   `grant_priority_on_resume` → `resume_site: FlushResumeSite`.

### Wire

**PROTOCOL 30 / HASH 67 unmoved.** No wire type changed — `make_distinct_slot_defaults` picks a
different *value* for an existing field. No sentinel re-pin owed.

### Gates

`cargo build --workspace`, `cargo test --all` (**3,875 / 0**), `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1,804 defs) —
all clean. 210 approved golden scripts, 0 new skips (SR-9c).

### Tests added (4, all fail-before-run)

`test_dp8_concede_under_a_suspended_batch_does_not_strand_priority`,
`test_dp8_reap_does_not_double_grant_priority_at_a_guarded_site`,
`test_dp8_default_answer_satisfies_cross_slot_distinctness`,
`script_replay::test_pump_skip_is_cross_step_and_kind_aware`; plus the missing priority
assertion added to `test_dp8_foreign_concede_does_not_step_over_the_suspended_batch` and the
rewrite of `test_dp8_illegal_target_rejected_state_untouched`.
