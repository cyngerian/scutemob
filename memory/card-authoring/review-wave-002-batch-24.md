# Card Review: Wave 2 Batch 24

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Scion of Draco
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic 12)
- **DSL correctness**: YES
- **Findings**:
  - None. Flying keyword present. Domain is an ability word (not a keyword in the engine), so no KeywordAbility variant expected. Two DSL gap TODOs correctly document the cost reduction and color-conditional keyword grant. Empty abilities (besides Flying) follow W5 policy.

## Card 2: Sting, the Glinting Dagger
- **Oracle match**: YES
- **Types match**: YES (Legendary Artifact -- Equipment)
- **Mana cost match**: YES (generic 2)
- **DSL correctness**: YES
- **Findings**:
  - None. All four abilities (static +1/+1 and haste, combat untap trigger, conditional first strike, equip) are DSL gaps and correctly documented as TODOs. Empty `abilities: vec![]` follows W5 policy.

## Card 3: Markov Baron
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Noble)
- **Mana cost match**: YES (2 generic + 1 black)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Missing Convoke keyword ability. Other card defs (e.g., `siege_wurm.rs`) use `AbilityDefinition::Keyword(KeywordAbility::Convoke)`. Convoke is a supported keyword in the engine and should be included in the abilities vec. Without it, the card cannot be cast with Convoke.
  - F2 (MEDIUM): Missing Madness ability definition. Other card defs (e.g., `fiery_temper.rs`) use both `AbilityDefinition::Keyword(KeywordAbility::Madness)` and `AbilityDefinition::Madness { cost: ManaCost { generic: 2, black: 1, ..Default::default() } }`. Without these, the card cannot be cast for its madness cost. The TODO comment only mentions the static +1/+1 lord ability as a DSL gap but omits the missing Convoke and Madness implementations.

## Card 4: Vampire of the Dire Moon
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire)
- **Mana cost match**: YES (1 black)
- **DSL correctness**: YES
- **Findings**:
  - None. Clean card -- both Deathtouch and Lifelink keywords present.

## Card 5: Goblin Ringleader
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin)
- **Mana cost match**: YES (3 generic + 1 red)
- **DSL correctness**: YES
- **Findings**:
  - None. Haste keyword present. ETB reveal-and-filter ability correctly documented as a DSL gap TODO. Empty abilities (besides Haste) follow W5 policy.

## Summary
- Cards with issues: Markov Baron (1 HIGH: missing Convoke keyword; 1 MEDIUM: missing Madness ability)
- Clean cards: Scion of Draco, Sting the Glinting Dagger, Vampire of the Dire Moon, Goblin Ringleader
