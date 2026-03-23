# Card Review: A-20 Pump-Buff Batch 1

**Reviewed**: 2026-03-23
**Cards**: 6
**Findings**: 0 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Return of the Wildspeaker
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (4G)
- **DSL correctness**: N/A (abilities are placeholder)
- **Findings**:
  - TODO on line 25: "EffectAmount::GreatestPowerAmong(filter) does not exist" -- VALID. No `GreatestPower` variant in DSL. Confirmed via grep: only two files reference this string, both card defs.
  - TODO on line 28: "EffectFilter for non-Human creatures you control does not exist" -- VALID. `EffectFilter` enum has no subtype-exclusion variant (only `OtherCreaturesYouControlWithSubtype` for inclusion). No `NonHuman` or `exclude_subtype` anywhere in engine.
  - Modal structure (min=1, max=1, two modes) is correct for "Choose one".
  - Both modes use `Effect::Nothing` as placeholder -- acceptable given genuine DSL gaps.

## Card 2: Mirari's Wake
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (3GW)
- **DSL correctness**: YES (for the implemented portion)
- **Findings**:
  - F1 (MEDIUM): **W5 policy concern** -- the +1/+1 static is implemented but the mana doubling triggered ability is not. This is a **partial implementation that does not produce wrong game state** -- the creature buff is correct on its own, and the mana ability is purely additive (missing it does not make the card do the wrong thing, it just does less). Acceptable under W5 policy since the partial effect is directionally correct.
  - TODO on line 23: "Mana doubling triggered ability not in DSL" -- VALID. No `ManaDoubl`, `WheneverTapLand`, or `TapForMana` trigger condition exists in the DSL. The mana-production trigger pattern is genuinely unsupported.
  - Static ability correctly uses `EffectLayer::PtModify`, `LayerModification::ModifyBoth(1)`, `EffectFilter::CreaturesYouControl`, `EffectDuration::WhileSourceOnBattlefield`.

## Card 3: Stromkirk Captain
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Soldier)
- **Mana cost match**: YES (1BR)
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: CLEAN
  - FirstStrike keyword present.
  - Two static abilities correctly split across Layer 7c (+1/+1) and Layer 6 (grant FirstStrike).
  - Both use `OtherCreaturesYouControlWithSubtype(SubType("Vampire"))` -- correctly excludes self and filters by subtype.
  - Duration `WhileSourceOnBattlefield` is correct for static abilities on permanents.

## Card 4: Goblin Chieftain
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin)
- **Mana cost match**: YES (1RR)
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**: CLEAN
  - Haste keyword present.
  - Two static abilities correctly split across Layer 7c (+1/+1) and Layer 6 (grant Haste).
  - Both use `OtherCreaturesYouControlWithSubtype(SubType("Goblin"))` -- correctly excludes self and filters by subtype.
  - Duration `WhileSourceOnBattlefield` is correct.

## Card 5: Triumph of the Hordes
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES (2GG)
- **DSL correctness**: YES
- **Findings**: CLEAN
  - Three `ApplyContinuousEffect` in a `Sequence`: +1/+1 (Layer 7c), Trample (Layer 6), Infect (Layer 6).
  - All use `EffectFilter::CreaturesYouControl` -- correct (oracle says "creatures you control", not "other").
  - All use `EffectDuration::UntilEndOfTurn` -- correct.
  - No targets required (affects all your creatures) -- `targets: vec![]` is correct.
  - Legal-but-wrong check: no token creation, no player targeting, no self-exclusion issues.

## Card 6: Tainted Strike
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES (B)
- **DSL correctness**: YES
- **Findings**: CLEAN
  - Two `ApplyContinuousEffect` in a `Sequence`: +1/+0 (Layer 7c via `ModifyPower(1)`), Infect (Layer 6).
  - Both use `EffectFilter::DeclaredTarget { index: 0 }` -- correct for targeted spell.
  - `targets: vec![TargetRequirement::TargetCreature]` -- correct (oracle says "target creature").
  - `EffectDuration::UntilEndOfTurn` -- correct.
  - Legal-but-wrong check: targets any creature (not just yours) -- matches oracle. No multiplayer issues.

## Summary
- **Cards with issues**: Mirari's Wake (1 MEDIUM -- partial implementation, acceptable under W5)
- **Clean cards**: Return of the Wildspeaker, Stromkirk Captain, Goblin Chieftain, Triumph of the Hordes, Tainted Strike
- **All TODOs validated as genuine DSL gaps** -- none are stale (KI-3 check passed)
- **No legal-but-wrong patterns found** -- all PlayerTargets, filters, and self-exclusions are correct
- **No KI pattern violations found**
