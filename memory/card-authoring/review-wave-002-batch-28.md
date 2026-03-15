# Card Review: Wave 2 Batch 28

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Tainted Observer
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES (2/3)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Flying and Toxic 1 correctly implemented as keywords. The triggered ability ("whenever another creature you control enters, you may pay {2}. If you do, proliferate") is correctly deferred with a TODO citing the DSL gap: no exclude-self filter on creature-enters trigger, and no optional-cost-payment-at-resolution pattern. This is the correct W5 policy (empty rather than overbroad per KI-2).

## Card 2: Florian, Voldaren Scion
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Vampire Noble)
- **Mana cost match**: YES ({1}{B}{R})
- **P/T match**: YES (3/3)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: First strike keyword correct. The triggered ability is correctly deferred with TODO. Three distinct DSL gaps identified: no postcombat main phase trigger condition, no opponents-life-lost-this-turn amount tracking, and no look-at-top-X-exile-one effect pattern. All accurately described.

## Card 3: Harald, King of Skemfar
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elf Warrior)
- **Mana cost match**: YES ({1}{B}{G})
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Menace keyword correct. ETB ability correctly deferred. The TODO accurately describes the DSL gap: no look-at-top-N with multi-subtype OR filter (Elf OR Warrior OR Tyvar). The oracle text uses the paper version (top five), not the Alchemy rebalanced version (top seven), which is correct.

## Card 4: Zurgo and Ojutai
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Orc Dragon)
- **Mana cost match**: YES ({2}{U}{R}{W})
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Flying and Haste keywords correct. Two abilities correctly deferred with TODOs: (1) conditional hexproof tied to ETB timestamp ("as long as it entered this turn"), and (2) pack-hunting-style trigger ("whenever one or more Dragons you control deal combat damage") with look-at-top-3 and optional bounce. Both TODOs accurately describe the DSL gaps.

## Card 5: Torch Courier
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin)
- **Mana cost match**: YES ({R})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Haste keyword correct. The activated ability ("sacrifice this creature: another target creature gains haste until end of turn") is correctly deferred with TODO citing the activated_ability_targets DSL gap. This is the right call -- the DSL cannot express targeted activated abilities.

## Summary
- Cards with issues: none
- Clean cards: Tainted Observer, Florian Voldaren Scion, Harald King of Skemfar, Zurgo and Ojutai, Torch Courier

All 5 cards are correctly authored. Oracle text matches Scryfall exactly in all cases. Mana costs, types, subtypes, supertypes, and P/T values are all correct. Keywords that are expressible (Flying, Haste, Menace, First Strike, Toxic 1) are implemented. Complex abilities that cannot be expressed in the current DSL are properly left out with accurate TODO comments describing the specific gaps. No KI pattern violations found.
