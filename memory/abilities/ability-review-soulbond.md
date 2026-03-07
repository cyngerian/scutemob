# Ability Review: Soulbond

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.95
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KW disc 129)
- `crates/engine/src/state/game_object.rs` (paired_with field)
- `crates/engine/src/state/hash.rs` (hash arms for paired_with, Soulbond KW, WhilePaired, SoulbondTrigger SOK, AbilityDefinition::Soulbond, SoulbondGrant)
- `crates/engine/src/state/stubs.rs` (SoulbondSelfETB, SoulbondOtherETB PTK; soulbond_pair_target field)
- `crates/engine/src/state/stack.rs` (SoulbondTrigger SOK disc 49)
- `crates/engine/src/state/mod.rs` (zone-change unpairing at both move_object_to_zone sites)
- `crates/engine/src/state/builder.rs` (paired_with: None)
- `crates/engine/src/state/continuous_effect.rs` (WhilePaired duration)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Soulbond disc 50, SoulbondGrant)
- `crates/engine/src/effects/mod.rs` (paired_with: None in token creation)
- `crates/engine/src/rules/sba.rs` (check_soulbond_unpairing)
- `crates/engine/src/rules/abilities.rs` (SoulbondSelfETB + SoulbondOtherETB trigger detection; flush PTK match arms)
- `crates/engine/src/rules/resolution.rs` (SoulbondTrigger resolution with fizzle check + CE registration; counter-spell arm)
- `crates/engine/src/rules/layers.rs` (WhilePaired in is_effect_active)
- `crates/engine/src/rules/replacement.rs` (WhilePaired in duration check)
- `crates/engine/src/testing/replay_harness.rs` (enrich_spec_from_def: Soulbond KW addition)
- `crates/engine/tests/soulbond.rs` (10 tests)
- `tools/replay-viewer/src/view_model.rs` (Soulbond KW + SoulbondTrigger SOK display)
- `tools/tui/src/play/panels/stack_view.rs` (SoulbondTrigger SOK display)

## Verdict: needs-fix

One MEDIUM finding: the resolution fizzle check (CR 702.95c) uses base characteristics
instead of layer-resolved characteristics for the "is still a creature" test. The SBA
correctly uses `calculate_characteristics`, but resolution does not. This creates a
scenario where a creature that has lost its creature type via a Layer 4 effect (e.g.,
Song of the Dryads, Imprisoned in the Moon) would still be paired by the trigger
resolution despite CR 702.95c saying "no longer a creature." Two LOW findings for
helper export and base-characteristic usage in trigger partner selection.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `resolution.rs:3025,3036` | **Resolution creature-type check uses base characteristics.** Should use `calculate_characteristics` per CR 702.95c. **Fix:** replace `o.characteristics.card_types` with `calculate_characteristics(state, id)` result. |
| 2 | LOW | `cards/helpers.rs` | **SoulbondGrant not exported from helpers.rs.** Card defs using `use crate::cards::helpers::*` cannot reference it. **Fix:** add `SoulbondGrant` to the re-export list. |
| 3 | LOW | `abilities.rs:2507,2578` | **Partner/source search uses base card_types.** Trigger-time partner selection checks `obj.characteristics.card_types` (base) not layer-resolved. Unlikely to cause issues in practice but inconsistent with the `entering_is_creature` check which uses `calculate_characteristics`. **Fix:** use `calculate_characteristics` for the creature-type check in partner filtering. |

### Finding Details

