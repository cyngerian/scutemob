# Card Review: Wave 2, Batch 37

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 2 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Crown of Skemfar
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Missing `AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature))`. All Aura card defs must include the Enchant keyword for cast-time targeting and SBA attachment enforcement. See `rancor.rs` for reference.
  - F2 (MEDIUM): TODO claims granting Reach to the attached creature is a DSL gap, but this is incorrect. The DSL supports `AbilityDefinition::Static { continuous_effect: ContinuousEffectDef { layer: EffectLayer::Ability, modification: LayerModification::AddKeyword(KeywordAbility::Reach), filter: EffectFilter::AttachedCreature, duration: EffectDuration::WhileSourceOnBattlefield } }` -- proven by Rancor granting Trample with the same pattern. The Reach grant should be implemented, not left as TODO. The count-based +1/+1 and graveyard-return abilities are legitimate DSL gaps.

## Card 2: Voidwing Hybrid
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - None. Flying and Toxic 1 are correctly implemented. The proliferate-triggered graveyard return is correctly documented as a DSL gap (no WhenYouProliferate trigger condition, no graveyard-return-self effect). Abilities are `vec![]` equivalent for the missing trigger -- the two implementable keywords are present.

## Card 3: Dragonlord Silumgar
- **Oracle match**: YES
- **Types match**: YES (Legendary supertype, Elder Dragon subtypes correct)
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - None. Flying and Deathtouch are correctly implemented. The ETB control-change effect is correctly documented as a DSL gap (no WhileYouControlSource duration, no multi-type creature-or-planeswalker target filter). TODO accurately describes both missing pieces.

## Card 4: Earthquake Dragon
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - None. Flying and Trample are correctly implemented. Both the cost-reduction static ability and the activated graveyard-return ability are correctly documented as DSL gaps. TODOs accurately describe the missing patterns (mana-value-sum cost reduction, compound activated cost with sacrifice + graveyard return).

## Card 5: Nullpriest of Oblivion
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F3 (HIGH): Missing `AbilityDefinition::Keyword(KeywordAbility::Kicker)`. The Kicker keyword is supported by the DSL (see `burst_lightning.rs`, `torch_slinger.rs`). It must be included for the engine to recognize Kicker as a valid additional cost. Without it, even once the kicked ETB trigger DSL gap is resolved, the card would not function correctly. The Lifelink and Menace keywords are correct. The kicked ETB graveyard-return trigger is a legitimate DSL gap.

## Summary
- Cards with issues: Crown of Skemfar (F1 HIGH missing Enchant keyword, F2 MEDIUM inaccurate TODO), Nullpriest of Oblivion (F3 HIGH missing Kicker keyword)
- Clean cards: Voidwing Hybrid, Dragonlord Silumgar, Earthquake Dragon
