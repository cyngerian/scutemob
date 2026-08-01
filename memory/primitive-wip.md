# Primitive WIP — PB-DX2 (gate the resolution-time commands nothing gates)

<!-- last_updated: 2026-08-01 -->

> Previous occupant: **PB-DX1 (the intervening-if dropped in the runtime lowering) — SHIPPED**,
> `scutemob-160`, PROTOCOL 31 → **32** / HASH 68 → **69**, tests **3,945** on main.
> Its WIP file is preserved verbatim at `memory/primitives/pb-wip-DX1-archive.md`.
> Authoritative queue: `memory/primitives/seed-rerank-2026-07-27.md` §4, **PB-DX1..PB-DX18**.

- **PB**: PB-DX2 — rank 2 of the PB-DX queue. Seeds **OOS-DP5-7** + **OOS-DP7-2**
  (+ riders **OOS-DP2-1**, **OOS-DP9-14**).
- **Task**: `scutemob-162`
- **Branch**: `feat/pb-dx2-gate-the-resolution-time-commands-nothing-gates-oos-d`
- **Class**: **CORRECTNESS — live exploit, trust boundary.** `Command::ChooseDredge` has no
  pending-state gate; `card: None` is a free card for any player at any time.
- **Phase**: implement
- **Plan**: `memory/primitives/pb-plan-DX2.md`
- **Review file**: `memory/primitives/pb-review-DX2.md`
- **Wire prediction**: the brief says wire-neutral (PROTOCOL 32 / HASH 69 unmoved). **Treat this as
  a hypothesis to be gate-computed, not a fact** — PB-DX1's "HASH only" prediction was half wrong.
  Any new `GameState` field is fed to the public state hash (PB-DP5's `pending_draws` moved
  HASH 63 → 64), so a new-field design **would** move HASH. The plan must state which design it
  takes and what the fingerprint consequence is *before* implementation, and state the falsifier.

## Premise re-verification (2026-08-01, base = `main` @ `27b0a1ec`)

| Cite in brief/seed | Status | Actual |
|---|---|---|
| `rules/engine.rs:534-544` — `ChooseDredge` arm checks only `validate_player_exists` | **HOLDS** | `engine.rs:534-544` verbatim; `validate_player_exists(&state, player)?` at `:538`, then straight to `handle_choose_dredge` at `:541`. No pending-state consult. |
| `replacement.rs:2925` — `handle_choose_dredge` validates the card, never a pending draw | **HOLDS** | `pub fn handle_choose_dredge` at `:2925`. `None` arm (`:2932-2939`) validates **nothing** and calls `draw_card_skipping_dredge`. `Some` arm (`:2940-3018`) validates graveyard zone / `Dredge(n)` / library ≥ n only. |
| `replacement.rs:619` — `DrawAction::DredgeAvailable` doc claims "The engine pauses until a `Command::ChooseDredge` is received." | **HOLDS** | `:617-620`. |
| `rules/events.rs:848-850` — `DredgeChoiceRequired` doc claims the same pause | **HOLDS** | `:845-853`. |
| `DrawStepOutcome::DredgeOffered` doc says the caller does NOT stop | **HOLDS** | `:764-767`; `perform_one_draw` at `:828` returns `(vec![event], DrawStepOutcome::DredgeOffered)` and records no pending state. |
| (extra site, not in the brief) `events.rs:1353-1355` already names OOS-DP7-2 | **HOLDS** | The `DiscardToHandSize` doc says of `DredgeChoiceRequired` "whose identical claim is not implemented (seed OOS-DP7-2)". A **third** doc site tied to this seed; it must be reconciled with whatever this batch does, or it becomes the new lying comment. |
| OOS-DP2-1 cite `rules/commander.rs:877-885` for `handle_keep_hand` | **DRIFTED** | `pub fn handle_keep_hand` is at `commander.rs:891`; the count-only check is in its body below that. Correct the cite on closure (the OOS-DP6-8 documentation-rot class). |
| OOS-DP9-14 — `pending_effect_choice` reaped only by `handle_concede` | **HOLDS** | `discharge_effect_choice_on_concede` at `engine.rs:2580-2583`; `drop_departed_trigger_flush` called at `engine.rs:2664`; `resolve_top_of_stack` (`resolution.rs:92`) has an entry `debug_assert!(state.pending_effect_choice.is_none())` and no reap. |
| Golden script `replacement/014` + `tests/mechanics_a_d/dredge.rs` reach `DredgeChoiceRequired` first | to be confirmed in plan/implement | — |

