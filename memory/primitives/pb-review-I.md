# Primitive Batch Review: PB-I --- Grant Flash

**Date**: 2026-04-05
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 702.8 (Flash), 601.3b (cast-as-though-flash), 601.5a (flash persists), 304.5 (instant timing), 307.1 (sorcery timing), 101.2 (restrictions override permissions)
**Engine files reviewed**: `state/stubs.rs`, `state/mod.rs`, `state/builder.rs`, `state/hash.rs`, `cards/card_definition.rs`, `effects/mod.rs`, `rules/replacement.rs`, `rules/casting.rs`, `rules/layers.rs`, `rules/turn_actions.rs`, `cards/helpers.rs`, `lib.rs`; `crates/simulator/src/legal_actions.rs`
**Card defs reviewed**: 4 (borne_upon_a_wind.rs, complete_the_circuit.rs, teferi_time_raveler.rs, yeva_natures_herald.rs)

## Verdict: needs-fix

The core engine implementation is correct and well-structured. CR 101.2 restriction-over-permission ordering is properly enforced by having `check_cast_restrictions` run before flash grant checks. Flash grant registration, filtering, duration expiry, and stale source cleanup are all implemented correctly in the engine. Two MEDIUM findings in the simulator's `legal_actions.rs` (missing source-validity check and missing `OpponentsCanOnlyCastAtSorcerySpeed` handling) and one MEDIUM test gap (Yeva ETB pipeline not actually tested). Card definitions match oracle text.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `simulator/legal_actions.rs:219` | **Simulator flash grant check missing source-validity.** Engine's `has_active_flash_grant` checks `WhileSourceOnBattlefield` duration and verifies source is on battlefield; simulator inline check does not. Stale grants (source left battlefield, not yet swept by `reset_turn_state`) would appear as legal actions. **Fix:** Add duration/source check mirroring engine's logic (lines 5403-5417 of casting.rs). |
| 2 | **MEDIUM** | `simulator/legal_actions.rs:932-970` | **Simulator missing OpponentsCanOnlyCastAtSorcerySpeed restriction.** `is_cast_restricted_by_stax` handles `MaxSpellsPerTurn`, `OpponentsCantCastDuringYourTurn`, etc. but has no arm for the new `OpponentsCanOnlyCastAtSorcerySpeed` variant. Opponents of Teferi controller will see illegal cast actions as legal in the simulator. **Fix:** Add match arm in `is_cast_restricted_by_stax` that returns `true` when `player != controller && (!is_own_main || !stack_empty)`. |
| 3 | **MEDIUM** | `tests/grant_flash.rs:807-856` | **Test 9 does not test ETB registration pipeline.** Named `test_yeva_static_flash_grant_registered_on_etb` but only verifies the card definition has `StaticFlashGrant`. Does not test that casting/placing Yeva actually populates `state.flash_grants` via `register_static_continuous_effects`. **Fix:** Rewrite test to cast Yeva (or use a mechanism that triggers `register_static_continuous_effects`) and assert `state.flash_grants.len() == 1` with correct filter/source/player. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | LOW | `complete_the_circuit.rs` | **Expected TODO for copy-spell-twice.** "When you next cast an instant or sorcery spell this turn, copy that spell twice" is documented as a separate primitive gap. Not introduced by PB-I. No fix needed now. |
| 5 | LOW | `teferi_time_raveler.rs` | **Pre-existing: -3 target too broad and not optional.** Oracle says "up to one target artifact, creature, or enchantment" but def uses `TargetPermanent` (any permanent, mandatory). Pre-existing issue not introduced by PB-I. No fix needed now. |

### Finding Details

#### Finding 1: Simulator flash grant check missing source-validity

**Severity**: MEDIUM
**File**: `crates/simulator/src/legal_actions.rs:219-236`
**CR Rule**: 601.3b -- "If an effect allows a player to cast a spell with certain qualities as though it had flash..."
**Issue**: The simulator's inline flash grant check iterates `state.flash_grants` and checks only player match and filter match. It does not check whether the grant's source is still on the battlefield for `WhileSourceOnBattlefield` grants. The engine's `has_active_flash_grant()` at `casting.rs:5403-5417` properly checks `grant.duration == WhileSourceOnBattlefield` and verifies the source object is in `ZoneId::Battlefield`. Without this check, the simulator may show casting a green creature at instant speed as legal even after Yeva has been destroyed (during the window before `reset_turn_state` sweeps the stale grant).
**Fix**: Add a source-validity check inside the `state.flash_grants.iter().any(|g| { ... })` closure, matching the engine's logic:
```rust
if matches!(g.duration, EffectDuration::WhileSourceOnBattlefield) {
    if let Some(src) = g.source {
        let on_bf = state.objects.get(&src)
            .map(|o| matches!(o.zone, ZoneId::Battlefield))
            .unwrap_or(false);
        if !on_bf { return false; }
    }
}
```
Import `EffectDuration` from `mtg_engine`.

