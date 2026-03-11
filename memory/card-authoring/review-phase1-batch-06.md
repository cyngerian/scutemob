# Card Review: Phase 1 Batch 06 (Mana-Producing Lands + Marble Diamond)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Glacial Fortress
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Conditional ETB ("unless you control a Plains or an Island") correctly documented as TODO with DSL gap comment. Mana ability produces W (pos0) or U (pos1) — correct.

## Card 2: Sunpetal Grove
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Conditional ETB ("unless you control a Forest or a Plains") correctly documented as TODO with DSL gap comment. Mana ability produces G (pos4) or W (pos0) — correct.

## Card 3: Marble Diamond
- **Oracle match**: YES
- **Types match**: YES (Artifact — not a land)
- **Mana cost match**: YES (`generic: 2, ..Default::default()`)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Unconditional enters-tapped (no TODO needed — fully expressible). Mana ability produces W (pos0) — correct. Correctly typed as Artifact with mana cost {2}.

## Card 4: Rockfall Vale
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Conditional ETB ("unless you control two or more other lands") correctly documented as TODO with DSL gap comment. Mana ability produces R (pos3) or G (pos4) — correct.

## Card 5: Sandsteppe Citadel
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Unconditional enters-tapped (no TODO needed). Tri-land with 3 choices: W (pos0), B (pos2), G (pos4) — all correct. Effect::Choose with 3 options properly models the tri-color mana ability.

## Summary
- Cards with issues: (none)
- Clean cards: Glacial Fortress, Sunpetal Grove, Marble Diamond, Rockfall Vale, Sandsteppe Citadel

All 5 cards are correctly authored. The mana_pool argument order (W, U, B, R, G, C) is correct in every case. The three conditional-ETB lands properly document the DSL gap with TODO comments. Marble Diamond correctly has Artifact type and mana cost {2}. Sandsteppe Citadel correctly offers all three color choices.