#### Finding 1: Resolution creature-type check uses base characteristics

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:3025,3036`
**CR Rule**: 702.95c -- "When the soulbond ability resolves, if either object that would be paired is no longer a creature, no longer on the battlefield, or no longer under the control of the player who controls the soulbond ability, neither object becomes paired."
**Issue**: The resolution fizzle check at lines 3025 and 3036 uses `o.characteristics.card_types.contains(&CardType::Creature)` which reads the base/printed card types, not the layer-resolved types. If a continuous effect has changed an object's type (e.g., Song of the Dryads turning it into a Forest, or Imprisoned in the Moon making it a colorless land), the base characteristics would still show "Creature" but the layer-resolved characteristics would not. The SBA at `sba.rs:1135-1140` correctly uses `calculate_characteristics` for the same check, creating an inconsistency. In the worst case, a soulbond trigger could pair a non-creature (layer-resolved) because its base type is still Creature.
**Fix**: Replace the creature-type checks at lines 3025 and 3036 with `calculate_characteristics(state, source_object)` / `calculate_characteristics(state, pair_target)` results, similar to how `check_soulbond_unpairing` does it in sba.rs.

#### Finding 2: SoulbondGrant not exported from helpers.rs

**Severity**: LOW
**File**: `crates/engine/src/cards/helpers.rs`
**CR Rule**: N/A (infrastructure)
**Issue**: The plan specified adding `SoulbondGrant` to `helpers.rs` for card definition files that use `use crate::cards::helpers::*`. `ChampionFilter` is already exported there (line 11), but `SoulbondGrant` was not added. This will block step 5 (card definition authoring) since card defs use the helpers prelude.
**Fix**: Add `SoulbondGrant` to the re-export list in `helpers.rs`, alongside `ChampionFilter`.

#### Finding 3: Trigger-time partner search uses base card_types

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:2507,2578`
**CR Rule**: 702.95a -- "you may pair this creature with another unpaired creature you control"
**Issue**: When selecting pairing partners at trigger time, the code checks `obj.characteristics.card_types.contains(&CardType::Creature)` (base types) for both the SelfETB partner search (line 2507) and the OtherETB soulbond source search (line 2578). This is inconsistent with the `entering_is_creature` check (line 2462-2471) which uses `calculate_characteristics`. An object that is a creature in base types but not after layer resolution (e.g., Song of the Dryads target) could be incorrectly selected as a partner. The resolution fizzle check (Finding 1) would also need to be fixed for this to matter in practice, since the resolution would currently also pass the creature check.
**Fix**: Use `calculate_characteristics(state, obj.id)` for the creature-type check in the partner/source filtering closures.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.95a (self-ETB trigger) | Yes | Yes | test_soulbond_self_etb_pairs_with_unpaired_creature |
| 702.95a (other-ETB trigger) | Yes | Yes | test_soulbond_other_etb_pairs_with_entering_creature |
| 702.95a (intervening-if) | Yes | Yes | test_soulbond_no_trigger_if_no_unpaired_partner |
| 702.95a ("for as long as") | Yes | Yes | test_soulbond_grants_apply_while_paired, test_soulbond_grants_removed_when_unpaired |
| 702.95b (symmetric pairing) | Yes | Yes | All pairing tests verify both directions |
| 702.95c (resolution fizzle) | Partial | Yes | Uses base types not layer-resolved (Finding 1) |
| 702.95d (one pair only) | Yes | Yes | test_soulbond_already_paired_cannot_repair |
| 702.95e (unpair: zone change) | Yes | Yes | test_soulbond_unpair_on_zone_change; move_object_to_zone clears both |
| 702.95e (unpair: control change) | Yes | Yes | test_soulbond_unpair_on_controller_change; SBA handles |
| 702.95e (unpair: stops being creature) | Yes | No | SBA checks via calculate_characteristics; no test |
| 702.95e (phasing) | Implicit | No | WhilePaired checks is_phased_in(); no dedicated test |

## Additional Observations (no action required)

1. **Plan simplification**: The plan called for `soulbond_owner: ObjectId` on SoulbondTrigger, but the implementation correctly omits it since `source_object` always IS the soulbond creature in both trigger paths. Good simplification.

2. **WhilePaired vs soulbond keyword removal**: The plan's "Interactions to Watch" section extensively discusses whether WhilePaired should check for soulbond keyword presence (Flowering Lumberknot ruling). The implementation correctly accepts this as a known V1 gap -- the CEs remain active as long as pairing exists, even if soulbond is removed by Humility. This is documented in the plan and acceptable.

3. **Two soulbond creatures**: Test 10 correctly verifies that when both creatures have soulbond, both triggers fire but only one succeeds (the other fizzles because the target is already paired). APNAP ordering determines which resolves first. Good coverage.

4. **Hash coverage**: Complete. `paired_with`, `KeywordAbility::Soulbond` (disc 129), `EffectDuration::WhilePaired`, `StackObjectKind::SoulbondTrigger` (disc 49), `AbilityDefinition::Soulbond` (disc 50), and `SoulbondGrant` all have hash implementations.

5. **Zone-change unpairing**: Both `move_object_to_zone` call sites (lines ~370 and ~524 in mod.rs) correctly clear `paired_with` on both the departing object AND its partner. Belt-and-suspenders with the SBA.
