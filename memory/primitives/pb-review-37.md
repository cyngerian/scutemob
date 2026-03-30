# Primitive Batch Review: PB-37 -- Complex Activated Abilities (Residual G-26)

**Date**: 2026-03-29
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 603.4 (intervening-if), CR 611.2a/b (continuous effect duration), CR 602.5b (activation restrictions)
**Engine files reviewed**: `card_definition.rs`, `game_object.rs`, `continuous_effect.rs`, `resolution.rs`, `effects/mod.rs`, `layers.rs`, `replacement.rs`, `turn_actions.rs`, `abilities.rs`, `hash.rs`, `player.rs`, `builder.rs`, `copy.rs`, `replay_harness.rs`
**Card defs reviewed**: 7 (the_one_ring, geological_appraiser, teferis_protection, elspeth_storm_slayer, kaito_dancing_shadow, ramos_dragon_engine, steel_hellkite)

## Verdict: needs-fix

One HIGH finding: `UntilYourNextTurn(PlayerId(0))` placeholder in card definitions is never resolved to the actual controller when `ApplyContinuousEffect` creates a `ContinuousEffect`. This means continuous effects for Elspeth and Kaito never expire unless the controller happens to be Player 0. Two MEDIUM findings (concede-orphaned effects, stale doc comment on `GrantPlayerProtection`). Four LOW findings (wrong CR citation, inaccurate doc comment, missing `was_cast` in mutate fallback, missing test for actual once-per-turn enforcement).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `effects/mod.rs:2064` | **UntilYourNextTurn PlayerId(0) never resolved.** `ApplyContinuousEffect` copies `effect_def.duration` verbatim into the new `ContinuousEffect`. Card defs use `PlayerId(0)` as placeholder. The expiry function matches `UntilYourNextTurn(active_player)` -- mismatch unless controller is Player 0. **Fix:** In the `ApplyContinuousEffect` handler (around line 2058-2069), add duration resolution: if `effect_def.duration` is `UntilYourNextTurn(_)`, replace the inner PlayerId with `ctx.controller`. |
| 2 | MEDIUM | `layers.rs:1227` | **Concede-orphaned UntilYourNextTurn effects never expire.** If a player concedes, their turn never arrives, so `expire_until_next_turn_effects` is never called for them. Continuous effects with `UntilYourNextTurn(conceded_player)` persist forever. **Fix:** Add a cleanup in the player elimination/concession path that removes all continuous effects with `UntilYourNextTurn(eliminated_player)` and clears their `temporary_protection_qualities`. |
| 3 | LOW | multiple files | **Wrong CR citation: 602.5g does not exist.** Code cites "CR 602.5g" throughout but the actual rule is CR 602.5b (which uses "Activate only once each turn" as an example of a restriction). **Fix:** Replace all "602.5g" references with "602.5b" in `card_definition.rs:205`, `game_object.rs:1027`, `abilities.rs:274,892`, `layers.rs:1226,1243`, `turn_actions.rs:1016`, `ramos_dragon_engine.rs:7`, `steel_hellkite.rs:7,47`. |
| 4 | LOW | `game_object.rs:1029` | **Inaccurate doc comment.** Comment says "Incremented when an activated ability with `once_per_turn == true` resolves." The counter is actually incremented at *activation* time (when the ability goes on the stack) in `abilities.rs:895`, not at resolution. **Fix:** Change "resolves" to "is activated (placed on the stack)". |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 5 | **HIGH** | `elspeth_storm_slayer.rs:80-81` | **Flying grant never expires.** Uses `UntilYourNextTurn(PlayerId(0))` in `ApplyContinuousEffect`. Since the PlayerId is never resolved (Finding 1), the flying grant expires only when Player 0's turn starts. For any other controller, flying persists indefinitely. **Fix:** Blocked on Finding 1 fix. |
| 6 | **HIGH** | `kaito_dancing_shadow.rs:37-38` | **CantBlock restriction never expires.** Same issue as Finding 5 -- `UntilYourNextTurn(PlayerId(0))` in `ApplyContinuousEffect` is never resolved to actual controller. **Fix:** Blocked on Finding 1 fix. |
| 7 | MEDIUM | `card_definition.rs:1786-1787` | **Stale doc comment on GrantPlayerProtection.** Comment says "Duration cleanup ('until your next turn') is deferred" and "Protection is granted permanently until expiration infrastructure is added (TODO)." This is no longer true -- PB-37 implemented the duration infrastructure. **Fix:** Update comment to describe the current behavior (duration field routes to `temporary_protection_qualities` when `Some`, cleared at untap step). |
| 8 | LOW | `the_one_ring.rs:28` | **Misleading comment.** Comment says "PlayerId(0) is a placeholder; the engine binds the actual controller when the trigger fires." For GrantPlayerProtection this works by accident (expiry is by player, not by PlayerId in duration), but the engine does NOT actually resolve the PlayerId. **Fix:** Clarify that the PlayerId in the duration is not used for GrantPlayerProtection expiry (which uses `temporary_protection_qualities` keyed on the target player). |

### Finding Details

#### Finding 1: UntilYourNextTurn PlayerId(0) never resolved in ApplyContinuousEffect

**Severity**: HIGH
**File**: `crates/engine/src/effects/mod.rs:2064`
**CR Rule**: CR 611.2a -- "A continuous effect generated by the resolution of a spell or ability lasts as long as stated by the spell or ability creating it"
**Issue**: When `ApplyContinuousEffect` creates a `ContinuousEffect` at line 2058-2069, it copies `effect_def.duration` verbatim. Card definitions (Elspeth, Kaito) use `EffectDuration::UntilYourNextTurn(PlayerId(0))` as a placeholder. The expiry function `expire_until_next_turn_effects` at `layers.rs:1232` filters for `UntilYourNextTurn(active_player)`. If the actual controller is `PlayerId(1)`, `PlayerId(2)`, or `PlayerId(3)`, the filter never matches and the effect persists forever.

