# Primitive Batch Review: PB-9 -- Hybrid Mana & X Costs

**Date**: 2026-03-17
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 107.3, 107.4e, 107.4f, 202.2d, 202.3e, 202.3f, 202.3g
**Engine files reviewed**: `state/game_object.rs` (HybridMana, PhyrexianMana, ManaCost), `rules/casting.rs` (flatten_hybrid_phyrexian, X-cost expansion, can_pay_cost), `state/hash.rs` (HybridMana/PhyrexianMana/ManaCost hashing), `rules/commander.rs` (compute_color_identity, add_colors_from_mana_cost), `rules/engine.rs` (CastSpell dispatch), `rules/command.rs` (hybrid_choices, phyrexian_life_payments fields), `effects/mod.rs` (devotion hybrid counting), `simulator/mana_solver.rs` (PipTracker), `simulator/legal_actions.rs` (can_afford)
**Card defs reviewed**: brokkos_apex_of_forever, connive, nethroi_apex_of_death, cut_ribbons, mockingbird, treasure_vault, kitchen_finks, boggart_ram_gang, fetid_heath, rugged_prairie, flooded_grove, twilight_mire, revitalizing_repast, blade_historian, ajani_sleeper_agent (15 total)

## Verdict: needs-fix

The engine-side implementation of HybridMana, PhyrexianMana, ManaCost.x_count, flatten_hybrid_phyrexian, mana_value(), color identity, and hash support is solid and CR-compliant. However, there is one HIGH finding (Brokkos main mana cost uses hybrid where oracle has separate colored pips), several MEDIUM findings (Connive missing Concoct half, mana_solver ignores hybrid/phyrexian/X, remaining TODOs on multiple cards), and several LOW findings (filter lands defaulting output, missing card abilities).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `simulator/mana_solver.rs:139` | **PipTracker ignores hybrid/phyrexian/x_count.** `from_cost()` copies only the 7 basic fields, discarding hybrid, phyrexian, and x_count. For pure-hybrid cards (Boggart Ram-Gang, Kitchen Finks), solver reports cost as payable with 0 colored mana. **Fix:** Flatten hybrid/phyrexian into colored pips in `from_cost()` (default: first color for hybrid, mana for phyrexian). For x_count, add `x_count * chosen_x` to generic (or accept X=0 as the deterministic default). |
| 2 | LOW | `state/game_object.rs:58` | **Missing {C/W} colorless-hybrid variant.** CR 107.4e lists `{C/W}, {C/U},...,{C/G}` monocolored hybrids payable with colorless or one colored mana. HybridMana only has ColorColor and GenericColor. No cards in current universe use this; add when needed. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **HIGH** | `brokkos_apex_of_forever.rs` | **Main mana cost wrong.** Oracle: `{2}{B}{G}{U}`. Def: `{2}{U/B}{G}{G}` (hybrid in main cost). Only the mutate cost has hybrid `{U/B}`. **Fix:** Change main cost to `ManaCost { generic: 2, blue: 1, black: 1, green: 1, ..Default::default() }`. |
| 4 | MEDIUM | `connive.rs` | **Missing Concoct half entirely.** Oracle: split card Connive `{2}{U/B}{U/B}` // Concoct `{3}{U}{B}`. Def only has Connive half cost and oracle text. Concoct ability (Surveil 3 + return creature from graveyard) missing. **Fix:** Add split card structure with both halves. At minimum, add Concoct oracle text and stub the second half's cost. |
| 5 | MEDIUM | `connive.rs` | **Connive effect not implemented.** Oracle: "Gain control of target creature with power 2 or less." The abilities vec is empty. **Fix:** Add spell ability with GainControl effect and TargetCreature target. |
| 6 | MEDIUM | `revitalizing_repast.rs` | **Front face effect not implemented.** Oracle: "Put a +1/+1 counter on target creature. It gains indestructible until end of turn." The abilities vec is empty. **Fix:** Add spell ability with AddCounters + grant Indestructible until EOT. |
| 7 | MEDIUM | `revitalizing_repast.rs` | **Missing MDFC back face.** Oracle: "Revitalizing Repast // Old-Growth Grove" is a modal DFC. Back face is a land (Old-Growth Grove). No `back_face` defined. **Fix:** Add back_face CardFace for Old-Growth Grove land. |
| 8 | MEDIUM | `blade_historian.rs` | **Static ability not implemented.** Oracle: "Attacking creatures you control have double strike." The abilities vec is empty. TODO comment exists noting DSL gap. **Fix:** Implement when conditional static grant infrastructure exists (EffectFilter for attacking creatures). Document as known DSL gap for now. |
| 9 | MEDIUM | `treasure_vault.rs:23-24` | **X-scaled token creation TODO.** The `{X}{X}` activated ability creates only 1 Treasure instead of X Treasures. x_count on cost is correct (2), but effect uses `treasure_token_spec(1)` instead of X-scaling. **Fix:** When EffectAmount::XValue is supported in token counts, update to create X treasures. |
| 10 | MEDIUM | `cut_ribbons.rs:8-13` | **Stale TODO comment about X cost.** The TODO says "ManaCost has no field for variable X costs" and "X is approximated as generic: 3". But the cost now correctly uses `x_count: 1` and `EffectAmount::XValue`. The TODO is outdated. **Fix:** Remove the stale TODO comment on lines 8-13. |
| 11 | LOW | `fetid_heath.rs:26` | **Filter land output approximated.** Oracle: "{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}." Def defaults to {W}{B}. TODO present. Correct approximation but lossy. |
| 12 | LOW | `rugged_prairie.rs:22` | **Filter land output approximated.** Same pattern as Fetid Heath. Defaults to {R}{W}. |
| 13 | LOW | `flooded_grove.rs:25` | **Filter land output approximated.** Same pattern. Defaults to {G}{U}. |
| 14 | LOW | `twilight_mire.rs:22` | **Filter land output approximated.** Same pattern. Defaults to {B}{G}. |
| 15 | LOW | `ajani_sleeper_agent.rs:31` | **Planeswalker abilities stubbed.** All three loyalty abilities have `Effect::Sequence(vec![])`. TODOs present. Expected -- planeswalker framework (PB-14) not yet complete. |
| 16 | LOW | `brokkos_apex_of_forever.rs:47` | **Cast-from-graveyard ability TODO.** "You may cast this card from your graveyard using its mutate ability" not implemented. Documented DSL gap. |
| 17 | LOW | `nethroi_apex_of_death.rs:43` | **Mutate trigger stubbed.** "Return creature cards with total power 10 or less" uses `Effect::Nothing`. Documented DSL gap (ReturnFromGraveyard with power constraint). |

