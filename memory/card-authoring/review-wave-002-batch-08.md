# Card Review: Wave 002, Batch 08

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 2 LOW

## Card 1: Ancient Brass Dragon
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic 5, black 2)
- **P/T match**: YES (7/6)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword implemented. Combat damage d20 trigger correctly identified as DSL gap (dice rolls not supported). Abilities left as `vec![]` with TODO per W5 policy.

## Card 2: Strix Serenade
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES (blue 1)
- **DSL correctness**: YES
- **Findings**: None. Correctly identifies two DSL gaps: multi-type spell target filter (artifact/creature/planeswalker) and "its controller creates a token" (opponent gets the token, not the caster). Abilities left as `vec![]` with TODO per W5 policy.

## Card 3: Fervor
- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES (generic 2, red 1)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): TODO comment says "Only EffectFilter::AllCreatures is available" but `EffectFilter::CreaturesControlledBy(PlayerId)` also exists. However, since CardDefinition is authored at definition time and the PlayerId is not known until runtime, the conclusion that this can't be expressed in a static card def is still correct. The TODO is slightly inaccurate in its enumeration of available filters but reaches the right conclusion.

## Card 4: Aven Mindcensor
- **Oracle match**: YES
- **Types match**: YES (Bird Wizard creature)
- **Mana cost match**: YES (generic 2, white 1)
- **P/T match**: YES (2/1)
- **DSL correctness**: YES
- **Findings**: None. Flash and Flying keywords implemented. Library search restriction replacement effect correctly identified as DSL gap. Abilities include both keywords with TODO for the replacement effect.

## Card 5: Ancient Silver Dragon
- **Oracle match**: YES
- **Types match**: YES (Elder Dragon creature)
- **Mana cost match**: YES (generic 6, blue 2)
- **P/T match**: YES (8/8)
- **DSL correctness**: YES
- **Findings**:
  - F2 (LOW): TODO comment mentions "d20 roll + variable card draw + no hand size limit" as a single DSL gap. These are actually three separate gaps (dice rolls, variable draw amount, continuous "no maximum hand size" effect). Grouping them is fine for a TODO but slightly imprecise.

## Summary
- Cards with issues: Fervor (1 LOW), Ancient Silver Dragon (1 LOW)
- Clean cards: Ancient Brass Dragon, Strix Serenade, Aven Mindcensor
