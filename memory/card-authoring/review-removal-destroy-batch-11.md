# Card Review: Removal/Destroy Batch 11

**Reviewed**: 2026-03-22
**Cards**: 2
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Patron of the Vein
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Shaman, no supertypes needed)
- **Mana cost match**: YES ({4}{B}{B} = generic 4, black 2)
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**: None

**Notes on TODO**: The card's second triggered ability is left as a TODO with two cited DSL gaps:
1. `WheneverCreatureDies` has no controller filter (cannot express "a creature an opponent controls dies") -- VALID gap, confirmed no filtered variant exists.
2. Exiling the specific dying creature requires LKI / event-based targeting not available in the DSL -- VALID gap.

Leaving this as `vec![]`-equivalent (ability omitted entirely) is correct per W5 policy: implementing only the ETB destroy without the death trigger does not produce wrong game state (the ETB is independently correct). The death-trigger omission means the card is weaker than it should be but does not create an incorrect state.

## Card 2: Acidic Slime
- **Oracle match**: YES
- **Types match**: YES (Creature -- Ooze, no supertypes needed)
- **Mana cost match**: YES ({3}{G}{G} = generic 3, green 2)
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: None

**Notes**: The ETB trigger uses `TargetPermanentWithFilter` with `has_card_types: vec![Artifact, Enchantment, Land]`. The engine applies OR semantics to `has_card_types` (confirmed in `effects/mod.rs`), which correctly matches "target artifact, enchantment, or land." Deathtouch keyword is present. Card is fully implemented.

## Summary
- Cards with issues: (none)
- Clean cards: Patron of the Vein, Acidic Slime
