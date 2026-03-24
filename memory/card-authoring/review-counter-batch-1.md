# Card Review: Counter Spells Batch 1

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 2 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Force of Will
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic: 3, blue: 2)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (MEDIUM): TODO claims "No AltCostKind variant for this pattern" for the pitch cost (pay 1 life, exile a blue card from hand). This is correct — no `AltCostKind::PitchCost` or similar exists in the DSL. The TODO is valid and the card counters the spell correctly when hard-cast. Since the partial implementation (hard-cast only) still produces correct game state for its supported mode, this is acceptable — the card just lacks the free-cast alternate mode.

**Verdict**: Acceptable. The CounterSpell effect is correct. The missing alt cost is a genuine DSL gap (no pitch-cost variant in AltCostKind). The card works correctly when hard-cast for {3}{U}{U}.

## Card 2: Mana Drain
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (blue: 2)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (MEDIUM — borderline): TODO says delayed trigger adding {C} equal to mana value requires "delayed triggers + mana-value tracking." This is a genuine DSL gap — no delayed trigger mechanism exists. The partial implementation counters the spell but does NOT add mana at next main phase. This means the card is strictly weaker than it should be (player misses the mana bonus), which is a gameplay disadvantage but not "wrong game state" in the W5 sense — the counter effect itself is correct. If this were a damage-dealing card that dealt less damage, it would be W5. For a mana-bonus miss, it's borderline acceptable since the primary effect (counter) is correct.

**Verdict**: Borderline acceptable. The counter works correctly. The missing delayed mana trigger is a genuine gap. Reclassify to HIGH if W5 policy strictly considers "missing upside for caster" as wrong game state.

## Card 3: Pact of Negation
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (all zeros = {0})
- **DSL correctness**: PARTIAL — W5 VIOLATION
- **Findings**:
  - F1 (HIGH / W5 / KI-2): The card counters a spell for {0} mana with NO upkeep payment trigger. Oracle requires "At the beginning of your next upkeep, pay {3}{U}{U}. If you don't, you lose the game." Without this trigger, the card is a **free unconditional counterspell with zero downside** — massively wrong game state. This is the textbook W5 violation: the card's entire design tension (free now, pay later or die) is missing. The TODO acknowledges the gap but the card should have `abilities: vec![]` per W5 policy. A free counter with no consequence is far worse than an uncastable card.

**Verdict**: FAIL. Must change to `abilities: vec![]` per W5 policy. A {0}-cost unconditional counter with no payment trigger produces catastrophically wrong game state.

## Card 4: Tibalt's Trickery
- **Oracle match**: YES (full oracle text matches Scryfall exactly)
- **Types match**: YES
- **Mana cost match**: YES (generic: 1, red: 1)
- **DSL correctness**: PARTIAL — W5 VIOLATION
- **Findings**:
  - F1 (HIGH / W5 / KI-2): The card counters the target spell but does NOT perform the "choose 1/2/3 at random, mill, exile until nonland, free cast" replacement effect for the spell's controller. Oracle text is very clear: after countering, the opponent gets compensation (a random free spell from their library). Without this, the card is a {1}{R} hard counter in RED — drastically wrong game state. Red does not get unconditional hard counters. The entire point of the card is that it counters but gives the opponent something back. The TODO acknowledges the complexity but the card should have `abilities: vec![]` per W5 policy.

**Verdict**: FAIL. Must change to `abilities: vec![]` per W5 policy. A {1}{R} unconditional hard counter in red with no compensation effect is fundamentally wrong — red should not have cheap hard counters.

## Card 5: Mana Leak
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic: 1, blue: 1)
- **DSL correctness**: WRONG — W5 concern
- **Findings**:
  - F1 (HIGH / W5 / KI-2): Uses `Effect::CounterSpell` (unconditional counter) but oracle says "Counter target spell **unless its controller pays {3}**." The TODO acknowledges this requires a `CounterUnlessPays` variant that doesn't exist. As implemented, this is a {1}{U} unconditional hard counter — strictly better than Counterspell itself ({U}{U}). This produces wrong game state: opponents cannot pay {3} to save their spells. The card should have `abilities: vec![]` per W5 policy.

**Verdict**: FAIL. Must change to `abilities: vec![]` per W5 policy. An unconditional counter for {1}{U} is strictly better than the oracle-intended conditional counter.

## Summary

- **Cards with issues**:
  - Pact of Negation (HIGH — free counter with no upkeep death trigger, W5 violation)
  - Tibalt's Trickery (HIGH — hard counter in red with no compensation, W5 violation)
  - Mana Leak (HIGH — unconditional counter instead of conditional "unless pays {3}", W5 violation)
  - Mana Drain (MEDIUM — missing delayed mana trigger, borderline W5)
  - Force of Will (MEDIUM — missing pitch alt cost, valid TODO, acceptable partial)
- **Clean cards**: None

### Recommended Actions

| Card | Action | Reason |
|------|--------|--------|
| Pact of Negation | `abilities: vec![]` | {0} free hard counter with no consequence = game-breaking |
| Tibalt's Trickery | `abilities: vec![]` | {1}{R} hard counter in red with no compensation = color-pie violation |
| Mana Leak | `abilities: vec![]` | Unconditional counter for {1}{U} strictly better than Counterspell |
| Mana Drain | Keep or `vec![]` | Counter part correct; missing mana bonus is player-disadvantage only |
| Force of Will | Keep as-is | Counter correct when hard-cast; missing alt cost is genuine DSL gap |

### DSL Gaps Identified (all valid)

1. **Pitch costs** (exile card from hand by color + life): no `AltCostKind` variant
2. **Delayed triggers** (next main phase, next upkeep): no delayed trigger mechanism
3. **Counter unless pays**: no `CounterUnlessPays` effect variant
4. **Random choice + complex library exile**: no random mill + exile-until pattern
