# Card Review: Wave 2 Batch 6

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Tiamat
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (2WUBRG)
- **P/T match**: YES (7/7)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Flying keyword implemented. ETB search ability correctly omitted with accurate TODO describing all 4 DSL gaps (variable count, subtype filter, name exclusion, hand destination). `abilities: vec![]` would be more correct per W5 policy (no-ability cards should have empty vec), but having only `Flying` is valid since Flying IS expressible.

## Card 2: Archetype of Imagination
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (4UU)
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Both static abilities correctly omitted with accurate TODO. `abilities: vec![]` is correct here since neither ability (grant flying to your creatures, remove flying from opponents' creatures) is expressible in the DSL.

## Card 3: Roil Elemental
- **Oracle match**: YES (uses em dash U+2014 for Landfall separator, matches Scryfall)
- **Types match**: YES
- **Mana cost match**: YES (3UUU)
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Flying implemented. Landfall control-change ability correctly omitted with accurate TODO describing the missing duration variant.

## Card 4: Dragonlord Dromoka
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elder Dragon)
- **Mana cost match**: YES (4GW)
- **P/T match**: YES (5/7)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Flying and Lifelink keywords implemented. Both "can't be countered" and "opponents can't cast spells during your turn" correctly omitted with accurate TODOs describing DSL gaps.

## Card 5: Balefire Dragon
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (5RR)
- **P/T match**: YES (6/6)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): The oracle_text in the card def uses "this creature" which matches Scryfall's current templating. However, the TODO could be slightly more precise about the three DSL gaps. Current TODO is adequate.

## Summary
- Cards with issues: Balefire Dragon (1 LOW, cosmetic only)
- Clean cards: Tiamat, Archetype of Imagination, Roil Elemental, Dragonlord Dromoka
- All 5 cards have correct oracle text, mana costs, types, and P/T
- All DSL gap TODOs are accurate and well-documented
- No KI pattern violations detected
