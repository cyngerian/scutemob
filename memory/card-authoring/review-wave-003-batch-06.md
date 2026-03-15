# Card Review: Wave 3 Batch 06 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Ghost Quarter
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - Colorless mana ability correctly implemented. Second ability (sacrifice + destroy + opponent search) correctly left as TODO with accurate description of DSL gaps (sacrifice-as-cost, opponent-controlled search).

## Card 2: Gloomlake Verge
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - First ability ({T}: Add {U}) correctly implemented with `mana_pool(0, 1, 0, 0, 0, 0)` (blue in position 2). Second ability correctly left as TODO -- conditional activation ("only if you control an Island or a Swamp") is indeed not expressible in the DSL.

## Card 3: Graven Cairns
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - Colorless mana ability correctly implemented. Filter ability correctly left as TODO -- hybrid mana cost ({B/R}) and choice-of-output mana abilities are not in the DSL.

## Card 4: Grim Backwoods
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - Colorless mana ability correctly implemented. Second ability correctly left as TODO. The TODO description is slightly verbose but accurately identifies the DSL gap (sacrifice-as-cost with mana cost and tap).

## Card 5: Hall of Heliod's Generosity
- **Oracle match**: YES
- **Types match**: YES (Legendary Land, uses `supertypes(&[SuperType::Legendary], &[CardType::Land])`)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): The `card_id` uses `cid("hall-of-heliods-generosity")` which drops the apostrophe from "Heliod's". This is consistent with slug conventions (no special characters in IDs) and not a bug, but worth noting for card lookup purposes.
  - Colorless mana ability correctly implemented. Second ability correctly left as TODO -- graveyard targeting with return-to-library is not in the DSL.

## Summary
- Cards with issues: Hall of Heliod's Generosity (1 LOW -- cosmetic slug note only)
- Clean cards: Ghost Quarter, Gloomlake Verge, Graven Cairns, Grim Backwoods
- All 5 cards have correct oracle text, correct type lines, correct mana costs, and accurate TODO descriptions for unimplemented abilities. No KI pattern violations detected.
