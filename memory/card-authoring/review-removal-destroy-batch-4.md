# Card Review: Removal/Destroy Batch 4

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 1 HIGH, 2 MEDIUM, 0 LOW

## Card 1: Vindicate
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES — {1}{W}{B} = generic:1 white:1 black:1
- **DSL correctness**: YES
- **Findings**: CLEAN. `TargetPermanent` is correct for "destroy target permanent" (no filter needed). Sorcery type, spell effect with DestroyPermanent, single declared target at index 0. All correct.

## Card 2: Cathar Commando
- **Oracle match**: YES
- **Types match**: YES — Creature - Human Soldier
- **Mana cost match**: YES — {1}{W} = generic:1 white:1
- **P/T match**: YES — 3/1
- **DSL correctness**: YES
- **Findings**: CLEAN. Flash keyword present. Activated ability with Cost::Sequence of Mana({1}) + SacrificeSelf. Target filter uses `has_card_types: vec![Artifact, Enchantment]` which has OR semantics per the TargetFilter doc comment -- correctly matches "target artifact or enchantment." DestroyPermanent with DeclaredTarget index 0. All correct.

## Card 3: Untimely Malfunction
- **Oracle match**: YES
- **Types match**: YES — Instant
- **Mana cost match**: YES — {1}{R} = generic:1 red:1
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (HIGH / KI-2): **W5 policy violation -- partial modal implementation produces wrong game state.** Mode 0 (destroy target artifact) works correctly, but modes 1 and 2 are empty `Effect::Sequence(vec![])`. A player who casts this choosing mode 1 or 2 gets a resolved spell that does nothing. The card should use `abilities: vec![]` since 2 of 3 modes are non-functional. The TODO comments about CantBlock and ChangeTarget/RetargetSpell DSL gaps are valid -- neither pattern exists in the engine.
  - F2 (MEDIUM / KI-9 variant): **Targets list is incomplete for modal spell.** The targets vec has only `TargetArtifact` (mode 0). Modes 1 and 2 have no target requirements declared. If the card were castable, mode selection with mode 1 or 2 would have no targets to declare, which is structurally wrong. (Moot given F1 recommendation to use `vec![]`.)

## Card 4: Season of Gathering
- **Oracle match**: YES
- **Types match**: YES — Sorcery
- **Mana cost match**: YES — {4}{G}{G} = generic:4 green:2
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**: CLEAN. The card uses `abilities: vec![]` with a detailed TODO explaining why: the Phyrexian-mana-as-mode-budget system, repeatable modes, dynamic draw count, and player choice of type are all genuinely not expressible in the current DSL. The TODO is valid and the vec![] approach is correct per W5 policy. No stale gap claims (KI-3 check passed).

## Card 5: Collective Resistance
- **Oracle match**: YES
- **Types match**: YES — Instant
- **Mana cost match**: YES — {1}{G} = generic:1 green:1
- **DSL correctness**: YES
- **Findings**:
  - F3 (MEDIUM): **Escalate mode_costs field is None but should not matter.** Escalate uses `AbilityDefinition::Escalate { cost }` (present at line 23) rather than `mode_costs` on ModeSelection (which is for Spree). The Escalate keyword marker is also present. ModeSelection has min_modes:1, max_modes:3, allow_duplicate_modes:false -- all correct for "choose one or more." Target indices 0/1/2 map correctly to the three modes. Mode 2 grants Hexproof + Indestructible via two ApplyContinuousEffect with EffectLayer::Ability and UntilEndOfTurn duration on DeclaredTarget index 2. All structurally correct.

  Actually, on closer inspection: CLEAN. No findings.

## Summary
- **Cards with issues**: Untimely Malfunction (1 HIGH, 1 MEDIUM)
- **Clean cards**: Vindicate, Cathar Commando, Season of Gathering, Collective Resistance

### Recommended Actions
1. **Untimely Malfunction** (F1, HIGH): Change `abilities` to `vec![]` since 2/3 modes cannot be implemented. The working mode 0 alone does not justify a partial implementation -- a player could legally choose mode 1 or 2 and get a no-op spell.
2. **Untimely Malfunction** (F2, MEDIUM): Moot if F1 is applied. If the card is kept partial, add placeholder target requirements for modes 1 and 2.
