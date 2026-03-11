# Card Review: Phase 1 Batch 04 (Lands - Conditional/Unconditional ETB Tapped)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 3 MEDIUM, 0 LOW

## Card 1: Smoldering Marsh
- **Oracle match**: YES
- **Types match**: YES (Land -- Swamp Mountain)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Unconditional `EntersTapped` replacement does not model "unless you control two or more basic lands." The card should enter untapped when the controller has 2+ basic lands. DSL lacks conditional ETB replacement. TODO comment should document this gap -- currently just says "CR 614.1c" with no mention of the condition being dropped.

## Card 2: Shattered Sanctum
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F2 (MEDIUM): Same as F1. Unconditional `EntersTapped` does not model "unless you control two or more other lands." TODO should document the missing condition.

## Card 3: Sundown Pass
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F3 (MEDIUM): Same as F1/F2. Unconditional `EntersTapped` does not model "unless you control two or more other lands." TODO should document the missing condition.

## Card 4: Nomad Outpost
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Unconditional ETB tapped is correct for this card. Mana choices (R/W/B) map to correct positions: R=pos3, W=pos0, B=pos2.

## Card 5: Seaside Citadel
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Unconditional ETB tapped is correct for this card. Mana choices (G/W/U) map to correct positions: G=pos4, W=pos0, U=pos1.

## Summary
- Cards with issues: Smoldering Marsh, Shattered Sanctum, Sundown Pass (all same issue: conditional ETB tapped modeled as unconditional)
- Clean cards: Nomad Outpost, Seaside Citadel

### Notes on Conditional ETB Lands
Smoldering Marsh, Shattered Sanctum, and Sundown Pass all have conditional enter-tapped clauses ("unless you control two or more basic/other lands"). The DSL's `ReplacementModification::EntersTapped` has no condition field, so these cards always enter tapped in the engine. This is a known DSL gap (`shock_etb` pattern from the authoring worklist). The card definitions should add a TODO comment explicitly noting that the condition is not enforced. Per W5 policy, the unconditional replacement is acceptable as a conservative approximation (always tapped is strictly worse for the controller, never produces incorrect game-winning state), but the missing TODO documentation makes it look like the implementation is complete when it is not.
