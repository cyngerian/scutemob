# Card Review: Wave 2 Batch 15

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 1 LOW

## Card 1: Molimo, Maro-Sorcerer
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic 4, green 3)
- **P/T**: Oracle says `*/*`. Def uses `Some(0)/Some(0)` as placeholder. Acceptable given DSL gap.
- **DSL correctness**: YES (Trample keyword present; dynamic P/T correctly left as TODO)
- **Findings**:
  - F1 (LOW): P/T is `Some(0)/Some(0)` but oracle is `*/*`. The card will be a 0/0 and die to SBA immediately, making it unplayable. This is the expected behavior per W5 policy (correct but incomplete > wrong behavior). TODO accurately describes the gap (Layer 7b CountLands modifier). No action needed.

## Card 2: Phyrexian Dreadnought
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature — Phyrexian Dreadnought)
- **Mana cost match**: YES (generic 1)
- **P/T match**: YES (12/12)
- **DSL correctness**: YES
- **Findings**: None. Trample implemented. ETB sacrifice trigger correctly left as TODO with accurate description. Clean card.

## Card 3: Danitha Capashen, Paragon
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Human Knight)
- **Mana cost match**: YES (generic 2, white 1)
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: None. All three keywords (FirstStrike, Vigilance, Lifelink) present. Cost reduction correctly left as TODO. Clean card.

## Card 4: Accorder's Shield
- **Oracle match**: YES
- **Types match**: YES (Artifact — Equipment)
- **Mana cost match**: YES (all zeros, i.e. {0})
- **P/T**: N/A (non-creature, correctly absent)
- **DSL correctness**: ISSUE
- **Findings**:
  - F2 (HIGH): The +0/+3 bonus is implemented as `ModifyToughness(3)` only. Oracle says "gets +0/+3" which means both power and toughness are modified (power by +0, toughness by +3). While +0 power is a no-op numerically, the standard DSL pattern for P/T modification is `ModifyPT(0, 3)` if that variant exists, or two separate effects. However, since +0 is a no-op, `ModifyToughness(3)` alone is functionally correct. **Downgrading**: on closer inspection this is functionally equivalent. No real bug. Reclassified below.
  - F2 (reclassified to MEDIUM): The Static ability grants toughness via `PtModify` layer but the power component (+0) is omitted. Functionally correct since +0 is identity, but if the DSL has `ModifyPT` this would be more precise. Minor style issue only.

## Card 5: Ogre Battledriver
- **Oracle match**: YES
- **Types match**: YES (Creature — Ogre Warrior)
- **Mana cost match**: YES (generic 2, red 2)
- **P/T match**: YES (3/3)
- **DSL correctness**: ISSUE
- **Findings**:
  - F3 (MEDIUM): The trigger says "another creature you control enters" but the TODO mentions `WheneverCreatureEntersBattlefield`. This trigger pattern should also filter for "another" (excludes self) and "you control" (controller only). The `ETBTriggerFilter` struct (added in Batch 12) supports `creature_only`, `controller_you`, and `exclude_self` fields, which would handle this correctly. The TODO does not mention `ETBTriggerFilter` as a partial solution -- it only cites `EffectTarget::TriggeringCreature` as the gap. The trigger condition itself may be expressible even if the effect targeting is not. TODO is partially inaccurate about the scope of the gap (KI-9 adjacent).

## Summary
- Cards with issues: Accorder's Shield (MEDIUM — style only), Ogre Battledriver (MEDIUM — TODO understates available DSL features)
- Cards with minor notes: Molimo, Maro-Sorcerer (LOW — 0/0 placeholder expected per policy)
- Clean cards: Phyrexian Dreadnought, Danitha Capashen, Paragon

### Revised counts after reclassification
**Findings**: 0 HIGH, 2 MEDIUM, 1 LOW
