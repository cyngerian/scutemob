# Card Review: Phase 1 Batch 10 (Lands)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Frontier Bivouac
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

Mana colors verified: G=(0,0,0,0,1,0), U=(0,1,0,0,0,0), R=(0,0,0,1,0,0). Tri-land Choose with 3 correct choices.

## Card 2: Shipwreck Marsh
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

Mana colors verified: U=(0,1,0,0,0,0), B=(0,0,1,0,0,0). Conditional ETB "unless you control two or more other lands" has TODO documenting the DSL gap (not flagged per review instructions).

## Card 3: Savage Lands
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

Mana colors verified: B=(0,0,1,0,0,0), R=(0,0,0,1,0,0), G=(0,0,0,0,1,0). Tri-land Choose with 3 correct choices.

## Card 4: Jungle Shrine
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

Mana colors verified: R=(0,0,0,1,0,0), G=(0,0,0,0,1,0), W=(1,0,0,0,0,0). Tri-land Choose with 3 correct choices.

## Card 5: Foreboding Ruins
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES (with caveat below)
- **Findings**:
  - F1 (LOW/KI-9): Missing TODO for conditional ETB. The oracle text says "As this land enters, you may reveal a Swamp or Mountain card from your hand. If you don't, this land enters tapped." The implementation uses unconditional EntersTapped with the comment "Enters tapped (CR 614.1c)" which is misleading -- it should have a TODO documenting the reveal-or-tapped DSL gap, similar to how Shipwreck Marsh documents its conditional ETB. The card always enters tapped instead of offering the reveal choice.

## Summary
- Cards with issues: Foreboding Ruins (1 LOW)
- Clean cards: Frontier Bivouac, Shipwreck Marsh, Savage Lands, Jungle Shrine