#### Finding 2: Simulator missing OpponentsCanOnlyCastAtSorcerySpeed restriction

**Severity**: MEDIUM
**File**: `crates/simulator/src/legal_actions.rs:932-970`
**CR Rule**: 101.2 -- "When a rule or effect allows or directs something to happen, and another effect states that it can't happen, the 'can't' effect takes precedence."
**Issue**: The function `is_cast_restricted_by_stax` handles several `GameRestriction` variants (`MaxSpellsPerTurn`, `OpponentsCantCastDuringYourTurn`, `OpponentsCantCastOrActivateDuringYourTurn`) but the `_ => {}` catch-all silently ignores the new `OpponentsCanOnlyCastAtSorcerySpeed`. When Teferi is on the battlefield, the simulator will still show instant-speed casting as legal for opponents.
**Fix**: Add a match arm before the `_ => {}`:
```rust
GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed => {
    if active_player == controller && player != controller {
        return true; // blocked unless it's their own main+empty stack
    }
    // Also block during opponent's own turn at non-sorcery speed:
    let is_own_main = active_player == player
        && matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain);
    if player != controller && (!is_own_main || !state.stack_objects.is_empty()) {
        return true;
    }
}
```
Note: the `is_cast_restricted_by_stax` function returns a blanket bool (not per-card), so this is a reasonable approximation. The sorcery-speed restriction allows casting during the opponent's own main phase with empty stack, so the check should only restrict when those conditions are NOT met.

#### Finding 3: Test 9 does not test ETB registration pipeline

**Severity**: MEDIUM
**File**: `crates/engine/tests/grant_flash.rs:807-856`
**CR Rule**: 601.3b -- flash grant registration via static ability
**Issue**: The test is named `test_yeva_static_flash_grant_registered_on_etb` and its doc comment says "Tests the full engine pipeline: `register_static_continuous_effects` -> `flash_grants`." However, the test only checks that `yeva_def_from_reg.abilities` contains `StaticFlashGrant`. It does NOT verify that placing Yeva on the battlefield actually results in a `FlashGrant` entry in `state.flash_grants`. The `GameStateBuilder` pre-places objects without running `register_static_continuous_effects`, so the pipeline is never exercised. This is a gap in integration test coverage for the `StaticFlashGrant` -> `register_static_continuous_effects` -> `flash_grants` path.
**Fix**: Either (a) rewrite the test to cast Yeva from hand (using `process_command(state, cast_spell_cmd(...))` + resolve), then assert `state.flash_grants.len() == 1` and verify the grant's `source`, `player`, `filter`, and `duration` fields; or (b) add a separate integration test that manually calls the replacement.rs registration path and checks the result. Option (a) is preferred as it tests the real game flow.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 702.8a (Flash keyword) | Yes (pre-existing) | Yes (pre-existing) | casting.rs line 537 |
| 601.3b (cast as though flash) | Yes | Yes | tests 1-6, 8 cover all filter variants |
| 601.5a (flash persists through casting) | N/A | N/A | Not testable without mid-cast state changes |
| 304.5 (instant timing) | Yes (pre-existing) | Yes | |
| 307.1 (sorcery timing) | Yes | Yes | Negative test (test 2) |
| 101.2 (restriction overrides permission) | Yes | Yes | test 8 (`test_teferi_restriction_overrides_flash_grant_for_opponents`) |
| 514.2 (end-of-turn cleanup) | Yes | Yes | test 6 (`test_grant_flash_until_end_of_turn_expires_at_cleanup`) |
| 611.2b (until-next-turn expiry) | Yes | Partial | Expiry code present in layers.rs; no dedicated test for UntilYourNextTurn flash grant expiry across turns |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Borne Upon a Wind | Yes | 0 | Yes | GrantFlash AllSpells + DrawCards sequence correct |
| Complete the Circuit | Partial | 1 (copy-spell-twice) | Partial | Flash grant correct; copy-twice is separate primitive gap |
| Teferi, Time Raveler | Yes | 0 | Yes | Passive restriction + +1 grant + -3 bounce all present; -3 target type filter is pre-existing LOW |
| Yeva, Nature's Herald | Yes | 0 | Yes | Flash + StaticFlashGrant GreenCreatures; mana cost/types/P-T all match oracle |

## Hash Discriminant Check

| Type | Discriminant | Collisions? |
|------|-------------|-------------|
| `GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed` | 8u8 | No (prev max: 7, `MaxNonartifactSpellsPerTurn`) |
| `Effect::GrantFlash` | 78u8 | No (prev max: 77, `RegisterReplacementEffect`) |
| `AbilityDefinition::StaticFlashGrant` | 72u8 | No (prev max: 71, `AdditionalLandPlays`) |
| `FlashGrantFilter` variants | 0/1/2 | N/A (new impl) |
| `FlashGrant` struct | hashes all 4 fields | OK |
| `GameState.flash_grants` | hashed at line 5803 | OK |
