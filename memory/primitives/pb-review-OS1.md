# Primitive Batch Review: PB-OS1 — Gain-Control Reversion (UntilEndOfTurn / UntilYourNextTurn)

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit**: `d2fbb77a` on `feat/pb-os1-gain-control-reversion-untilendofturnuntilyournexttur`
**CR Rules**: 514.2, 611.2a/b/c, 613.7/613.7b
**Engine files reviewed**: `crates/engine/src/rules/layers.rs`
**Card defs reviewed**: none edited (correctness fix; roster: `sarkhan_vol.rs`, `zealous_conscripts.rs`, `karrthus_tyrant_of_jund.rs` all read to verify scope)
**Test files reviewed**: `crates/engine/tests/primitives/primitive_pb32.rs`, `crates/engine/tests/primitives/pb_os1_gain_control_reversion.rs`, `crates/engine/tests/primitives/main.rs`

## Verdict: needs-fix

The engine fix is **correct, minimal, and CR-faithful in both passes**; the tests are non-vacuous and well-targeted; scope discipline is clean (no wire/hash change, no card-def edit, no out-of-scope reversion added). The **code is collectable as-is**. The one finding is a MEDIUM guidance/documentation defect, not an engine defect: the runner's follow-up flag and a committed test doc comment mischaracterize `karrthus_tyrant_of_jund` as a latent "Indefinite gain-control never reverts" bug. It is **not** a bug — Karrthus's control change is permanent by oracle text and by Scryfall ruling, and `Indefinite` (never reverting) is the correct model. The coordinator must **not** file the proposed karrthus follow-up seed, and the misleading comment should be corrected.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| — | — | — | No engine defects. Both expiry passes collect the removed `SetController` targets before `state.continuous_effects = keep;` and call `recompute_object_controller` after — ordering correct in both. `recompute_object_controller` unchanged and still private/in-module. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | MEDIUM | `karrthus_tyrant_of_jund.rs` (roster reasoning, not the def) | **Spurious "latent bug" flag.** `karrthus` correctly uses `EffectDuration::Indefinite`; the runner's WIP note + the `pb_os1_gain_control_reversion.rs` doc comment claim it is a distinct out-of-scope bug that "arguably should be `WhileYouControlSource`." It is not — Karrthus grants permanent control (CR 611.2a; Scryfall ruling: control "lasts indefinitely... doesn't wear off during the cleanup step, and it doesn't expire if Karrthus leaves the battlefield"). **Fix:** Do NOT file the karrthus follow-up seed. Correct the test's doc comment (lines ~99-107, 154-156) to state karrthus is *correctly permanent* (not the same "for as long as you control" pattern as Silumgar/Olivia). The roster **count of 2 is correct** and needs no change. |

### Finding Details

#### Finding 1: Karrthus is not a latent gain-control-reversion bug

**Severity**: MEDIUM (guidance/documentation; no shipped-behavior impact)
**File**: `crates/engine/tests/primitives/pb_os1_gain_control_reversion.rs:99-107,154-156`; `memory/primitive-wip.md` steps 7 and phase note
**Oracle**: "When Karrthus enters, gain control of all Dragons, then untap all Dragons." — no "for as long as" / "until" clause.
**CR / Ruling**: CR 611.2a — a resolution continuous effect with no stated duration lasts until end of game. Scryfall ruling (2020-08-07): "The control-change effect of Karrthus's triggered ability lasts indefinitely. It doesn't wear off during the cleanup step, and it doesn't expire if Karrthus leaves the battlefield."
**Issue**: The def models this with `EffectDuration::Indefinite`, which `is_effect_active` (`layers.rs:515`) keeps active forever and neither expiry pass touches. That is the **correct** behavior. The runner flagged it as "a distinct, out-of-scope bug (Indefinite GainControl never reverts)" and suggested it "arguably should be `WhileYouControlSource` like Dragonlord Silumgar/Olivia Voldaren." Those two cards read "gain control ... for as long as you control [source]" — a genuinely conditional duration — which is a *different oracle pattern* from Karrthus's unconditional grant. Acting on the flag would convert a correct card into a wrong one (control would incorrectly revert when Karrthus dies).
**Fix**: Coordinator: do not open the karrthus seed. Correct the roster-test doc comment to note karrthus is intentionally permanent (Indefinite) per CR 611.2a + ruling, distinct from the "for as long as you control" (WhileYouControlSource) pattern. Roster assertion (`affected.len() == 2`) is correct and stays.

