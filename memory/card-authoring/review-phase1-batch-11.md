# Card Review: Phase 1 Batch 11 (Lands -- Tri-land, Snarls, Taplands, Hand-lands)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Mystic Monastery
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Tri-land with 3-color Choose (U/R/W). All three mana_pool calls correct: U=`(0,1,0,0,0,0)`, R=`(0,0,0,1,0,0)`, W=`(1,0,0,0,0,0)`. Enters-tapped replacement correct.

## Card 2: Furycalm Snarl
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES (conditional ETB "unless" is known DSL gap, not flagged per policy)
- **Findings**: None
- **Notes**: Mana_pool calls correct: R=`(0,0,0,1,0,0)`, W=`(1,0,0,0,0,0)`. Always-tapped is the documented simplification for conditional ETB lands.

## Card 3: Foul Orchard
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Mana_pool calls correct: B=`(0,0,1,0,0,0)`, G=`(0,0,0,0,1,0)`. Enters-tapped replacement correct.

## Card 4: Choked Estuary
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES (conditional ETB "unless" is known DSL gap, not flagged per policy)
- **Findings**: None
- **Notes**: Mana_pool calls correct: U=`(0,1,0,0,0,0)`, B=`(0,0,1,0,0,0)`. Always-tapped is the documented simplification for conditional ETB lands.

## Card 5: Frostboil Snarl
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES (conditional ETB "unless" is known DSL gap, not flagged per policy)
- **Findings**: None
- **Notes**: Mana_pool calls correct: U=`(0,1,0,0,0,0)`, R=`(0,0,0,1,0,0)`. Always-tapped is the documented simplification for conditional ETB lands.

## Summary
- Cards with issues: (none)
- Clean cards: Mystic Monastery, Furycalm Snarl, Foul Orchard, Choked Estuary, Frostboil Snarl

All 5 cards are clean. Oracle text matches Scryfall exactly. Mana_pool argument order verified correct for all color combinations (W=0, U=1, B=2, R=3, G=4, C=5). Types, subtypes, and mana costs all correct. No P/T fields present (correct for lands). Enters-tapped replacement pattern is consistent across all cards. The 3 conditional ETB lands (Furycalm Snarl, Choked Estuary, Frostboil Snarl) use the always-tapped simplification, which is a known DSL gap excluded from this review per policy.
