# Card Review: removal-damage-each batch 4

**Reviewed**: 2026-03-22
**Cards**: 1
**Findings**: 2 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Goblin Chainwhirler

- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Warrior, no supertypes needed)
- **Mana cost match**: YES ({R}{R}{R} = red: 3)
- **P/T match**: YES (3/3)
- **DSL correctness**: NO

- **Findings**:

  - F1 (HIGH / KI-2): **W5 policy violation -- partial implementation produces wrong game state.**
    The ETB trigger only deals 1 damage to each opponent via `ForEachTarget::EachOpponent`.
    Oracle says "it deals 1 damage to each opponent **and each creature and planeswalker they control**."
    The creature/planeswalker damage is missing entirely. In a real game, Goblin Chainwhirler
    would enter and fail to ping any opposing creatures or planeswalkers, which is a significant
    gameplay difference (e.g., it won't kill opposing 1-toughness creatures). This is a wrong-game-state
    partial implementation.

  - F2 (HIGH / KI-3): **Stale TODO -- DSL supports this pattern.** The TODO on line 19-20 says
    "Requires ForEach over opponent permanents (creatures + planeswalkers)" but this IS expressible
    using `ForEachTarget::EachPermanentMatching(TargetFilter { has_card_types: vec![CardType::Creature, CardType::Planeswalker], controller: TargetController::Opponent, ..Default::default() })`.
    Combined with `Effect::Sequence`, the full ability can be expressed as a sequence of two
    ForEach effects: one over EachOpponent (DealDamage 1) and one over EachPermanentMatching
    for creatures+planeswalkers opponents control (DealDamage 1).

  - **Fix**: Replace the single ForEach with `Effect::Sequence(vec![...])` containing both
    ForEach effects, and remove the TODO comment.

## Summary

- Cards with issues: Goblin Chainwhirler (2 HIGH)
- Clean cards: (none)
