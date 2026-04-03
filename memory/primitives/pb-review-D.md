# Primitive Batch Review: PB-D -- Chosen Creature Type

**Date**: 2026-04-02
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 205.3m (creature types), 601.2f (cost reduction), 614.1c (ETB replacement)
**Engine files reviewed**: `state/continuous_effect.rs`, `state/hash.rs`, `state/game_object.rs`, `cards/card_definition.rs`, `rules/layers.rs`, `rules/abilities.rs`, `rules/casting.rs`, `effects/mod.rs`, `testing/replay_harness.rs`
**Card defs reviewed**: 8 (morophon_the_boundless, vanquishers_banner, patchwork_banner, heralds_horn, kindred_dominance, three_tree_city, etchings_of_the_chosen, pact_of_the_serpent) + cavern_of_souls (deferred check)

## Verdict: needs-fix

Implementation is solid overall. All 8 card defs match oracle text correctly. Engine
primitives are well-designed with proper fallback chains (ctx -> source permanent). Hash
discriminants are unique and sequential. EffectContext propagation is correct across all
construction sites and ForEach inner contexts. One MEDIUM finding (Morophon cost reduction
filter scope) and two LOW findings.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `casting.rs:5851` | **Morophon cost reduction filter too narrow.** `HasChosenCreatureSubtype` requires `CardType::Creature`, but Morophon's oracle says "Spells of the chosen type" (not "creature spells"). Kindred spells with matching subtypes are excluded. **Fix:** see Finding 1 below. |
| 2 | LOW | `effects/mod.rs:1036` | **ExileAll/BounceAll missing `check_chosen_subtype_filter`.** Not currently used by any card def with chosen subtype, but inconsistent with DestroyAll/PermanentCount. **Fix:** add `&& check_chosen_subtype_filter(state, ctx, filter, &chars)` to ExileAll and BounceAll filter chains for forward-compatibility. |
| 3 | LOW | `effects/mod.rs:6121-6138` | **TopCardIsCreatureOfChosenType uses raw characteristics.** Correct for library cards (CR 400.2: no continuous effects apply in library), but a comment noting why `calculate_characteristics` is intentionally not used would improve clarity. **Fix:** add a brief comment: `// CR 400.2: cards in library have printed characteristics; no layer calculation needed.` |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| (none) | -- | -- | All 8 card defs match oracle text. No issues found. |

### Finding Details

#### Finding 1: Morophon cost reduction filter too narrow

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:5848-5857`
**CR Rule**: 205.3m -- "Creatures and kindreds share their lists of subtypes; these subtypes are called creature types."
**Oracle**: Morophon: "Spells of the chosen type you cast cost {W}{U}{B}{R}{G} less to cast."
**Issue**: The `SpellCostFilter::HasChosenCreatureSubtype` match at line 5851 checks `spell_chars.card_types.contains(&CardType::Creature)`. Morophon's oracle says "Spells of the chosen type", not "creature spells of the chosen type." Per CR 205.3m, kindred (formerly tribal) cards share creature subtypes, so a Kindred - Elf Sorcery should also get the reduction. The current filter excludes all non-creature spells even if they have the correct creature subtype.

Herald's Horn explicitly says "creature spells you cast of the chosen type", so `HasChosenCreatureSubtype` is correct for Herald's Horn. But Morophon is broader.

**Impact**: LOW in practice -- Kindred/Tribal cards are rare in Commander. But the oracle text mismatch is technically incorrect.
**Fix**: Either (a) add a new `SpellCostFilter::HasChosenSubtype` variant that checks subtype without requiring `CardType::Creature`, use it for Morophon's modifier, and keep `HasChosenCreatureSubtype` for Herald's Horn/Urza's Incubator, or (b) rename and extend `HasChosenCreatureSubtype` to accept an optional `require_creature: bool` parameter. Option (a) is simpler and more explicit. Alternatively, this can be deferred as a known LOW gap since no Kindred cards are currently in the card pool.

#### Finding 2: ExileAll/BounceAll missing chosen subtype filter

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:1036`, `effects/mod.rs:1140`
**Issue**: `DestroyAll` (line 857) and `PermanentCount` (line 5155) call `check_chosen_subtype_filter`, but `ExileAll` and `BounceAll` do not. If a future card uses "exile all creatures that aren't of the chosen type" with `exclude_chosen_subtype: true`, it would silently ignore the filter.
**Fix**: Add `&& check_chosen_subtype_filter(state, ctx, filter, &chars)` after `matches_filter` in `ExileAll` (line 1036) and `BounceAll` (line 1140).

#### Finding 3: TopCardIsCreatureOfChosenType missing CR 400.2 comment

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:6121-6138`
**Issue**: The condition reads `card.characteristics.card_types` and `card.characteristics.subtypes` directly instead of using `calculate_characteristics`. This is correct for library cards per CR 400.2 (printed characteristics apply), but the reason is not documented.
**Fix**: Add comment before the characteristic reads: `// CR 400.2: Library cards have printed characteristics; layer calculation not needed.`

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 205.3m (creature types) | Yes | Yes | test_chosen_type_anthem_basic, test_chosen_type_anthem_other, test_chosen_type_destroy_all_except |
| 601.2f (cost reduction) | Yes | Yes (def check) | test_chosen_type_cost_reduction_colored, test_chosen_type_cost_reduction_generic |
| 614.1c (ETB replacement) | Yes | Yes | test_chosen_creature_type_etb_sets_type |
| 613.4c (Layer 7c P/T mod) | Yes | Yes | test_chosen_type_anthem_basic uses calculate_characteristics |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Morophon, the Boundless | Yes* | 0 | Yes* | *"Spells" vs "creature spells" filter gap (Finding 1) |
| Vanquisher's Banner | Yes | 0 | Yes | All three abilities correctly implemented |
| Patchwork Banner | Yes | 0 | Yes | Choose type + anthem + tap-for-any |
| Herald's Horn | Yes | 0 | Yes | Cost reduction + upkeep conditional draw |
| Kindred Dominance | Yes | 0 | Yes | Sequence(ChooseCreatureType, DestroyAll) |
| Three Tree City | Yes | 0 | Yes | AddManaOfAnyColorAmount + ChosenTypeCreatureCount |
| Etchings of the Chosen | Yes | 0 | Yes | Anthem + CreatureOfChosenType sac cost + indestructible |
| Pact of the Serpent | Yes | 0 | Yes | ChosenTypeCreatureCount for draw + life loss |
| Cavern of Souls | Partial | 1 (deferred) | Partial | "can't be countered" rider deferred -- documented |

## Test Coverage Assessment

12 tests covering:
- ETB type choice + ctx propagation (1 test)
- Anthem filters -- inclusive and exclusive (2 tests)
- Cost reduction -- colored and generic (2 def-check tests)
- Cast trigger filter definition (1 def-check test)
- DestroyAll exclude_chosen_subtype (2 tests -- direct + Sequence)
- TopCardIsCreatureOfChosenType true/false (2 tests)
- ChosenTypeCreatureCount via GainLife (1 integration test)
- Pact of the Serpent definition (1 def-check test)

Missing: No integration test for the actual casting cost reduction (only definition checks).
No integration test for SacrificeFilter::CreatureOfChosenType validation (Etchings ability).
No integration test for AddManaOfAnyColorAmount execution (Three Tree City).
These are test gaps but not blocking -- the engine dispatch code is exercised by the
ChosenTypeCreatureCount and DestroyAll integration tests, and the definition checks
confirm the card defs are wired correctly.
