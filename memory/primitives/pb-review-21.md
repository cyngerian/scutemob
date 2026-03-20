# Primitive Batch Review: PB-21 -- Fight & Bite

**Date**: 2026-03-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 701.14, 701.14a, 701.14b, 701.14c, 701.14d
**Engine files reviewed**: `crates/engine/src/cards/card_definition.rs` (Effect::Fight, Effect::Bite), `crates/engine/src/effects/mod.rs` (dispatch + helpers), `crates/engine/src/state/hash.rs` (discriminants 58-59)
**Card defs reviewed**: brash_taunter.rs, bridgeworks_battle.rs, ram_through.rs, frontier_siege.rs (4 total)

## Verdict: needs-fix

One MEDIUM finding in the engine (is_creature_on_battlefield bypasses layer system). Two MEDIUM findings in card defs (Bridgeworks Battle optional targeting and missing MDFC back face). Three LOW findings (test gaps and Brash Taunter "another" constraint). The core Fight/Bite engine logic is correct per CR 701.14 -- damage source is the creature (not the spell), self-fight handles 2x power, simultaneous power read before damage, all-or-nothing validation per 701.14b, deathtouch/lifelink/wither/infect correctly keyed off creature source.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `effects/mod.rs:3842` | **is_creature_on_battlefield bypasses layer system.** Uses `obj.characteristics.card_types` (base) instead of `calculate_characteristics()`. Inconsistent with `get_creature_power` which uses layers. An animated land would not be recognized as a creature during animation. **Fix:** Use `calculate_characteristics(state, id)` to check creature type. |
| 2 | LOW | `tests/fight_bite.rs` | **Missing test_fight_target_not_creature.** Plan specified a test for a target that stops being a creature before fight resolves (e.g., animated land animation ends). Not implemented. **Fix:** Add test verifying CR 701.14b when a target loses creature type. |
| 3 | LOW | `tests/fight_bite.rs` | **Missing test_bite_negative_power.** Plan specified a test for negative-power bite. The code handles this correctly (clamped to 0), but no test verifies it. **Fix:** Add test with a creature whose power is reduced below 0, verify no damage dealt. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | MEDIUM | `bridgeworks_battle.rs` | **"Up to one" optional targeting not supported.** Oracle: "It fights up to one target creature you don't control." Ruling confirms: "You can cast Bridgeworks Battle targeting only the creature you control." Card def requires mandatory second target, preventing casting when no opponent creature exists. TODO is documented. **Fix:** No immediate fix possible (DSL gap). Keep TODO. |
| 5 | MEDIUM | `bridgeworks_battle.rs` | **Missing MDFC back face.** Bridgeworks Battle is an MDFC with Tanglespan Bridgeworks (Land) back face. No `back_face` field set. Card cannot be played as a land. **Fix:** Add `back_face: Some(CardFace { ... })` with land type for Tanglespan Bridgeworks. Pre-existing gap, not Fight/Bite specific. |
| 6 | LOW | `brash_taunter.rs` | **"Another target creature" not enforced.** Oracle says "fights another target creature" -- source cannot target itself. `TargetRequirement::TargetCreature` does not exclude self. Pre-existing DSL gap (no "another" target filter). **Fix:** Document as known limitation. No DSL support for "another target" restriction. |

### Finding Details

#### Finding 1: is_creature_on_battlefield bypasses layer system

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:3835-3845`
**CR Rule**: 701.14b -- "If one or both creatures instructed to fight are no longer on the battlefield or are no longer creatures, neither of them fights or deals damage."
**Issue**: The helper `is_creature_on_battlefield` reads `obj.characteristics.card_types` which is the base (printed) characteristics, not the layer-calculated characteristics. Meanwhile, `get_creature_power` at line 3849 correctly uses `calculate_characteristics()` for the power value. This inconsistency means:
- An animated land (base type: Land, gains Creature via continuous effect) would NOT be recognized as a creature by `is_creature_on_battlefield`, so fights involving animated lands would incorrectly do nothing.
- A permanent that lost Creature type via a continuous effect (hypothetical) would still be treated as a creature if its base type includes Creature.

**Fix**: Replace the body of `is_creature_on_battlefield` with:
```rust
fn is_creature_on_battlefield(state: &GameState, id: ObjectId) -> bool {
    let on_bf = state.objects.get(&id)
        .map(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
        .unwrap_or(false);
    if !on_bf { return false; }
    crate::rules::layers::calculate_characteristics(state, id)
        .map(|c| c.card_types.contains(&CardType::Creature))
        .unwrap_or(false)
}
```

#### Finding 4: Bridgeworks Battle optional targeting

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/bridgeworks_battle.rs:17-18`
**Oracle**: "It fights up to one target creature you don't control."
**Ruling (2024-06-07)**: "You can cast Bridgeworks Battle targeting only the creature you control."
**Issue**: The card def uses a mandatory `TargetRequirement::TargetCreatureWithFilter` for the second target. This means the spell cannot be cast unless a valid opponent creature exists, which contradicts the "up to one" oracle text. The TODO at line 17-18 correctly identifies this gap.
**Fix**: No fix possible within current DSL (no optional target mechanism). Keep the TODO as-is. This is a DSL limitation, not a Fight/Bite primitive issue.

#### Finding 5: Bridgeworks Battle missing MDFC back face

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/bridgeworks_battle.rs`
**Oracle**: Bridgeworks Battle // Tanglespan Bridgeworks is a Modal Double-Faced Card. The back face is "Tanglespan Bridgeworks" -- a Land that enters tapped and taps for {G}.
**Issue**: No `back_face` field is set. The card uses `..Default::default()` which leaves `back_face: None`. This means the card cannot be played as a land from hand.
**Fix**: Add `back_face: Some(CardFace { name: "Tanglespan Bridgeworks".to_string(), types: types(&[CardType::Land]), ... })`. This is a pre-existing gap unrelated to Fight/Bite.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 701.14a | Yes | Yes | test_fight_basic, test_fight_one_dies, test_fight_both_die |
| 701.14b (fight) | Yes | Yes | test_fight_creature_left_battlefield |
| 701.14b (bite) | Yes | Yes | test_bite_source_creature_killed_before_resolution |
| 701.14c | Yes | Yes | test_fight_self (3/4 takes 6 damage, dies) |
| 701.14d | Yes | Yes | test_fight_non_combat_damage (life unchanged) |
| Deathtouch | Yes | Yes | test_fight_deathtouch |
| Lifelink | Yes | Yes | test_fight_lifelink, test_bite_lifelink |
| Wither/Infect | Yes (code) | No | deal_creature_power_damage handles it; no test |
| Bite one-sided | Yes | Yes | test_bite_basic (source takes 0 damage) |
| Zero power | Yes | Yes | test_bite_zero_power |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| brash_taunter | Partial | 1 (damage reflection trigger) | Partial | Fight ability correct; damage trigger is separate DSL gap; "another" not enforced (LOW) |
| bridgeworks_battle | Partial | 1 (up to one targeting) | Partial | Fight + pump correct; optional target not supported; MDFC back face missing |
| ram_through | Partial | 1 (trample excess damage) | Partial | Bite correct; Ram Through-specific trample clause deferred (not Fight/Bite gap) |
| frontier_siege | No impl | 3 (modal ETB, mana trigger, conditional fight ETB) | No | Correctly notes Fight is now available; all blocking gaps are non-Fight/Bite |

## Previous Findings (first review)

N/A -- first review of PB-21.
