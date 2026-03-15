# Card Review: Wave 2 Batch 7

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Skrelv, Defector Mite
- **Oracle match**: YES
- **Types match**: YES (Legendary Artifact Creature -- Phyrexian Mite)
- **Mana cost match**: YES ({W})
- **P/T match**: YES (1/1)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Comment on line 8 says "Toxic 1 and CantBlock are implemented" but CantBlock is NOT actually in the abilities vec. The card only has `Toxic(1)`. The "can't block" restriction is missing and the TODO on line 33 contradicts the header comment on line 8. The header comment is misleading -- it implies CantBlock is present when it is not.
  - F2 (LOW): The TODO on line 33 says "no CantBlock keyword variant" which is an accurate DSL gap description. However the header comment on line 8 must be corrected to say only Toxic 1 is implemented, not CantBlock.
  - F3 (LOW): The activated ability TODO (lines 10-15) accurately describes 4 DSL gaps (Phyrexian mana, choose a color, hexproof from color, color-based block restriction). Correct to omit.

## Card 2: Alseid of Life's Bounty
- **Oracle match**: YES
- **Types match**: YES (Enchantment Creature -- Nymph)
- **Mana cost match**: YES ({W})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - No issues. Lifelink keyword is correctly implemented. Activated ability correctly omitted with accurate TODO describing the color-choice DSL gap. The ability requires both sacrifice cost, targeting "creature or enchantment you control", and interactive color selection -- all beyond current DSL.

## Card 3: Legolas's Quick Reflexes
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({G})
- **P/T match**: N/A (non-creature, correctly absent)
- **DSL correctness**: NO
- **Findings**:
  - F4 (MEDIUM): Split Second keyword is in the abilities vec, but the card's spell effect (untap + grant hexproof/reach + temporary triggered ability) is entirely missing with only a TODO. Per W5 policy, if the primary spell effect cannot be expressed, the card should have `abilities: vec![]` so it is not castable as a do-nothing spell. Currently casting this would apply Split Second (preventing responses) but do nothing else, which is worse than not being castable at all -- it gives the controller a free "lock out responses" with no downside. This is a KI-3 variant: a partial implementation that makes the card castable when its core effect is missing.

## Card 4: Hellkite Courser
- **Oracle match**: YES
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES ({4}{R}{R})
- **P/T match**: YES (6/5)
- **DSL correctness**: YES
- **Findings**:
  - No issues. Flying keyword correctly implemented. ETB ability correctly omitted with accurate TODO describing command zone manipulation DSL gap. The card is a 6/5 flyer even without its ETB, so having it be a castable creature with Flying is acceptable (it does not create degenerate game states like Legolas's Quick Reflexes does).

## Card 5: Malakir Bloodwitch
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Shaman)
- **Mana cost match**: YES ({3}{B}{B})
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**:
  - No issues. Flying and ProtectionFrom(FromColor(White)) correctly implemented. ETB drain ability correctly omitted with accurate TODO describing the subtype-count DSL gap. The card is a reasonable 4/4 flying pro-white creature even without its ETB.

## Summary
- Cards with issues: Skrelv, Defector Mite (misleading comment, 1 HIGH); Legolas's Quick Reflexes (partial implementation makes spell castable as do-nothing, 1 MEDIUM)
- Clean cards: Alseid of Life's Bounty, Hellkite Courser, Malakir Bloodwitch
