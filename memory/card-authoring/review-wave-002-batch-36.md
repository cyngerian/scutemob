# Card Review: Wave 002 Batch 36

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Volatile Stormdrake
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (1U)
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword present. Hexproof-from-sources variant and ETB exchange-control + energy mechanics correctly identified as DSL gaps with accurate TODOs. Empty abilities beyond Flying is correct per W5 policy (no approximations).

## Card 2: Gilded Drake
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (1U)
- **P/T match**: YES (3/3)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword present. ETB exchange-control with conditional sacrifice correctly identified as DSL gap. TODO accurately describes the missing functionality.

## Card 3: Grief
- **Oracle match**: YES
- **Types match**: YES (Elemental Incarnation)
- **Mana cost match**: YES (2BB)
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**:
  - F1 (MEDIUM): Evoke keyword is present as `AbilityDefinition::Keyword(KeywordAbility::Evoke)`, but Evoke requires an associated alternative cost ("Exile a black card from your hand"). The engine's Evoke implementation (P2, KW Evoke) handles the sacrifice-on-ETB part, but the Evoke keyword alone does not encode the specific alternative cost. Verify that the engine's Evoke handling derives the cost from oracle text or if an `evoke_cost` field is needed on the card definition. If the engine already handles this via the keyword alone (with cost parsed elsewhere), this is acceptable. Otherwise, this Evoke keyword entry is incomplete.

## Card 4: Thieving Skydiver
- **Oracle match**: YES
- **Types match**: YES (Merfolk Rogue)
- **Mana cost match**: YES (1U)
- **P/T match**: YES (2/1)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword present. Kicker {X} noted as handled at cast time. ETB conditional control-steal with mana-value filter correctly identified as DSL gap with accurate TODO. Comment about Kicker being handled by AltCostKind::Kicker is reasonable, though technically Kicker {X} with "X can't be 0" is a more complex variant than standard Kicker.

## Card 5: Vito, Thorn of the Dusk Rose
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature - Vampire Cleric)
- **Mana cost match**: YES (2B)
- **P/T match**: YES (1/3)
- **DSL correctness**: YES
- **Findings**: None. Both abilities (life-gain trigger and activated lifelink-grant) correctly identified as DSL gaps with thorough TODOs. Empty `abilities: vec![]` is correct per W5 policy since neither ability is expressible. TODO comments are unusually detailed and accurate -- they identify the specific missing TriggerCondition and EffectTarget variants.

## Summary
- Cards with issues: Grief (1 MEDIUM -- Evoke keyword may need cost data)
- Clean cards: Volatile Stormdrake, Gilded Drake, Thieving Skydiver, Vito Thorn of the Dusk Rose
