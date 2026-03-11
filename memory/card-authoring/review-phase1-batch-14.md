# Card Review: Phase 1 Batch 14 -- Mana Lands

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 1 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Snow-Covered Swamp
- **Oracle match**: YES
- **Types match**: NO
- **Mana cost match**: YES (none)
- **DSL correctness**: YES (mana ability correct)
- **Findings**:
  - F1 (HIGH): Missing supertypes Basic and Snow. Card uses `types_sub(&[CardType::Land], &["Swamp"])` but should use `full_types(&[SuperType::Basic, SuperType::Snow], &[CardType::Land], &["Swamp"])`. Scryfall type line is "Basic Snow Land -- Swamp". Both `SuperType::Basic` and `SuperType::Snow` exist in the DSL. Compare with `forest.rs` which correctly uses `full_types(&[SuperType::Basic], &[CardType::Land], &["Forest"])` -- Snow-Covered Swamp needs both Basic and Snow.

## Card 2: Volcanic Island
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

## Card 3: Plateau
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

## Card 4: Scrubland
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

## Card 5: Underground Sea
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

## Summary
- Cards with issues: Snow-Covered Swamp (missing Basic + Snow supertypes)
- Clean cards: Volcanic Island, Plateau, Scrubland, Underground Sea

### Notes
- All four dual lands correctly use `Effect::Choose` with two `AddMana` options for their two-color mana abilities.
- All `mana_pool()` argument positions verified correct (white=0, blue=1, black=2, red=3, green=4, colorless=5).
- Snow-Covered Island (`snow_covered_island.rs`) was not in this review batch but appears to have the same missing-supertypes issue (uses `types_sub` instead of `full_types` with Basic + Snow). Flagging for awareness.
