# Card Review: Removal/Destroy Batch 9

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 1 HIGH, 0 MEDIUM, 2 LOW

## Card 1: Glissa Sunslayer
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Phyrexian Zombie Elf)
- **Mana cost match**: YES ({1}{B}{G})
- **P/T match**: YES (3/3)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-3): TODO claims "Effect::RemoveCounters does not exist in the DSL" but `Effect::RemoveCounter` (singular) DOES exist (card_definition.rs line 1078, effects/mod.rs line 1493). However, Glissa's mode 2 says "Remove up to three counters from target permanent" without specifying a counter type -- `RemoveCounter` requires a specific `CounterType` and a fixed `count`, so the gap is real but misstated. The TODO should be updated to accurately describe the gap: "RemoveCounter requires a specific CounterType; Glissa needs any-type removal of up to N counters." Current mode 2 is a no-op `Sequence(vec![])` which produces wrong game state (W5 violation) -- a player choosing mode 2 gets nothing. Since the other two modes work correctly and this is a genuine DSL gap (not a stale TODO), severity is HIGH for the W5 wrong-game-state aspect but the TODO itself is not fully stale. Recommend: either (a) keep the card with corrected TODO text explaining the real gap, or (b) set `abilities: vec![]` per W5 if the partial implementation is deemed unacceptable.
  - F2 (LOW): The `targets` vec on the triggered ability declares two targets (index 0 = TargetEnchantment, index 1 = TargetPermanent) but only mode 1 uses index 0. Mode 2's placeholder doesn't reference index 1. If mode 2 were implemented, the target would need to be declared when the trigger goes on the stack, before the mode is chosen. Modal triggered abilities in MTG choose modes as the trigger is put on the stack (CR 603.3c), so targets for all modes aren't required -- only the chosen mode's targets. The current structure may not match how modal triggers work in the engine.

## Card 2: Atomize
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({2}{B}{G})
- **DSL correctness**: YES
- **Findings**: None. Clean card. Correctly uses `TargetPermanentWithFilter` with `non_land: true` for "nonland permanent" (KI-1 check passes). Proliferate effect correctly follows destroy.

## Card 3: Caustic Caterpillar
- **Oracle match**: YES
- **Types match**: YES (Creature -- Insect)
- **Mana cost match**: YES ({G})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None. Clean card. Activated ability correctly uses `Cost::Sequence` with mana + `SacrificeSelf`. Target filter uses `has_card_types: vec![CardType::Artifact, CardType::Enchantment]` which has OR semantics, correctly matching "artifact or enchantment".

## Card 4: Murderous Cut
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({4}{B})
- **DSL correctness**: YES
- **Findings**: None. Clean card. Delve keyword correctly declared. Spell effect targets creature with `TargetCreature`. Structure is correct with keyword + spell ability as separate entries.

## Card 5: Reclamation Sage
- **Oracle match**: YES
- **Types match**: YES (Creature -- Elf Shaman)
- **Mana cost match**: YES ({2}{G})
- **P/T match**: YES (2/1)
- **DSL correctness**: YES
- **Findings**:
  - F3 (LOW): Oracle says "you may destroy target artifact or enchantment" -- the "you may" makes this optional. The def comment (line 17) acknowledges this is handled by "targeting being optional at resolution." This is acceptable engine behavior if the engine allows fizzling when no legal target exists, but strictly speaking "you may" means the controller can choose not to destroy even with a valid target. Minor behavioral difference; documenting for completeness.

## Summary
- Cards with issues: Glissa Sunslayer (1 HIGH, 1 LOW), Reclamation Sage (1 LOW)
- Clean cards: Atomize, Caustic Caterpillar, Murderous Cut
