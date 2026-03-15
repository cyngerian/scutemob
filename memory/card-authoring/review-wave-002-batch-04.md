# Card Review: Wave 002 Batch 04

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Rhythm of the Wild
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (1RG)
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - None. Card correctly uses `abilities: vec![]` with accurate TODO comments explaining DSL gaps (uncounterable spells + blanket riot grant). Oracle text matches Scryfall exactly.

## Card 2: Shadowspear
- **Oracle match**: YES
- **Types match**: YES (Legendary Artifact -- Equipment)
- **Mana cost match**: YES ({1})
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): The activated ability ({1} to remove hexproof/indestructible) is omitted with a TODO. The TODO accurately describes the DSL gap. The static abilities (P/T boost + keywords) and Equip cost are correctly implemented.

## Card 3: Sword of Vengeance
- **Oracle match**: YES
- **Types match**: YES (Artifact -- Equipment)
- **Mana cost match**: YES ({3})
- **DSL correctness**: YES
- **Findings**:
  - None. Clean implementation. `ModifyPower(2)` correctly represents "+2/+0". Four keywords (FirstStrike, Vigilance, Trample, Haste) granted via `AddKeywords`. Equip {3} correctly implemented.

## Card 4: Hope of Ghirapur
- **Oracle match**: YES
- **Types match**: YES (Legendary Artifact Creature -- Thopter)
- **Mana cost match**: YES ({1})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - None. Flying keyword implemented. Sacrifice ability correctly omitted with accurate TODO describing the tracking requirement ("target player who was dealt combat damage by this creature this turn" + "until your next turn" duration).

## Card 5: Concordant Crossroads
- **Oracle match**: YES
- **Types match**: NO
- **Mana cost match**: YES ({G})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F2 (HIGH): Missing `World` supertype. The card is a "World Enchantment" but uses `types(&[CardType::Enchantment])` instead of `supertypes(&[SuperType::World], &[CardType::Enchantment])`. `SuperType::World` exists in the DSL (see `state/types.rs` line 35), and the `supertypes()` helper exists in `helpers.rs` line 46. Another card (`tombstone_stairwell.rs`) already uses this exact pattern. The TODO comment says "SuperType::World may not be available" -- this is incorrect; it is available.
  - F3 (MEDIUM): The TODO comment on line 8 ("SuperType::World may not be available in helpers.rs") is inaccurate (KI-9 variant). The type exists and is used elsewhere. The TODO should be removed and the type line fixed.
  - F4 (LOW): Uses `AddKeyword` (singular) instead of `AddKeywords` (plural with collection). Both variants exist in the DSL so this is not wrong, but inconsistent with the pattern used in the other equipment cards in this batch. Not a bug.

## Summary
- Cards with issues: Concordant Crossroads (1 HIGH: missing World supertype, 1 MEDIUM: inaccurate TODO)
- Clean cards: Rhythm of the Wild, Shadowspear, Sword of Vengeance, Hope of Ghirapur
