# Card Review: Wave 3 Batch 01 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 2 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Abstergo Entertainment
- **Oracle match**: YES
- **Types match**: YES (Legendary Land with SuperType::Legendary)
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Two mana abilities correctly implemented ({T}: Add {C} and {1},{T}: Add any color). Third ability (exile-self, return historic card, exile all graveyards) correctly left as TODO with accurate DSL gap description. Clean card.

## Card 2: Access Tunnel
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. Second ability ({3},{T}: unblockable for power<=3 creature) correctly left as TODO. DSL gap description is accurate: Activated has no targets field, and "can't be blocked" as a granted effect is not expressible. Clean card.

## Card 3: Arch of Orazca
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. Ascend and the conditional draw ability correctly left as TODOs. DSL gap description is accurate: city's blessing (designation) tracking exists (Designations bitfield from type consolidation) but conditional activation restriction on Activated abilities is not wired up. Clean card.

## Card 4: Battlefield Forge
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): **W5 policy violation -- partial ability produces wrong game state.** The second ability implements `Effect::Choose` between {R} and {W} mana but omits the "This land deals 1 damage to you" self-damage clause. This means the controller gets colored mana for free when they should take 1 damage. Per W5 policy ("wrong/approximate behavior corrupts game state"), the second ability should be removed and left as a TODO-only comment, or the entire `abilities` vec should only contain the colorless tap. The colorless {C} ability is correct and can stay; only the pain-mana ability is problematic.

## Card 5: Blazemire Verge
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: NO
- **Findings**:
  - F2 (HIGH): **W5 policy violation -- missing activation restriction produces wrong game state.** The second ability adds {R} unconditionally, but the oracle text requires "Activate only if you control a Swamp or a Mountain." Without this restriction, the card produces {R} in situations where it legally cannot. Per W5 policy, this ability should be removed and left as a TODO-only comment. The first ability ({T}: Add {B}) is unconditional and correctly implemented.

## Summary
- Cards with issues: Battlefield Forge (1 HIGH), Blazemire Verge (1 HIGH)
- Clean cards: Abstergo Entertainment, Access Tunnel, Arch of Orazca

### Issue Details

Both HIGH findings are the same pattern: a mana ability is implemented without its restriction/cost, giving the controller an advantage they should not have. The fix for both is the same -- remove the partially-implemented ability entry from the `abilities` vec, leaving only the TODO comment describing the DSL gap.

- **Battlefield Forge**: Remove the `Effect::Choose` ability (lines 25-41), keep the TODO comment explaining the pain-land damage gap.
- **Blazemire Verge**: Remove the second `AbilityDefinition::Activated` (lines 25-32), keep the TODO comment explaining the activation-condition gap.
