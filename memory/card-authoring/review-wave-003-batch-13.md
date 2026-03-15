# Card Review: Wave 3 Batch 13 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 0 LOW

## Card 1: The Seedcore
- **Oracle match**: YES
- **Types match**: YES (Land -- Sphere)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - Ability 1 ({T}: Add {C}) correctly implemented.
  - Ability 2 ({T}: Add any color) implemented as AddManaAnyColor with TODO noting mana-spend restriction is not expressible. Acceptable -- the mana production works, restriction is a DSL gap.
  - Ability 3 (Corrupted pump) left as TODO comment. TODO accurately describes two DSL gaps: activated ability with targets and conditional activation based on opponent poison counters. Correct decision to omit rather than approximate.

## Card 2: Thespian's Stage
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - Ability 1 ({T}: Add {C}) correctly implemented.
  - Ability 2 (copy target land) left as TODO. Accurately describes the gap -- "become a copy of target land" copy effect is not expressible. Correct.

## Card 3: Three Tree City
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - Supertype Legendary correctly set via `supertypes()`.
  - Ability 1 ({T}: Add {C}) correctly implemented.
  - Both the ETB creature-type choice and the count-based mana ability are left as TODOs with accurate descriptions. Correct.

## Card 4: Treasure Vault
- **Oracle match**: YES
- **Types match**: YES (Artifact Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - Dual types (Artifact + Land) correctly specified via `types_sub`.
  - Ability 1 ({T}: Add {C}) correctly implemented.
  - Ability 2 ({X}{X} sacrifice for Treasures) left as TODO. Accurately describes the gap: X cost and X-scaled token creation. Correct.

## Card 5: Twilight Mire
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: YES
- **Findings**:
  - F1 (MEDIUM): The second ability's TODO says "hybrid mana costs ({B/G}) not expressible in Cost::Mana (ManaCost struct has no hybrid field)". While the hybrid cost part is true, the description also says "Triple-choice mana output also not expressible with current Choose." This is partially accurate -- the filter land ability produces a choice of three two-mana outputs, which is distinct from a simple AddMana or AddManaAnyColor. However, even if hybrid costs were added, the multi-option mana output (choose one of BB/BG/GG) would still be a gap. The TODO is directionally correct but slightly imprecise about the nature of the output gap. Low practical impact since the ability is correctly omitted entirely.
  - Ability 1 ({T}: Add {C}) correctly implemented.

## Summary
- Cards with issues: Twilight Mire (1 MEDIUM -- imprecise TODO description, no behavioral impact)
- Clean cards: The Seedcore, Thespian's Stage, Three Tree City, Treasure Vault
