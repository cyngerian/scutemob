# Card Review: Wave 2 Batch 38

**Reviewed**: 2026-03-13
**Cards**: 2
**Findings**: 2 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Nezumi Prowler
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature -- Rat Ninja)
- **Mana cost match**: YES ({1}{B})
- **P/T match**: YES (3/1)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Missing `AbilityDefinition::Ninjutsu { cost: ManaCost { generic: 1, black: 1, ..Default::default() } }`. The card only has `Keyword(KeywordAbility::Ninjutsu)` which is the keyword marker, but the engine's `get_ninjutsu_cost()` in `abilities.rs` looks up the cost from `AbilityDefinition::Ninjutsu { cost }`. Without it, Ninjutsu activation will fail (no cost found). Reference: `ninja_of_the_deep_hours.rs` lines 15-18 has both the Keyword marker and the Ninjutsu cost entry.
  - TODO accuracy: The TODO about the ETB trigger DSL gap is accurate. The ETB grants deathtouch + lifelink to a target creature you control until end of turn, which requires targeted triggered ability + keyword granting -- correctly identified as inexpressible.

## Card 2: Mindleecher
- **Oracle match**: YES
- **Types match**: YES (Creature -- Nightmare)
- **Mana cost match**: YES ({4}{B}{B})
- **P/T match**: YES (5/5)
- **DSL correctness**: NO
- **Findings**:
  - F2 (HIGH): Missing `AbilityDefinition::MutateCost { cost: ManaCost { generic: 4, black: 1, ..Default::default() } }`. The card only has `Keyword(KeywordAbility::Mutate)` which is the keyword marker, but mutate activation requires the `MutateCost` ability definition with the actual cost. Without it, Mutate cannot be activated. Reference: `gemrazer.rs` lines 25-29 has both the Keyword marker and the MutateCost entry.
  - TODO accuracy: The TODO about the mutate trigger DSL gap is accurate. Exiling face-down cards from opponents' libraries with play-while-exiled permission is correctly identified as inexpressible in the current DSL.

## Summary
- Cards with issues: Nezumi Prowler (F1), Mindleecher (F2)
- Clean cards: (none)
- Both HIGH findings follow the same pattern: alternative cost ability definitions (`Ninjutsu`, `MutateCost`) are missing, leaving only the keyword marker. The keyword marker alone is insufficient -- the engine needs the cost-bearing ability definition to actually activate the alternative cost.