## Detailed Verification Against the Review Charge

**1. Correctness of the fix (ordering load-bearing in BOTH passes)** — CONFIRMED.
- `expire_end_of_turn_effects` (`layers.rs:1590-1614`): collects `reverted` (filter `UntilEndOfTurn` + `EffectLayer::Control` + `SetController` + `SingleObject`) **before** `state.continuous_effects = keep;` (`:1609`), then loops `recompute_object_controller` **after** (`:1612-1614`). Correct.
- `expire_until_next_turn_effects` (`layers.rs:1658-1682`): identical shape, gated on `EffectDuration::UntilYourNextTurn(active_player)`; collect before `keep` reassignment (`:1677`), recompute after (`:1680-1682`). Correct.
- If reversed, `recompute_object_controller` would re-observe the still-`is_effect_active` expiring effect (both durations hardcode `true` at `:514`/`:518`) and no-op the revert. The chosen ordering avoids this. `recompute_object_controller` (`:1840`) is byte-unchanged and remains a private in-module `fn` (no visibility widening).

**2. Non-vacuity of the canary + real negative test** — CONFIRMED.
- `test_gain_control_until_eot_expires` (`primitive_pb32.rs:375-380`) now asserts `controller == p2`, i.e. exactly the field the fix mutates. Deleting the engine change (not the assertion) makes `recompute_object_controller` never run, so `controller` stays `p1` → assert fails. Runner's git-stash evidence (`left: PlayerId(1) right: PlayerId(2)`) matches this exactly.
- `test_gain_control_until_eot_stacked_control_persists` (`:389-462`) is a genuine negative test. Effect B is `WhileSourceOnBattlefield` with `source: Some(source_id)`, and `source_id` is a battlefield permanent controlled by p3 and phased in (builder default), so `is_effect_active`'s `WhileSourceOnBattlefield` arm (`layers.rs:503-508`: `zone == Battlefield && is_phased_in()`) returns `true`. `recompute_object_controller` therefore folds B and lands on p3, not owner p2. A naive "always revert to owner on expiry" fix fails the `controller == p3` assertion. Test also confirms the `UntilEndOfTurn` effect is removed and the `WhileSourceOnBattlefield` effect remains. Valid.
- `test_gain_control_until_next_turn_reverts_at_untap` (`:467-520`) proves the 514.2-vs-611.2b timing split: `expire_end_of_turn_effects` leaves an `UntilYourNextTurn(p1)` steal intact (controller stays p1, effect not removed), and only `expire_until_next_turn_effects(state, p1)` reverts to p2. Confirms Change 2's `active_player` gating.

**3. Roster sweep validity** — CONFIRMED (with the count corrected to 2, per Finding 1).
- `pb_os1_gain_control_reversion_roster` walks `mtg_engine::all_cards()` and recurses every `Effect` combinator that can nest `GainControl` (`Sequence`, `Conditional`, `Repeat`, `ForEach`, `Choose`, `MayPayOrElse`, `MayPayThenEffect`) across every `Effect`-bearing `AbilityDefinition` arm incl. modal `ModeSelection.modes`. Not a source grep.
- `sarkhan_vol.rs`: `−2` `LoyaltyAbility → Sequence → GainControl { DeclaredTarget, UntilEndOfTurn }` — in scope; walk reaches it (LoyaltyAbility → Sequence recursion). Confirmed.
- `zealous_conscripts.rs`: ETB `Triggered → Sequence → GainControl { DeclaredTarget, UntilEndOfTurn }` — in scope; walk reaches it. Confirmed.
- `karrthus_tyrant_of_jund.rs`: `GainControl { AllPermanentsMatching, Indefinite }` — genuinely out of scope for these two passes, and correctly so (Finding 1). Note the executor (`effects/mod.rs:5497`) emits a per-object `SingleObject` effect regardless of the `EffectTarget` breadth, so `AllPermanentsMatching` is not itself a coverage problem — `Indefinite` is why karrthus is (correctly) untouched.
- The load-bearing executor assumption holds: **every** `GainControl` resolution produces `EffectFilter::SingleObject(obj_id)` (`effects/mod.rs:5484-5497`), so the fix's `filter_map(SingleObject)` collect matches reality for all present cards, including multi-object grants.

