# Card Review: Phase 1 Batch 08 — Dual Lands

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Golgari Guildgate
- **Oracle match**: YES
- **Types match**: YES — Land - Gate (Gate subtype present)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Enters tapped unconditionally (correct, no condition needed). Mana colors B=pos2, G=pos4 correct.

## Card 2: Rootbound Crag
- **Oracle match**: YES
- **Types match**: YES — Land (no subtypes, correct)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Conditional ETB ("unless you control a Mountain or a Forest") documented as TODO with DSL gap note. Mana colors R=pos3, G=pos4 correct.

## Card 3: Canopy Vista
- **Oracle match**: YES
- **Types match**: YES — Land - Forest Plains (basic land subtypes present)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Conditional ETB ("unless you control two or more basic lands") documented as TODO with DSL gap note. Mana colors G=pos4, W=pos0 correct. Intrinsic mana ability from Forest/Plains subtypes modeled as explicit Activated ability (functionally correct).

## Card 4: Cinder Glade
- **Oracle match**: YES
- **Types match**: YES — Land - Mountain Forest (basic land subtypes present)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Conditional ETB documented as TODO. Mana colors R=pos3, G=pos4 correct.

## Card 5: Prairie Stream
- **Oracle match**: YES
- **Types match**: YES — Land - Plains Island (basic land subtypes present)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Conditional ETB documented as TODO. Mana colors W=pos0, U=pos1 correct.

## Summary
- Cards with issues: (none)
- Clean cards: Golgari Guildgate, Rootbound Crag, Canopy Vista, Cinder Glade, Prairie Stream
- Note: All 4 conditional-ETB lands properly document the DSL gap with TODO comments. The unconditional enters-tapped on Golgari Guildgate is fully implemented.
