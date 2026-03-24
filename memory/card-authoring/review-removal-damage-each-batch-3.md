# Card Review: removal-damage-each batch 3

**Reviewed**: 2026-03-22
**Cards**: 6
**Findings**: 1 HIGH, 2 MEDIUM, 2 LOW

## Card 1: Guttersnipe
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({2}{R} = generic 2, red 1)
- **P/T match**: YES (2/2)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (MEDIUM): `WheneverYouCastSpell { during_opponent_turn: false }` lacks a spell-type filter (instant/sorcery only). The TODO documents this correctly. The trigger fires on ALL spells cast, not just instants/sorceries. This is overbroad but the card does document the limitation. Since the trigger fires too often (wrong game state), this should be `vec![]` per W5 policy (KI-2). However, the TODO is present and the overbroadness is documented, so marking MEDIUM rather than HIGH.
  - F2 (LOW): The `during_opponent_turn: false` field is misleading. The oracle text does not restrict to "during your turn only" -- it fires whenever you cast instant/sorcery regardless of whose turn it is. `during_opponent_turn: false` means the trigger does NOT require it to be an opponent's turn (i.e., it fires on any turn), which is correct behavior. No actual bug, but the field name is confusing in context.

## Card 2: Molten Gatekeeper
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature -- Golem)
- **Mana cost match**: YES ({2}{R} = generic 2, red 1)
- **P/T match**: YES (2/3)
- **DSL correctness**: YES
- **Findings**:
  - No issues. The `WheneverCreatureEntersBattlefield` with `controller: TargetController::You` correctly models "another creature you control enters" (engine auto-applies `exclude_self` for creature ETB triggers on creatures). Unearth keyword + AltCastAbility correctly provided. ForEach/EachOpponent/DealDamage pattern is correct.

## Card 3: Vindictive Vampire
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire)
- **Mana cost match**: YES ({3}{B} = generic 3, black 1)
- **P/T match**: YES (2/3)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F3 (MEDIUM, KI-9): `WheneverCreatureDies` has no filter -- fires on ALL creature deaths (any player, including self dying). Oracle says "another creature you control dies." The TODO documents this is overbroad. Per W5 policy, overbroad triggers that produce wrong game state should use `vec![]`. However, the card has a documented TODO and the overbroadness is acknowledged. Marking MEDIUM per KI-9.

## Card 4: General Kreat, the Boltbringer
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Goblin Soldier, supertype Legendary present)
- **Mana cost match**: YES ({2}{R} = generic 2, red 1)
- **P/T match**: YES (2/2)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F4 (HIGH, KI-2): First ability ("whenever one or more Goblins you control attack, create a 1/1 red Goblin creature token that's tapped and attacking") is completely missing from `abilities`. Only a TODO comment exists. The second ability (ETB creature trigger dealing 1 damage to each opponent) IS implemented. This is a **partial implementation producing wrong game state** -- the card is playable as a 2/2 that deals 1 damage on creature ETB but never creates Goblin tokens on attack. A player would put this card in their Goblin tribal deck expecting token generation and get none. Per W5 policy (KI-2), a card with half its abilities is worse than `vec![]` because it misleads the player about the card's power level and game impact.
  - F5 (LOW): The TODO for the first ability is valid -- "tapped and attacking" token creation is indeed a DSL gap (PB-22 S4 covers tapped-attacking tokens but the Goblin attack subtype filter trigger is also missing).

## Card 5: Tectonic Giant
- **Oracle match**: YES
- **Types match**: YES (Creature -- Elemental Giant)
- **Mana cost match**: YES ({2}{R}{R} = generic 2, red 2)
- **P/T match**: YES (3/4)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F6 (HIGH, KI-2): This is a **partial implementation producing wrong game state**. The oracle has a modal triggered ability with TWO trigger conditions (attacks OR becomes target of opponent's spell) and TWO modes (3 damage to each opponent OR impulse draw). The implementation only has the attack trigger with forced damage mode -- no modal choice, no "becomes target" trigger. This means:
    1. The card always deals 3 damage when attacking (player cannot choose impulse draw mode)
    2. The card never triggers when targeted by opponent's spell
    This is wrong game state -- the card is significantly weaker than it should be (missing a key defensive trigger) and removes player agency (no modal choice). Should be `vec![]` per W5 policy.

## Card 6: Creeping Bloodsucker
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire)
- **Mana cost match**: YES ({1}{B} = generic 1, black 1)
- **P/T match**: YES (1/2)
- **DSL correctness**: YES
- **Findings**:
  - No issues. `AtBeginningOfYourUpkeep` trigger with `DrainLife { amount: Fixed(1) }` correctly models "deals 1 damage to each opponent. You gain life equal to the damage dealt this way." DrainLife is the exact semantic match for this pattern.

## Summary

- **Cards with issues**: Guttersnipe (1 MEDIUM, 1 LOW), Vindictive Vampire (1 MEDIUM), General Kreat (1 HIGH, 1 LOW), Tectonic Giant (1 HIGH)
- **Clean cards**: Molten Gatekeeper, Creeping Bloodsucker

### HIGH findings requiring action:
1. **General Kreat** (F4): Partial implementation -- attack-trigger token creation missing, only ETB damage implemented. Should be `vec![]` per W5/KI-2.
2. **Tectonic Giant** (F6): Partial implementation -- only attack trigger with forced damage mode, missing "becomes target" trigger and modal choice. Should be `vec![]` per W5/KI-2.

### MEDIUM findings (documented DSL gaps):
1. **Guttersnipe** (F1): `WheneverYouCastSpell` lacks spell-type filter (instant/sorcery). Documented with TODO.
2. **Vindictive Vampire** (F3): `WheneverCreatureDies` is overbroad (fires on all deaths, not just "another creature you control"). Documented with TODO.
