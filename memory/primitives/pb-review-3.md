# Primitive Batch Review: PB-3 -- Shockland ETB (pay-or-tapped)

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 614.1c (replacement effects for "As this enters"), 305.7 (basic land type mana abilities)
**Engine files reviewed**: `replacement_effect.rs:107`, `replacement.rs:1530-1555`, `hash.rs:1572-1575`
**Card defs reviewed**: 10 (blood_crypt, breeding_pool, godless_shrine, hallowed_fountain, overgrown_tomb, sacred_foundry, steam_vents, stomping_ground, temple_garden, watery_grave)

## Verdict: clean

All 10 shockland card definitions correctly implement their oracle text. The
`ReplacementModification::EntersTappedUnlessPayLife(u32)` engine primitive is correctly
defined, hashed, and dispatched. The deterministic fallback (always enters tapped) is a
documented pre-M10 simplification that is conservative (prevents free mana). Land subtypes,
mana abilities, and oracle text all match Scryfall oracle text exactly. No TODOs remain.
Three tests cover positive behavior, all-card integration, and variant discrimination.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|

No findings.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|

No findings.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 614.1c  | Yes         | Yes     | test_shockland_enters_tapped_deterministic_fallback, test_all_shocklands_enter_tapped |
| 305.7   | Yes (intrinsic mana abilities modeled as Activated) | Implicit (mana abilities present in card defs) | Consistent with basic land pattern (forest.rs) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Blood Crypt | Yes | 0 | Yes | Land -- Swamp Mountain; {B} or {R} |
| Breeding Pool | Yes | 0 | Yes | Land -- Forest Island; {G} or {U} |
| Godless Shrine | Yes | 0 | Yes | Land -- Plains Swamp; {W} or {B} |
| Hallowed Fountain | Yes | 0 | Yes | Land -- Plains Island; {W} or {U} |
| Overgrown Tomb | Yes | 0 | Yes | Land -- Swamp Forest; {B} or {G} |
| Sacred Foundry | Yes | 0 | Yes | Land -- Mountain Plains; {R} or {W} |
| Steam Vents | Yes | 0 | Yes | Land -- Island Mountain; {U} or {R} |
| Stomping Ground | Yes | 0 | Yes | Land -- Mountain Forest; {R} or {G} |
| Temple Garden | Yes | 0 | Yes | Land -- Forest Plains; {G} or {W} |
| Watery Grave | Yes | 0 | Yes | Land -- Island Swamp; {U} or {B} |

## Test Coverage

Three tests in `crates/engine/tests/replacement_effects.rs`:

1. **test_shockland_enters_tapped_deterministic_fallback** -- Blood Crypt enters tapped, PermanentTapped event emitted
2. **test_all_shocklands_enter_tapped** -- All 10 shocklands enter tapped in deterministic mode
3. **test_shockland_uses_pay_life_variant_not_enters_tapped** -- Verifies Blood Crypt uses `EntersTappedUnlessPayLife(2)`, not `EntersTapped`

No negative-case test gap: the deterministic fallback always produces the same result (enters tapped), so there is no "doesn't apply" case to test. A future M10 test should verify the interactive pay-life path when implemented.
