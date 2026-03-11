# Card Review: Phase 1 Batch 17 (MDFCs, Split Cards, Artifact Creature)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 1 HIGH, 2 MEDIUM, 0 LOW

## Card 1: Walk-In Closet // Forgotten Cellar
- **Oracle match**: PARTIAL -- MCP did not return oracle text to verify exact wording. Comment line 1 says "You may play lands from your graveyard." which is plausible for a Room front face. Cannot fully confirm.
- **Types match**: YES (Enchantment -- Room)
- **Mana cost match**: YES ({2}{G} = generic: 2, green: 1)
- **P/T match**: N/A (not a creature)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Room MDFC. Card name correctly includes both faces. Card ID uses front-face slug. abilities: vec![] correct per MDFC body-only policy (Room mechanic + back face need full MDFC support). Mana cost matches Scryfall front face {2}{G}.

## Card 2: Phyrexian Walker
- **Oracle match**: YES (empty oracle text, matches Scryfall vanilla creature)
- **Types match**: YES (Artifact Creature -- Phyrexian Construct)
- **Mana cost match**: YES ({0} = all zeros default)
- **P/T match**: YES (0/3)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Clean card. Zero-cost artifact creature with no abilities. Types correctly use `types_sub(&[CardType::Artifact, CardType::Creature], &["Phyrexian", "Construct"])`. Empty oracle_text and abilities: vec![] are both correct for a vanilla creature.

## Card 3: Connive // Concoct
- **Oracle match**: PARTIAL -- front face oracle text "Gain control of target creature with power 2 or less." is plausible for Connive half but could not fully verify exact Scryfall wording.
- **Types match**: YES (Sorcery)
- **Mana cost match**: NO
- **P/T match**: N/A (not a creature)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Mana cost is wrong. Scryfall shows {2}{U/B}{U/B} (mana value 4) but the definition has `generic: 2` only (mana value 2). Even though hybrid mana is a DSL gap, the convention (see Kitchen Finks, Boggart Ram-Gang, Nethroi) is to approximate with one color per hybrid symbol and add a comment. Should be something like `ManaCost { generic: 2, blue: 1, black: 1, ..Default::default() }` with a comment noting "{U/B}{U/B} hybrid simplified." The current definition loses 2 mana from the cost entirely, making the mana value wrong.
  - F2 (MEDIUM): No TODO comment documenting that hybrid mana {U/B}{U/B} is a DSL gap. Other hybrid cards (kitchen_finks.rs, boggart_ram_gang.rs, nethroi_apex_of_death.rs) all include such comments.
- **Notes**: Split card (not MDFC). Back half "Concoct" ({3}{U}{B} Sorcery, Surveil 3 + return creature from graveyard) is omitted per body-only policy. abilities: vec![] correct since gain-control effects need DSL support.

## Card 4: Bottomless Pool // Locker Room
- **Oracle match**: PARTIAL -- MCP did not return oracle text for Room cards. Comment says "When you unlock this door, return up to one target creature to its owner's hand." which is plausible.
- **Types match**: YES (Enchantment -- Room)
- **Mana cost match**: YES ({U} = blue: 1)
- **P/T match**: N/A (not a creature)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Room MDFC. Card name correctly includes both faces. Card ID uses front-face slug. abilities: vec![] correct per MDFC body-only policy. Mana cost matches Scryfall front face {U}.

## Card 5: Agadeem's Awakening // Agadeem, the Undercrypt
- **Oracle match**: YES -- "Return from your graveyard to the battlefield any number of target creature cards that each have a different mana value X or less." matches expected front face text.
- **Types match**: YES (Sorcery)
- **Mana cost match**: PARTIAL
- **P/T match**: N/A (not a creature)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F3 (MEDIUM): Mana cost {X}{B}{B}{B} is represented as `ManaCost { black: 3, ..Default::default() }`. The {B}{B}{B} portion is correct, but there is no indication that the card has an X cost. The ManaCost struct lacks an X field (known DSL gap), but the definition should include a TODO comment documenting this, similar to how hybrid cards document their approximation. The mana value calculation will return 3 instead of the correct "3+X" (CR 202.3e: X is 0 everywhere except the stack), so the static MV=3 is technically correct for non-stack contexts, but the X-cost nature of the card should be documented.
- **Notes**: MDFC with Land back face. Card name correctly includes both faces. Card ID uses front-face slug. abilities: vec![] correct per MDFC body-only policy (X-cost targeting + return-from-graveyard + back face all need DSL support).

## Summary
- Cards with issues: Connive // Concoct (1 HIGH, 1 MEDIUM), Agadeem's Awakening // Agadeem, the Undercrypt (1 MEDIUM)
- Clean cards: Walk-In Closet // Forgotten Cellar, Phyrexian Walker, Bottomless Pool // Locker Room

The HIGH finding on Connive // Concoct is significant: the mana cost is missing 2 hybrid mana symbols entirely, making the mana value 2 instead of the correct 4. This affects mana value checks, casting cost validation, and any interaction that cares about MV. The fix is straightforward: approximate the hybrid as `blue: 1, black: 1` and add a comment per project convention.

The MEDIUM on Agadeem's Awakening is lower severity because ManaCost correctly produces MV=3 for non-stack contexts (X=0 per CR 202.3e), but the missing TODO means a future implementer won't know the card has an X cost.
