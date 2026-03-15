# Card Review: Wave 3 Batch 02 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Bleachbone Verge
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: First ability ({T}: Add {B}) correctly implemented. Second ability ({T}: Add {W} with Plains/Swamp condition) left as TODO — accurate, conditional activation based on controlling a land subtype is not in the DSL. The implemented {B} ability is the unconditional one, which is correct per oracle text. No placeholder/no-op issue (W5 policy: empty rather than approximate).

## Card 2: Blinkmoth Nexus
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented with mana_pool(0,0,0,0,0,1). Both animation and Blinkmoth pump abilities left as TODO with accurate descriptions. Land animation and creature-subtype-filtered targeted pump are genuine DSL gaps.

## Card 3: Boseiju, Who Endures
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Supertype Legendary correctly applied via `supertypes()`. {T}: Add {G} correctly implemented with mana_pool(0,0,0,0,1,0). Channel ability left as TODO with thorough description covering the discard-as-cost, variable mana cost (legendary creature scaling), and multi-type target filter gaps. All accurate DSL gaps. The Channel keyword itself is not added to keywords, which is acceptable since the ability is unimplemented (adding the keyword without the ability would be misleading).

## Card 4: Buried Ruin
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. The sacrifice-and-return ability is left as TODO with accurate DSL gap description (return_from_graveyard with type filter, sacrifice-self cost). Both are genuine gaps.

## Card 5: Cabal Coffers
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: All abilities left as `vec![]` per W5 policy since the only ability is the count-based mana production which is not expressible in the DSL. TODO comment accurately describes the gap (count controlled Swamps for scaling mana output). This is correct — implementing a partial or fixed-amount version would be wrong behavior.

## Summary
- Cards with issues: (none)
- Clean cards: Bleachbone Verge, Blinkmoth Nexus, Boseiju Who Endures, Buried Ruin, Cabal Coffers
- All 5 cards have correct oracle text, type lines, and mana costs matching Scryfall
- All TODO comments accurately describe genuine DSL gaps
- No KI pattern violations detected
- No placeholder/no-op approximations (W5 policy compliant)
