# Primitive WIP — PB-DP8 (DP-6 / OOS-M11-4: triggered-ability target choice) · SHIPPED

<!-- last_updated: 2026-07-26 -->

> Previous occupant: **PB-DP7 (DP-3: cleanup discard has no `Command`) — SHIPPED**
> `scutemob-155`, merge `8f890611`, PROTOCOL 27 → **28**, HASH 64 → **65**, tests **3,837**.

- **PB**: PB-DP8 — **DP-6 / OOS-M11-4** (CR **603.3d** / **601.2c** / **603.3b**).
- **Task**: `scutemob-156`
- **Branch**: `feat/pb-dp8-triggered-ability-target-choice-surface-the-84-def-ag`
- **Class**: AGENCY (Tier 1 top, class **B**). Rank 8 of the PB-DP suite; the suite's **second**
  wire change.
- **Phase**: **implement COMPLETE → review**
- **Plan**: `memory/primitives/pb-plan-DP8.md`
- **Review file**: `memory/primitives/pb-review-DP8.md`
- **Baseline**: PROTOCOL **28**, HASH **65**, tests **3,837**
- **Shipped**: PROTOCOL **29**, HASH **66**, tests **3,858**, **0 card-def edits, 0 completeness
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
