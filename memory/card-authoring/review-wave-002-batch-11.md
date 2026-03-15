# Card Review: Wave 2 Batch 11

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Jhoira's Familiar
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature -- Bird)
- **Mana cost match**: YES ({4})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**:
  - None. Flying keyword implemented. Cost-reduction static ability correctly documented as TODO with accurate DSL gap description (historic spell cost reduction requires cost-reduction layer effect filtered by Artifact/Legendary/Saga).

## Card 2: Iroas, God of Victory
- **Oracle match**: YES
- **Types match**: YES (Legendary Enchantment Creature -- God)
- **Mana cost match**: YES ({2}{R}{W})
- **P/T match**: YES (7/4)
- **DSL correctness**: YES
- **Findings**:
  - None. Indestructible keyword implemented. Three TODOs correctly document DSL gaps: (1) devotion-based type removal (Layer 4), (2) continuous menace grant to all creatures you control (Layer 6), (3) blanket damage prevention for attacking creatures you control. All three are accurate gap descriptions.

## Card 3: Skithiryx, the Blight Dragon
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Phyrexian Dragon Skeleton)
- **Mana cost match**: YES ({3}{B}{B})
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**:
  - None. Flying and Infect keywords implemented. Two TODOs correctly document DSL gaps: (1) activated ability granting haste to self until end of turn, (2) activated regeneration ability. Both are accurate.

## Card 4: Legion Loyalist
- **Oracle match**: YES (em dash correctly encoded as Unicode U+2014)
- **Types match**: YES (Creature -- Goblin Soldier)
- **Mana cost match**: YES ({R})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): Oracle text uses Unicode em dash for "Battalion ---" separator. Scryfall oracle text uses the same encoding, so this is correct, but worth noting the encoding choice for consistency across card defs.
  - TODO accurately describes the DSL gap: Battalion trigger condition (self + 2 others attacking) and the compound effect (first strike + trample + token-block restriction) are both inexpressible.

## Card 5: Plague Stinger
- **Oracle match**: YES
- **Types match**: YES (Creature -- Phyrexian Insect Horror)
- **Mana cost match**: YES ({1}{B})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - None. Both Flying and Infect keywords implemented. Card is fully expressible in current DSL -- no TODOs needed.

## Summary
- Cards with issues: Legion Loyalist (1 LOW -- cosmetic only)
- Clean cards: Jhoira's Familiar, Iroas God of Victory, Skithiryx the Blight Dragon, Plague Stinger
- All 5 cards have correct oracle text, mana costs, types, and P/T values matching Scryfall data.
- No KI pattern violations detected (no overbroad triggers, no GainLife(0) placeholders, no incorrect target filters, no compilation issues).
- TODOs are accurate and descriptive where present.
