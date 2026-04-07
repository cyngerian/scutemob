# Primitive Batch Review: PB-H -- Mass Reanimate

**Date**: 2026-04-06
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 400.7, 101.4, 603.6a, 701.17a, 704.5m
**Engine files reviewed**: `crates/engine/src/cards/card_definition.rs` (lines 1930-1963), `crates/engine/src/effects/mod.rs` (lines 4622-4997), `crates/engine/src/state/hash.rs` (lines 5212-5232)
**Card defs reviewed**: 5 (splendid_reclamation.rs, open_the_vaults.rs, eerie_ultimatum.rs, world_shaper.rs, living_death.rs)

## Verdict: needs-fix

One MEDIUM finding (missing planned test for replacement-exile exclusion in Living Death)
and three LOW findings. No HIGH findings. Engine logic is correct per CR rules.
The ETB chain matches the established `PutLandFromHandOntoBattlefield` pattern exactly.
Living Death step-1 ID tracking correctly uses new ObjectIds from `move_object_to_zone`.
Step-2 sacrifice correctly does NOT contaminate the step-1 exiled set. Hash discriminants
79/80 are unique within the Effect enum.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `effects/mod.rs:4761` | **APNAP order is sort-by-PlayerId, not true APNAP.** Player order uses `player_order.sort()` which sorts by PlayerId numeric value. True APNAP starts with the active player then proceeds in turn order. In 4-player Commander the active player is not necessarily PlayerId(1). However, the engine consistently uses PlayerId sort order as its APNAP approximation elsewhere (DestroyAll, ExileAll, BounceAll), so this is consistent. **Fix:** No fix needed now; note for future APNAP correctness audit. |
| 2 | LOW | `effects/mod.rs:4806` | **Living Death step 2 creature filter uses `obj.characteristics.card_types` for phased-in check but `calculate_characteristics` for card_types.** The filter at line 4806 checks `obj.characteristics.card_types.contains(&CardType::Creature)` for the battlefield snapshot, but line 4818 uses `calculate_characteristics` for the sacrifice logic. The snapshot should also use layer-resolved characteristics for consistency, since a non-creature permanent could become a creature via continuous effects. However, this inconsistency is benign in practice -- if layers make something a creature, it should be sacrificed. **Fix:** Change line 4807 filter to use `calculate_characteristics(state, *id)` instead of `obj.characteristics.card_types` for the battlefield creature snapshot. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | LOW | `world_shaper.rs` | **Attack trigger missing "you may" optionality.** Oracle says "you may mill three cards" but the Triggered ability has no mechanism to model optionality. This is a pre-existing DSL limitation (AbilityDefinition::Triggered has no `optional` field), not a PB-H regression. **Fix:** No fix for PB-H; track as a DSL gap for future work. |

## Test Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| 4 | MEDIUM | `tests/mass_reanimate.rs` | **Missing planned test: replacement-exile exclusion.** The plan specified `test_living_death_replacement_exile_not_returned` to verify that creatures sacrificed in step 2 and exiled by a replacement effect (e.g., Leyline of the Void) are NOT returned in step 3. This test was not implemented. The engine code handles this correctly (line 4858 comment explicitly notes this), but without a test, the 2018-03-16 ruling's second instruction is not verified by any assertion. **Fix:** Add `test_living_death_replacement_exile_not_returned` -- set up a replacement effect that redirects graveyard zone changes to exile, run LivingDeath, verify the replacement-exiled cards remain in exile and are not on the battlefield. |

### Finding Details

#### Finding 1: APNAP order is sort-by-PlayerId

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:4761`
**CR Rule**: 101.4 -- "If multiple players would make choices and/or take actions at the same time, the active player makes any choices required, then the next player in turn order..."
**Issue**: `player_order.sort()` sorts by PlayerId numeric value, which may not match true APNAP order (active player first, then turn order). The active player might be PlayerId(3) in a 4-player game.
**Fix**: No fix needed for PB-H -- this is consistent with the engine's existing APNAP approximation used in all mass effects. A future APNAP correctness audit should address this globally.

#### Finding 2: Step 2 creature filter inconsistency

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:4806`
**CR Rule**: 400.7 -- layer-resolved characteristics determine what is a "creature"
**Issue**: The battlefield creature snapshot (line 4804-4812) uses `obj.characteristics.card_types` to check if something is a creature, but the sacrifice logic (line 4818) correctly uses `calculate_characteristics`. Animate effects (e.g., Gideon Jura becoming a creature) would be missed by the snapshot filter but caught by the sacrifice logic. In practice this is a no-op because the snapshot determines the iteration set.
**Fix**: Change the snapshot filter to use `calculate_characteristics(state, *id).unwrap_or_else(|| obj.characteristics.clone()).card_types.contains(&CardType::Creature)` for full layer correctness.

#### Finding 3: World Shaper "you may" not modeled

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/world_shaper.rs:17`
**Oracle**: "Whenever World Shaper attacks, you may mill three cards."
**Issue**: The "you may" optionality is not represented in the triggered ability. The engine will always mill 3 cards on attack without giving the player a choice. This is a pre-existing DSL limitation, not a PB-H regression.
**Fix**: No fix for PB-H. Track as a DSL gap for future AbilityDefinition::Triggered `optional` field.

#### Finding 4: Missing replacement-exile test

**Severity**: MEDIUM
**File**: `crates/engine/tests/mass_reanimate.rs`
**CR Rule**: 2018-03-16 ruling -- "If a replacement effect (such as that of Leyline of the Void) causes any of the sacrificed creatures to be exiled instead of put into a graveyard, those cards aren't returned to the battlefield."
**Issue**: The plan specified this test but it was not implemented. The engine code at line 4858 has a comment confirming the correct behavior, but no test verifies it. This is the most important correctness property of Living Death's interaction with replacement effects.
**Fix**: Add a test that registers a zone-change replacement effect redirecting graveyard-bound permanents to exile, then runs LivingDeath, and asserts that: (a) the replacement-exiled cards are in exile, not on the battlefield; (b) step-1 exiled cards ARE on the battlefield; (c) the replacement-exiled card IDs differ from step-1 exiled IDs.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 400.7 (new object identity) | Yes | Yes | test_living_death_step1_id_tracking |
| 101.4 (APNAP simultaneous) | Partial | Yes | PlayerId sort, not true APNAP; test_mass_reanimate_multiplayer |
| 603.6a (simultaneous ETB) | Yes | Yes | test_mass_reanimate_etb_triggers_fire |
| 701.17a (sacrifice) | Yes | Yes | test_living_death_basic, test_living_death_two_players_full_swap |
| 704.5m (unattached Aura SBA) | Documented | No | Open the Vaults Aura approximation; deferred to M10+ |
| 2018-03-16 ruling (replacement exile) | Yes (code) | **No** | Finding 4: test missing |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| splendid_reclamation | Yes | 0 | Yes | Lands return tapped, controller's GY only |
| open_the_vaults | Yes | 1 (M10+ Aura) | Partial | Auras enter unattached then SBA removes; acceptable approximation |
| eerie_ultimatum | Yes | 1 (M10+ selection) | Partial | Returns ALL qualifying cards (max greed); interactive choice deferred |
| world_shaper | Yes | 0 | Partial | "you may" not modeled (LOW, pre-existing DSL gap) |
| living_death | Yes | 0 | Yes | Three-step sequence correct per 2018-03-16 ruling |
