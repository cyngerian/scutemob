# Card Review: F-4 Session 1 Batch C

**Reviewed**: 2026-03-22
**Cards**: 6
**Findings**: 2 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Tainted Field
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: YES
- **Findings**: CLEAN
  - Colorless mana ability correct (no activation_condition).
  - W/B split into two activated abilities with `Condition::ControlLandWithSubtypes(vec![SubType("Swamp")])` -- correct pattern for "Activate only if you control a Swamp."
  - Mana pool values correct: `(1,0,0,0,0,0)` = W, `(0,0,1,0,0,0)` = B. (WUBRGC order confirmed.)

## Card 2: Tainted Isle
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: YES
- **Findings**: CLEAN
  - Same pattern as Tainted Field. U = `(0,1,0,0,0,0)`, B = `(0,0,1,0,0,0)`. Correct.
  - Activation condition matches.

## Card 3: Tainted Wood
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: YES
- **Findings**: CLEAN
  - Same pattern. B = `(0,0,1,0,0,0)`, G = `(0,0,0,0,1,0)`. Correct.
  - Activation condition matches.

## Card 4: Glistening Sphere
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (`{3}` = generic 3)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-2 W5): **Corrupted ability produces 1 mana instead of 3.** The Corrupted ability uses `Effect::AddManaAnyColor` which produces 1 mana. Oracle says "Add three mana of any one color." The DSL has `Effect::AddManaChoice { player, count: EffectAmount }` which supports this exactly. Should be `Effect::AddManaChoice { player: PlayerTarget::Controller, count: EffectAmount::Fixed(3) }`. Current implementation gives free mana at 1/3 the correct rate -- wrong game state.
  - ETB tapped replacement: correct (`ReplacementModification::EntersTapped`, `is_self: true`).
  - Proliferate ETB trigger: correct.
  - Normal mana ability (1 of any color): correct.
  - Activation condition `Condition::OpponentHasPoisonCounters(3)`: correct for Corrupted.

## Card 5: Minamo, School at Water's Edge
- **Oracle match**: YES
- **Types match**: YES (Legendary Land via `supertypes`)
- **Mana cost match**: YES (none, land)
- **DSL correctness**: NO (target filter limitation)
- **Findings**:
  - F2 (MEDIUM): **Target filter cannot constrain to legendary permanents.** The TODO on line 36 correctly identifies that `TargetFilter` lacks a supertype constraint field. The current filter is `TargetFilter { ..Default::default() }` which matches ANY permanent, not just legendary ones. This is a genuine DSL gap -- `TargetFilter` has no `supertypes` or `is_legendary` field. The TODO is valid and should remain.
  - Note: The target uses `TargetPermanentWithFilter` (not `TargetCreature`), which is correct since "legendary permanent" includes all permanent types.
  - Blue mana ability: correct `(0,1,0,0,0,0)`.
  - Cost structure `Cost::Sequence([Mana({U}), Tap])`: correct.
  - `UntapPermanent` with `DeclaredTarget { index: 0 }`: correct targeting pattern.

## Card 6: Wirewood Lodge
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: NO (target too narrow)
- **Findings**:
  - F3 (HIGH): **Uses `TargetCreatureWithFilter` but oracle says "target Elf" (not "target Elf creature").** An Elf that is not a creature (e.g., a tribal Elf enchantment, or a creature that lost its creature type) would not be targetable. Should use `TargetPermanentWithFilter(TargetFilter { has_subtype: Some(SubType("Elf")), ..Default::default() })` instead. In practice this rarely matters since almost all Elves are creatures, but it is technically incorrect per oracle text.
  - Colorless mana ability: correct `(0,0,0,0,0,1)`.
  - Cost structure `Cost::Sequence([Mana({G}), Tap])`: correct.
  - `UntapPermanent` with `DeclaredTarget { index: 0 }`: correct targeting pattern.

## Summary
- **Cards with issues**: Glistening Sphere (1 HIGH), Minamo School at Water's Edge (1 MEDIUM), Wirewood Lodge (1 HIGH)
- **Clean cards**: Tainted Field, Tainted Isle, Tainted Wood

### Issue Index

| ID | Card | Severity | Pattern | Description |
|----|------|----------|---------|-------------|
| F1 | Glistening Sphere | HIGH | KI-2 (W5) | Corrupted mana ability produces 1 instead of 3; use `AddManaChoice` with `EffectAmount::Fixed(3)` |
| F2 | Minamo, School at Water's Edge | MEDIUM | Valid TODO | TargetFilter lacks supertype constraint; cannot filter to legendary permanents |
| F3 | Wirewood Lodge | HIGH | Wrong target type | `TargetCreatureWithFilter` should be `TargetPermanentWithFilter` -- oracle says "target Elf" not "target Elf creature" |
