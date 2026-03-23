# Card Review: Token Create Batch 4 (S50-S52)

**Reviewed**: 2026-03-23
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Mogg War Marshal
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Warrior)
- **Mana cost match**: YES ({1}{R})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None

Notes: Echo {1}{R} correctly implemented as `KeywordAbility::Echo(ManaCost { generic: 1, red: 1 })`. The oracle says "When this creature enters or dies" which is a single triggered ability, but splitting it into two separate triggers (WhenEntersBattlefield + WhenDies) is the correct DSL representation since the engine handles them as distinct trigger conditions. Token spec is correct: 1/1 red Goblin creature token. Legal-but-wrong checks all pass -- tokens go to controller (correct for "create").

## Card 2: Hordeling Outburst
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({1}{R}{R})
- **DSL correctness**: YES
- **Findings**: None

Notes: Spell effect creates 3 Goblin tokens (count: 3). Token spec correct: 1/1 red Goblin creature. Clean implementation.

## Card 3: Dragon Fodder
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({1}{R})
- **DSL correctness**: YES
- **Findings**: None

Notes: Spell effect creates 2 Goblin tokens (count: 2). Token spec correct: 1/1 red Goblin creature. Clean implementation.

## Card 4: Talrand's Invocation
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({2}{U}{U})
- **DSL correctness**: YES
- **Findings**: None

Notes: Spell effect creates 2 Drake tokens (count: 2). Token spec correct: 2/2 blue Drake creature with Flying keyword. Clean implementation.

## Card 5: Squee, Dubious Monarch
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Goblin Noble, supertype Legendary present)
- **Mana cost match**: YES ({2}{R})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: None

Notes: Haste keyword present. Attack trigger correctly uses WhenAttacks with a 1/1 red Goblin token that has `tapped: true` and `enters_attacking: true` -- matches oracle "tapped and attacking". The TODO for the graveyard alt-cast ability is valid: the pattern requires paying {3}{R} AND exiling four other cards from graveyard as an alternate casting cost from the graveyard zone, which is a compound cost+zone combination not covered by existing AltCostKind variants. This is a legitimate DSL gap. Legal-but-wrong checks: token goes to controller (correct), token enters tapped and attacking (correct for "that's tapped and attacking").

## Summary
- Cards with issues: (none)
- Clean cards: Mogg War Marshal, Hordeling Outburst, Dragon Fodder, Talrand's Invocation, Squee, Dubious Monarch
- All 5 cards have correct oracle text, types, mana costs, and token specifications.
- Squee's TODO for graveyard alt-cast is a legitimate DSL gap (not stale).