This does NOT affect `GrantPlayerProtection` (The One Ring, Teferi's Protection) because that path stores protections in `temporary_protection_qualities` and expiry is based on which player owns the protections, not the PlayerId in the duration enum.
**Fix**: In `effects/mod.rs` around line 2058-2069, after building the `ContinuousEffect`, resolve the duration:
```rust
let resolved_duration = match effect_def.duration {
    EffectDuration::UntilYourNextTurn(_) => {
        EffectDuration::UntilYourNextTurn(ctx.controller)
    }
    other => other,
};
// ... then use resolved_duration instead of effect_def.duration
```

#### Finding 2: Concede-orphaned UntilYourNextTurn effects

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/layers.rs:1227`
**CR Rule**: CR 104.3a -- player leaving the game
**Issue**: The plan's Risks section explicitly flagged this: "If the player whose 'next turn' we're waiting for concedes, the effect should expire immediately." This was not addressed. If Player 1 casts Teferi's Protection and then Player 1 concedes, the continuous effects with `UntilYourNextTurn(PlayerId(1))` and their `temporary_protection_qualities` will never be cleaned up. For protections this is moot (the player is gone), but for continuous effects affecting the board (e.g., Elspeth's flying grant on all creatures), those effects would persist forever.
**Fix**: In the player elimination/concession handling, add a sweep that removes continuous effects with `UntilYourNextTurn(conceded_player)`. This can be deferred if no current card creates an `UntilYourNextTurn` effect that outlives the player (e.g., Elspeth's flying only matters for the controller's creatures).

#### Finding 3: Wrong CR citation throughout

**Severity**: LOW
**File**: Multiple (10+ sites)
**CR Rule**: Actual rule is CR 602.5b, not CR 602.5g (which does not exist)
**Issue**: All code comments and doc strings cite "CR 602.5g" for the once-per-turn restriction. CR 602.5 only has subrules a through e. The correct citation is CR 602.5b, which says: "If an activated ability has a restriction on its use (for example, 'Activate only once each turn'), the restriction continues to apply to that object even if its controller changes."
**Fix**: Global search-replace "602.5g" -> "602.5b" in all affected files.

#### Finding 4: Doc comment says "resolves" but counter increments at activation

**Severity**: LOW
**File**: `crates/engine/src/state/game_object.rs:1029`
**Issue**: Comment says "Incremented when an activated ability with `once_per_turn == true` resolves." The actual code in `abilities.rs:895` increments the counter at activation time (when the ability is placed on the stack), which is correct per CR 602.5b (the restriction prevents beginning to activate, so tracking must happen at activation, not resolution). The comment is wrong.
**Fix**: Change "resolves" to "is activated (placed on the stack)" at `game_object.rs:1029`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 603.4 (intervening-if) | Yes | Yes | test_was_cast_condition_true_when_cast, test_was_cast_condition_false_when_not_cast |
| CR 611.2a (continuous effect duration) | Partial | Yes | UntilYourNextTurn expiry tested; PlayerId resolution bug (Finding 1) |
| CR 602.5b (activation restrictions) | Yes | Partial | Counter tracking and reset tested; no test exercises actual once-per-turn *rejection* via `handle_activate_ability` |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| the_one_ring | Yes | 0 | Yes (for protection) | PlayerId(0) works by accident for GrantPlayerProtection path |
| geological_appraiser | Yes | 0 | Yes | Clean implementation |
| teferis_protection | Partial | 3 | Partial | Life-total-can't-change, phase-out, self-exile all still TODO (pre-existing) |
| elspeth_storm_slayer | Yes | 0 | **No** | Flying grant never expires due to Finding 1 |
| kaito_dancing_shadow | Partial | 2 | **No** | CantBlock never expires (Finding 1); CantAttack TODO; combat trigger TODO; LTB token trigger TODO |
| ramos_dragon_engine | Partial | 1 | Partial | once_per_turn correct; spell-cast trigger for counters still TODO (pre-existing) |
| steel_hellkite | Partial | 2 | Partial | once_per_turn correct; main {X} effect still Effect::Nothing (pre-existing) |

## Test Gaps

| Gap | Severity | Description |
|-----|----------|-------------|
| No once-per-turn enforcement test | LOW | Tests verify counter defaults to 0 and resets, but no test attempts a second activation and verifies it's rejected. Should add a test that builds an object with `abilities_activated_this_turn = 1` and a `once_per_turn: true` ability, then calls `handle_activate_ability` and asserts `Err`. |
| No integration test with actual casting | LOW | All WasCast tests set `was_cast` manually. No test casts a spell through `process_command` and verifies `was_cast = true` on the resulting permanent. |
| No UntilYourNextTurn with resolved PlayerId | MEDIUM | All duration tests use the correct PlayerId. No test catches the PlayerId(0) placeholder bug because they construct effects directly rather than going through `ApplyContinuousEffect`. |

## Pre-existing Issues Noted (not PB-37 scope)

- `resolution.rs:6526-6530`: Mutate fallback (target illegal) path does not set `was_cast = true`. The spell was cast but the permanent enters without the flag. LOW pre-existing.
- `simulator/legal_actions.rs`: Does not check `once_per_turn` -- bots may attempt illegal activations. LOW pre-existing.
