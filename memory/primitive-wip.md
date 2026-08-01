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
- **Phase**: closed (fix cycle applied — see "Fix cycle" section at the foot of this file and
  `memory/primitives/pb-review-DX2.md`'s "Fix cycle" appendix)
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
- [x] Step 8 — Phase 2: reconciled all five §5 doc sites (`replacement.rs`
      `DredgeAvailable`/`DredgeOffered`, `events.rs` `DredgeChoiceRequired` /
      `CleanupDiscardChoiceRequired` / `MiracleRevealChoiceRequired`). Verified
      by reading: no surviving comment on the dredge or miracle path claims a
      pause, a block, or a guarantee the code does not make.
- [x] Step 9 — Phase 3: `commander.rs:891` `handle_keep_hand` per-entry hand
      guard (§8.1). T10, T11, T12, T13 all pass; `bare_lookup_ratchet` green,
      unmoved (`expect_zone`, not a bare lookup); full `commander::`/mulligan
      suite (27 tests) green.
- [x] Step 10 — Phase 4: `resolution.rs:90` reap above the entry
      `debug_assert!` (§8.2), narrowed to a dead owner only. T14 now passes;
      T15 (`#[should_panic]`, live owner) written and passes, proving the
      reap did not silence the assert.
- [x] Step 11 — Phase 5 gates: `core` test group (449 tests, including
      `hash_schema::*` and `protocol_schema::*`) green; `git diff --stat` over
      `rules/protocol.rs` / `state/hash.rs` is EMPTY. T16 added and passes
      (`HASH_SCHEMA_VERSION == 69`, `PROTOCOL_VERSION == 32`, both unmoved).
- [x] Step 12 — full suite (3,971 passing / 0 failing, +16 over this
      worktree's own 3,955 baseline), `cargo clippy --workspace --all-targets
      -- -D warnings` clean, `cargo fmt --check` clean, `check-defs-fmt.sh`
      clean (1,804 defs), `cargo build --workspace` clean. **211/211 golden
      scripts pass, 60 retired (pre-existing, unrelated) — but
      `replacement/014_golgari_grave_troll_dredge.json` did NOT stay
      unchanged as plan §9.4 predicted.** Its `turn_based_action: draw_card`
      entry is (and always was) purely informational per
      `script_schema.rs`'s documented contract — no driver dispatches an
      engine Command off it — so the script never actually attempted a real
      draw; its `choose_dredge` succeeded pre-PB-DX2 purely on the exploit
      the batch closes. Fixed per plan §11 step 12's own instruction ("SR-9c
      forbids silent skips... any changed expectation needs a one-line CR
      citation... do not adjust a script to fit"): initial_state now starts
      at Upkeep and a leading `priority_round` (both players pass) drives the
      REAL Upkeep→Draw transition and its draw turn-based action (CR 504.1),
      mirroring `dredge.rs`'s `pass_all` unit-test pattern. A new dispute
      entry (append-only, the existing one preserved) documents the finding
      and fix with CR citations. All downstream assertions unchanged and
      still pass.
- [x] Step 13 — Phase 6: roster enumeration (temp in-test one-off, deleted
      after running, per plan §6/§11 step 13 — a permanent gate for 1 card
      would be theatre): **exactly 1 distinct Complete card**,
      `golgari_grave_troll.rs`, `Dredge(6)` — matches the plan's prediction
      exactly (`all_cards()` + `effective_abilities(both faces)` reports it
      twice, once per face, because the card is single-faced and both faces
      resolve to the same ability list; 1 distinct card, 0 flips, 0 def
      edits). Benches (throwaway worktree at merge base `27b0a1ec`, PB-DP9's
      method): base `full_turn_4p` 229.1 µs / `priority_cycle_4p` 26.0 µs /
      `sba_check` 14.8 µs; branch (3 runs, high ambient noise from a
      concurrent worktree build) 219.6-254.8 µs / 24.6 µs / 15.2 µs — all
      within noise of the base, no regression (criterion's own
      change-detection on a clean re-run reported "No change in performance
      detected" / an *improvement*, not a regression).
- [x] Step 14 — Phase 7: bookkeeping. Seeds **OOS-DX2-1..6** filed in
      `docs/audits/decision-point-audit.md` §8.1; **OOS-DP5-7**, **OOS-DP7-2**,
      **OOS-DP2-1**, **OOS-DP9-14** all marked CLOSED with the two stale-cite
      corrections applied in-row (OOS-DP2-1's `commander.rs:877-885` → `:891`;
      OOS-DP9-14's `drop_departed_trigger_flush`-placement claim corrected to
      name `handle_concede`); **OOS-DP7-1**'s row updated (dredge/miracle pair
      now answered). `memory/workstream-state.md` and `CLAUDE.md`'s Current
      State snapshot both updated. `Phase:` moved to `implement-complete`
      above (this batch's task instructions specify `implement-complete`,
      not `review` — no automatic review-phase handoff was requested).

## Fix cycle (2026-08-01, same day)

`memory/primitives/pb-review-DX2.md` — verdict **needs-fix**, 1 HIGH / 7 MEDIUM / 7 LOW.
All 15 findings applied; full disposition table and argument for each is in that file's
"Fix cycle" appendix. Summary:

- **Finding 1 (HIGH)** — the implement-phase "fold guard" let an unanswered dredge offer
  accumulate WITHOUT BOUND across turns and be cashed in one command at an arbitrary later
  moment, out of priority, while `events.rs`'s own doc denied it. **Fixed by replacing the
  fold with a discharge**: `replacement::perform_one_draw` now auto-resolves (as an implicit
  decline, `resolve_declined_pending_draw`) any stale `PendingDraw` for a player the instant
  another draw event arrives for them — unconditionally, before even examining what the new
  draw needs. This bounds `pending_draws` to the single most-recently-offered draw's own
  remainder (never a running sum), conserves every draw (nothing destroyed, only completed
  at a different moment than a human answer would have chosen), and — found while designing
  the fix, not prescribed by the review — closes **OOS-DX2-3** (two entries per player) as a
  full side effect, since both `pending_draws.push_back` sites are now downstream of the
  discharge. **Two early `return`s inside `perform_one_draw`'s `Proceed` arm had to be
  restructured into nested `match` tail expressions** — a bare `return` there would have
  skipped the new `events.extend(draw_events)` step and silently dropped the discharge's own
  events; caught during implementation, not by a test.
- **Findings 2, 3, 6, 7, 12** — doc-vs-code, all fixed. `PendingDraw`'s declaration doc and the
  `GameState.pending_draws` field doc now name both producers and both consumers;
  `handle_order_replacements` gained the four-case table from plan §3.3; `perform_remaining_draws`
  relocated ABOVE `resolve_pending_draw`'s doc block (Rust attaches a doc comment to the
  immediately following item — the bug was exactly this); `memory/gotchas-rules.md` rewritten
  for the gated handler; `effects/mod.rs` names both consumers.
- **Findings 4, 5** (test-validity MEDIUMs, treated as fix-phase HIGHs per
  `memory/conventions.md`) — three new tests added (T17 cross-player rejection, T18/T19 the two
  untested cross-kind cells of plan §3.3's four-case table), and `dredge.rs` test 9 rewritten to
  reach a real offer first before naming an invalid card, restoring coverage of
  `handle_choose_dredge`'s `Some`-arm validations.
- **Findings 9, 10, 13, 14** — 9 (wrong CR 104.4b justification) and 13/14 (silent no-op /
  discarded fields in the fold) are moot: the code they described no longer exists after
  Finding 1's rewrite. 10 (decline not sticky) is genuinely a feature, not a bug — documented
  in-source with a CR 616.1e citation and pinned by new test T19.
- **Finding 11** — golden script `replacement/014`'s three surviving stale-prose sites (lines
  6, 205, 216) reconciled.
- **Finding 15** — (a) no action, already correctly documented; (b) `OOS-DX2-2`'s stale cite
  corrected (`resolve_pending_draw:1402` → `perform_remaining_draws:1495`); (c) `dredge.rs:150`
  and its enclosing test's doc comment corrected.

**Extra, beyond the 15 findings**: two more "the engine pauses" sites the review did not name
— `check_would_draw_replacement`'s own doc comment, and the top-level `perform_one_draw`
doc's new "Per-player invariant" section — were found and fixed for consistency while
reconciling the family Finding 1/2/3/7/12 all touch.

**Verification**: wire-neutrality confirmed (`git diff --stat` over `rules/protocol.rs` +
`state/hash.rs` empty; `protocol_schema` / `hash_schema` gates green). Tests 3,971 → **3,974**
(+3: T17/T18/T19; T7 rewritten in place for the new discharge mechanism, not counted as new).
`cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --check` and
`tools/check-defs-fmt.sh` clean; `cargo build --workspace` clean.
