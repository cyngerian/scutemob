# Card Review: Wave 3 Batch 11 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Slayers' Stronghold
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Colorless mana ability is correct. The pump/keyword activated ability is correctly left as TODO -- it requires a targeted activated ability with temporary P/T buff and keyword grants, which the DSL cannot express. TODO accurately describes the gap.

## Card 2: Sokenzan, Crucible of Defiance
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Red mana ability is correct (mana_pool position 4 = red). Channel ability correctly left as TODO -- requires discard-from-hand activation cost, token creation with temporary haste, and variable cost reduction, none of which are expressible together. TODO accurately describes the gaps.

## Card 3: Strip Mine
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Colorless mana ability is correct. The sacrifice-to-destroy-land ability is correctly left as TODO -- requires Cost::SacrificeSelf and targeted land destruction. TODO accurately describes the gap.

## Card 4: Sulfurous Springs
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): **Partial implementation violates W5 policy.** The second activated ability implements `Effect::Choose` for adding {B} or {R} but omits the mandatory "This land deals 1 damage to you" side effect. This makes the colored mana free when it should cost 1 life, which is strictly better than the real card and corrupts game state. Per W5 policy ("no simplifications -- wrong/approximate behavior corrupts game state"), the second ability should be removed from the `abilities` vec and left as a TODO-only comment until the DSL can express mana-ability-with-damage-side-effect. The colorless mana ability (first ability) is correct and should remain.

## Card 5: Sunken Ruins
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Colorless mana ability is correct. The filter ability is correctly left as TODO -- requires hybrid mana cost ({U/B}) and a three-way mana output choice, neither of which is expressible. TODO accurately describes the gap.

## Summary
- Cards with issues: Sulfurous Springs (1 MEDIUM -- partial painland implementation gives free colored mana)
- Clean cards: Slayers' Stronghold, Sokenzan Crucible of Defiance, Strip Mine, Sunken Ruins
