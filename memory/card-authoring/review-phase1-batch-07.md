# Card Review: Phase 1 Batch 07 (Dual/Tri Lands)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Gilt-Leaf Palace
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none -- Land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Missing TODO for reveal-Elf conditional ETB. The card says "As this land enters, you may reveal an Elf card from your hand. If you don't, this land enters tapped." This is NOT the same as the "unless you control N lands" pattern -- it requires revealing a card of a specific creature type. The current implementation uses unconditional `EntersTapped` but has no TODO comment documenting the DSL gap. The three "unless" lands correctly have TODO comments; this card should too. Suggested fix: add `// TODO: Conditional ETB -- reveal an Elf card or enters tapped. DSL gap: no reveal-from-hand condition on ReplacementModification::EntersTapped`.

## Card 2: Arcane Sanctum
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none -- Land)
- **DSL correctness**: YES
- **Findings**: None. Unconditional enters-tapped is correctly modeled. Three mana choices (W/U/B) are correct: `mana_pool(1,0,0,0,0,0)`, `mana_pool(0,1,0,0,0,0)`, `mana_pool(0,0,1,0,0,0)`.

## Card 3: Stormcarved Coast
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none -- Land)
- **DSL correctness**: YES
- **Findings**: None. Conditional ETB has proper TODO documenting the DSL gap. Mana colors correct: U=`mana_pool(0,1,0,0,0,0)`, R=`mana_pool(0,0,0,1,0,0)`.

## Card 4: Concealed Courtyard
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none -- Land)
- **DSL correctness**: YES
- **Findings**: None. Conditional ETB has proper TODO documenting the DSL gap. Mana colors correct: W=`mana_pool(1,0,0,0,0,0)`, B=`mana_pool(0,0,1,0,0,0)`.

## Card 5: Deathcap Glade
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none -- Land)
- **DSL correctness**: YES
- **Findings**: None. Conditional ETB has proper TODO documenting the DSL gap. Mana colors correct: B=`mana_pool(0,0,1,0,0,0)`, G=`mana_pool(0,0,0,0,1,0)`.

## Summary
- Cards with issues: Gilt-Leaf Palace (1 MEDIUM -- missing TODO for reveal-Elf condition)
- Clean cards: Arcane Sanctum, Stormcarved Coast, Concealed Courtyard, Deathcap Glade
