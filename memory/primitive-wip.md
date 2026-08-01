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
- **Phase**: plan
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
