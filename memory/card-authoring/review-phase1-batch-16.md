# Card Review: Phase 1 Batch 16 — Body-Only MDFCs and Room

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Disciple of Freyalise // Garden of Freyalise
- **Oracle match**: YES
- **Types match**: YES — Creature - Elf Druid (front face)
- **Mana cost match**: YES — {3}{G}{G}{G} = generic 3, green 3
- **P/T match**: YES — 3/3
- **DSL correctness**: YES — `abilities: vec![]` correct (MDFC needs back_face support)
- **Findings**: None

## Card 2: Fell the Profane // Fell Mire
- **Oracle match**: YES
- **Types match**: YES — Instant (front face)
- **Mana cost match**: YES — {2}{B}{B} = generic 2, black 2
- **DSL correctness**: YES — `abilities: vec![]` correct (MDFC needs back_face support)
- **Findings**: None

## Card 3: Sink into Stupor // Soporific Springs
- **Oracle match**: YES
- **Types match**: YES — Instant (front face)
- **Mana cost match**: YES — {1}{U}{U} = generic 1, blue 2
- **DSL correctness**: YES — `abilities: vec![]` correct (MDFC needs back_face support)
- **Findings**: None

## Card 4: Witch Enchanter // Witch-Blessed Meadow
- **Oracle match**: YES
- **Types match**: YES — Creature - Human Warlock (front face)
- **Mana cost match**: YES — {3}{W} = generic 3, white 1
- **P/T match**: YES — 2/2
- **DSL correctness**: YES — `abilities: vec![]` correct (MDFC needs back_face support)
- **Findings**: None

## Card 5: Funeral Room // Awakening Hall
- **Oracle match**: PARTIAL — see F1
- **Types match**: YES — Enchantment - Room (front face, using `types_sub`)
- **Mana cost match**: YES — {2}{B} = generic 2, black 1 (front face only)
- **DSL correctness**: YES — `abilities: vec![]` correct (Room needs door/unlock support)
- **Findings**:
  - F1 (MEDIUM): Oracle text includes reminder text for Room mechanic ("You may cast either half..."). While not incorrect per se, this is inconsistent with most other card definitions in the codebase which omit reminder text. The front face oracle text should be just "Whenever a creature you control dies, each opponent loses 1 life and you gain 1 life." without the reminder paragraph. However, since the card has `abilities: vec![]` anyway, this is cosmetic.
  - F2 (LOW): Funeral Room is a Room (Duskmourn), not a standard MDFC. Rooms use a different mechanic (doors/unlocking) than MDFCs (modal double-faced). The TODO reason differs from the other 4 cards: this needs Room/door/unlock support rather than MDFC back_face support. The current `abilities: vec![]` is correct either way.
  - F3 (LOW): The back face (Awakening Hall) has mana cost {6}{B}{B} and its own ability. When Room support is added, both halves will need to be represented. This is not an error in the current definition but worth noting for future implementation.

## Summary
- Cards with issues: Funeral Room // Awakening Hall (1 MEDIUM cosmetic, 2 LOW informational)
- Clean cards: Disciple of Freyalise // Garden of Freyalise, Fell the Profane // Fell Mire, Sink into Stupor // Soporific Springs, Witch Enchanter // Witch-Blessed Meadow
