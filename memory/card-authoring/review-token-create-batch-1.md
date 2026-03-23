# Card Review: Token-Create Batch 1 (S44-S45, first 5)

**Reviewed**: 2026-03-23
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Xorn
- **Oracle match**: YES
- **Types match**: YES (Creature -- Elemental)
- **Mana cost match**: YES ({2}{R})
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**: None. TODO for replacement effect on Treasure token creation is a valid DSL gap (no `ReplacementModification` variant for "create additional tokens"). Empty abilities correct per W5 policy.

## Card 2: Adeline, Resplendent Cathar
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Knight, SuperType::Legendary present)
- **Mana cost match**: YES ({1}{W}{W})
- **P/T match**: YES (`*/4` -- `power: None, toughness: Some(4)` correct for CDA creature)
- **DSL correctness**: YES
- **Findings**: None. Vigilance keyword present. Two valid TODOs: (1) CDA for power equal to creature count is a DSL gap; (2) attack trigger creating tokens per-opponent entering attacking against specific opponents is a DSL gap (ForEach EachOpponent + per-target token placement). Empty abilities (beyond Vigilance) correct per W5 policy.

## Card 3: Oketra's Monument
- **Oracle match**: YES
- **Types match**: YES (Legendary Artifact, SuperType::Legendary present)
- **Mana cost match**: YES ({3})
- **P/T match**: N/A (non-creature, no P/T -- correct)
- **DSL correctness**: YES
- **Findings**: None. Two valid TODOs: (1) Cost reduction needs combined color+type filter ("white creature spells") but `SpellCostFilter` only supports single-variant filters (`HasColor` OR `HasCardType`, not both). Valid DSL gap. (2) `WheneverYouCastSpell` has no creature-only filter. Valid DSL gap. Empty abilities correct per W5 policy.

## Card 4: Alandra, Sky Dreamer
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Merfolk Wizard, SuperType::Legendary present)
- **Mana cost match**: YES ({2}{U}{U})
- **P/T match**: YES (2/4)
- **DSL correctness**: YES
- **Findings**: None. Two valid TODOs: (1) "draw your second card each turn" requires ordinal draw-count trigger, not supported by DSL; (2) "draw your fifth card each turn" same gap, plus dynamic hand-size buff. Empty abilities correct per W5 policy.

## Card 5: Skyknight Vanguard
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Knight)
- **Mana cost match**: YES ({R}{W})
- **P/T match**: YES (1/2)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword present. Attack trigger with CreateToken is correctly implemented: 1/1 white Soldier token with `tapped: true` and `enters_attacking: true`. Token spec matches oracle (name, types, color, P/T, entering tapped and attacking). No keywords on the token is correct (oracle specifies none). Trigger fires on self-attack (WhenAttacks) which matches "Whenever this creature attacks."

## Summary
- Cards with issues: None
- Clean cards: Xorn, Adeline Resplendent Cathar, Oketra's Monument, Alandra Sky Dreamer, Skyknight Vanguard
- All 4 cards with empty/partial abilities have valid DSL gap TODOs and comply with W5 policy
- Skyknight Vanguard is the only fully implemented card -- token spec verified correct
