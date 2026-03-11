# Card Review: Phase 1 Batch 15 (MDFCs and Split Cards -- body-only)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Boggart Trawler // Boggart Bog
- **Oracle match**: YES
- **Types match**: YES (Creature - Goblin)
- **Mana cost match**: YES (2B = generic: 2, black: 1)
- **P/T match**: YES (3/1)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: MDFC with Land back face. Front face oracle text "When this creature enters, exile target player's graveyard." matches Scryfall. abilities: vec![] correct per MDFC body-only policy (ETB trigger + back face both need full MDFC support).

## Card 2: Malakir Rebirth // Malakir Mire
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES (B = black: 1)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: MDFC with Land back face. Front face oracle text matches Scryfall exactly. abilities: vec![] correct per MDFC body-only policy (complex replacement effect + back face need full MDFC support).

## Card 3: Sea Gate Restoration // Sea Gate, Reborn
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES (4UUU = generic: 4, blue: 3)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: MDFC with Land back face. Front face oracle text "Draw cards equal to the number of cards in your hand plus one. You have no maximum hand size for the rest of the game." matches Scryfall. abilities: vec![] correct per MDFC body-only policy (variable draw + no-max-hand-size + back face need full MDFC support).

## Card 4: Commit // Memory
- **Oracle match**: YES
- **Types match**: YES (Instant, front half only)
- **Mana cost match**: YES (3U = generic: 3, blue: 1)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Aftermath split card (not MDFC). Front half "Commit" oracle text "Put target spell or nonland permanent into its owner's library second from the top." matches Scryfall. Back half "Memory" (4UU Sorcery, Aftermath) is omitted. abilities: vec![] correct per body-only policy (library tuck + Aftermath back half need full split card support).

## Card 5: Brightclimb Pathway // Grimclimb Pathway
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: MDFC Pathway land. Front face "{T}: Add {W}." matches Scryfall. Back face (Grimclimb Pathway, "{T}: Add {B}.") is omitted. abilities: vec![] correct per MDFC body-only policy (mana ability + back face need full MDFC support). Note: unlike single-faced lands which implement mana abilities, this card uses vec![] because the back face is integral to the card's identity and both faces should be implemented together.

## Summary
- Cards with issues: (none)
- Clean cards: Boggart Trawler // Boggart Bog, Malakir Rebirth // Malakir Mire, Sea Gate Restoration // Sea Gate, Reborn, Commit // Memory, Brightclimb Pathway // Grimclimb Pathway

All 5 cards are clean. Oracle text matches Scryfall for all front faces. Mana costs, types, and P/T (where applicable) are all correct. The abilities: vec![] approach is correct for all cards since they are MDFCs or split cards requiring back_face/Aftermath support for full implementation. Card names correctly include both faces with " // " separator. Card IDs correctly use front-face slug only. No power/toughness fields on non-creature cards (correct). Boggart Trawler correctly has P/T as a creature.
