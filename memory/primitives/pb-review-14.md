# Primitive Batch Review: PB-14 -- Planeswalker Support + Emblems

**Date**: 2026-03-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 306 (Planeswalkers), CR 606 (Loyalty Abilities), CR 704.5i (0-loyalty SBA), CR 114 (Emblems)
**Engine files reviewed**: `rules/engine.rs` (handle_activate_loyalty_ability), `rules/sba.rs` (check_planeswalker_sbas), `rules/combat.rs` (combat damage to planeswalkers), `rules/replacement.rs` (starting loyalty ETB), `rules/turn_actions.rs` (reset_turn_state), `rules/resolution.rs` (LoyaltyAbility SOK resolution), `rules/command.rs` (ActivateLoyaltyAbility), `cards/card_definition.rs` (LoyaltyAbility, LoyaltyCost), `state/game_object.rs` (loyalty_ability_activated_this_turn), `state/hash.rs` (LoyaltyCost, LoyaltyAbility SOK), `state/stack.rs` (LoyaltyAbility SOK), `effects/mod.rs` (non-combat damage to planeswalkers), `testing/replay_harness.rs` (activate_loyalty_ability), `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`
**Card defs reviewed**: 2 (ajani_sleeper_agent.rs, tyvar_jubilant_brawler.rs)

## Verdict: needs-fix

One HIGH-severity bug: combat damage to planeswalkers sets `damage_marked` instead of removing loyalty counters, making combat damage to planeswalkers completely ineffective. One MEDIUM: emblem support (CR 114) is entirely missing despite being in the batch specification. The core loyalty ability framework (activation, cost payment, timing restrictions, resolution, SBA, non-combat damage, hash support, turn reset, match arms) is solid and correct.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `combat.rs:1690-1695` | **Combat damage to planeswalkers uses damage_marked instead of removing loyalty counters.** CR 306.8 says "Damage dealt to a planeswalker results in that many loyalty counters being removed from it." The non-combat path in `effects/mod.rs:296-306` correctly removes Loyalty counters. But combat damage at `combat.rs:1693` sets `obj.damage_marked += final_dmg` which has no effect on planeswalker survival. **Fix:** Replace `obj.damage_marked += final_dmg` with loyalty counter removal logic matching `effects/mod.rs:298-306` (read current Loyalty counter, saturating_sub, write back). |
| 2 | **MEDIUM** | engine-wide | **Emblem creation (CR 114) not implemented.** The batch spec lists "Emblem creation (CR 114)" as a deliverable. No `Effect::CreateEmblem`, no emblem object type, no command zone emblem support exists. Ajani's -6 ability creates an emblem and is currently a TODO. **Fix:** Implement `Effect::CreateEmblem { controller: PlayerTarget, abilities: Vec<...> }` that creates an emblem object in the command zone per CR 114.1-114.5. Alternatively, defer explicitly to a future PB if emblem support is complex. |
| 3 | **LOW** | `game_object.rs:893` | **loyalty_ability_activated_this_turn is a bool field instead of Designations bitflag.** Per conventions.md, new boolean designations should use the Designations bitflags (u16, room for 8 more). This predates the convention and works correctly. **Fix:** Migrate to `Designations::LOYALTY_USED` flag. Low priority. |
| 4 | **LOW** | `replay_harness.rs:586` | **TODO: x_value field not wired for -X abilities.** The `activate_loyalty_ability` harness action hardcodes `x_value: None` with a TODO comment. No current card def uses MinusX, so this is non-blocking. **Fix:** Add `x_value` field to `PlayerAction` schema and wire it through when MinusX planeswalkers are authored. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | LOW | `ajani_sleeper_agent.rs` | **All 3 loyalty abilities have empty Effect::Sequence.** TODOs correctly document what's missing: +1 reveal-conditional, -3 distributed counters+vigilance, -6 emblem creation. Loyalty costs (+1/-3/-6) and starting loyalty (4) match oracle text. Mana cost with Phyrexian hybrid is correct. Compleated TODO is appropriate (separate mechanic). No incorrect game state -- abilities are stubs, not wrong implementations. |
| 2 | LOW | `tyvar_jubilant_brawler.rs` | **Both loyalty abilities have empty Effect::Sequence.** TODOs correctly document: static haste-for-abilities, +1 untap, -2 mill+return. Loyalty costs (+1/-2) and starting loyalty (3) match oracle text. Mana cost {1}{B}{G} is correct. No incorrect game state. |

### Finding Details

#### Finding 1: Combat damage to planeswalkers uses damage_marked instead of removing loyalty counters

