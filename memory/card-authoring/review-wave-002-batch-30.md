# Card Review: Wave 2 Batch 30

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Teneb, the Harvester
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (3WBG correct)
- **P/T match**: YES (6/6)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword is present. Combat damage trigger correctly left as TODO with accurate DSL gap description (no return_from_graveyard effect, no optional mana payment trigger pattern). Empty abilities beyond Flying follows W5 policy.

## Card 2: Tyvar, Jubilant Brawler
- **Oracle match**: YES (unicode minus \u2212 used for -2 ability, matches Scryfall)
- **Types match**: YES (Legendary Planeswalker -- Tyvar)
- **Mana cost match**: YES (1BG correct)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): No `loyalty` field on CardDefinition struct, so starting loyalty 3 cannot be represented. The TODO correctly documents this as a planeswalker DSL gap. Not a card def error -- the struct simply lacks the field.

## Card 3: Dragonlord Kolaghan
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elder Dragon; "Elder" and "Dragon" are separate creature subtypes)
- **Mana cost match**: YES (4BR correct)
- **P/T match**: YES (6/5)
- **DSL correctness**: YES
- **Findings**: None. Flying and Haste keywords present. Static haste-grant and name-matching triggered ability correctly left as TODOs with accurate DSL gap descriptions.

## Card 4: Thalia, Guardian of Thraben
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Soldier)
- **Mana cost match**: YES (1W correct)
- **P/T match**: YES (2/1)
- **DSL correctness**: YES
- **Findings**: None. First strike keyword present. Cost-increase static ability correctly left as TODO with accurate DSL gap description.

## Card 5: Jagged-Scar Archers
- **Oracle match**: YES
- **Types match**: YES (Creature -- Elf Archer)
- **Mana cost match**: YES (1GG correct)
- **DSL correctness**: ACCEPTABLE (see findings)
- **Findings**:
  - F2 (MEDIUM): P/T set to `Some(0)/Some(0)` but oracle text says `*/*`. The card has a characteristic-defining ability (CDA) setting P/T equal to the number of Elves you control. Using `0/0` as a placeholder means the creature will die to SBAs immediately upon entering the battlefield (0 toughness) rather than having its P/T defined by the CDA. This is a meaningful behavioral difference -- the card is effectively unplayable with `0/0`. Consider using `None/None` if the engine treats that as "to be determined by CDA", or document this as a known limitation in the TODO.
  - F3 (LOW): Both abilities (CDA for P/T and tap-to-deal-damage activated ability) correctly left as empty vec with accurate TODO descriptions of DSL gaps (no CountCreaturesYouControlWithSubtype, no EffectAmount::SelfPower, no flying TargetFilter).

## Summary
- Cards with issues: Jagged-Scar Archers (F2 MEDIUM -- 0/0 placeholder for */* makes card die to SBAs), Tyvar (F1 LOW -- no loyalty field)
- Clean cards: Teneb the Harvester, Dragonlord Kolaghan, Thalia Guardian of Thraben
