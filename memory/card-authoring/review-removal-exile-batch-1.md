# Card Review: Removal/Exile Batch 1

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Despark
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (W=1, B=1)
- **DSL correctness**: YES
- **Findings**: None. Target filter uses `min_cmc: Some(4)` which correctly models "mana value 4 or greater." Clean.

## Card 2: Deadly Rollick
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic=3, B=1)
- **DSL correctness**: YES (for the implemented portion)
- **Findings**:
  - F1 (LOW): TODO claims DSL gap for "if you control a commander, may cast without paying mana cost." This is a genuine DSL gap -- there is no `AltCostKind` variant for commander-conditional free casting. The TODO is valid. The core exile effect is correctly implemented and the card is still castable for {3}{B}, so the missing alt cost does not produce wrong game state (the card is just slightly worse than it should be -- you always pay full price). This is acceptable under current policy since the partial impl does not create incorrect game behavior; the player simply cannot access the discount.

## Card 3: Utter End
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic=2, W=1, B=1)
- **DSL correctness**: YES
- **Findings**: None. Uses `TargetPermanentWithFilter(TargetFilter { non_land: true, .. })` correctly for "nonland permanent." Clean.

## Card 4: Resculpt
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic=1, U=1)
- **DSL correctness**: N/A (abilities empty)
- **Findings**:
  - F1 (MEDIUM): The TODO states that `Effect::CreateToken` always creates for the spell controller, so targeting an opponent's permanent gives the token to the wrong player. This is a genuine DSL gap -- `CreateToken { spec }` has no `player` or `controller` field to redirect token creation to the target's controller. The `abilities: vec![]` placeholder is correct per W5 policy since partial implementation (exile without token, or token to wrong player) would produce wrong game state. However, the target filter is also missing -- the card needs `TargetRequirement::TargetPermanentWithFilter(TargetFilter { card_types: vec![CardType::Artifact, CardType::Creature], .. })` for "artifact or creature." Since abilities are empty anyway, this is moot but worth noting for when the DSL gap is closed.

## Card 5: Ephemerate
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (W=1)
- **DSL correctness**: YES (for the implemented portion)
- **Findings**:
  - F1 (LOW): TODO claims Rebound is not in the DSL. This is a genuine DSL gap -- `KeywordAbility::Rebound` does not exist in the enum (confirmed via grep). The TODO is valid. The flicker effect is correctly implemented: uses `Effect::Flicker` with `return_tapped: false`, targets `TargetCreatureWithFilter(TargetFilter { controller: TargetController::You, .. })` for "creature you control", and the engine's Flicker implementation already returns under owner's control (matching oracle). The missing Rebound means the card works once but does not get its free recast next upkeep -- this is a missing upside, not wrong game state, so partial implementation is acceptable.

## Summary
- **Cards with issues**: Deadly Rollick (1 LOW), Resculpt (1 MEDIUM), Ephemerate (1 LOW)
- **Clean cards**: Despark, Utter End
- **Notes**:
  - Two genuine DSL gaps confirmed: commander-conditional free cast (Deadly Rollick, Flawless Maneuver cycle), Rebound keyword (Ephemerate). Neither is in the Now-Expressible Patterns list.
  - Resculpt's W5-policy empty abilities are correct -- `CreateToken` lacks a player target field. This gap also affects Pongify, Rapid Hybridization, Stroke of Midnight, and Beast Within.
