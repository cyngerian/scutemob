# Wave 1 Batches 16-17 Review

**Reviewed**: 2026-03-12
**Cards**: 7
**Findings**: 1 HIGH, 1 MEDIUM, 5 LOW

---

## Card: Sacred Foundry
- **card_id**: witchs-cottage -> `cid("sacred-foundry")` -- correct
- **name**: correct
- **types/subtypes**: correct -- `Land - Plains Mountain` (subtype order differs from Scryfall's "Mountain Plains" but order is not mechanically significant)
- **oracle_text**: correct -- matches Scryfall exactly
- **abilities**: Skeleton with TODOs. Both the basic land type mana ability (handled by engine for Plains/Mountain subtypes) and the shock ETB ("pay 2 life or enter tapped") are DSL gaps (shock_etb pattern). TODOs accurately describe both abilities.
- **Verdict**: PASS

## Card: Steam Vents
- **card_id**: correct
- **name**: correct
- **types/subtypes**: correct -- `Land - Island Mountain` matches Scryfall
- **oracle_text**: correct -- matches Scryfall exactly
- **abilities**: Skeleton with TODOs. Same shock_etb DSL gap as Sacred Foundry. TODOs accurate.
- **Verdict**: PASS

## Card: Field of the Dead
- **card_id**: correct
- **name**: correct
- **types/subtypes**: correct -- `Land` with no subtypes
- **oracle_text**: correct -- matches Scryfall exactly
- **abilities**: Skeleton with TODOs. Three abilities: (1) ETB tapped -- implementable in DSL but left as TODO, (2) `{T}: Add {C}` mana ability -- implementable but left as TODO, (3) landfall trigger with intervening-if "seven or more lands with different names" condition and Zombie token creation -- count_threshold DSL gap. TODO on line 14 is truncated but describes the trigger.
- **Findings**:
  - F1 (MEDIUM): ETB tapped and `{T}: Add {C}` mana ability are both implementable in the current DSL but left as empty TODOs. These should be implemented to make the card minimally functional as a tapped colorless land. The triggered ability is correctly a DSL gap.
  - F2 (LOW): TODO comment on line 14 is truncated mid-sentence ("if you control seven or m"). Not harmful but imprecise.
- **Verdict**: MEDIUM

## Card: Witch's Cottage
- **card_id**: `cid("witchs-cottage")` -- correct (no apostrophe in slug, matches filename)
- **name**: correct -- "Witch's Cottage"
- **types/subtypes**: correct -- `Land - Swamp`
- **oracle_text**: correct -- matches Scryfall exactly
- **abilities**: Skeleton with TODOs. Three abilities: (1) Swamp mana ability (engine handles via subtype), (2) conditional ETB tapped ("unless you control three or more other Swamps") -- DSL gap, (3) conditional ETB trigger ("when this land enters untapped") with targeted graveyard-to-library effect -- targeted_trigger DSL gap. TODOs accurate.
- **Verdict**: PASS

## Card: Emeria, the Sky Ruin
- **card_id**: correct
- **name**: correct -- "Emeria, the Sky Ruin"
- **types/subtypes**: WRONG -- uses `types(&[CardType::Land])` but Emeria, the Sky Ruin is a **Legendary Land**. Should use `supertypes(&[SuperType::Legendary], &[CardType::Land])`.
- **oracle_text**: correct -- matches Scryfall exactly
- **abilities**: Skeleton with TODOs. Three abilities: (1) ETB tapped -- implementable but left as TODO, (2) upkeep trigger with intervening-if and targeted return-from-graveyard -- complex DSL gap (count_threshold + targeted_trigger + return_from_graveyard), (3) `{T}: Add {W}` mana ability -- implementable but left as TODO. TODOs accurate.
- **Findings**:
  - F1 (HIGH): Missing `SuperType::Legendary` supertype. Emeria, the Sky Ruin has type line "Legendary Land". The definition uses `types(&[CardType::Land])` which omits the Legendary supertype entirely. This affects legend rule enforcement (CR 704.5j) -- two copies of Emeria would not trigger the legend rule SBA.
  - F2 (MEDIUM): ETB tapped and `{T}: Add {W}` mana ability are both implementable in the current DSL but left as empty TODOs. The upkeep trigger is correctly a DSL gap.
- **Verdict**: HIGH

## Card: Den of the Bugbear
- **card_id**: correct
- **name**: correct
- **types/subtypes**: correct -- `Land` with no subtypes
- **oracle_text**: correct -- matches Scryfall exactly
- **abilities**: Skeleton with TODOs. Three abilities: (1) conditional ETB tapped ("if you control two or more other lands") -- DSL gap, (2) `{T}: Add {R}` mana ability -- implementable but left as TODO, (3) creature-land animation activated ability -- complex DSL gap (animate_land). TODOs accurate.
- **Findings**:
  - F3 (LOW): `{T}: Add {R}` mana ability is implementable in the current DSL but left as TODO.
- **Verdict**: LOW

## Card: Mortuary Mire
- **card_id**: correct
- **name**: correct
- **types/subtypes**: correct -- `Land` with no subtypes
- **oracle_text**: correct -- matches Scryfall exactly
- **abilities**: Skeleton with TODOs. Three abilities: (1) ETB tapped -- implementable but left as TODO, (2) ETB trigger targeting creature in graveyard to put on top of library -- targeted_trigger DSL gap, (3) `{T}: Add {B}` mana ability -- implementable but left as TODO. TODOs accurate.
- **Findings**:
  - F4 (LOW): ETB tapped and `{T}: Add {B}` mana ability are both implementable but left as TODOs.
- **Verdict**: LOW

---

## Summary

| Severity | Count | Cards |
|----------|-------|-------|
| HIGH | 1 | Emeria, the Sky Ruin (missing Legendary supertype) |
| MEDIUM | 1 | Field of the Dead (implementable ETB tapped + mana ability skipped); Emeria also has this issue (counted under HIGH) |
| LOW | 5 | Field of the Dead (truncated TODO), Den of the Bugbear (mana ability skippable), Mortuary Mire (implementable abilities skipped), Sacred Foundry (clean), Steam Vents (clean) |

**HIGH: 1 | MEDIUM: 1 | LOW: 3**

### Cards with issues
- **Emeria, the Sky Ruin**: HIGH -- missing Legendary supertype (affects legend rule). Also has implementable abilities left as TODO.
- **Field of the Dead**: MEDIUM -- ETB tapped and colorless mana ability are implementable but left as TODO.
- **Den of the Bugbear**: LOW -- red mana ability implementable but left as TODO.
- **Mortuary Mire**: LOW -- ETB tapped and black mana ability implementable but left as TODO.

### Clean cards
- Sacred Foundry
- Steam Vents
- Witch's Cottage
