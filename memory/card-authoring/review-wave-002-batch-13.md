# Card Review: Wave 2 Batch 13

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Phoenix Chick
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (R)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None. Flying and Haste implemented as keywords. "Can't block" and the graveyard-return triggered ability correctly documented as DSL gaps with accurate TODO descriptions. Empty abilities for unimplemented parts follow W5 policy (abilities omitted, not approximated).

## Card 2: Sylvan Messenger
- **Oracle match**: YES
- **Types match**: YES (Creature - Elf)
- **Mana cost match**: YES (3G)
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: None. Trample implemented as keyword. The ETB "reveal top four, collect Elves" ability correctly documented as DSL gap. TODO accurately describes the three sub-requirements (reveal top N, subtype filter, split destination).

## Card 3: Shrieking Drake
- **Oracle match**: YES
- **Types match**: YES (Creature - Drake)
- **Mana cost match**: YES (U)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None. Flying implemented as keyword. The ETB bounce ability ("return a creature you control to its owner's hand") correctly documented as DSL gap. TODO is accurate.

## Card 4: Broodcaller Scourge
- **Oracle match**: YES
- **Types match**: YES (Creature - Dragon)
- **Mana cost match**: YES (5GG)
- **P/T match**: YES (5/7)
- **DSL correctness**: YES
- **Findings**: None. Flying implemented as keyword. The combat damage trigger with damage-amount-based mana value filtering correctly documented as DSL gap. TODO accurately identifies all three sub-requirements.

## Card 5: Greymond, Avacyn's Stalwart
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature - Human Soldier, uses full_types with SuperType::Legendary)
- **Mana cost match**: YES (2WW)
- **P/T match**: YES (3/4)
- **DSL correctness**: YES
- **Findings**: None. Both abilities (modal ETB choose-two grant and conditional +2/+2 static) correctly documented as DSL gaps. The `abilities: vec![]` with inline TODO comments follows W5 policy. Supertypes correctly include Legendary.

## Summary
- Cards with issues: (none)
- Clean cards: Phoenix Chick, Sylvan Messenger, Shrieking Drake, Broodcaller Scourge, Greymond Avacyn's Stalwart
