# Ability Review: Forage

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.61
**Files reviewed**: `crates/engine/src/state/game_object.rs:94-108`, `crates/engine/src/state/hash.rs:1364-1371`, `crates/engine/src/rules/abilities.rs:338-409`, `crates/engine/tests/forage.rs`

## Verdict: clean

The forage implementation is correct and well-tested. CR 701.61a is faithfully implemented as a composite activation cost with deterministic choice between sacrificing a Food artifact (subtype-checked via `calculate_characteristics`) and exiling 3 cards from the controller's graveyard. The hash coverage is complete, phased-out permanents are filtered, and all seven tests cover the key paths and edge cases. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:388-396` | **Source-as-Food edge case untested.** If the source permanent itself has Food subtype, it could be sacrificed to pay its own forage cost. Accepted but untested. **Fix:** Add a test where the source is a Food artifact creature with a forage ability; verify the ability still resolves from the stack after source is sacrificed. |
| 2 | LOW | `abilities.rs:393` | **Missing sacrifice-specific event.** The Food sacrifice path emits `PermanentDestroyed` but there is no `PermanentSacrificed` event type in the engine. This means "whenever you sacrifice" triggers cannot distinguish sacrifice from destruction. This is a pre-existing engine-wide gap, not forage-specific. **Fix:** Deferred. When a `PermanentSacrificed` event is added engine-wide, update the forage sacrifice path to emit it. |
| 3 | LOW | `forage.rs:173-178` | **Incorrect CR citation in assertion message.** The assertion message says "CR 702.61a" but the correct rule is CR 701.61a (keyword actions are in 701, not 702). **Fix:** Change the string from `"CR 702.61a: AbilityActivated event expected"` to `"CR 701.61a: AbilityActivated event expected"`. |

### Finding Details

#### Finding 1: Source-as-Food edge case untested

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:388-396`
**CR Rule**: 701.61a -- "To forage means 'Exile three cards from your graveyard or sacrifice a Food.'"
**Issue**: If the source of the forage ability is itself a Food artifact (e.g., a Food artifact creature), the deterministic fallback could pick it as the Food to sacrifice. The ability would still resolve from the stack (source identity is captured before costs are paid), but this path is not tested. The code does not exclude the source from Food candidates, which is technically correct (you can sacrifice the source to pay its own cost), but the interaction deserves a test to confirm the engine handles it gracefully.
**Fix**: Add a test where the source is a Food artifact creature with a forage ability and no other Food is available. Verify the ability goes on the stack, the source is sacrificed, and the ability resolves correctly.

#### Finding 2: Missing sacrifice-specific event

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:393`
**CR Rule**: 701.17a -- "To sacrifice a permanent, its controller moves it from the battlefield directly to its owner's graveyard."
**Issue**: The forage Food sacrifice emits `PermanentDestroyed` rather than a sacrifice-specific event. The engine has no `PermanentSacrificed` event variant, so "whenever you sacrifice" triggers cannot fire from forage. This is a pre-existing engine-wide limitation that affects all sacrifice-as-cost paths (sacrifice_self also uses `CreatureDied`/`PermanentDestroyed`), not unique to forage.
**Fix**: Deferred to engine-wide sacrifice event rework. No action needed in the forage implementation itself.

#### Finding 3: Typo in CR citation

**Severity**: LOW
**File**: `crates/engine/tests/forage.rs:177`
**CR Rule**: 701.61a
**Issue**: The assertion message string contains "CR 702.61a" but should be "CR 701.61a". Section 701 is keyword actions; 702 is keyword abilities. Forage is a keyword action.
**Fix**: Change the string literal from `"CR 702.61a: AbilityActivated event expected"` to `"CR 701.61a: AbilityActivated event expected"`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.61a (sacrifice Food) | Yes | Yes | test_forage_sacrifice_food, test_forage_food_is_artifact_subtype_not_just_token |
| 701.61a (exile 3 from GY) | Yes | Yes | test_forage_exile_three_from_graveyard |
| 701.61a (insufficient resources) | Yes | Yes | test_forage_insufficient_resources |
| 701.61a (Food = artifact subtype) | Yes | Yes | test_forage_food_is_artifact_subtype_not_just_token, test_forage_non_food_artifact_rejected |
| 701.61a (deterministic choice) | Yes | Yes | test_forage_prefers_food_when_both_available |
| CR 602.2 (cost paid at activation) | Yes | Yes | test_forage_requires_mana_cost_too; all tests verify cost timing |
| Phased-out filter | Yes | No | `is_phased_in()` filter present in code but no phasing test (acceptable -- phasing is tested elsewhere) |
| Layer-resolved Food check | Yes | No | `calculate_characteristics` used in code but no continuous-effect test (acceptable -- layer system tested elsewhere) |

## Implementation Quality Notes

- **Hash coverage**: `forage` field correctly hashed in `ActivationCost::hash_into` at hash.rs:1369.
- **No new KeywordAbility/StackObjectKind**: Correct -- forage is a keyword action (CR 701), not a keyword ability (CR 702). Uses existing `Activated` ability definition with `cost.forage: true`.
- **Phased-out filtering**: Food search correctly calls `obj.is_phased_in()` (line 347).
- **Layer-resolved types**: Food subtype check uses `calculate_characteristics` (line 350), correctly handling continuous effects that might add/remove the Food subtype.
- **Deterministic ordering**: Both Food IDs and graveyard IDs are sorted by ObjectId for deterministic behavior (lines 362, 376).
- **Error message**: Clear error message citing CR 701.61a (line 383-384).
- **Test quality**: Seven tests covering positive (sacrifice Food, exile 3), negative (insufficient resources, non-Food artifact, missing mana), edge case (non-token Food), and deterministic behavior. All tests cite CR rules in comments.
