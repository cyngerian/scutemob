# Card Review: Wave 2, Batch 19

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Neriv, Heart of the Storm
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic 1, red 1, white 1, black 1)
- **P/T match**: YES (4/5)
- **DSL correctness**: YES
- **Findings**:
  - Flying keyword is present. Replacement effect correctly left as TODO with accurate description of the DSL gap (damage doubling conditioned on "entered this turn"). No KI issues.

## Card 2: Phyrexian Crusader
- **Oracle match**: YES
- **Types match**: YES (Phyrexian Zombie Knight)
- **Mana cost match**: YES (generic 1, black 2)
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**:
  - All abilities implemented: FirstStrike, ProtectionFrom(Red), ProtectionFrom(White), Infect. Clean card.

## Card 3: Razorkin Needlehead
- **Oracle match**: YES
- **Types match**: YES (Human Assassin)
- **Mana cost match**: YES (red 2)
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): Both abilities left as TODO with `abilities: vec![]`. The TODOs accurately describe the DSL gaps: conditional first strike (only during your turn) requires a layer-6 conditional keyword grant, and the opponent-draws-card trigger requires a `WheneverOpponentDrawsCard` TriggerCondition variant. Correct per W5 policy.

## Card 4: The Ur-Dragon
- **Oracle match**: YES
- **Types match**: YES (Legendary Dragon Avatar)
- **Mana cost match**: YES (generic 4, white 1, blue 1, black 1, red 1, green 1)
- **P/T match**: YES (10/10)
- **DSL correctness**: YES
- **Findings**:
  - Flying keyword present. Eminence and attack trigger correctly left as TODO. TODO comments accurately describe all three DSL gaps: Eminence cost reduction from command zone, dynamic draw count based on attacking Dragons, and putting a permanent from hand onto battlefield. No KI issues.

## Card 5: Thundermane Dragon
- **Oracle match**: YES
- **Types match**: YES (Dragon)
- **Mana cost match**: YES (generic 3, red 1)
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**:
  - Flying keyword present. Library-top permission and cast-from-top abilities correctly left as TODO with accurate gap descriptions. No KI issues.

## Summary
- Cards with issues: Razorkin Needlehead (1 LOW -- empty abilities, correct per W5 policy)
- Clean cards: Neriv Heart of the Storm, Phyrexian Crusader, The Ur-Dragon, Thundermane Dragon
