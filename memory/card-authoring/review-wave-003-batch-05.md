# Card Review: Wave 3 Batch 05 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Flooded Grove
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Colorless tap ability correctly implemented. TODO accurately describes the hybrid mana cost + 3-way choice gap. Per W5 policy, leaving the second ability unimplemented with TODO is correct.

## Card 2: Forbidden Orchard
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Mana ability correctly uses `AddManaAnyColor`. TODO accurately describes two DSL gaps: mana-trigger and targeted_trigger (target opponent). Note: the triggered ability is technically a triggered mana ability under CR 605.1b (triggers from activating a mana ability), which does not use the stack -- this subtlety is not mentioned in the TODO but is not a correctness issue for the implemented portion.

## Card 3: Gaea's Cradle
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Correctly has empty abilities with accurate TODO describing the variable mana production gap. Per W5 policy, `abilities: vec![]` is correct when the only ability cannot be expressed.

## Card 4: Geier Reach Sanitarium
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**:
  - F1 (MEDIUM): Uses `full_types(&[SuperType::Legendary], &[CardType::Land], &[])` while Gaea's Cradle and Gemstone Caverns use `supertypes(&[SuperType::Legendary], &[CardType::Land])`. Both are functionally equivalent (empty subtypes), but inconsistent style across the batch. Not a correctness bug -- `full_types` with empty subtypes slice produces the same result as `supertypes`.

## Card 5: Gemstone Caverns
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. Both abilities are complex DSL gaps (opening-hand replacement effect, conditional mana based on counter). TODOs accurately describe the gaps. Empty abilities is correct per W5 policy -- implementing just the colorless tap without the luck-counter conditional would produce wrong behavior (always colorless, never any-color), which is arguably acceptable as a partial implementation, but the opening-hand leyline-style ability is the bigger gap and leaving both out is the safer choice.

## Summary
- Cards with issues: Geier Reach Sanitarium (1 MEDIUM -- style inconsistency only)
- Clean cards: Flooded Grove, Forbidden Orchard, Gaea's Cradle, Gemstone Caverns
