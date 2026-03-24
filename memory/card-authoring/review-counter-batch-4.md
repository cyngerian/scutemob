# Card Review: Counter Batch 4

**Reviewed**: 2026-03-22
**Cards**: 1
**Findings**: 1 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Transcendent Dragon

- **Oracle match**: YES
- **Types match**: YES (Creature -- Dragon, no supertypes needed)
- **Mana cost match**: YES ({4}{U}{U} = generic 4, blue 2)
- **P/T match**: YES (4/3)
- **Keywords match**: YES (Flash, Flying both present)
- **DSL correctness**: NO

### Findings

- **F1 (HIGH)**: Missing `intervening_if` for "if you cast it" condition. The oracle text
  says "When this creature enters, **if you cast it**, counter target spell." This is a
  classic intervening-if condition (CR 603.4). The def has `intervening_if: None` but should
  have something like `intervening_if: Some(Condition::WasCast)` or equivalent. Without this,
  the trigger fires even when the creature enters via reanimation, flicker, etc., which is
  wrong game state (W5 policy violation). The TODO comment acknowledges this complexity but
  the ability is still partially implemented rather than being `vec![]` with a TODO.

- **F2 (MEDIUM)**: Partial implementation produces wrong game state (KI-2 / W5 policy).
  The triggered ability as implemented will:
  1. Fire on any ETB (not just cast) -- missing intervening-if
  2. Counter the spell but NOT exile it instead of putting it into the graveyard
  3. NOT grant the free cast from exile

  This is a three-part failure. The counter-to-exile replacement and the free cast from
  exile are acknowledged in the TODO comment but the ability is still wired up. Per W5
  policy, a partial implementation that produces wrong game state should use `abilities: vec![]`
  (keeping only the keyword abilities for Flash and Flying) or the full triggered ability
  should be replaced with a TODO-only comment.

  **Recommended fix**: Keep Flash and Flying keywords, replace the triggered ability with
  a comment-only TODO:
  ```rust
  abilities: vec![
      AbilityDefinition::Keyword(KeywordAbility::Flash),
      AbilityDefinition::Keyword(KeywordAbility::Flying),
      // TODO: ETB triggered — counter target spell (if cast), exile instead of graveyard,
      // free cast from exile. Requires intervening-if WasCast + counter-to-exile + PlayExiledCard.
  ],
  ```

  Alternatively, if the engine supports `Condition::WasCast` for intervening-if AND
  counter-to-exile is expressible, the full ability could be implemented. But the partial
  version (counter without exile+free-cast, and without the if-cast check) is strictly
  worse than no implementation because it changes game state incorrectly.

## Summary

- **Cards with issues**: Transcendent Dragon (1 HIGH, 1 MEDIUM)
- **Clean cards**: (none)

### Issue Summary

| ID | Severity | Card | Issue |
|----|----------|------|-------|
| F1 | HIGH | Transcendent Dragon | Missing intervening-if for "if you cast it" -- triggers on all ETBs |
| F2 | MEDIUM | Transcendent Dragon | Partial impl (counter without exile+free-cast) produces wrong game state per W5 policy |
