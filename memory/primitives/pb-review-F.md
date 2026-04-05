# Primitive Batch Review: PB-F --- Damage Multiplier

**Date**: 2026-04-05
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 614.1, 614.1a, 701.10g, 616.1
**Engine files reviewed**: `state/replacement_effect.rs`, `state/game_object.rs`, `state/mod.rs`, `state/hash.rs`, `state/builder.rs`, `rules/replacement.rs`, `rules/layers.rs`, `rules/abilities.rs`, `rules/lands.rs`, `rules/resolution.rs`, `effects/mod.rs`, `cards/card_definition.rs`
**Card defs reviewed**: 3 (fiery_emancipation.rs, lightning_army_of_one.rs, neriv_heart_of_the_storm.rs)

## Verdict: clean

All engine changes are correct per CR rules. The TripleDamage variant multiplies by 3 (not 2x2). The new DamageTargetFilter variants (ToPlayerOrTheirPermanents, FromControllerCreaturesEnteredThisTurn) are correctly implemented with proper filter matching. The entered_turn field is set at all ETB sites: move_object_to_zone (state/mod.rs), add_object for tokens, resolution.rs (6 sites), effects/mod.rs (1 explicit site + 2 token sites relying on add_object). The RegisterReplacementEffect dispatches correctly with proper PlayerId(0) placeholder resolution (damaged_player for ToPlayerOrTheirPermanents, controller for duration and other filters). The replacement effect expiry in expire_until_next_turn_effects correctly filters both continuous_effects and replacement_effects. Hash discriminants are unique within their respective enum contexts (TripleDamage: 18, ToPlayerOrTheirPermanents: 6, FromControllerCreaturesEnteredThisTurn: 7, RegisterReplacementEffect: 77). All three card defs match their oracle text exactly. Tests cover positive cases, negative cases, multiplier stacking, and serde round-trips. No findings at any severity.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| (none) | | | |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| (none) | | | |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 614.1 | Yes | Yes | test_triple_damage_basic, test_neriv_*, test_to_player_or_their_permanents_* |
| 614.1a | Yes | Yes | All damage multiplier tests verify "instead" replacement semantics |
| 701.10g | Yes | Yes | test_triple_damage_basic (x3), test_double_and_triple_stack (x2 then x3 = x6) |
| 616.1 | Yes | Yes | test_double_and_triple_stack verifies multiplicative stacking |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| fiery_emancipation.rs | Yes | 0 | Yes | FromControllerSources + TripleDamage matches oracle exactly |
| lightning_army_of_one.rs | Yes | 0 | Yes | Stagger triggered ability with RegisterReplacementEffect; DoubleDamage + ToPlayerOrTheirPermanents + UntilYourNextTurn |
| neriv_heart_of_the_storm.rs | Yes | 0 | Yes | Static replacement with FromControllerCreaturesEnteredThisTurn + DoubleDamage |

## Design Notes (informational, not findings)

1. **apply_damage_prevention TripleDamage arm (replacement.rs:1952)**: This arm exists for defensive completeness but is currently unreachable through normal game flow. The `find_applicable` path constructs event triggers with target-specific filters (Player/Permanent), which don't match source-based filters (FromControllerSources, etc.) via `event_damage_target_matches_filter`. Damage multipliers are handled exclusively through `apply_damage_doubling`. This is a pre-existing pattern (DoubleDamage had the same unreachable arm), not introduced by this batch.

2. **CR 616.1 player choice for multiplier ordering**: The engine applies all matching multipliers in registration order without player choice. Since multiplication is commutative, this produces the correct result for all current cards. If a future card introduces an additive damage modification alongside a multiplier, player choice would matter. Acceptable pre-M10 simplification, noted in the plan.

3. **builder.rs entered_turn = None**: Test-placed permanents via GameStateBuilder get `entered_turn: None`, meaning they don't match Neriv's "entered this turn" filter. Tests correctly handle this by manually setting entered_turn on source creatures. This is the intended design -- builder objects represent pre-existing permanents.
