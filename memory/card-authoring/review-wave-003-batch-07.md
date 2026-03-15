# Card Review: Wave 3 Batch 07 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Hanweir Battlements
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
  - Colorless mana tap ability correctly implemented.
  - TODO for haste-granting activated ability accurately describes DSL gap (no targets field on Activated).
  - TODO for meld ability accurately describes DSL gap (Meld mechanic not implemented).

## Card 2: Haven of the Spirit Dragon
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
  - Colorless mana tap ability correctly implemented.
  - TODO for restricted any-color mana accurately describes DSL gap (mana spending restrictions).
  - TODO for return-from-graveyard ability accurately describes DSL gaps (return_from_graveyard with name filter, sacrifice-self cost).

## Card 3: High Market
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
  - Colorless mana tap ability correctly implemented.
  - TODO for sacrifice-a-creature-to-gain-life ability accurately describes DSL gap (Cost::SacrificeCreature).

## Card 4: Inkmoth Nexus
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
  - Colorless mana tap ability correctly implemented.
  - TODO for land animation ability accurately describes DSL gap (land animation effect adding types/subtypes/P&T/keywords until end of turn).

## Card 5: Inventors' Fair
- **Oracle match**: YES
- **Types match**: YES (Legendary Land, uses `supertypes(&[SuperType::Legendary], &[CardType::Land])`)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
  - Colorless mana tap ability correctly implemented.
  - Legendary supertype correctly set.
  - TODO for upkeep trigger accurately describes DSL gap (intervening_if with count_threshold for artifacts).
  - TODO for sacrifice-to-tutor ability accurately describes DSL gaps (sacrifice-self cost, artifact-typed search, conditional activation).

## Summary
- Cards with issues: (none)
- Clean cards: Hanweir Battlements, Haven of the Spirit Dragon, High Market, Inkmoth Nexus, Inventors' Fair

All 5 cards have correct oracle text, type lines, and mana costs. Each card correctly implements the {T}: Add {C} mana ability with proper colorless mana pool values `mana_pool(0, 0, 0, 0, 0, 1)`. All TODO comments accurately describe genuine DSL gaps and do not hide correctness bugs. No known-issue patterns (KI-1 through KI-10) are present in any of these definitions.
