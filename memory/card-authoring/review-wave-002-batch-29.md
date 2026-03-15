# Card Review: Wave 2, Batch 29

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Battle Squadron
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: NO
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): P/T should be `None`/`None` for a `*/*` creature (characteristic-defining ability). The definition uses `power: Some(0), toughness: Some(0)`, which would make it a 0/0 that dies to SBAs before the CDA can apply. Star P/T creatures should use `None` so the engine knows the value is defined by an ability, not a fixed number. Using `Some(0)` as a placeholder makes the card die immediately on ETB.
  - F2 (LOW): TODO accurately describes the DSL gap (CDA for P/T = creature count). Acceptable.

## Card 2: Gifted Aetherborn
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES (2/3)
- **DSL correctness**: YES
- **Findings**: None. Clean card.

## Card 3: Goblin War Drums
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: None. TODO accurately describes the DSL gap (static ability granting Menace to all creatures you control). Clean card.

## Card 4: Archon of Emeria
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES (2/3)
- **DSL correctness**: YES (empty abilities with TODOs)
- **Findings**: None. Both TODOs accurately describe the DSL gaps (spell-frequency restriction, conditional ETB-tapped replacement for opponent nonbasic lands). Flying keyword is correctly present. Clean card.

## Card 5: Karlach, Fury of Avernus
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Tiefling Barbarian)
- **Mana cost match**: YES
- **P/T match**: YES (5/4)
- **DSL correctness**: YES
- **Findings**:
  - F3 (LOW): Choose a Background keyword is correctly implemented. TODO accurately describes the DSL gaps for the attack trigger (AddCombatPhase, GrantKeywordToAttackers, first-combat-phase condition). Clean.

## Summary
- Cards with issues: Battle Squadron (1 HIGH -- `*/*` encoded as `Some(0)/Some(0)` instead of `None/None`)
- Clean cards: Gifted Aetherborn, Goblin War Drums, Archon of Emeria, Karlach Fury of Avernus