### Finding Details

#### Finding 3: Brokkos Main Mana Cost Uses Hybrid Incorrectly

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/brokkos_apex_of_forever.rs:18-23`
**Oracle**: Brokkos, Apex of Forever -- Mana Cost: `{2}{B}{G}{U}`
**Issue**: The card def encodes the main mana cost as `ManaCost { generic: 2, green: 2, hybrid: vec![HybridMana::ColorColor(ManaColor::Blue, ManaColor::Black)] }` which represents `{2}{U/B}{G}{G}`. The oracle text shows the main cost is `{2}{B}{G}{U}` -- three separate colored pips, no hybrid. The hybrid `{U/B}` only appears in the mutate cost `{2}{U/B}{G}{G}`, which is correctly encoded in the MutateCost ability. This error affects color-pip counting for devotion, mana payment validation, and mana value calculation (MV 5 correct for both, but payment semantics differ -- hybrid allows either color, separate pips require both).
**Fix**: Change line 18-23 to:
```rust
mana_cost: Some(ManaCost {
    generic: 2,
    blue: 1,
    black: 1,
    green: 1,
    ..Default::default()
}),
```

#### Finding 1: Mana Solver Ignores Hybrid/Phyrexian/X

**Severity**: MEDIUM
**File**: `crates/simulator/src/mana_solver.rs:139-148`
**CR Rule**: 107.4e -- hybrid mana can be paid with either component color
**Issue**: `PipTracker::from_cost()` only copies the 7 standard ManaCost fields (white, blue, black, red, green, colorless, generic). It completely ignores `hybrid`, `phyrexian`, and `x_count`. For a card like Boggart Ram-Gang with cost `{R/G}{R/G}{R/G}` (all pips in hybrid vec, zero in colored fields), the solver sees a zero-cost spell and returns `Some(vec![])`. The `can_afford` fast-path partially compensates by checking `pool.total() >= cost.mana_value()`, but this only ensures enough total mana exists -- it does not enforce that the mana is of a valid color for hybrid payment.
**Fix**: In `PipTracker::from_cost()`, flatten hybrid pips into colored requirements (default: first color for ColorColor, color for GenericColor) and phyrexian pips into colored requirements (default: mana payment). For x_count, add `x_count * 0` (X=0 default) to generic. Alternatively, call `flatten_hybrid_phyrexian` with default choices before constructing PipTracker.

#### Finding 10: Stale TODO in Cut // Ribbons

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/cut_ribbons.rs:8-13`
**Issue**: The TODO comment says "ManaCost has no field for variable X costs" and "X is approximated here as generic: 3". But the actual code on line 48 correctly uses `ManaCost { black: 2, x_count: 1, ..Default::default() }` and line 54 uses `EffectAmount::XValue`. The TODO is stale and misleading -- the card has already been properly fixed to use the PB-9 x_count primitive.
**Fix**: Remove lines 8-13 (the stale TODO block). The implementation is correct.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 107.3 (X costs) | Yes | Yes | test_x_mana_value_off_stack, test_double_x_mana_value |
| 107.3a (X chosen at cast time) | Yes | Partial | x_value on CastSpell, consumed in casting.rs; no integration test casting an X spell |
| 107.4e (hybrid mana) | Yes | Yes | test_hybrid_payment_first_color, test_hybrid_payment_second_color, test_hybrid_generic_color_* |
| 107.4f (Phyrexian mana) | Yes | Yes | test_phyrexian_payment_mana, test_phyrexian_payment_life, test_phyrexian_mixed_payment |
| 202.2d (hybrid/Phyrexian color) | Yes | Yes | colors_from_mana_cost handles hybrid+phyrexian |
| 202.3e (X=0 off stack) | Yes | Yes | test_x_mana_value_off_stack |
| 202.3f (hybrid MV) | Yes | Yes | test_hybrid_color_color_mana_value, test_hybrid_generic_color_mana_value |
| 202.3g (Phyrexian MV) | Yes | Yes | test_phyrexian_mana_value |
| 903.4 (color identity) | Yes | Yes | test_hybrid_color_identity, test_phyrexian_color_identity, test_hybrid_phyrexian_color_identity |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| brokkos_apex_of_forever | **No** | 2 | **No** | Main cost has hybrid {U/B} instead of separate {U}+{B} (HIGH) |
| connive | **No** | 0 | **No** | Missing Concoct half, Connive effect not implemented |
| nethroi_apex_of_death | Partial | 1 | Partial | Mutate trigger stubbed (Effect::Nothing) -- DSL gap |
| cut_ribbons | Yes | 1 (stale) | Yes | x_count + XValue correct; stale TODO comment remains |
| mockingbird | Partial | 1 | Partial | Flying correct, ETB copy effect is DSL gap |
| treasure_vault | Partial | 2 | **No** | Activated ability creates 1 treasure instead of X |
| kitchen_finks | Yes | 0 | Yes | Hybrid cost and abilities correct |
| boggart_ram_gang | Yes | 0 | Yes | Triple hybrid cost and keywords correct |
| fetid_heath | Partial | 1 | Partial | Filter output approximated to one option |
| rugged_prairie | Partial | 1 | Partial | Filter output approximated to one option |
| flooded_grove | Partial | 1 | Partial | Filter output approximated to one option |
| twilight_mire | Partial | 1 | Partial | Filter output approximated to one option |
| revitalizing_repast | **No** | 0 | **No** | Front face effect + back face both missing |
| blade_historian | Partial | 0 | **No** | Static ability not implemented (DSL gap) |
| ajani_sleeper_agent | Partial | 4 | Partial | PW abilities stubbed, Compleated not implemented |

## Notes

The engine infrastructure for hybrid mana, Phyrexian mana, and X costs is well-designed and CR-compliant. The HybridMana and PhyrexianMana enums cover all standard variants, the flatten_hybrid_phyrexian function correctly resolves payment choices, mana_value() follows CR 202.3e-g, color identity includes hybrid/Phyrexian colors, and hash support is complete. The main issues are in card definitions (wrong mana cost on Brokkos, missing abilities/halves on several cards) and the simulator's mana_solver not accounting for the new ManaCost fields.

Many of the card def TODOs (filter land output, planeswalker abilities, Mockingbird copy, Nethroi return-from-graveyard) are pre-existing DSL gaps that PB-9 was not expected to solve. These are tracked as LOW.
