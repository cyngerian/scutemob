# Card Review: Token-Create Batch 2 (S44-S46)

**Reviewed**: 2026-03-23
**Cards**: 5
**Findings**: 2 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Hero of Bladehold
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-3): TODO claims "Battle cry keyword not in DSL KeywordAbility enum" but `KeywordAbility::BattleCry` EXISTS (see `types.rs:485`, used in `signal_pest.rs`). The card is missing `AbilityDefinition::Keyword(KeywordAbility::BattleCry)`. This means the card currently has NO battle cry -- each other attacking creature does NOT get +1/+0. **Wrong game state**: combat damage is incorrect when Hero attacks with other creatures.
  - F2 (MEDIUM): The token-creation trigger correctly uses `TriggerCondition::WhenAttacks` and the token spec correctly has `tapped: true, enters_attacking: true, count: 2` with white Soldier 1/1. This part is correct.

## Card 2: Grave Titan
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-3): TODO on line 41 claims "WhenDeclaredAttacker trigger not in DSL" but `TriggerCondition::WhenAttacks` EXISTS and is widely used (leonin_warleader.rs, goldspan_dragon.rs, etc.). The card is missing the attack trigger for token creation. Oracle says "enters OR attacks" -- the ETB half is implemented but the attack half is not. **Wrong game state**: Grave Titan does not create tokens when attacking, only on ETB.
  - Fix: Add a second `AbilityDefinition::Triggered` with `TriggerCondition::WhenAttacks` and the same `Effect::CreateToken` spec (two 2/2 black Zombie tokens).

## Card 3: Goblin Instigator
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None. Clean card. ETB trigger creates 1/1 red Goblin token. All fields correct.

## Card 4: Doomed Traveler
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None. Clean card. Dies trigger creates 1/1 white Spirit with flying. Token spec correctly includes `KeywordAbility::Flying` in keywords. All fields correct.

## Card 5: Empty the Warrens
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None. Clean card. Spell effect creates two 1/1 red Goblin tokens. Storm keyword present via `KeywordAbility::Storm`. All fields correct.

## Summary
- Cards with issues: Hero of Bladehold (1 HIGH), Grave Titan (1 HIGH)
- Clean cards: Goblin Instigator, Doomed Traveler, Empty the Warrens

### Fix Priority
1. **Hero of Bladehold** -- Add `AbilityDefinition::Keyword(KeywordAbility::BattleCry)` to abilities vec. The stale TODO should be removed.
2. **Grave Titan** -- Add a second triggered ability with `TriggerCondition::WhenAttacks` and the same CreateToken spec. The stale TODO should be removed.

Both are KI-3 (stale TODO claiming DSL gap that was already closed). Both produce wrong game state because abilities that ARE expressible are missing.
