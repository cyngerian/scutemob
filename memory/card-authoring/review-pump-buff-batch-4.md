# Card Review: A-20 Pump-Buff Batch 4

**Reviewed**: 2026-03-23
**Cards**: 6
**Findings**: 1 HIGH, 2 MEDIUM, 2 LOW

## Card 1: Goldnight Commander
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. Trigger uses `WheneverCreatureEntersBattlefield` with `controller: TargetController::You` filter. The "another" exclusion is handled by the harness `ETBTriggerFilter` `exclude_self` auto-injection. Effect correctly uses `CreaturesYouControl` (all, including self) with `ModifyBoth(1)` and `UntilEndOfTurn`. No issues.

## Card 2: Forerunner of the Legion
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. ETB search uses `SearchLibrary` with Vampire subtype filter, `Library` destination at `Top`, `reveal: true`, `shuffle_before_placing: true`. The oracle says "shuffle and put that card on top" which matches (shuffle first, then place on top). Second trigger uses `WheneverCreatureEntersBattlefield` with Vampire subtype filter + `TargetController::You`, targets a creature with `TargetRequirement::TargetCreature`, and applies +1/+1 via `DeclaredTarget { index: 0 }`. "Another" exclusion handled by harness. Correct.

## Card 3: Crucible of Fire
- **Oracle match**: YES
- **Types match**: YES (Enchantment, no subtypes)
- **Mana cost match**: YES
- **DSL correctness**: MINOR IMPRECISION
- **Findings**:
  - F1 (LOW): Uses `OtherCreaturesYouControlWithSubtype("Dragon")` but oracle says "Dragon creatures you control" (not "other"). Comment in code acknowledges this -- the source is an Enchantment so the "other" exclusion never fires. Functionally correct. A `CreaturesYouControlWithSubtype` filter would be more precise but doesn't exist in the DSL. No gameplay impact.

## Card 4: Kolaghan, the Storm's Fury
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Dragon)
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL (TODO)
- **Findings**:
  - F2 (MEDIUM): TODO claims "WheneverCreatureYouControlAttacks with subtype filter does not exist." This is a genuine DSL gap. The existing `WhenAttacks` is self-only (fires when this creature attacks). There is no `WheneverACreatureYouControlAttacks` variant with subtype filter. TODO is valid. However, the triggered ability is omitted entirely, meaning Kolaghan only has Flying + Dash. **W5 policy check**: Kolaghan without the attack trigger is a 4/5 flyer with dash for 5 mana. The missing trigger (all creatures get +1/+0) is significant but the card is still a reasonable creature without it. The partial implementation does not produce *wrong* game state -- it just doesn't do anything on attack. Acceptable per W5 as a TODO placeholder, not a wrong-state issue.
  - Dash implementation looks correct: `KeywordAbility::Dash` + `AltCastAbility { kind: AltCostKind::Dash, cost: {3BR} }`.

## Card 5: Mikaeus, the Unhallowed
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Zombie Cleric)
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL (TODOs)
- **Findings**:
  - F3 (HIGH): Two abilities omitted with TODOs, leaving only `Keyword(Intimidate)`. The card has three key abilities: (1) Intimidate, (2) "Whenever a Human deals damage to you, destroy it", (3) "Other non-Human creatures you control get +1/+1 and have undying." With only Intimidate, Mikaeus is a 5/5 intimidate creature for 6 mana that does nothing else. **W5 policy**: This is a **wrong game state** issue. Mikaeus is a combo piece -- the undying grant is the entire reason the card is played. An opponent seeing Mikaeus on the battlefield expects their non-Human creatures to NOT get undying. The static +1/+1 buff is also missing, meaning combat math is wrong for all other non-Human creatures. **Both TODOs are genuine DSL gaps**: (a) trigger on damage-dealt-to-you by creature with subtype doesn't exist, (b) `OtherCreaturesYouControlWithSubtype` exists for P/T but can't *exclude* a subtype (non-Human), and granting Undying as a keyword via static effect isn't supported. Per W5, abilities should be `vec![]` since the partial implementation (intimidate-only 5/5) produces misleading board state where other creatures don't get +1/+1 or undying.
  - F4 (MEDIUM): Per W5 policy, `abilities: vec![]` would be more correct than `vec![Keyword(Intimidate)]` alone. A 5/5 creature with intimidate but without its signature undying-granting ability produces wrong combat math and wrong death behavior for all other non-Human creatures the controller has. Recommend changing to `abilities: vec![]` with consolidated TODO.

## Card 6: Sword of the Paruns
- **Oracle match**: YES
- **Types match**: YES (Artifact -- Equipment)
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL (TODO)
- **Findings**:
  - F5 (LOW): TODO claims DSL lacks `Condition::EquippedCreatureIsTapped` and `EffectFilter::TappedCreaturesYouControl`. This is a genuine DSL gap -- there are no tapped/untapped conditional static effects or tapped/untapped subset filters in EffectFilter. Only `Equip` keyword is present. The card is essentially a vanilla equipment with no effects. **W5 policy check**: Without the static +2/+0 or +0/+2 effects and without the tap/untap activated ability, this is just an equip-3 equipment that does nothing. This doesn't produce *wrong* game state since there are no partial effects -- it simply does nothing. Having just the Equip keyword is acceptable as a placeholder since equipping it won't buff anything (no misleading board state).

## Summary
- **Cards with issues**: Mikaeus the Unhallowed (1 HIGH: W5 wrong-state, partial impl should be vec![]; 1 MEDIUM), Kolaghan the Storm's Fury (1 MEDIUM: genuine TODO), Crucible of Fire (1 LOW: filter imprecision), Sword of the Paruns (1 LOW: genuine TODO)
- **Clean cards**: Goldnight Commander, Forerunner of the Legion

### Action Items
1. **F3 (HIGH)**: Mikaeus -- change `abilities` from `vec![Keyword(Intimidate)]` to `vec![]` with consolidated TODO explaining all three DSL gaps (damage-by-subtype trigger, non-Human exclusion filter, keyword granting via static). The intimidate-only implementation produces wrong combat expectations.
2. **F4 (MEDIUM)**: Same fix as F3 -- consolidate into single `vec![]`.
3. **F2 (MEDIUM)**: Kolaghan -- no change needed, TODO is valid and partial impl is acceptable per W5 (flying+dash without attack trigger is not misleading).
4. **F1, F5 (LOW)**: No changes needed. Crucible workaround is functionally correct. Sword placeholder is not misleading.
