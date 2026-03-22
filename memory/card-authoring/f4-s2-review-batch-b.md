# Card Review: F4-S2 Batch B (Bounce Lands + Shrieking Drake)

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Orzhov Basilica
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes needed)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

Verified: ETB tapped replacement present; bounce trigger targets land you control with OwnerOf for hand zone; mana_pool(1,0,1,0,0,0) = {W}{B} correct.

## Card 2: Rakdos Carnarium
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes needed)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

Verified: ETB tapped replacement present; bounce trigger targets land you control with OwnerOf for hand zone; mana_pool(0,0,1,1,0,0) = {B}{R} correct.

## Card 3: Selesnya Sanctuary
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes needed)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

Verified: ETB tapped replacement present; bounce trigger targets land you control with OwnerOf for hand zone; mana_pool(1,0,0,0,1,0) = {G}{W} correct.

## Card 4: Simic Growth Chamber
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes needed)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None

Verified: ETB tapped replacement present; bounce trigger targets land you control with OwnerOf for hand zone; mana_pool(0,1,0,0,1,0) = {G}{U} correct.

## Card 5: Shrieking Drake
- **Oracle match**: YES
- **Types match**: YES (Creature -- Drake)
- **Mana cost match**: YES ({U} = blue: 1)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None

Verified: Flying keyword present; ETB trigger targets creature you control (not "another" -- can self-bounce, correct); OwnerOf for hand zone (multiplayer-correct); no extra abilities.

## Summary
- Cards with issues: (none)
- Clean cards: Orzhov Basilica, Rakdos Carnarium, Selesnya Sanctuary, Simic Growth Chamber, Shrieking Drake

All 5 cards are structurally identical bounce-land patterns (4 lands) plus one ETB-bounce creature, all correctly implemented. Key correctness points verified:
- mana_pool argument order (W,U,B,R,G,C) matches oracle for all 4 lands
- OwnerOf used (not ControllerOf) for "its owner's hand" -- multiplayer correct
- Bounce trigger does not exclude self (oracle says "a land/creature you control", not "another")
- ETB tapped replacement effect present on all 4 bounce lands
- Shrieking Drake has Flying keyword + correct creature subtype + correct P/T
