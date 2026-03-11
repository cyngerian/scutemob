# Card Review: Phase 1 Batch 03 (ETB Tapped Dual/Multiplayer Lands)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 0 LOW

## Card 1: Morphic Pool
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Replacement effect `EntersTapped` is unconditional -- always enters tapped. Oracle says "unless you have two or more opponents." The DSL lacks a conditional replacement that checks opponent count. Should have a TODO comment documenting this gap. In Commander (4-player), this land almost always enters untapped, so the current behavior is the opposite of the common case.

## Card 2: Sea of Clouds
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Same unconditional `EntersTapped` issue as Morphic Pool. Oracle says "unless you have two or more opponents." No TODO documenting the gap. Land always enters tapped instead of almost always entering untapped in multiplayer.

## Card 3: Bountiful Promenade
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Same unconditional `EntersTapped` issue. Oracle says "unless you have two or more opponents." No TODO documenting the gap.

## Card 4: Spire Garden
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Same unconditional `EntersTapped` issue. Oracle says "unless you have two or more opponents." No TODO documenting the gap.

## Card 5: Drowned Catacomb
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Replacement effect `EntersTapped` is unconditional -- always enters tapped. Oracle says "unless you control an Island or a Swamp." The DSL lacks a conditional replacement that checks for controlled land subtypes. Should have a TODO comment documenting this gap. The check-land cycle (Drowned Catacomb, Glacial Fortress, etc.) should almost never always-enter-tapped; this is functionally wrong in most games.

## Summary

- **Cards with issues**: Morphic Pool, Sea of Clouds, Bountiful Promenade, Spire Garden, Drowned Catacomb (all 5)
- **Clean cards**: (none)

### Common Issue: Unconditional EntersTapped (5 cards)

All 5 cards use `ReplacementModification::EntersTapped` unconditionally, but each has a condition under which it should enter untapped:

- **Battlebond lands** (Morphic Pool, Sea of Clouds, Bountiful Promenade, Spire Garden): "unless you have two or more opponents" -- in Commander, this is almost always true, so these lands should almost always enter *untapped*. The current implementation makes them always enter tapped, which is the opposite of their intended behavior in the target format.

- **Check land** (Drowned Catacomb): "unless you control an Island or a Swamp" -- requires checking controller's battlefield for land subtypes.

**DSL gap**: `ReplacementModification::EntersTapped` has no conditional variant. Two new conditions are needed:
1. `Condition::HasTwoOrMoreOpponents` (for Battlebond cycle)
2. `Condition::ControlsLandWithSubtype(Vec<SubType>)` (for check-land cycle)

**Recommendation**: Per W5 policy, cards with wrong/approximate behavior corrupt game state. These should either:
- (a) Have `abilities: vec![]` with a TODO explaining the DSL gap (prevents incorrect gameplay), or
- (b) Have a TODO comment on the replacement effect clearly documenting that the condition is missing and the land always enters tapped (current approach, but undocumented).

Option (b) is acceptable if the project tolerates "always tapped" as a conservative approximation. Option (a) is stricter but removes the mana ability entirely. Given that these are lands whose primary purpose is mana production, option (b) with clear TODO comments is the pragmatic choice -- the mana ability itself is correct, only the ETB condition is missing.
