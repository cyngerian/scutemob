# Primitive Batch Review: PB-EF11 — low-yield singletons (WheelDraw greatest-discarded + spell-only single-target)

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: scutemob-112
**Branch**: feat/pb-ef11-low-yield-singletons-wheeldraw-greatest-discarded-sp
**CR Rules**: 121.1 (draw), 115.7a/115.7b (change target(s)/a target), 115.10 (targeting), 601.2c (target legality/announce), 702.140 (Mutate — mutating creature spell IS a spell)
**Engine files reviewed**: `crates/card-types/src/cards/card_definition.rs` (WheelDraw, TargetRequirement); `crates/engine/src/effects/mod.rs` (WheelHand executor, ChangeTargets executor); `crates/engine/src/rules/casting.rs` (validation early-return + `valid` match arm + internal precision test); `crates/engine/src/rules/abilities.rs` (battlefield auto-target match arm); `crates/engine/src/state/hash.rs` (both HashInto arms + version/history); `crates/engine/src/rules/protocol.rs` (version/history)
**Card defs reviewed**: `windfall.rs`, `misdirection.rs` (2 defs)
**Tests reviewed**: `tests/primitives/pb_ef11_wheel_greatest_discarded.rs`, `tests/primitives/pb_ef11_spell_single_target.rs`, `casting.rs` internal `test_target_spell_with_single_target_self_and_kind_check`

## Verdict: needs-fix (LOW only)

