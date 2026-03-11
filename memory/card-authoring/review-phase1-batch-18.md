# Card Review: Phase 1 Batch 18 — MDFCs (Pathways + Battle/Regent)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Barkchannel Pathway // Tidechannel Pathway
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Findings**: None

## Card 2: Blightstep Pathway // Searstep Pathway
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Findings**: None

## Card 3: Bloomvine Regent // Claim Territory
- **Oracle match**: YES
- **Types match**: YES (Creature — Dragon)
- **Mana cost match**: YES (3GG)
- **P/T match**: YES (4/5)
- **DSL correctness**: YES
- **Findings**: None
- **Note**: Front face has Flying + triggered ability. `abilities: vec![]` is correct per W5 policy since the triggered ability (Dragon ETB lifegain) requires trigger filtering not yet in the DSL. Flying alone could be expressed but partial implementation is worse than empty.

## Card 4: Bridgeworks Battle // Tanglespan Bridgeworks
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES (2G)
- **DSL correctness**: YES
- **Findings**: None
- **Note**: Sorcery with fight effect. `abilities: vec![]` is correct — the card's effect is a spell effect (not an ability), and fight targeting is not yet expressible.

## Card 5: Clearwater Pathway // Murkwater Pathway
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None — land)
- **DSL correctness**: YES
- **Findings**: None

## Summary
- Cards with issues: none
- Clean cards: Barkchannel Pathway, Blightstep Pathway, Bloomvine Regent, Bridgeworks Battle, Clearwater Pathway
