# Card Review: Wave 2 Batch 34

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 2 MEDIUM, 1 LOW

## Card 1: Twilight Prophet
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Flying keyword present. Ascend and upkeep drain trigger correctly documented as DSL gaps. `abilities: vec![]` would be the W5 policy result but Flying is expressible and included, which is correct.

## Card 2: Zealous Conscripts
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Haste keyword present. ETB gain-control trigger correctly documented as targeted_trigger DSL gap.

## Card 3: Hammer of Nazahn
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Equip {4} is expressible in the DSL. Multiple equipment cards (Swiftfoot Boots, Whispersilk Cloak, Sword of Vengeance) implement Equip via `AbilityDefinition::Activated` with `cost: Cost::Mana(...)` and `effect: Effect::AttachEquipment`. The TODO on line 27 incorrectly claims "Equip {4} activated ability not in DSL." The Equip ability should be implemented, not left as a TODO.
  - F2 (MEDIUM): The equipped-creature continuous effect (+2/+0 and indestructible) is partially expressible. Other equipment cards implement P/T bonuses and keyword grants via `Effect::ModifyPowerToughness` and `Effect::GrantKeyword` on `EffectFilter::EquippedCreature`. The TODO on line 25-26 should be more specific about what IS expressible (the stat buff and indestructible grant) vs. what isn't (the ETB auto-attach trigger).
  - F3 (LOW): The ETB trigger for auto-attaching equipment when any equipment enters is correctly identified as a DSL gap (watches for "self or another Equipment" entering).

## Card 4: Tatyova, Steward of Tides
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Both abilities correctly documented as DSL gaps: continuous flying grant to land-creatures requires type-combination filter, and landfall animate-land requires targeted_trigger + count_threshold + animate effect.

## Card 5: Emrakul, the Promised End
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F4 (MEDIUM): Protection from instants may be partially expressible. The engine has `KeywordAbility::Protection` and `ProtectionFilter` (DEBT framework in protection.rs). If `ProtectionFilter` supports card-type-based protection (from instants), this should be implemented rather than left as a TODO. Needs verification against the ProtectionFilter enum variants.
  - F5 (LOW): TODO accuracy is good. Cast trigger for controlling an opponent and extra turn mechanics are correctly identified as major DSL gaps.

## Summary
- Cards with issues: Hammer of Nazahn (1 HIGH, 1 MEDIUM, 1 LOW), Emrakul the Promised End (1 MEDIUM, 1 LOW)
- Clean cards: Twilight Prophet, Zealous Conscripts, Tatyova Steward of Tides
