# Card Review: Wave 2 Batch 27

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Ixhel, Scion of Atraxa
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Phyrexian Angel)
- **Mana cost match**: YES ({1}{W}{B}{G} = generic 1, white 1, black 1, green 1)
- **P/T match**: YES (2/5)
- **DSL correctness**: YES
- **Findings**: None. Keywords Flying, Vigilance, Toxic(2) correctly implemented. Corrupted end-step trigger correctly deferred with accurate TODO describing per-opponent conditional exile and play-from-exile tracking gaps.

## Card 2: Smoke Shroud
- **Oracle match**: YES
- **Types match**: YES (Enchantment -- Aura)
- **Mana cost match**: YES ({1}{U} = generic 1, blue 1)
- **P/T match**: N/A (non-creature, correctly absent)
- **DSL correctness**: YES
- **Findings**: None. Enchant(Creature) keyword correct. +1/+1 split into two Layer 7c static effects (ModifyPower and ModifyToughness) with AttachedCreature filter -- correct pattern. Flying grant via Layer 6 AddKeyword -- correct. Ninja-enters graveyard-return trigger correctly deferred with accurate TODO.

## Card 3: Drana and Linvala
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Vampire Angel)
- **Mana cost match**: YES ({1}{W}{W}{B} = generic 1, white 2, black 1)
- **P/T match**: YES (3/4)
- **DSL correctness**: YES
- **Findings**: None. Flying and Vigilance keywords correct. Both static abilities (opponent ability suppression and ability copying) correctly deferred with accurate TODOs describing specific DSL gaps.

## Card 4: Goblin Motivator
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Warrior)
- **Mana cost match**: YES ({R} = red 1)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None. Activated ability correctly deferred with accurate TODO citing the activated_ability_targets DSL gap. Abilities vec is empty per W5 policy (no partial implementations).

## Card 5: Moria Marauder
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Warrior)
- **Mana cost match**: YES ({R}{R} = red 2)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None. DoubleStrike keyword correct. Combat damage trigger correctly deferred with accurate TODO citing multi_type_filter and non-self trigger gaps.

## Summary
- Cards with issues: (none)
- Clean cards: Ixhel Scion of Atraxa, Smoke Shroud, Drana and Linvala, Goblin Motivator, Moria Marauder
