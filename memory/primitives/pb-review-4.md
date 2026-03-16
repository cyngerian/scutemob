# Primitive Batch Review: PB-4 -- Sacrifice as Activation Cost

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 602.2 (activated ability costs), CR 701.21 (sacrifice), CR 605.1a (mana abilities)
**Engine files reviewed**: `state/game_object.rs` (SacrificeFilter enum, ActivationCost struct), `rules/abilities.rs` (sacrifice validation+execution), `rules/command.rs` (sacrifice_target field), `state/hash.rs` (SacrificeFilter hash), `testing/replay_harness.rs` (flatten_cost_into, enrich_spec_from_def)
**Card defs reviewed**: 26 cards from PB-4 spec + 3 additional cards using Cost::Sacrifice

## Verdict: needs-fix

PB-4 engine changes are well-implemented: SacrificeFilter enum covers the needed variants,
abilities.rs correctly validates controller/zone/type-match with layer-resolved characteristics,
hash.rs is complete, and tests cover positive/negative/missing-target cases. However, 13 of
26 card defs still have TODOs for the sacrifice ability (blocked on other primitives, not PB-4
itself). The main PB-4-specific finding is that `flatten_cost_into` has a silent default
fallback to `SacrificeFilter::Creature` that could mask bugs. Additionally, three mana-producing
sacrifice abilities (Phyrexian Tower, Ashnod's Altar, Phyrexian Altar) are modeled as
stack-using activated abilities when CR 605.1a says they are mana abilities.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `replay_harness.rs:2696` | **Silent fallback in flatten_cost_into.** `_ => SacrificeFilter::Creature` silently converts any unrecognized TargetFilter to Creature. If a card def uses an unexpected filter shape, the wrong sacrifice type is enforced with no warning. **Fix:** Replace `_ => SacrificeFilter::Creature` with a `_ => panic!("unhandled TargetFilter in Cost::Sacrifice: {:?}", filter)` or log a warning, so bugs are caught early. |
| 2 | **MEDIUM** | `replay_harness.rs:1976` | **Mana abilities with sacrifice-another cost use the stack.** Phyrexian Tower ({T}, Sacrifice a creature: Add {B}{B}), Ashnod's Altar, and Phyrexian Altar produce mana and don't target, so they are mana abilities per CR 605.1a. But `enrich_spec_from_def` only recognizes `Cost::Tap`-only abilities as mana abilities. These three cards incorrectly resolve via the stack. **Fix:** Extend the mana-ability detection in `enrich_spec_from_def` to recognize `Cost::Sacrifice(filter)` and `Cost::Sequence([Cost::Tap, Cost::Sacrifice(filter)])` with `Effect::AddMana`-family effects as mana abilities. Also extend `ManaAbility` struct to support `sacrifice_filter`, and extend `mana.rs` to handle it. Note: this is a known architectural gap, not introduced by PB-4. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | MEDIUM | `command_beacon.rs` | **Missing sacrifice ability.** Oracle: "{T}, Sacrifice this land: Put your commander into your hand from the command zone." TODO remains. Blocked on command-zone-to-hand effect, not PB-4. **Fix:** Implement when Effect::ReturnCommanderToHand is available. |
| 4 | MEDIUM | `strip_mine.rs` | **Missing sacrifice ability.** Oracle: "{T}, Sacrifice this land: Destroy target land." TODO remains. Blocked on PB-5 (targeted activated). **Fix:** Implement when PB-5 targeting is available. |
| 5 | MEDIUM | `wasteland.rs` | **Missing sacrifice ability.** Oracle: "{T}, Sacrifice this land: Destroy target nonbasic land." TODO remains. Blocked on PB-5. **Fix:** Same as strip_mine. |
| 6 | MEDIUM | `scavenger_grounds.rs` | **Missing sacrifice ability.** Oracle: "{2}, {T}, Sacrifice a Desert: Exile all graveyards." TODO remains. Blocked on PB-19 (mass exile). **Fix:** Implement when Effect::ExileAllGraveyards is available. |
| 7 | MEDIUM | `boromir_warden_of_the_tower.rs` | **Missing sacrifice ability.** Oracle: "Sacrifice Boromir: Creatures you control gain indestructible until end of turn. The Ring tempts you." TODO remains. Blocked on PB-6 (mass indestructible grant). **Fix:** Implement when mass-keyword-grant effect is available. |
| 8 | MEDIUM | `etchings_of_the_chosen.rs` | **Missing sacrifice ability + static.** Oracle: "Creatures you control of the chosen type get +1/+1" and "{1}, Sacrifice a creature of the chosen type: Target creature you control gains indestructible until end of turn." Both TODO. Blocked on chosen-subtype dynamic filter. **Fix:** Implement when EffectFilter::ChosenSubtype is available. |
| 9 | MEDIUM | `torch_courier.rs` | **Missing sacrifice ability.** Oracle: "Sacrifice this creature: Another target creature gains haste until end of turn." TODO remains. Blocked on PB-5 (targeted). **Fix:** Implement when PB-5 targeting is available. |
| 10 | MEDIUM | `vampire_hexmage.rs` | **Missing sacrifice ability.** Oracle: "Sacrifice this creature: Remove all counters from target permanent." TODO remains. Blocked on PB-5 (targeted). **Fix:** Implement when PB-5 targeting + Effect::RemoveAllCounters is available. |
| 11 | MEDIUM | `hope_of_ghirapur.rs` | **Missing sacrifice ability.** Oracle: "Sacrifice Hope of Ghirapur: Until your next turn, target player who was dealt combat damage by Hope of Ghirapur this turn can't cast noncreature spells." TODO remains. Blocked on combat-damage tracking + cast restriction. **Fix:** Implement when those primitives are available. |
| 12 | MEDIUM | `skemfar_elderhall.rs` | **Missing sacrifice ability.** Oracle: "{2}{B}{B}{G}, {T}, Sacrifice this land: Up to one target creature you don't control gets -2/-2 until end of turn. Create two 1/1 green Elf Warrior creature tokens." TODO remains. Blocked on targeted debuff + token creation combo. **Fix:** Implement when PB-5 targeting is available. |
| 13 | MEDIUM | `the_world_tree.rs` | **Missing sacrifice ability + static.** Oracle: "{W}{W}{U}{U}{B}{B}{R}{R}{G}{G}, {T}, Sacrifice: Search for any number of God cards" and "As long as you control 6+ lands, lands you control have {T}: Add any color." Both TODO. Blocked on multi-card search and count-threshold grant. **Fix:** Implement when those primitives are available. |
| 14 | MEDIUM | `alexios_deimos_of_kosmos.rs` | **Missing "can't be sacrificed" restriction.** Oracle says "can't be sacrificed" but no such restriction exists in DSL. Also missing upkeep trigger (each player's upkeep). Sacrifice cost is not relevant here (the card PREVENTS sacrifice, not requiring it). The TODO is documented. **Fix:** Implement when sacrifice-restriction primitive is available. |
| 15 | LOW | `treasure_vault.rs:31` | **Approximated token count.** Oracle: "Create X Treasure tokens" but effect creates only 1 Treasure. TODO documented. **Fix:** Replace `treasure_token_spec(1)` with X-scaled version when EffectAmount::XValue is wired to token creation count. |
| 16 | LOW | `inventors_fair.rs:30` | **Missing activation condition.** Oracle: "Activate only if you control three or more artifacts." No condition enforced. TODO documented. **Fix:** Add activation condition when PB-18 stax/restrictions covers this pattern. |
| 17 | LOW | `haven_of_the_spirit_dragon.rs:40` | **Incomplete target filter.** Oracle: "Dragon creature card or Ugin planeswalker card" but def only targets Dragon creatures. TODO documented. Ugin union filter not expressible. **Fix:** Add name-based filter union when available. |
| 18 | LOW | `crop_rotation.rs:24` | **Missing spell additional cost.** "As an additional cost to cast this spell, sacrifice a land" is a SPELL cost, not an activated ability cost. PB-4's sacrifice_filter is for activated abilities only. TODO correctly identifies this as a spell-additional-cost gap. **Fix:** Not PB-4 scope; needs AdditionalCost::Sacrifice on spell casting. |
| 19 | LOW | `deadly_dispute.rs:16` | **Missing spell additional cost.** Same as crop_rotation -- sacrifice is a spell additional cost, not PB-4 activated cost. **Fix:** Not PB-4 scope. |
| 20 | LOW | `flare_of_fortitude.rs:14` | **Missing alt cost + spell effect.** Alternative cost (sacrifice nontoken white creature) and complex effect. Not PB-4 scope. **Fix:** Needs AltCostKind extension. |

### Finding Details

#### Finding 1: Silent fallback in flatten_cost_into

**Severity**: MEDIUM
**File**: `crates/engine/src/testing/replay_harness.rs:2696`
**CR Rule**: CR 602.2 -- sacrifice costs must match the specified filter
**Issue**: The `flatten_cost_into` function converts `Cost::Sacrifice(TargetFilter)` to `SacrificeFilter` but defaults to `SacrificeFilter::Creature` for any unrecognized `has_card_type` value (including `None`). This means a card def with `Cost::Sacrifice(TargetFilter::default())` (no type filter) would silently become "sacrifice a creature" instead of "sacrifice any permanent." If a future card def has a non-standard sacrifice filter, the bug would be silent.
**Fix**: Replace `_ => SacrificeFilter::Creature` with a more explicit fallback that either panics in debug builds or logs a warning: `_ => { debug_assert!(false, "unhandled TargetFilter in Cost::Sacrifice: {:?}", filter); SacrificeFilter::Creature }`.

#### Finding 2: Mana abilities with sacrifice-another cost use the stack

**Severity**: MEDIUM
**File**: `crates/engine/src/testing/replay_harness.rs:1976`
**CR Rule**: CR 605.1a -- "An activated ability is a mana ability if it meets all of the following criteria: it doesn't require a target [...], it could put mana into a player's mana pool when it resolves, and it's not a loyalty ability."
**Issue**: Phyrexian Tower, Ashnod's Altar, and Phyrexian Altar all have non-targeting abilities that produce mana. Per CR 605.1a, these are mana abilities that should resolve immediately without using the stack. However, because `enrich_spec_from_def` only classifies `Cost::Tap`-only abilities as mana abilities, these three cards' sacrifice-for-mana abilities are registered as `ActivatedAbility` objects that go through the stack in `abilities.rs`. This means opponents can respond to what should be a mana ability, which is incorrect game state.
**Fix**: This is a pre-existing architectural gap, not introduced by PB-4. Extend `ManaAbility` to support `sacrifice_filter: Option<SacrificeFilter>`. Extend `mana.rs` to handle sacrifice-filter validation and execution. Extend `enrich_spec_from_def` to detect `Cost::Sacrifice(creature)` + `Effect::AddMana` as a mana ability pattern. This should be tracked as a separate issue rather than blocking PB-4 closure.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 602.2 (activated ability costs) | Yes | Yes | 4 tests in abilities.rs: valid creature, reject artifact, reject opponent's, missing target |
| 701.21a (sacrifice definition) | Yes | Yes | Sacrifice moves to owner's graveyard, cannot sacrifice what you don't control |
| 605.1a (mana ability classification) | Partial | No | Sacrifice-for-mana abilities incorrectly use the stack (Finding 2) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| phyrexian_tower | Yes | 0 | Partial | Sac ability implemented but uses stack (should be mana ability) |
| high_market | Yes | 0 | Yes | Sac-a-creature: gain life -- correct |
| grim_backwoods | Yes | 0 | Yes | Sac-a-creature: draw -- correct |
| ghost_quarter | Yes | 0 | Yes | Sac-self + destroy target land + search -- all implemented |
| buried_ruin | Yes | 0 | Yes | Sac-self + return artifact from GY -- correct |
| gingerbrute | Yes | 1 | Partial | Food sac ability correct; evasion ability TODO (PB-5) |
| viscera_seer | Yes | 0 | Partial | Correct but uses stack (should be non-mana ability -- actually correct here since Scry is not mana) |
| ashnods_altar | Yes | 0 | Partial | Uses stack (should be mana ability per CR 605.1a) |
| phyrexian_altar | Yes | 0 | Partial | Uses stack (should be mana ability per CR 605.1a) |
| command_beacon | Partial | 1 | No | Missing sacrifice ability entirely |
| strip_mine | Partial | 1 | No | Missing sacrifice ability entirely |
| wasteland | Partial | 1 | No | Missing sacrifice ability entirely |
| scavenger_grounds | Partial | 1 | No | Missing sacrifice ability entirely |
| boromir_warden_of_the_tower | Partial | 2 | No | Missing sacrifice ability + counter-spell trigger |
| etchings_of_the_chosen | Partial | 2 | No | Missing static + sacrifice ability |
| torch_courier | Partial | 1 | No | Missing sacrifice ability |
| vampire_hexmage | Partial | 1 | No | Missing sacrifice ability |
| hope_of_ghirapur | Partial | 1 | No | Missing sacrifice ability |
| skemfar_elderhall | Partial | 1 | Partial | ETB tapped + mana correct; sacrifice TODO |
| the_world_tree | Partial | 2 | No | ETB tapped + mana correct; static + sacrifice TODO |
| alexios_deimos_of_kosmos | Partial | 3 | No | Trample + MustAttack correct; 3 abilities TODO |
| treasure_vault | Partial | 1 | Partial | Sac-self correct but creates 1 treasure not X |
| inventors_fair | Partial | 2 | Partial | Sac-self + search correct; missing upkeep trigger + activation condition |
| haven_of_the_spirit_dragon | Partial | 1 | Partial | All abilities present; target filter incomplete (Ugin) |
| maelstrom_of_the_spirit_dragon | Yes | 0 | Yes | All abilities correct |
| crop_rotation | Partial | 1 | No | Spell sac cost not PB-4 scope |
| deadly_dispute | Partial | 1 | No | Spell sac cost not PB-4 scope |
| flare_of_fortitude | No | 2 | No | Alt cost + spell effect not PB-4 scope |
| camellia_the_seedmiser | Partial | 1 | Partial | Forage ability correct; sacrifice-trigger TODO |

## Notes

1. **PB-4 engine changes are solid.** The SacrificeFilter enum, ActivationCost integration,
   abilities.rs validation (zone, controller, type match via layers), hash.rs, and Command
   sacrifice_target field are all correctly implemented with proper error handling (no
   unwrap in library code). The 4 tests cover the key positive and negative cases.

2. **13 of 26 cards still have TODOs** for their sacrifice abilities. In every case, the
   TODO is blocked on a DIFFERENT primitive (PB-5 targeting, PB-6 static grants, PB-19
   mass exile, etc.), not on PB-4 itself. The `Cost::SacrificeSelf` and
   `Cost::Sacrifice(TargetFilter)` DSL primitives are available and used correctly where
   applicable.

3. **3 cards (Crop Rotation, Deadly Dispute, Flare of Fortitude)** require sacrifice as a
   SPELL additional cost, not an activated ability cost. These are out of PB-4 scope.

4. **Mana ability classification (Finding 2)** is a pre-existing architectural gap that
   affects Phyrexian Tower, Ashnod's Altar, and Phyrexian Altar. This should be tracked
   separately and does not block PB-4 closure.