**4. Scope discipline** — CONFIRMED.
- Engine change adds no new `Effect`/`EffectDuration`/`EffectFilter`/enum variant or struct field — it wires an existing helper into two existing functions. This is consistent with the plan/WIP claim of **no `PROTOCOL_VERSION` / `HASH_SCHEMA_VERSION` bump** (nothing serde-visible or hash-schema-visible changed). No card-def edits. No `WhileSourceOnBattlefield` reversion added (explicitly deferred — OOS-EF9-1's remaining half). *Caveat:* I did not re-run `cargo build --workspace` / the sentinel-hash / protocol-fingerprint suites myself (no shell in this review); the runner reports all gates green (WIP step 9), and the change surface supports the no-bump claim.

**5. Anything missed** — reviewed, nothing material.
- **No separate "this turn" duration.** CR 514.2 ends both "until end of turn" and "this turn" effects at cleanup, but `EffectDuration` (`card-types/.../continuous_effect.rs:44-78`) has no distinct "this turn" variant — the engine models both as `UntilEndOfTurn`. So `expire_end_of_turn_effects` already covers every cleanup-ending control duration. No gap.
- **Idempotency / double-revert:** two `UntilEndOfTurn` `SetController` effects on one object would push its id into `reverted` twice → two `recompute_object_controller` calls. Both are idempotent (each reads the post-removal effect set). Harmless; runner chose not to dedup, which the plan explicitly permits.
- **Other control durations:** `WhileYouControlSource` reversion is handled by `expire_while_you_control_source_effects` (PB-EF9); `WhileSourceOnBattlefield` reversion is the still-open half of OOS-EF9-1 (correctly deferred). Consistent.
- **Dead/zone-changed target:** `recompute_object_controller` early-returns if the object id is absent (CR 400.7 new-object case); the stale `UntilEndOfTurn` effect is still dropped by the `keep` filter. ObjectIds are monotonic (`next_object_id`), so no id-recycling aliasing. Safe.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 514.2 (cleanup ends UntilEndOfTurn; control reverts) | Yes | Yes | `test_gain_control_until_eot_expires` |
| 611.2a (no-duration control is permanent) | Yes (Indefinite) | Yes (implicitly, roster) | karrthus correctly untouched |
| 611.2b (UntilYourNextTurn reverts at untap) | Yes | Yes | `test_gain_control_until_next_turn_reverts_at_untap` |
| 611.2c (fixed affected set; keep 2nd active controller) | Yes | Yes | `test_gain_control_until_eot_stacked_control_persists` |
| 613.7/613.7b (timestamp-order fold) | Yes | Yes (via stacked test's timestamp 100 vs 101) | `recompute_object_controller` sorts by timestamp |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| sarkhan_vol | Yes | 0 | Yes (now reverts) | in-scope, `−2` UntilEndOfTurn steal |
| zealous_conscripts | Yes | 0 | Yes (now reverts) | in-scope, ETB UntilEndOfTurn steal |
| karrthus_tyrant_of_jund | Yes | 0 | Yes (permanent control, correctly unchanged) | Indefinite is correct; NOT a bug — see Finding 1 |

## Action for the coordinator before collection

- The engine/tests/defs are correct and the batch is collectable.
- **Do not file the proposed karrthus "Indefinite gain-control never reverts" follow-up seed** — it is not a bug (Finding 1).
- Correct the misleading doc comment in `pb_os1_gain_control_reversion.rs` (optional but recommended; a committed-source inaccuracy that could invite a regression). Not blocking.
- Carry forward the one genuine follow-up: `WhileSourceOnBattlefield` gain-control reversion (the remaining half of OOS-EF9-1), correctly deferred by this PB.
