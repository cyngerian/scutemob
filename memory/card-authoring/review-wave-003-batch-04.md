# Card Review: Wave 3 Batch 04 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Crucible of the Spirit Dragon
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None for land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): TODO comment on line 15-16 claims "{T}: Add {C} is NOT in the oracle text" but Scryfall oracle text clearly includes "{T}: Add {C}." as the first ability. The first ability ({T}: Add {C}) is a standard colorless mana ability and IS expressible in the DSL. It should be implemented, not left as TODO. The other two abilities (storage counters, X-removal mana with Dragon restriction) are legitimately inexpressible.
  - F2 (MEDIUM): Because the TODO incorrectly dismisses the first ability, the card has `abilities: vec![]` when it should have at least the colorless tap ability implemented. This makes the land produce no mana at all, which is incorrect behavior per W5 policy (wrong behavior is worse than empty abilities only when the card literally has zero expressible abilities -- here, one of three is expressible).

## Card 2: Darkwater Catacombs
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES
- **Findings**: None. Clean implementation of {1}, {T}: Add {U}{B}. Mana pool order correct (0W, 1U, 1B, 0R, 0G, 0C). Cost::Sequence with generic 1 + Tap is correct.

## Card 3: Deserted Temple
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES
- **Findings**: None. First ability ({T}: Add {C}) correctly implemented. Second ability ({1},{T}: Untap target land) correctly left as TODO with accurate description of the DSL gap (no UntapPermanent effect). Mana pool order correct (0W, 0U, 0B, 0R, 0G, 1C).

## Card 4: Eiganjo, Seat of the Empire
- **Oracle match**: YES
- **Types match**: YES (Legendary supertype correctly set via `full_types`)
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES
- **Findings**: None. First ability ({T}: Add {W}) correctly implemented. Channel ability correctly left as TODO with accurate description (Channel keyword + variable cost reduction not in DSL). Mana pool order correct (1W, 0U, 0B, 0R, 0G, 0C).

## Card 5: Fetid Heath
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES
- **Findings**: None. First ability ({T}: Add {C}) correctly implemented. Second ability ({W/B},{T}: three-choice mana output) correctly left as TODO with accurate description of DSL gaps (hybrid cost + three-way mana choice). Mana pool order correct (0W, 0U, 0B, 0R, 0G, 1C).

## Summary
- Cards with issues: Crucible of the Spirit Dragon (1 HIGH, 1 MEDIUM -- colorless tap ability incorrectly omitted due to wrong TODO comment)
- Clean cards: Darkwater Catacombs, Deserted Temple, Eiganjo Seat of the Empire, Fetid Heath
