# Card Review: Wave 3 Batch 10 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Rugged Prairie
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. TODO for hybrid mana cost ability ({R/W}, {T}: Add {R}{R}/{R}{W}/{W}{W}) is accurate -- hybrid mana in costs and triple-choice output are genuine DSL gaps.

## Card 2: Scavenger Grounds
- **Oracle match**: YES
- **Types match**: YES (Land -- Desert)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. TODO for sacrifice-a-Desert activated ability with exile-all-graveyards effect is accurate -- both the typed sacrifice cost and mass graveyard exile are DSL gaps.

## Card 3: Secluded Courtyard
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. Both TODOs are accurate: ETB creature type choice and mana spending restrictions are genuine DSL gaps. Good decision not to implement unrestricted AddManaAnyColor as that would produce incorrect gameplay (W5 policy: no approximate behavior).

## Card 4: Shivan Reef
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. TODO for painland ability is accurate -- the second ability needs a choice (U or R) combined with self-damage, which requires a Sequence/Choose pattern not currently in the DSL.

## Card 5: Shizo, Death's Storehouse
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {B} correctly implemented with mana_pool(0, 0, 1, 0, 0, 0) -- black is the third position, which is correct. TODO for fear-granting activated ability is accurate on both counts: Activated lacks a targets field, and Fear is not a KeywordAbility variant.

## Summary
- Cards with issues: (none)
- Clean cards: Rugged Prairie, Scavenger Grounds, Secluded Courtyard, Shivan Reef, Shizo Death's Storehouse