**Nothing in the brief was falsified.** One line-number drift (OOS-DP2-1 → `commander.rs:891`).

## Implementation progress (plan §11 step numbering)

- [x] Step 1 — Phase 0 probes T1, T2, T5, T10 written in
      `crates/engine/tests/primitives/pb_dx2_command_gates.rs`; all four FAIL
      pre-fix as predicted (T1/T2: `ChooseDredge` succeeds with no gate; T5:
      3 `DredgeChoiceRequired`, 0 `CardDrawn`; T10: p2's hand card moved to
      p1's library bottom). Failure text captured in
      `memory/primitives/pb-review-DX2.md` inputs (see runner report).
- [x] Step 2 — Phase 0 T14 written in-src at the foot of
      `crates/engine/src/rules/resolution.rs`
      (`dx2_pending_effect_choice_reap_tests`); FAILS pre-fix with the entry
      `debug_assert!` panic, exactly as predicted. T15 (live-owner
      `#[should_panic]`) already passes pre-fix (its row says "passes before
      and after").
- [x] Step 3 — extracted `perform_remaining_draws` in `replacement.rs`;
      `resolve_pending_draw`'s tail re-expressed on it, `DredgeOffered` added
      to its `matches!` set. Pure refactor — full dredge/primitives corpus
      stayed green.
- [x] Step 4 — `DredgeAvailable` arm in `perform_one_draw` records a
      `PendingDraw` with the fold guard (§4.2). T4 passes; T7 initially wrote
      the wrong assertion (expected zero `DredgeChoiceRequired` on the second
      offer instead of one folded-entry), corrected to assert exactly one
      event + one entry + `remaining == 2`.
- [x] Step 5 — `effects/mod.rs::draw_cards_for_player` break set gains
      `DredgeOffered`. T5's "stops at first offer" half now passes.
- [x] Step 6 — `handle_choose_dredge` rewritten per §4.4 (steps 0-4b);
      `draw_card_skipping_dredge` deleted; `check_would_draw_replacement`'s
      doc reference and every other prose mention of the deleted function
      (7 more sites across `replacement.rs`, `effects/mod.rs`,
      `card-types/.../replacement_effect.rs`, `dredge.rs`,
      `pb_dp5_pending_draw_choice.rs`) updated so
      `rg -n 'draw_card_skipping_dredge' crates/` → 0. T1, T2, T3, T6, T8, T9
      all pass; full dredge/mechanics_e_l/primitives corpus green except T10
      (KeepHand rider, Phase 3).
- [x] Step 7 — `cargo build --workspace` clean (no exhaustive-match sites to
      update — no enum variant changed).
- [ ] Step 8 — Phase 2: reconcile all five §5 doc sites.
- [ ] Step 9 — Phase 3: `commander.rs:891` `handle_keep_hand` per-entry hand
      guard (§8.1). Run T10, T11, T12, T13; confirm `bare_lookup_ratchet`
      unmoved.
- [ ] Step 10 — Phase 4: `resolution.rs:90` reap above the entry
      `debug_assert!` (§8.2). T14 should then pass; write and run T15.
- [ ] Step 11 — Phase 5 gates: `core` test group green, no edits to
      `rules/protocol.rs` / `state/hash.rs`. Add T16.
- [ ] Step 12 — full suite / clippy / fmt / check-defs-fmt / workspace build
      / golden scripts all green.
- [ ] Step 13 — Phase 6: roster enumeration + bench check.
- [ ] Step 14 — Phase 7: bookkeeping (seeds, closures, wip/workstream-state).
