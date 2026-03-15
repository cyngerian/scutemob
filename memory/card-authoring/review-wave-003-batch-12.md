# Card Review: Wave 3 Batch 12 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Tainted Field
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none — land)
- **DSL correctness**: YES
- **Findings**:
  - Clean. The {T}: Add {C} ability is correctly implemented. The conditional {W}/{B} ability is correctly left as TODO with an accurate description of the DSL gap (activation condition requiring Swamp control check). Per W5 policy, not implementing the conditional ability avoids corrupting game state.

## Card 2: Tainted Isle
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none — land)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Same pattern as Tainted Field. {T}: Add {C} implemented; conditional {U}/{B} correctly left as TODO.

## Card 3: Tainted Wood
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none — land)
- **DSL correctness**: YES
- **Findings**:
  - Clean. Same pattern as Tainted Field/Isle. {T}: Add {C} implemented; conditional {B}/{G} correctly left as TODO.

## Card 4: Takenuma, Abandoned Mire
- **Oracle match**: YES
- **Types match**: YES (Legendary Land via `supertypes()`)
- **Mana cost match**: YES (none — land)
- **DSL correctness**: YES
- **Findings**:
  - Clean. {T}: Add {B} correctly produces `mana_pool(0, 0, 1, 0, 0, 0)` (black in position 3). Channel ability correctly left as TODO with accurate description of multiple DSL gaps: discard-self cost, mill, return-from-graveyard with multi-type filter, variable cost reduction.

## Card 5: Temple of the False God
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none — land)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): The TODO comment says "producing two colorless from a single tap" is not expressible, but `mana_pool(0, 0, 0, 0, 0, 2)` would produce {C}{C}. The actual blocker is only the activation condition ("five or more lands"). The decision to leave the entire ability as TODO is still correct per W5 policy (unconditional {C}{C} would be wrong), but the TODO slightly overstates the gap.

## Summary
- Cards with issues: Temple of the False God (1 LOW — minor TODO inaccuracy)
- Clean cards: Tainted Field, Tainted Isle, Tainted Wood, Takenuma Abandoned Mire
