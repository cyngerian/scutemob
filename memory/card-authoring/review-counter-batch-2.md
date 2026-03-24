# Card Review: Counter Batch 2

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 3 HIGH, 2 MEDIUM, 1 LOW

## Card 1: Mana Tithe
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (W)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-2 W5): Card uses `Effect::CounterSpell` which unconditionally counters the target spell. Oracle says "Counter target spell unless its controller pays {1}." The DSL lacks `CounterUnlessPays`, but using unconditional `CounterSpell` is **strictly stronger** than the real card -- it removes the opponent's option to pay {1} to save their spell. This produces wrong game state (free hard counter for {W} instead of a soft counter). Should be `abilities: vec![]` with TODO until `CounterUnlessPays` is added.

## Card 2: Abjure
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (U)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-2 W5): Card has a TODO noting that "sacrifice a blue permanent" as additional cost is not in the DSL, but implements `Effect::CounterSpell` anyway. This means Abjure acts as a 1-mana unconditional counterspell with **no sacrifice cost**. The sacrifice of a blue permanent is the core balancing mechanism. Without it, this is a strictly-better Counterspell at {U}. Should be `abilities: vec![]` with TODO until color-filtered sacrifice-as-additional-cast-cost is supported. Note: `Cost::Sacrifice(TargetFilter)` exists for activation costs and TargetFilter has a `colors` field, but there is no equivalent `AdditionalCastCost::Sacrifice(TargetFilter)` for spell casting costs.

## Card 3: Archmage's Charm
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (UUU)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM / KI-10): Card uses `Effect::Nothing` as a placeholder. This makes the card castable but does absolutely nothing on resolution. However, since the card does nothing at all (no partial effect), it is less dangerous than cards that do the wrong thing. The TODO correctly identifies modal + gain-control + MV-filter as gaps.
  - F2 (LOW / KI-3 partial): The TODO claims modal spells are a gap, but modal spells ARE supported in the DSL (see Izzet Charm as reference: `ModeSelection` struct with `modes` vec). Mode 1 (CounterSpell) and Mode 2 (DrawCards) are fully expressible. Mode 3 (gain control of nonland permanent with MV <= 1) requires a `GainControl` effect variant (which does not exist in the DSL) and a `max_mana_value` field on TargetFilter (also missing). So the TODO is partially stale: 2 of 3 modes could be implemented now, but Mode 3 blocks full implementation. The card should either implement modes 1+2 with a TODO on mode 3, or remain `vec![]` per W5 if partial modal implementation is considered wrong-state.

## Card 4: Access Denied
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (3UU)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-2 W5): Card implements only `Effect::CounterSpell` but omits the token creation ("Create X 1/1 colorless Thopter artifact creature tokens with flying, where X is that spell's mana value"). The counter half works correctly, but the caster never gets the Thopter tokens that are the card's primary payoff. This is a partial implementation producing wrong game state: the opponent's spell is countered but the caster doesn't get compensated with tokens. The TODO correctly identifies the gap (tracking countered spell's MV + variable-count token creation). Should be `abilities: vec![]` with TODO until the full effect can be expressed.

## Card 5: Siren Stormtamer
- **Oracle match**: YES
- **Types match**: YES (Creature -- Siren Pirate Wizard)
- **Mana cost match**: YES (U)
- **P/T match**: YES (1/1)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (MEDIUM): The activated ability (sacrifice + counter spell/ability targeting you or your creature) is left as a TODO comment, not even an empty ability entry. The TODO is valid: there is no `TargetSpellOrAbility` target requirement in the DSL, and the "that targets you or a creature you control" filter is also not expressible. Flying keyword is correctly implemented. The card is castable as a 1/1 flyer for {U}, which is a minor but non-harmful partial implementation (a 1/1 flyer with no activated ability is weaker than the real card, not stronger). Acceptable as-is since the creature body is correct and the missing ability only makes the card weaker.

## Summary
- **Cards with issues**: Mana Tithe (1 HIGH), Abjure (1 HIGH), Archmage's Charm (1 MEDIUM + 1 LOW), Access Denied (1 HIGH), Siren Stormtamer (1 MEDIUM)
- **Clean cards**: None

### HIGH findings requiring action:
1. **Mana Tithe**: Unconditional counter instead of "unless pays {1}" -- W5 violation, should be `vec![]`
2. **Abjure**: Missing sacrifice-a-blue-permanent cost makes this a 1-mana hard counter -- W5 violation, should be `vec![]`
3. **Access Denied**: Counter without Thopter token creation -- W5 violation, should be `vec![]`

### Pattern note:
All three HIGH findings share the same root cause: "counter unless pays" and "counter + secondary effect" patterns are implemented as bare `CounterSpell`, which either removes a cost the opponent should be able to pay (Mana Tithe) or omits a benefit the caster should receive (Access Denied), or omits a cost the caster should pay (Abjure). The consistent fix is `abilities: vec![]` for any counterspell variant where the DSL cannot express the full card behavior.
