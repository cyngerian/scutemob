# Card Review: Wave 2 Batch 20

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Timeline Culler
- **Oracle match**: YES
- **Types match**: YES (Creature -- Drix Warlock)
- **Mana cost match**: YES ({B}{B})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: None. Haste keyword implemented. Warp ability correctly left as TODO with accurate DSL gap description (no AltCostKind::Warp, no cast-from-exile loop, no end-step exile replacement).

## Card 2: Vampire Hexmage
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Shaman)
- **Mana cost match**: YES ({B}{B})
- **P/T match**: YES (2/1)
- **DSL correctness**: YES
- **Findings**: None. First strike keyword implemented. Activated ability (sacrifice self to remove all counters from target permanent) correctly left as TODO -- DSL lacks RemoveAllCounters effect and self-sacrifice activation cost.

## Card 3: Wonder
- **Oracle match**: YES
- **Types match**: YES (Creature -- Incarnation)
- **Mana cost match**: YES ({3}{U})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword implemented. Graveyard static ability (grant flying while in graveyard + control an Island) correctly left as TODO -- DSL has no graveyard-zone static effect with land-type condition.

## Card 4: Vorinclex, Monstrous Raider
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Phyrexian Praetor)
- **Mana cost match**: YES ({4}{G}{G})
- **P/T match**: YES (6/6)
- **DSL correctness**: YES
- **Findings**: None. Trample and Haste keywords implemented. Counter-doubling and counter-halving replacement effects correctly left as TODO -- DSL has no counter-modification replacement effect.

## Card 5: Kodama of the East Tree
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Spirit)
- **Mana cost match**: YES ({4}{G}{G})
- **P/T match**: YES (6/6)
- **DSL correctness**: YES
- **Findings**: None. Reach and Partner keywords implemented. Triggered ability (permanent enters -> put permanent card from hand with equal/lesser MV) correctly left as TODO -- DSL lacks mana-value comparison filter and self-exclusion for "wasn't put with this ability" intervening-if condition.

## Summary
- Cards with issues: (none)
- Clean cards: Timeline Culler, Vampire Hexmage, Wonder, Vorinclex Monstrous Raider, Kodama of the East Tree
