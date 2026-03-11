# Card Review: Phase 1 Batch 13 — Mana-Producing Lands

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 1 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Ancient Den
- **Oracle match**: YES (`{T}: Add {W}.`)
- **Types match**: YES (`Artifact Land` -- uses `types(&[CardType::Artifact, CardType::Land])`)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**: None. Clean card.

## Card 2: Savannah
- **Oracle match**: YES (`({T}: Add {G} or {W}.)`)
- **Types match**: YES (`Land -- Forest Plains` -- uses `types_sub(&[CardType::Land], &["Forest", "Plains"])`)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES. Mana pool args correct: G = `mana_pool(0,0,0,0,1,0)`, W = `mana_pool(1,0,0,0,0,0)`.
- **Findings**: None. Clean card.

## Card 3: Taiga
- **Oracle match**: YES (`({T}: Add {R} or {G}.)`)
- **Types match**: YES (`Land -- Mountain Forest` -- uses `types_sub(&[CardType::Land], &["Mountain", "Forest"])`)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES. Mana pool args correct: R = `mana_pool(0,0,0,1,0,0)`, G = `mana_pool(0,0,0,0,1,0)`.
- **Findings**: None. Clean card.

## Card 4: Snow-Covered Island
- **Oracle match**: YES (`({T}: Add {U}.)`)
- **Types match**: **NO** -- Missing supertypes Basic and Snow.
- **Mana cost match**: YES (None)
- **DSL correctness**: YES (mana ability correct: U = `mana_pool(0,1,0,0,0,0)`)
- **Findings**:
  - F1 (HIGH): **Missing supertypes `Basic` and `Snow`**. Oracle type line is `Basic Snow Land -- Island`. Definition uses `types_sub(&[CardType::Land], &["Island"])` which omits both supertypes. Should use `full_types(&[SuperType::Basic, SuperType::Snow], &[CardType::Land], &["Island"])`. This affects basic land search (e.g. fetchlands), snow-matters cards, and the "basic" designation which allows unlimited copies in a deck.

## Card 5: Tundra
- **Oracle match**: YES (`({T}: Add {W} or {U}.)`)
- **Types match**: YES (`Land -- Plains Island` -- uses `types_sub(&[CardType::Land], &["Plains", "Island"])`)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES. Mana pool args correct: W = `mana_pool(1,0,0,0,0,0)`, U = `mana_pool(0,1,0,0,0,0)`.
- **Findings**: None. Clean card.

## Summary
- Cards with issues: Snow-Covered Island (1 HIGH -- missing Basic+Snow supertypes)
- Clean cards: Ancient Den, Savannah, Taiga, Tundra