The batch is correct on every load-bearing axis. Both engine primitives match CR text and the two card defs match oracle text exactly, are `Complete`, and register real behavior (invariant #9 satisfied). F1's two-pass executor computes the shared cross-player max and preserves `ThatMany`/`Fixed` byte-identically via an outer `match draw` wrapper with a forced `unreachable!()` on the inner arm. F2's validation is genuinely spell-only — the `is_spell` gate (`Spell | MutatingCreatureSpell`) captures every spell-representing `StackObjectKind` (including copies, which are `Spell { is_copy: true }`, and adventure/split casts) while excluding activated/loyalty/triggered abilities. Hash arms are append-only (WheelDraw disc 2, TargetRequirement disc 19, no collision); PROTOCOL 15→16→17 and HASH 53→54→55 are per-commit, history rows appended, digests machine-forced from the schema tests. Only two **LOW** findings, both cosmetic/out-of-scope; neither blocks collection and neither requires a fix phase.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/primitives/pb_ef11_spell_single_target.rs:247-253` | **Near-vacuous post-cast assertion in the accepts test.** The `\|\| matches!(s.kind, StackObjectKind::Spell { .. })` disjunct is always true (the "other" stack object is itself a `Spell`), so the `any(...)` is trivially satisfied regardless of whether the test spell landed. Acceptance is still genuinely verified by the `unwrap_or_else(panic)` on the cast `Result`, so this is not a coverage hole — cosmetic only. **Fix (optional):** drop the disjunct and assert `s.id == test_spell_id` alone. |
| 2 | LOW (out-of-scope) | `crates/engine/src/effects/mod.rs:6310-6344` | **Pre-existing ChangeTargets object-retarget limitation, inherited by Misdirection.** The object branch picks the smallest-ObjectId object in the original target's zone without re-checking the redirected spell's own `TargetRequirement` (CR 115.7a "another *legal* target"). Shared with Bolt Bend, documented at `effects/mod.rs:6315`, and explicitly scoped out by the plan (§Risks). For Misdirection's object-targeting spells this can apply a type-illegal target in multiplayer, but it is not a PB-EF11 regression and is deferred to M10 interactive targeting. **Acceptable/out-of-scope — no action.** |

## Card Definition Findings

None. Both defs match oracle text exactly, use the new primitives correctly, are `Complete`, carry no TODOs, and register real behavior.

### Finding Details

#### Finding 1: Near-vacuous post-cast assertion (LOW)
**File**: `crates/engine/tests/primitives/pb_ef11_spell_single_target.rs:247-253`
**Issue**: `assert!(state.stack_objects().iter().any(|s| s.id == test_spell_id || matches!(s.kind, StackObjectKind::Spell { .. })))` — the second disjunct is unconditionally true because `build_base_state(false, 1)` seeds a `StackObjectKind::Spell` "Other Stack Object". The assertion can never fail on this state. The real acceptance signal is the `unwrap_or_else(panic)` on line 240-246, which does fire if `TargetSpellWithSingleTarget` validation rejects a legal single-target spell, so the test is not vacuous overall.
**Fix**: Replace the disjunction with `s.id == test_spell_id` so the assertion pins that the cast object actually reached the stack. Optional; the cast-succeeds path is already covered.

#### Finding 2: ChangeTargets smallest-ObjectId retarget does not re-validate the redirected spell's TargetRequirement (LOW, out-of-scope)
**File**: `crates/engine/src/effects/mod.rs:6310-6344`
**CR Rule**: 115.7a — "each target can be changed only to another *legal* target."
**Issue**: The deterministic retarget picks `candidates.sort()`'s first ObjectId in the original zone, filtering only on "different object, same zone" — not on whether the new object satisfies the redirected spell's targeting criteria. Documented as a KNOWN LIMITATION in-code; Bolt Bend already ships with it.
**Fix**: None for this batch. Confirmed acceptable and out-of-scope per the plan; tracked for M10. The integration test correctly uses a player-target scenario where the smallest-ObjectId fallback is legal.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 121.1 (draw = greatest discarded, shared max) | Yes | Yes | `test_greatest_discarded_all_draw_max`, non-vacuous decoy, empty-hands edge, windfall integration |
| ThatMany/Fixed byte-identity | Yes (outer `match` + `unreachable!()`) | Yes | Regression-guarded by untouched `pb_ac9_wheel_and_misc.rs` |
| 115.7a/115.7b (single-target, change a target) | Yes (must_change:true) | Yes | Misdirection retarget integration observes target change + `TargetsChanged` event |
| 115.10 / ruling (spell can't target itself) | Yes (self_id guard) | Yes | Internal precision test asserts "self-targeting" error message |
| 601.2c (exactly one target) | Yes (`target_count != 1`) | Yes | Decoy `test_spell_single_target_rejects_two_target_spell` |
| Spell-only (reject abilities) | Yes (`is_spell` gate) | Yes | Decoy `test_spell_single_target_rejects_activated_ability` + internal "is not a spell" msg |
| 702.140 (MutatingCreatureSpell is a spell) | Yes (included in `is_spell`) | Partial | Not directly cast in a test, but the arm is present and correct; no card exercises it here |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| windfall.rs | Yes | 0 | Yes | {2}{U} Sorcery; `WheelHand{EachPlayer, Discard, GreatestDiscarded}`; `targets: vec![]` (Windfall targets nothing); Complete |
| misdirection.rs | Yes | 0 | Yes | {3}{U}{U} Instant; Pitch alt-cost `ExileFromHand{Blue}` with **no** `PayLife` (correct vs Force of Will, which does pay 1 life); `ChangeTargets{must_change:true}` + `TargetSpellWithSingleTarget`; Complete |

## Adversarial checks performed (all pass)

- **Exhaustive-match completeness**: the two new `=> false` arms (casting.rs:6427, abilities.rs:7363) are correct, not lazy — both sites reach the real spell-only validation via the early-return block at casting.rs:6196; battlefield auto-targeting (abilities.rs) genuinely does not apply to stack-object spell targets. Inner WheelHand `match draw` uses a forced `unreachable!()`, not a silent `=> false`. `cargo build --workspace` (per WIP) confirms no missed site.
- **is_spell semantic gate (oracle-vs-filter)**: walked `StackObjectKind` fully — only `Spell` and `MutatingCreatureSpell` represent a spell being cast; copies are `Spell{is_copy:true}` (accepted); all other variants are abilities/triggers (correctly rejected). Matches Misdirection oracle "target **spell**" exactly and is strictly narrower than the sibling `TargetSpellOrAbilityWithSingleTarget` — the whole point of the new variant.
- **Hash discriminants**: WheelDraw `GreatestDiscarded => 2` (after Fixed=1); TargetRequirement `TargetSpellWithSingleTarget => 19` (after TargetOpponent=18). Both append-only, no collision, existing arms unchanged.
- **Version bumps machine-forced**: HASH 53→54 (Commit 1) / 54→55 (Commit 2); PROTOCOL 15→16 / 16→17. Per-commit (bisectable), history doc lines + HASH/PROTOCOL history rows appended. No straggler sentinels (only match on old versions is the unrelated `sr27_adversarial_demo.sh` canary). Digests recomputed from failing schema tests per WIP, not hand-authored.
- **Two-pass ordering / APNAP**: all disposals precede all draws (required for the shared max); order within each pass follows `resolve_player_target_list` (APNAP for EachPlayer). Single-controller sorcery resolution, so no priority window between disposal and draw — correct.
- **Empty-hand safety**: `counts.iter().copied().max().unwrap_or(0)` — no panic; empty hand contributes 0 and does not raise the max. Pinned by `test_greatest_discarded_empty_hands` (also asserts no `CardDrawn` events).
- **Decoy separation (F2)**: the count-check decoy uses a real `Spell` kind (isolates count) and the kind-check decoy uses exactly 1 target (isolates kind), so neither masks the other. WIP records both verified non-vacuous by locally reverting each guard.
- **Invariant #9**: both defs `Complete`; windfall registers a Spell effect, misdirection registers AltCastAbility + Spell — neither is inert.

## Previous Findings (re-review only)

N/A — first review of PB-EF11.
