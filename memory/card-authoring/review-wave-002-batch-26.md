# Card Review: Wave 2 Batch 26

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 1 LOW

## Card 1: Warren Instigator
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Double strike keyword is present. The triggered ability (put a Goblin from hand onto battlefield when dealing damage to an opponent) is correctly identified as a DSL gap and left unimplemented with an accurate TODO. Per W5 policy, `abilities` contains only the keyword and omits the trigger -- correct.

## Card 2: Scion of the Ur-Dragon
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Dragon Avatar)
- **Mana cost match**: YES (WUBRG)
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Flying keyword is present. The activated ability (search library for Dragon, put to graveyard, become copy) is correctly identified as a DSL gap with an accurate TODO describing the search-to-graveyard + self-copy interaction.

## Card 3: Vampire Nighthawk
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Shaman)
- **Mana cost match**: YES ({1}{B}{B})
- **P/T match**: YES (2/3)
- **DSL correctness**: YES
- **Findings**:
  - Clean. All three keywords (Flying, Deathtouch, Lifelink) correctly implemented. Fully expressible in DSL with no gaps.

## Card 4: Abomination of Llanowar
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elf Horror)
- **Mana cost match**: YES ({1}{B}{G})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (MEDIUM): P/T should be `None`/`None` (or use a sentinel) for a `*/*` creature with a CDA, not `Some(0)`/`Some(0)`. Using `0` means the card enters as a 0/0 and dies to SBAs immediately, which is incorrect behavior. The oracle P/T is `*/*`. The TODO correctly identifies the CDA gap, but the power/toughness values make the card functionally broken if cast. Per W5 policy, since the CDA cannot be expressed, the card should still be castable but should document the behavioral discrepancy. However, using 0/0 actively corrupts game state (instant SBA death), which is worse than a reasonable placeholder or `None`. This needs attention.
  - F2 (LOW): The TODO accurately describes the DSL gap (cross-zone subtype counting). No issue with the description itself.

## Card 5: Wingcrafter
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Wizard)
- **Mana cost match**: YES ({U})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Soulbond with Flying grant is correctly expressed using `AbilityDefinition::Soulbond` with `SoulbondGrant` targeting the Ability layer with `AddKeyword(Flying)`. Matches the Batch 10 Soulbond infrastructure.

## Summary
- **Cards with issues**: Abomination of Llanowar (1 MEDIUM: 0/0 P/T for `*/*` CDA creature)
- **Clean cards**: Warren Instigator, Scion of the Ur-Dragon, Vampire Nighthawk, Wingcrafter
