# Card Review: Wave 2 Batch 25

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Wrathful Red Dragon
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic:3, red:2)
- **P/T match**: YES (5/5)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword present. Triggered ability correctly omitted with accurate TODO describing WhenDealtDamage trigger with subtype filter and variable damage amount as DSL gaps.

## Card 2: Ajani, Sleeper Agent
- **Oracle match**: YES
- **Types match**: YES (Legendary Planeswalker -- Ajani)
- **Mana cost match**: NO
- **DSL correctness**: YES (abilities correctly empty with TODOs)
- **Findings**:
  - F1 (MEDIUM): Mana cost {1}{G}{G/W/P}{W} approximated as generic:1 + green:1 + white:1 = CMC 3, but actual CMC is 4. The hybrid Phyrexian pip {G/W/P} is entirely dropped rather than approximated as either green:2 or white:2. This means the card costs 1 less mana than it should in-game. The TODO documents the Phyrexian hybrid gap but the approximation should include the pip as either an extra green or white to preserve CMC. Consider generic:1 + green:2 + white:1 or generic:1 + green:1 + white:2.
  - F2 (LOW): No `loyalty` field on CardDefinition struct, so starting loyalty of 4 cannot be represented. This is a structural DSL gap, not a card def error. TODO documents this.

## Card 3: Nighthawk Scavenger
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Rogue)
- **Mana cost match**: YES (generic:1, black:2)
- **P/T match**: PARTIAL -- power:1 is the base value from P/T line "1+*/3" but the CDA is unimplementable. Acceptable baseline.
- **DSL correctness**: YES
- **Findings**:
  - F3 (LOW): Power set to 1 as static value; actual P/T is "1+*/3" where power is a CDA equal to 1 plus number of card types among opponents' graveyards. TODO accurately documents this as a DSL gap (EffectLayer::PtCda). The static value of 1 is the minimum and reasonable as a placeholder.

## Card 4: Karrthus, Tyrant of Jund
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Dragon)
- **Mana cost match**: YES (generic:4, black:1, red:1, green:1)
- **P/T match**: YES (7/7)
- **DSL correctness**: YES
- **Findings**: None. Flying and Haste keywords present. ETB triggered ability (mass control change + untap) and static ability (other Dragons have haste) correctly omitted with accurate TODOs describing subtype-filtered control change and keyword grant as DSL gaps.

## Card 5: Brave the Sands
- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES (generic:1, white:1)
- **P/T match**: N/A (non-creature, correctly absent)
- **DSL correctness**: YES
- **Findings**: None. Both static abilities (global vigilance grant, additional blocker) correctly omitted with accurate TODOs describing global keyword grant and additional blocker assignment as DSL gaps.

## Summary
- Cards with issues: Ajani, Sleeper Agent (1 MEDIUM mana cost, 1 LOW loyalty gap), Nighthawk Scavenger (1 LOW CDA power)
- Clean cards: Wrathful Red Dragon, Karrthus Tyrant of Jund, Brave the Sands
