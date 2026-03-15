# Card Review: Wave 3 Batch 03 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 2 MEDIUM, 0 LOW

## Card 1: Cabal Stronghold
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: YES
- **Findings**:
  - No issues. The {T}: Add {C} ability is correctly implemented. The count-based mana ability ({3},{T}: Add {B} per basic Swamp) is correctly left as a TODO -- count-based mana generation is not in the DSL. TODO is accurate.

## Card 2: Cascade Bluffs
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: YES
- **Findings**:
  - No issues. The {T}: Add {C} ability is correctly implemented. The hybrid-cost filter ability is correctly left as a TODO -- hybrid mana costs and multi-choice mana output are not in the DSL. TODO is accurate.

## Card 3: Cavern of Souls
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): The `{T}: Add {C}` mana ability is expressible in the DSL but is missing from the abilities vec. The ETB choice and conditional mana abilities are correctly left as TODOs, but the basic colorless mana tap should be implemented. Currently `abilities: vec![]` makes the card produce no mana at all. Should include the colorless tap ability like all other lands in this batch.

## Card 4: Caves of Koilos
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: NO
- **Findings**:
  - F2 (MEDIUM): The second ability ({T}: Add {W} or {B}) is implemented without the "This land deals 1 damage to you" drawback. Per W5 policy, a card should not have wrong/approximate behavior -- producing {W} or {B} for free (no damage) is strictly better than the actual card. The mana choice part alone without the self-damage violates the W5 rule against partial implementations that alter game correctness. The second ability should either be removed (left as TODO) or the self-damage should be added. The TODO comment accurately describes the gap but the partial implementation creates incorrect game state.

## Card 5: Command Beacon
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: YES
- **Findings**:
  - F3 (MEDIUM): The sacrifice ability is correctly left as a TODO. However, the TODO states "sacrifice cost + command zone targeting not expressible in DSL." Sacrifice as a cost IS available in the DSL (`Cost::SacrificeThis` or similar). The real gap is the "put your commander into your hand from the command zone" effect (zone-specific move from command zone to hand). The TODO description slightly misidentifies the blocker -- worth correcting to avoid confusion in future DSL expansion work.

## Summary
- Cards with issues: Cavern of Souls (HIGH -- missing implementable colorless mana ability), Caves of Koilos (MEDIUM -- painland damage omitted from partial implementation), Command Beacon (MEDIUM -- slightly inaccurate TODO description)
- Clean cards: Cabal Stronghold, Cascade Bluffs
