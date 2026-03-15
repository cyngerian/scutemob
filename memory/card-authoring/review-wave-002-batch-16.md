# Card Review: Wave 2 Batch 16

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Cathar's Shield
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic 0)
- **DSL correctness**: YES
- **Findings**: None -- clean. Equipment pattern (static toughness + static keyword grant + activated equip) is correct and consistent with other equipment cards.

## Card 2: Spidersilk Net
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic 0)
- **DSL correctness**: YES
- **Findings**: None -- clean. Same equipment pattern as Cathar's Shield, correctly grants reach instead of vigilance.

## Card 3: Tombstone Stairwell
- **Oracle match**: YES
- **Types match**: YES (World Enchantment via `supertypes()` helper)
- **Mana cost match**: YES (generic 2, black 2)
- **DSL correctness**: YES (within limits)
- **Findings**:
  - F1 (LOW): Dual CumulativeUpkeep entries (Keyword + AbilityDefinition::CumulativeUpkeep) is consistent with established convention (Mystic Remora uses the same pattern), so not a bug.
  - F2 (LOW): TODO comments accurately describe 4 DSL gaps: each-player-upkeep trigger, count-based token creation, token-origin tracking for destruction, and combined end-step/LTB trigger. All are genuine gaps and correctly prevent incomplete behavior per W5 policy.

## Card 4: Balan, Wandering Knight
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Cat Knight)
- **Mana cost match**: YES (generic 2, white 2)
- **P/T match**: YES (3/3)
- **DSL correctness**: YES (within limits)
- **Findings**:
  - F3 (MEDIUM): The TODO comment for the conditional double strike says "Condition::AttachedEquipmentCount threshold static -- not in DSL". This is accurate. However, the TODO for the activated ability says "attach all Equipment you control to Balan" and describes a mass-equip effect. This is also accurate. Both abilities are correctly omitted per W5 policy (empty abilities would silently fail). No incorrect behavior shipped. Severity MEDIUM only because this card has two significant abilities beyond first strike that are unimplemented, making it a very incomplete representation -- but that is the correct approach given DSL limitations.

## Card 5: Indomitable Archangel
- **Oracle match**: YES (em dash encoded as `\u{2014}`, matches Scryfall)
- **Types match**: YES (Creature -- Angel)
- **Mana cost match**: YES (generic 2, white 2)
- **P/T match**: YES (4/4)
- **DSL correctness**: YES (within limits)
- **Findings**:
  - F4 (LOW): TODO accurately describes the Metalcraft conditional static gap. The comment mentions "EffectFilter::CreaturesYouControl filtered to artifacts" which is slightly imprecise (should be "artifacts you control", not creatures), but the TODO correctly identifies the DSL gap and the ability is correctly omitted.

## Summary
- Cards with issues: Balan, Wandering Knight (1 MEDIUM -- correct incomplete representation, no fix needed), Tombstone Stairwell (2 LOW), Indomitable Archangel (1 LOW)
- Clean cards: Cathar's Shield, Spidersilk Net
- No action required -- all cards follow W5 policy correctly. The identified issues are accurate TODO documentation and expected DSL gaps.
