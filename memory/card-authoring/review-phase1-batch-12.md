# Card Review: Phase 1 Batch 12 (Lands)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Darkslick Shores
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES (known TODO for conditional ETB)
- **Findings**: None (conditional ETB "unless" is a documented DSL gap with TODO — not flagged per instructions)

## Card 2: Dryad Arbor
- **Oracle match**: YES
- **Types match**: YES — `types_sub(&[CardType::Land, CardType::Creature], &["Forest", "Dryad"])` correctly encodes "Land Creature -- Forest Dryad"
- **Mana cost match**: YES (None — no mana cost)
- **P/T match**: YES (1/1)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Missing color indicator. Dryad Arbor is green via color indicator (CR 204). Since it has no mana cost, the engine's `colors_from_mana_cost()` will treat it as colorless. `CardDefinition` lacks a `color_indicator` field (only `CardFace` has one, for DFC back faces). This is a DSL gap that needs a struct addition. **Action**: Add TODO comment to the card definition noting the missing green color indicator, and file the DSL gap for adding `color_indicator: Option<Vec<Color>>` to `CardDefinition`.
  - F2 (MEDIUM): Redundant explicit mana ability. The "Forest" subtype intrinsically grants "{T}: Add {G}" (CR 305.6). The explicit `Activated` ability is not incorrect but is redundant — the engine should grant this from the basic land subtype. If the engine does NOT currently derive mana abilities from basic land subtypes, then this explicit ability is necessary and correct. Verify engine behavior. If derived, remove to avoid double-mana bugs.

## Card 3: Bayou
- **Oracle match**: YES
- **Types match**: YES — `types_sub(&[CardType::Land], &["Swamp", "Forest"])` correctly encodes "Land -- Swamp Forest"
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Mana pool args**: Correct. {B} = `mana_pool(0, 0, 1, 0, 0, 0)`, {G} = `mana_pool(0, 0, 0, 0, 1, 0)` (W=0, U=1, B=2, R=3, G=4, C=5)
- **Findings**: None

## Card 4: Badlands
- **Oracle match**: YES
- **Types match**: YES — `types_sub(&[CardType::Land], &["Swamp", "Mountain"])` correctly encodes "Land -- Swamp Mountain"
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Mana pool args**: Correct. {B} = `mana_pool(0, 0, 1, 0, 0, 0)`, {R} = `mana_pool(0, 0, 0, 1, 0, 0)`
- **Findings**: None

## Card 5: Tropical Island
- **Oracle match**: YES
- **Types match**: YES — `types_sub(&[CardType::Land], &["Forest", "Island"])` correctly encodes "Land -- Forest Island"
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Mana pool args**: Correct. {G} = `mana_pool(0, 0, 0, 0, 1, 0)`, {U} = `mana_pool(0, 1, 0, 0, 0, 0)`
- **Findings**: None

## Summary
- Cards with issues: Dryad Arbor (1 HIGH: missing color indicator DSL gap, 1 MEDIUM: potentially redundant Forest mana ability)
- Clean cards: Darkslick Shores, Bayou, Badlands, Tropical Island
