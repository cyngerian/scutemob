# Primitive Batch Review: PB-EF9 — `EffectDuration::WhileYouControlSource`

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 611.2b, 611.2c, 613.1b, 613.7, 702.26e, 400.7
**Engine files reviewed**: `crates/card-types/src/state/continuous_effect.rs`,
`crates/engine/src/rules/layers.rs` (`is_effect_active` arm +
`expire_while_you_control_source_effects` + `recompute_object_controller`),
`crates/engine/src/effects/mod.rs` (GainControl / ApplyContinuousEffect / ExchangeControl),
`crates/engine/src/rules/sba.rs` (call site), `crates/engine/src/rules/replacement.rs`
(second `is_effect_active`), `crates/engine/src/state/hash.rs`,
`crates/engine/src/rules/protocol.rs`
**Card defs reviewed**: olivia_voldaren.rs, dragonlord_silumgar.rs (Complete flips),
roil_elemental.rs, kellogg_dangerous_mind.rs (stay partial) — 4/4
**Test file reviewed**: `crates/engine/tests/primitives/pb_ef9_while_you_control_source.rs` (9 tests)

## Verdict: needs-fix

The primitive is correct on every safety-critical axis the plan flagged: never-resumes is
enforced by one-shot imperative removal (not a live check), the phased-out source deliberately
skips `is_phased_in()`, `recompute_object_controller` reapplies stacked control in timestamp
order rather than snapping to owner, placeholder resolution binds "you" to `ctx.controller` at
creation, and both Complete card defs match oracle text (target filters verified against
dispatch — Silumgar's `has_card_types` is OR-semantics, correctly "creature or planeswalker").
The DECOY test uses a P/T (non-control) `WhileSourceOnBattlefield` effect and is non-vacuous.
No HIGH findings. **One MEDIUM**: the single pre-loop call site catches control *change* of a
source (steal/destroy-via-resolution) but **not** the source *leaving via a state-based action*
(the most common exit for Olivia/Silumgar: dying to combat damage or 0 toughness), because the
SBA that removes the source runs *inside* the loop, after the pre-loop expiry has already run —
so reversion lags by one `check_and_apply_sbas` invocation and the borrowed permanent is under
the wrong controller during the intervening priority window. The existing tests do not cover
the SBA-death path (all use `test_util::move_object_to_zone`, which routes like a resolution).
Two LOW notes.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `sba.rs:75` | **Reversion lags when the source dies as a state-based action.** Expiry runs once pre-loop; a source that leaves *inside* the SBA loop (lethal combat damage / 0 toughness) is not observed until the next `check_and_apply_sbas`. **Fix:** run the expiry at the top of each loop iteration (or re-run until stable) and add a source-dies-via-SBA test. |
| 2 | LOW | `layers.rs:1754` | **Only `EffectFilter::SingleObject` reverts control.** `expire_*` collects affected ids solely from `SingleObject`; a future `WhileYouControlSource` authored via `ApplyContinuousEffect` with a broader filter would be removed but not revert control. No card hits it today. **Fix:** none required now; note for the next author. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | LOW | `olivia_voldaren.rs` | **"Target Vampire" modelled as `TargetCreatureWithFilter{has_subtype: Vampire}`.** Narrows to creatures; a (vanishingly rare) non-creature Vampire permanent would be excluded. Practically correct — all Vampire permanents are creatures. **Fix:** none required. |

### Finding Details

#### Finding 1 (MEDIUM): Control reversion lags a source that leaves via a state-based action

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/sba.rs:75` (call site);
`crates/engine/src/rules/layers.rs:1733` (`expire_while_you_control_source_effects`)
**CR Rule**: 611.2b — the effect ends the moment the "for as long as you control [source]"
duration ends (source leaves). 704.5g — a creature with lethal damage / 0 toughness is put
into the graveyard **as a state-based action**.

**Issue**: `check_and_apply_sbas` calls the expiry once, *before* the SBA fixpoint loop:

```
super::layers::expire_while_you_control_source_effects(state);   // pre-loop, ONCE
loop { let events = apply_sbas_once(state); if events.is_empty() { break; } ... }
```

The plan's placement rationale — "control never changes as a *result* of an SBA" — is true but
incomplete. The ended-condition is `!(zone == Battlefield && controller == pid)`; an SBA
(creature death) changes `zone`, so an SBA **can** flip the effect to ended. When the source
dies during combat (turn-based damage → SBA 704.5g), the sequence in the enclosing
`check_and_apply_sbas` is: pre-loop expiry (source still alive → no-op) → loop pass 1 moves the
source to the graveyard → loop breaks. The borrowed permanent is **not** reverted in that call.
Priority is then granted (combat damage step) with the borrowed permanent still under its
borrower; it only reverts at the *next* `check_and_apply_sbas` (end of combat). This is
observably wrong game state during one priority window — a player could tap / activate /
sacrifice a creature they should no longer control. It is the canonical way Olivia/Silumgar
lose the source, so it is frequently hit. (It self-heals and is strictly better than the prior
never-reverts behavior, hence MEDIUM not HIGH.)

The steal path and destroy-*spell* path are correct: both change the source's controller/zone
*during resolution*, before `check_and_apply_sbas` runs, so the pre-loop pass catches them. The
tests exercise only those paths — `move_object_to_zone` is called manually *before*
`check_and_apply_sbas`, mirroring a resolution, never an SBA death — so the gap is untested.

**Fix**: Invoke `expire_while_you_control_source_effects(state)` at the **top of each loop
iteration** in `check_and_apply_sbas` (before `apply_sbas_once`), not only pre-loop. This is
loop-safe: on a steady-state pass it is idempotent and produces nothing; on the pass after the
source dies it reverts control, and that same iteration's `apply_sbas_once` then observes any
consequent SBAs (e.g. an Aura now on an illegal object), so termination still keys off
`apply_sbas_once` returning empty. Add a regression test where the source dies via SBA (set the
source to lethal marked damage or drop its toughness to 0 with a continuous effect, then call
`check_and_apply_sbas`) and assert the borrowed permanent reverts within that single call.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 611.2b (source leaves ends effect) | Yes | Partial | `ends_when_source_leaves` covers manual-move; SBA-death path untested (Finding 1) |
| 611.2c (never resumes; fixed affected set) | Yes | Yes | `does_not_resume` — non-vacuous (mutation-tested per wip); `is_effect_active` returns `true`, removal is one-shot |
| 613.1b (control is Layer 2) | Yes | Yes | GainControl builds Layer-2 SetController; integration tests |
| 613.7 (timestamp order for stacked control) | Yes | Yes | `recompute_object_controller` sorts by timestamp; `multiplayer` reverts to owner under no remaining effect |
| 702.26e (phased-out source still controlled) | Yes | Yes | `survives_source_phase_out`; `is_phased_in()` deliberately NOT copied — confirmed |
| 400.7 (source new id on zone change → ended) | Yes | Yes | `unwrap_or(true)`; mutation-tested (`unwrap_or(false)` failed exactly 3 tests) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| olivia_voldaren | Yes | 0 | Yes | Complete; `{1}{R}` + `{3}{B}{B}` both correct; "target Vampire"→creature filter is LOW |
| dragonlord_silumgar | Yes | 0 | Yes | Complete; `has_card_types: [Creature, Planeswalker]` verified OR-semantics (effects/mod.rs:8342) |
| roil_elemental | Yes | 1 (correct) | N/A (partial) | Stays partial; "you may" costless-optional wrapper genuinely inexpressible — verified: MayPayOrElse is a gated stub, MayPayThenEffect auto-pays (no true decline). Not authored as mandatory (would be legal-but-wrong). Correct call. |
| kellogg_dangerous_mind | Yes | 1 (correct) | N/A (partial) | Stays partial; residual blocker = sacrifice-N-of-subtype cost (`Cost::Sacrifice` has no count field). First strike/haste/Treasure-on-attack all present. Note refreshed correctly. |

## Verification of Reviewer's Scrutiny Points

1. **Never-resumes (611.2c)** — CONFIRMED. `is_effect_active` arm returns `true` (layers.rs:525,
   no live control check); termination is solely the `retain`-out in
   `expire_while_you_control_source_effects` (layers.rs:1762-1769). `does_not_resume` test is
   non-vacuous (mutation #2 in wip: skipping the removal step failed 8/9 tests).
2. **Phased-out source (702.26e)** — CONFIRMED. Ended-condition (layers.rs:1748) tests only
   `zone == Battlefield && controller == pid`, deliberately NOT `is_phased_in()`. Mutation #3
   (reinstating the check) failed exactly `survives_source_phase_out`.
3. **DECOY test** — CONFIRMED non-vacuous and correctly scoped. Uses a Layer-7 `ModifyBoth(+1)`
   `WhileSourceOnBattlefield` effect on a third creature; asserts power 3 before AND after the
   source's control change, plus `is_effect_active == true`, plus a sanity assert that the
   WhileYouControlSource borrow DID end in the same scenario. A SetController decoy was
   deliberately avoided (would entangle with OOS-EF9-1). Proves the two durations diverge only
   on control change.
4. **recompute under stacked control** — CONFIRMED. Reapplies remaining active `SetController`
   effects in `timestamp` order (layers.rs:1793-1810), not a snap-to-owner. Just-removed effect
   already retained out before recompute (step 2 precedes step 3). `multiplayer` test proves
   reversion to owner p3, not thief p2 / borrower p1.
5. **Call-site sufficiency** — GAP FOUND (Finding 1). Pre-loop catches resolution-driven control
   change and resolution-driven source-leave, but not SBA-driven source-leave (creature death).
6. **Placeholder resolution** — CONFIRMED. GainControl (effects/mod.rs:5371) and
   ApplyContinuousEffect (effects/mod.rs:3159) both resolve `PlayerId(0)` → `ctx.controller`.
   ExchangeControl left as a `// NOTE:` only (effects/mod.rs:5407) — safe, no card uses it.
7. **Card defs vs oracle** — CONFIRMED via MCP. olivia + silumgar Complete and correct;
   roil's optional-wrapper inexpressibility independently verified against
   `card_definition.rs:1721/1742` (both stubs); kellogg's residual is the sacrifice-count cost.
   No legal-but-wrong (roil NOT shipped as a mandatory steal).
8. **OOS-EF9-1** — Correctly deferred; documented in `primitive-wip.md` and `pb-plan-EF9.md`.
   Not yet appended to `ef-batch-plan-2026-07-17.md` §9 OOS registry — should be added at
   collection. Tests do NOT depend on the broken WhileSourceOnBattlefield-control-never-reverts
   behavior (DECOY uses a P/T effect precisely to avoid it).
9. **Version bumps** — SANE. PROTOCOL 13→14 (fingerprint const at protocol.rs:161 matches the
   appended v14 history row at :290-295; existing rows untouched). HASH 51→52 with both
   decl+stream fingerprints. Both machine-forced (EffectDuration is in the SR-8 closure and
   hashed into `continuous_effects`). Hash arm uses discriminant 5, distinct from all others.
   Replacement.rs second `is_effect_active` also got the `=> true` arm (exhaustiveness).

## Test Adequacy

9 tests, all mutation-tested for non-vacuity (per wip, 4 mutate/restore cycles with byte-identical
`layers.rs` md5 after). Positive, negative (decoy, does_not_resume), phased-out, multiplayer, and
two card-integration paths (activated + ETB, steal + death). **One gap**: no test drives the
source's departure through a state-based action (all use manual `move_object_to_zone`) — this is
exactly the path Finding 1 identifies as lagging. Add one when applying the fix.
