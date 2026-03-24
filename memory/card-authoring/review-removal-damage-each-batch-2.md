# Card Review: removal-damage-each batch 2

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 2 HIGH, 2 MEDIUM, 2 LOW

## Card 1: Glint-Horn Buccaneer
- **Oracle match**: YES (matches current Scryfall oracle using "this creature")
- **Types match**: YES (Creature -- Minotaur Pirate)
- **Mana cost match**: YES ({1}{R}{R} = generic 1, red 2)
- **P/T match**: YES (2/4)
- **DSL correctness**: ACCEPTABLE (TODOs are valid)
- **Findings**:
  - F1 (LOW): TODO claims "no WheneverYouDiscard trigger in DSL" -- this is a genuine DSL gap. No `WheneverYouDiscard` trigger condition exists in TriggerCondition enum. Valid TODO.
  - F2 (LOW): Activation condition "only if attacking" -- PB-22 S1 added `activation_condition: Some(Condition::...)` but there may not be a `Condition::IsAttacking` variant. TODO is likely valid but worth verifying when authoring.

## Card 2: Brallin, Skyshark Rider
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Shaman)
- **Mana cost match**: YES ({3}{R} = generic 3, red 1)
- **P/T match**: YES (3/3)
- **DSL correctness**: ACCEPTABLE (TODOs are valid)
- **Findings**:
  - No issues beyond the same valid "WheneverYouDiscard" DSL gap as Glint-Horn Buccaneer.
  - PartnerWith keyword correctly implemented.

## Card 3: Incendiary Command
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({3}{R}{R} = generic 3, red 2)
- **DSL correctness**: NO
- **Findings**:
  - F3 (HIGH / KI-10 / W5 policy): Card uses `AbilityDefinition::Spell { effect: Effect::Nothing, ... }` which makes the card castable but does literally nothing. This is a no-op placeholder that produces wrong game state (W5/W6 policy violation). Modal spells ARE supported (PB-11 added modes, Batch 11 added Modal Choice 700.2). The card should either use proper modal spell support with `modes: Some(...)` containing the 4 modes (choose 2), OR if the individual mode effects are too complex (wheel effect for mode 4), should use `abilities: vec![]` to prevent casting.
  - F4 (MEDIUM): TODO says "requires modal spell support with per-mode targets" but modal spells with per-mode targets are supported since Batch 11. The "choose two" pattern is expressible. However, mode 4 (wheel: discard hand, draw that many) likely requires a DSL extension. The TODO is partially stale (KI-3) for the modal framework but valid for the wheel effect specifically.

## Card 4: Omnath, Locus of Creation
- **Oracle match**: NO
- **Types match**: YES (Legendary Creature -- Elemental)
- **Mana cost match**: YES ({R}{G}{W}{U} = red 1, green 1, white 1, blue 1)
- **P/T match**: YES (4/4)
- **DSL correctness**: PARTIAL (ETB correct, landfall TODO valid)
- **Findings**:
  - F5 (HIGH / KI-18): Oracle text in def uses "Omnath, Locus of Creation" but current Scryfall oracle uses just "Omnath" (updated to use shortened name per modern oracle templating). The `oracle_text` field reads: `"When Omnath, Locus of Creation enters, draw a card.\nLandfall — Whenever a land you control enters, you gain 4 life if this is the first time this ability has resolved this turn..."` but should read: `"When Omnath enters, draw a card.\nLandfall — Whenever a land you control enters, you gain 4 life if this is the first time this ability has resolved this turn..."` (just "Omnath" in the ETB line, and "you control enters" not "enters under your control").
  - F6 (MEDIUM): The ETB trigger is correctly implemented with `WhenEntersBattlefield` + `DrawCards { player: PlayerTarget::Controller, count: Fixed(1) }`. The Landfall TODO claiming "per-object ability resolution counter per turn" is a genuine DSL gap -- no resolution-count-tracking mechanism exists.

## Card 5: Warleader's Call
- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({1}{R}{W} = generic 1, red 1, white 1)
- **DSL correctness**: YES
- **Findings**:
  - No issues found. Both abilities are correctly implemented:
    - Static +1/+1 to creatures you control uses correct Layer 7c (`PtModify`), `EffectFilter::CreaturesYouControl`, `EffectDuration::WhileSourceOnBattlefield`.
    - Triggered ability uses `WheneverCreatureEntersBattlefield` with `controller: TargetController::You` filter, and `ForEach { over: EachOpponent }` with `DealDamage` -- correctly targets each opponent. The `DeclaredTarget { index: 0 }` inside `ForEach::EachOpponent` is the correct pattern (references iteration variable, not targets vec -- see `feedback_foreach_player_target.md`).

## Summary

- **Cards with issues**: Incendiary Command (HIGH: no-op placeholder), Omnath Locus of Creation (HIGH: stale oracle text; MEDIUM: valid TODO), Glint-Horn Buccaneer (LOW: valid TODOs)
- **Clean cards**: Warleader's Call, Brallin Skyshark Rider

### Action Items
1. **Incendiary Command**: Change `abilities` to `vec![]` to prevent casting with no effect (W5/W6 policy). Keep TODO explaining which modes need DSL work.
2. **Omnath, Locus of Creation**: Update `oracle_text` to match current Scryfall (use "Omnath" not "Omnath, Locus of Creation" in the ETB line; verify exact wording of landfall text).
