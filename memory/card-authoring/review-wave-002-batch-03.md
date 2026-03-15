# Card Review: Wave 002 Batch 03

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Bloodghast
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Spirit)
- **Mana cost match**: YES ({B}{B})
- **P/T match**: YES (2/1)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: `abilities: vec![]` is correct per W5 policy. Three DSL gaps documented in TODO: (1) "can't block" static ability, (2) conditional haste based on opponent life total threshold, (3) Landfall return-from-graveyard trigger. All three are accurate descriptions of gaps.

## Card 2: Twinflame Tyrant
- **Oracle match**: YES
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES ({3}{R}{R})
- **P/T match**: YES (3/5)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Flying keyword correctly implemented. Damage doubling replacement effect correctly identified as a DSL gap in TODO. The TODO accurately describes the missing primitive (replacement effect for damage doubling to opponents and their permanents).

## Card 3: Perennial Behemoth
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature -- Beast)
- **Mana cost match**: YES ({5})
- **P/T match**: YES (2/7)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: `KeywordAbility::Unearth` is present as a marker keyword. Two DSL gaps documented: (1) "play lands from graveyard" zone-permission static, (2) Unearth activation from graveyard zone. Both are accurate. The Unearth keyword marker allows the engine to recognize the card has Unearth even if the full activation is not yet wired.

## Card 4: Devilish Valet
- **Oracle match**: YES
- **Types match**: YES (Creature -- Devil Warrior)
- **Mana cost match**: YES ({2}{R})
- **P/T match**: YES (1/3)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Trample and Haste keywords correctly implemented. Alliance trigger ("double this creature's power until end of turn") correctly identified as a DSL gap requiring multiplicative power modification (LayerModification). TODO is accurate.

## Card 5: Ancient Greenwarden
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elemental; uses `full_types` with `SuperType::Legendary`)
- **Mana cost match**: YES ({4}{G}{G})
- **P/T match**: YES (5/7)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Reach keyword correctly implemented. Two DSL gaps documented: (1) "play lands from graveyard" zone-permission static (same gap as Perennial Behemoth), (2) land-ETB trigger doubling continuous effect. Both TODOs are accurate and well-described.

## Summary
- Cards with issues: (none)
- Clean cards: Bloodghast, Twinflame Tyrant, Perennial Behemoth, Devilish Valet, Ancient Greenwarden

All 5 cards are correctly authored. Oracle text matches Scryfall exactly. Mana costs, types, subtypes, supertypes, and P/T values are all correct. Implementable keywords (Flying, Reach, Trample, Haste, Unearth) are properly included. Unimplementable abilities correctly use `vec![]` or omit the triggered/static abilities with accurate TODO comments describing the DSL gaps. No KI pattern violations detected.
