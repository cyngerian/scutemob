# Card Review: Wave 2 Batch 10

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Faeburrow Elder
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({1}{G}{W})
- **P/T match**: YES (0/0)
- **DSL correctness**: YES
- **Findings**: None. Vigilance keyword correctly implemented. Two TODOs accurately describe DSL gaps for the CDA (+1/+1 per color among permanents) and the multi-color mana ability. Base 0/0 P/T is correct as printed.

## Card 2: Lightning, Army of One
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Soldier)
- **Mana cost match**: YES ({1}{R}{W})
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**: None. First strike, trample, lifelink correctly implemented as keywords. Stagger ability TODO accurately describes the DSL gap (damage-doubling replacement effect with "until your next turn" duration scoped to a specific player and their permanents).

## Card 3: Ram Through
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({1}{G})
- **P/T match**: N/A (non-creature)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): **W5 policy violation -- approximated effect should be empty abilities.** The card uses `AbilityDefinition::Spell` with two `TargetCreature` targets, but oracle text requires "target creature you control" for target[0] and "target creature you don't control" for target[1]. The current `TargetRequirement::TargetCreature` does not enforce controller restrictions, meaning a player could target their own creature with both slots or an opponent's creature for the power source. Per W5 policy ("no simplifications -- wrong/approximate behavior corrupts game state"), the abilities should be `vec![]` with a TODO explaining that `TargetRequirement` lacks "you control" / "you don't control" variants. The trample excess-damage clause is also not modeled. File: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/ram_through.rs`, line 21-33.

## Card 4: Blade Historian
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Cleric)
- **Mana cost match**: APPROXIMATED -- {R/W}{R/W}{R/W}{R/W} hybrid cost rendered as `red: 2, white: 2`. ManaCost struct lacks hybrid mana support. Documented in comment at line 4.
- **P/T match**: YES (2/3)
- **DSL correctness**: YES
- **Findings**: None. Empty abilities correct per W5 policy. TODO accurately describes the DSL gap for the conditional continuous effect granting double strike to attacking creatures you control.

## Card 5: Berserk
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({G})
- **P/T match**: N/A (non-creature)
- **DSL correctness**: YES
- **Findings**: None. Empty abilities correct per W5 policy. Three TODOs accurately describe DSL gaps: (1) timing restriction "only before the combat damage step", (2) EffectAmount::TargetPower for +X/+0 where X is target's power, (3) delayed conditional destroy at next end step.

## Summary
- Cards with issues: Ram Through (1 HIGH -- approximated effect violates W5 policy)
- Clean cards: Faeburrow Elder, Lightning Army of One, Blade Historian, Berserk
