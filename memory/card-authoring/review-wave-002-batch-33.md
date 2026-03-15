# Card Review: Wave 2 Batch 33

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Frontier Siege
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (3G correct)
- **DSL correctness**: YES
- **Findings**: None. Empty abilities with TODO correctly documents that modal ETB choice (Khans/Dragons), main-phase mana triggers, and conditional fight-on-ETB are all outside the DSL. Compliant with W5 policy.

## Card 2: Qarsi Revenant
- **Oracle match**: YES
- **Types match**: YES (Creature - Vampire, 3/3)
- **Mana cost match**: YES (1BB correct)
- **DSL correctness**: YES
- **Findings**: None. Keywords Flying, Deathtouch, Lifelink correctly implemented. Renew graveyard activated ability correctly left as TODO -- requires graveyard-zone activation, exile-self cost, and keyword counter placement, none of which are in the DSL.

## Card 3: Throatseeker
- **Oracle match**: YES
- **Types match**: YES (Creature - Vampire Ninja, 3/2)
- **Mana cost match**: YES (2B correct)
- **DSL correctness**: YES
- **Findings**: None. Empty abilities with TODO correctly describes the DSL gap: static continuous effect requiring both creature-type filter (Ninja) and combat-state filter (unblocked attacker). Compliant with W5 policy.

## Card 4: Druid Class
- **Oracle match**: YES (matches non-Alchemy version from Scryfall)
- **Types match**: YES (Enchantment - Class)
- **Mana cost match**: YES (1G correct)
- **DSL correctness**: YES
- **Findings**: None. Empty abilities with TODO correctly documents that Class level-up mechanic, extra land play, and animate-land effects are all outside the DSL. Compliant with W5 policy.

## Card 5: Multani, Yavimaya's Avatar
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature - Elemental Avatar, 0/0)
- **Mana cost match**: YES (4GG correct)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): The comment header on line 2 says "P/T = number of lands you control + lands in graveyard" which is a reasonable shorthand but technically Multani's base P/T is 0/0 with a +1/+1 buff per land, not a characteristic-defining ability setting P/T. The actual code and oracle text are correct; this is just a comment clarity issue. (KI-9 adjacent)

## Summary
- Cards with issues: Multani, Yavimaya's Avatar (1 LOW, comment only)
- Clean cards: Frontier Siege, Qarsi Revenant, Throatseeker, Druid Class
- All 5 cards correctly follow W5 policy: keywords that exist in the DSL are implemented; abilities that cannot be faithfully expressed use empty `vec![]` with accurate TODO comments describing the specific DSL gaps.
