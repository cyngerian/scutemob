# Card Review: Wave 2 Batch 5

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Dragon Tempest
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: CLEAN. Both triggered abilities are correctly omitted with accurate TODO comments describing the DSL gaps (creature-type-filtered ETB trigger and count-based EffectAmount with subtype filter). Empty `abilities: vec![]` follows W5 policy.

## Card 2: Tamiyo's Safekeeping
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Target requirement uses `TargetRequirement::TargetPermanent` but oracle text says "target permanent **you control**". Should be `TargetRequirement::TargetPermanentWithFilter(TargetFilter { controller: TargetController::You, ..Default::default() })`. Without this filter, the spell can illegally target opponents' permanents. (KI-1 variant — wrong target filter)
  - The spell effect itself (hexproof + indestructible via ApplyContinuousEffect + GainLife) is correctly structured: Sequence with continuous effect on DeclaredTarget{0} and GainLife to Controller.

## Card 3: Scourge of Valkas
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: CLEAN. Flying keyword is implemented. Both non-keyword abilities (Dragon ETB damage trigger and {R}: +1/+0 pump) are correctly omitted with accurate TODO comments. The TODO for the pump ability is slightly conservative — EffectFilter::Source with Activated ability may work — but erring on caution is acceptable per W5 policy.

## Card 4: Goblin Warchief
- **Oracle match**: YES
- **Types match**: YES (Goblin Warrior)
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: CLEAN. Both abilities (cost reduction for Goblin spells, haste grant to Goblins) require subtype-filtered continuous effects not in the DSL. Correctly omitted with accurate TODO comments.

## Card 5: Darksteel Mutation
- **Oracle match**: YES
- **Types match**: YES (Enchantment — Aura)
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - F2 (LOW): Missing `Keyword(KeywordAbility::Enchant)` in abilities. Aura cards typically include the Enchant keyword ability for cast-time Enchant enforcement. This is a minor gap — the card won't be targetable during casting without it, but since the complex layer effects are also omitted, the card is effectively a no-op either way. (KI-9 — missing TODO noting Enchant keyword omission)

## Summary
- Cards with issues: Tamiyo's Safekeeping (1 HIGH), Darksteel Mutation (1 LOW)
- Clean cards: Dragon Tempest, Scourge of Valkas, Goblin Warchief