**Severity**: HIGH
**File**: `crates/engine/src/rules/combat.rs:1690-1695`
**CR Rule**: CR 306.8 -- "Damage dealt to a planeswalker results in that many loyalty counters being removed from it."
**Issue**: The `CombatDamageTarget::Planeswalker(pw_id)` arm in the combat damage application loop writes `obj.damage_marked += final_dmg`. The `damage_marked` field is only checked by creature SBAs (CR 704.5g lethal damage). The planeswalker SBA (CR 704.5i) checks `CounterType::Loyalty` counters. So combat damage to a planeswalker is silently lost -- a 5/5 creature attacking a 3-loyalty planeswalker would deal 5 "damage" that does nothing, and the planeswalker survives with 3 loyalty.

The non-combat damage path in `effects/mod.rs:296-306` handles this correctly:
```rust
let cur = obj.counters.get(&CounterType::Loyalty).copied().unwrap_or(0);
let new_val = cur.saturating_sub(final_dmg);
obj.counters.insert(CounterType::Loyalty, new_val);
```

**Fix**: Replace `combat.rs:1690-1695` with:
```rust
CombatDamageTarget::Planeswalker(pw_id) => {
    // CR 306.8: Damage to a planeswalker removes that many loyalty counters.
    if let Some(obj) = state.objects.get_mut(pw_id) {
        let cur = obj.counters.get(&CounterType::Loyalty).copied().unwrap_or(0);
        let new_val = cur.saturating_sub(final_dmg);
        obj.counters.insert(CounterType::Loyalty, new_val);
    }
}
```

#### Finding 2: Emblem creation (CR 114) not implemented

**Severity**: MEDIUM
**File**: engine-wide (no implementation exists)
**CR Rule**: CR 114.1 -- "Some effects put emblems into the command zone. An emblem is a marker used to represent an object that has one or more abilities, but usually no other characteristics."
**Issue**: The PB-14 batch specification includes "Emblem creation (CR 114)" and the primitive-card-plan.md DSL gap analysis lists "Emblem creation | 11 | Bundle with PB-14 (planeswalker)". No emblem creation Effect, no emblem object model, and no command zone emblem handling exists. Ajani's -6 ability depends on this.
**Fix**: Implement `Effect::CreateEmblem` that creates an emblem object in the command zone with specified abilities. The emblem should be a GameObject with `zone: ZoneId::CommandZone`, no card types, no mana cost, no color (CR 114.3), owned and controlled by the specified player (CR 114.2). Abilities on emblems function from the command zone (CR 114.4). If this is too large for a fix phase, create a dedicated PB-14.5 or track as a known gap.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 306.5b (starting loyalty ETB) | Yes | No (direct) | replacement.rs:999-1010; tested indirectly via planeswalker.rs setup |
| CR 306.5d (loyalty ability timing) | Yes | Yes | test_loyalty_sorcery_speed_cr606_3 |
| CR 306.6 (attack planeswalkers) | Yes | No | combat.rs validates AttackTarget::Planeswalker |
| CR 306.7 (damage redirection removed) | N/A | N/A | Correctly not implemented (rule removed 2019) |
| CR 306.8 (damage removes loyalty) | **Partial** | Yes (non-combat) | Non-combat: effects/mod.rs correct. Combat: **BUG** (Finding 1) |
| CR 306.9 / 704.5i (0 loyalty SBA) | Yes | Yes | test_planeswalker_zero_loyalty_sba_cr704_5i |
| CR 606.3 (timing: main phase, empty stack, once/turn) | Yes | Yes | 3 tests: sorcery_speed, needs_empty_stack, once_per_turn |
| CR 606.4 (loyalty cost payment) | Yes | Yes | test_loyalty_plus_cost, test_loyalty_minus_cost |
| CR 606.6 (insufficient loyalty for negative cost) | Yes | Yes | test_loyalty_insufficient_counters |
| CR 114 (Emblems) | **No** | No | Finding 2 |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| ajani_sleeper_agent | Yes | 4 (Compleated, +1 reveal, -3 distribute, -6 emblem) | Yes (stub) | All abilities are empty stubs; loyalty/cost/types correct |
| tyvar_jubilant_brawler | Yes | 3 (static haste, +1 untap, -2 mill+return) | Yes (stub) | All abilities are empty stubs; loyalty/cost/types correct |

**Note on card count**: The batch spec says "31+ cards" but only 2 planeswalker card defs exist. The remaining ~29 are un-authored and will be done in Phase 2 Wave W-O after all primitives are complete. The engine framework is in place for authoring them.

## Test Summary

10 tests in `planeswalker.rs` covering:
- SBA: 0 loyalty dies (704.5i)
- Non-combat damage removes loyalty (306.8)
- +N cost adds counters (606.4)
- -N cost removes counters (606.4)
- Once per turn restriction (606.3)
- Main phase requirement (606.3)
- Insufficient loyalty for negative cost (606.6)
- Empty stack requirement (606.3)
- Zero cost doesn't change loyalty
- Turn boundary resets flag
- Resolution executes effect
- Can't activate opponent's planeswalker

**Test gap**: No test for combat damage to planeswalkers (the bug in Finding 1 would have been caught by such a test). No game script for planeswalker interaction.
