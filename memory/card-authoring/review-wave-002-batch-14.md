# Card Review: Wave 2 Batch 14

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Alexios, Deimos of Kosmos
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (3R)
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**:
  - None. Trample keyword implemented; remaining abilities (forced attack, sacrifice restriction, owner attack restriction, each-player upkeep trigger) correctly identified as DSL gaps with accurate TODOs. Empty abilities beyond Trample is correct per W5 policy since partial behavior would be misleading.

## Card 2: Ghalta, Primal Hunger
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (10GG)
- **P/T match**: YES (12/12)
- **DSL correctness**: YES
- **Findings**:
  - None. Trample keyword implemented. Cost reduction by total creature power correctly identified as a DSL gap with TODO. Clean card.

## Card 3: Goldhound
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (R)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - None. First Strike, Menace keywords implemented. Mana ability ({T}, Sacrifice: add one mana of any color) is implemented with Cost::Sequence([Tap, Sacrifice]) and Effect::AddManaAnyColor. Clean card.

## Card 4: Temur Sabertooth
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (2GG)
- **P/T match**: YES (4/3)
- **DSL correctness**: YES
- **Findings**:
  - None. Activated ability correctly omitted with accurate TODO describing the "you may ... if you do" conditional branching DSL gap. Empty abilities vec is correct per W5 policy.

## Card 5: Scryb Ranger
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (1G)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): The TODO comment says "Effect::UntapPermanent not in DSL" but `Effect::UntapPermanent` does exist (used by other cards). The actual blocking gap is Cost::ReturnPermanentToHand (bouncing a Forest as a cost) and the once-per-turn activation restriction. The TODO is slightly inaccurate but the decision to omit the ability is still correct since the cost side cannot be expressed.

## Summary
- Cards with issues: Scryb Ranger (1 LOW -- slightly inaccurate TODO comment)
- Clean cards: Alexios Deimos of Kosmos, Ghalta Primal Hunger, Goldhound, Temur Sabertooth
