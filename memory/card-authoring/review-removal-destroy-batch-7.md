# Card Review: Removal/Destroy Batch 7

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 2 HIGH, 1 MEDIUM, 0 LOW

## Card 1: In Garruk's Wake
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic: 7, black: 2)
- **DSL correctness**: YES
- **Findings**: None. Clean implementation using `DestroyAll` with two filters
  (creatures + planeswalkers, both `TargetController::Opponent`). Correct for
  "you don't control" semantics.

## Card 2: Elspeth, Storm Slayer
- **Oracle match**: YES
- **Types match**: YES (Legendary Planeswalker -- Elspeth, loyalty 5)
- **Mana cost match**: YES (generic: 3, white: 2)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-3): TODO claims "Token doubling replacement effect not in DSL" but
    `ReplacementModification::DoubleTokens` with `ReplacementTrigger::WouldCreateTokens`
    exists and is used by Adrix and Nev, Twincasters. The static ability should be
    implemented as `AbilityDefinition::Replacement` with `DoubleTokens`, not left as TODO.
  - F2 (MEDIUM / KI-2): The 0-loyalty ability only adds +1/+1 counters but does NOT grant
    flying. Oracle says "Those creatures gain flying until your next turn." The TODO notes
    `UntilYourNextTurn` duration is not expressible, which is correct -- that duration does
    not exist in `EffectDuration`. However, the partial implementation (counters without
    flying) produces a different game state than the real card. Per W5 policy, this ability
    should either implement both parts or be `vec![]` with a TODO. Currently it silently
    gives counters without flying, which is wrong game state. Recommend keeping as-is with
    a clear TODO comment acknowledging the partial behavior, since counters are the primary
    effect and flying is secondary/temporary. Alternatively, could use `UntilEndOfTurn` as
    a conservative approximation (shorter duration than oracle). Flagging as MEDIUM rather
    than HIGH because the counter placement is correct and the gap is duration-specific.

## Card 3: Fracture
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES (white: 1, black: 1)
- **DSL correctness**: YES
- **Findings**: None. Uses `TargetPermanentWithFilter` with `has_card_types` (plural, OR
  semantics) for artifact/enchantment/planeswalker targeting. Correct approach.

## Card 4: Saw in Half
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES (generic: 2, black: 1)
- **DSL correctness**: NO
- **Findings**:
  - F3 (HIGH / KI-2): Partial implementation produces wrong game state. The card only
    implements the destroy effect but not the "if that creature dies this way, create two
    half-stat copy tokens" clause. This means the spell just destroys a creature with no
    upside, which is strictly worse than the real card. The TODO correctly identifies that
    `CreateTokenCopy` lacks stat-modification support (halved P/T), which is a genuine DSL
    gap. Per W5 policy, the card should use `abilities: vec![]` to prevent casting until
    the full effect can be implemented. Currently it functions as a 3-mana unconditional
    creature removal spell, which is wrong game state.

## Card 5: Goblin Trashmaster
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Warrior, 3/3)
- **Mana cost match**: YES (generic: 2, red: 2)
- **DSL correctness**: YES
- **Findings**: None. Static lord effect uses correct
  `OtherCreaturesYouControlWithSubtype(Goblin)` filter at Layer 7c. Activated ability
  correctly uses `Cost::Sacrifice` with Goblin subtype filter and `TargetArtifact` target.

## Summary
- Cards with issues: Elspeth, Storm Slayer (1 HIGH, 1 MEDIUM); Saw in Half (1 HIGH)
- Clean cards: In Garruk's Wake, Fracture, Goblin Trashmaster
