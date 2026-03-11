# Card Review: Phase 1 Batch 09 (Mana Rocks & Dual Lands)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Fire Diamond
- **Oracle match**: YES
- **Types match**: YES (Artifact, no subtypes)
- **Mana cost match**: YES ({2} generic)
- **DSL correctness**: YES
- **Findings**: None

Enters tapped replacement + tap for {R} mana ability. mana_pool(0,0,0,1,0,0) correctly produces red. Clean card.

## Card 2: Blooming Marsh
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None

Conditional ETB ("unless two or fewer other lands") documented with TODO -- excluded per review instructions. mana_pool values correct: B=(0,0,1,0,0,0), G=(0,0,0,0,1,0). Clean card.

## Card 3: Dreamroot Cascade
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None

Conditional ETB ("unless two or more other lands") documented with TODO -- excluded per review instructions. mana_pool values correct: G=(0,0,0,0,1,0), U=(0,1,0,0,0,0). Clean card.

## Card 4: Haunted Mire
- **Oracle match**: YES
- **Types match**: YES (Land -- Swamp Forest)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None

Enters tapped unconditionally. Subtypes "Swamp" and "Forest" correctly specified via `types_sub`. mana_pool values correct: B=(0,0,1,0,0,0), G=(0,0,0,0,1,0). The explicit mana ability is technically redundant with intrinsic basic land type mana abilities, but functionally correct for this engine. Clean card.

## Card 5: Shineshadow Snarl
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (None)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM/KI-9): Missing TODO for conditional ETB. The oracle text reads "As this land enters, you may reveal a Plains or Swamp card from your hand. If you don't, this land enters tapped." This is a conditional ETB (reveal-or-tapped pattern), but the comment says `// Enters tapped (CR 614.1c)` as if it were unconditional. The implementation always enters tapped, which is incorrect behavior. Should have a TODO comment documenting the DSL gap (conditional ETB with reveal check), consistent with how Blooming Marsh and Dreamroot Cascade document their conditional ETB gaps.

mana_pool values correct: W=(1,0,0,0,0,0), B=(0,0,1,0,0,0).

## Summary
- Cards with issues: Shineshadow Snarl (1 MEDIUM)
- Clean cards: Fire Diamond, Blooming Marsh, Dreamroot Cascade, Haunted Mire
