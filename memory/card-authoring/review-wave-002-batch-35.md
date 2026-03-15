# Card Review: Wave 2 Batch 35

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 1 LOW

## Card 1: Brash Taunter
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (4R)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: Clean. Indestructible keyword present. Two TODOs accurately describe gaps: WhenDealtDamage trigger requires damage-amount variable forwarding (not in DSL), and the activated fight ability requires activated_ability_targets (Activated has no targets field). Both correctly left as `vec![]` with TODOs per W5 policy.

## Card 2: Eomer, King of Rohan
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Noble)
- **Mana cost match**: YES (3RW)
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: Clean. Double strike keyword present. TODOs accurately describe two gaps: ETB counter placement based on creature count (count_threshold gap) and ETB trigger with monarch + damage-equal-to-power (monarch mechanic not in DSL). Correctly left as `vec![]`.

## Card 3: Great Oak Guardian
- **Oracle match**: YES
- **Types match**: YES (Creature -- Treefolk)
- **Mana cost match**: YES (5G)
- **P/T match**: YES (4/5)
- **DSL correctness**: YES
- **Findings**: Clean. Flash and Reach keywords present. TODO accurately describes the ETB trigger gap: targeting a player and buffing all their creatures +2/+2 plus untapping them requires targeted_trigger with ForEach. Correctly left as `vec![]`.

## Card 4: Fell Stinger
- **Oracle match**: YES
- **Types match**: YES (Creature -- Zombie Scorpion)
- **Mana cost match**: YES (2B)
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**: Clean. Deathtouch and Exploit keywords present. TODO accurately describes that the exploit trigger effect (target player draws 2, loses 2) requires targeting a player (targeted_trigger gap). Correctly left as `vec![]`.

## Card 5: Endurance
- **Oracle match**: YES
- **Types match**: YES (Creature -- Elemental Incarnation)
- **Mana cost match**: YES (1GG)
- **P/T match**: YES (3/4)
- **DSL correctness**: MINOR ISSUE
- **Findings**:
  - F1 (MEDIUM): Evoke keyword is present but has no associated evoke cost definition. The Evoke keyword alone tells the engine this card has evoke, but without specifying the alternative cost (exile a green card from hand), the evoke casting path may not work correctly. The engine's Evoke implementation (Batch 6 / M9.5) may handle this via the keyword alone or may require additional cost specification -- verify against other evoke card defs. If the keyword is sufficient (cost encoded elsewhere or hardcoded), this is a non-issue.
  - F2 (LOW): TODO comment says "shuffle their graveyard to the bottom of their library" but the oracle text says "puts all the cards from their graveyard on the bottom of their library in a random order." This is not shuffling -- it is placing cards on the bottom in a random order (the library itself is not shuffled). The TODO is slightly inaccurate in its description but correctly identifies the DSL gap.

## Summary
- Cards with issues: Endurance (1 MEDIUM evoke cost, 1 LOW inaccurate TODO wording)
- Clean cards: Brash Taunter, Eomer King of Rohan, Great Oak Guardian, Fell Stinger
