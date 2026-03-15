# Card Review: Wave 2 Batch 12

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 1 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Mockingbird
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (partial -- `{X}{U}` represented as `blue: 1` with TODO for X, acceptable per W5 policy)
- **DSL correctness**: YES
- **Findings**: None. Flying keyword is implemented. The ETB copy effect is correctly identified as a DSL gap and abilities are left as `vec![Flying]` only. The TODO accurately describes what is missing (ETB replacement copy with X-cost mana value filter + type/keyword overlay).

## Card 2: Boromir, Warden of the Tower
- **Oracle match**: YES
- **Types match**: YES (Legendary supertype, Human Soldier subtypes)
- **Mana cost match**: YES (`{2}{W}` = generic 2, white 1)
- **DSL correctness**: YES
- **Findings**: None. Vigilance keyword is implemented. Both the triggered counter-spell ability (opponent casts with no mana spent) and the activated sacrifice ability (grant indestructible + Ring tempts) are correctly identified as DSL gaps with accurate TODO descriptions. Abilities vec contains only Vigilance, which is correct per W5 policy.

## Card 3: Vampire Cutthroat
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (`{B}` = black 1)
- **DSL correctness**: YES
- **Findings**: None. Both Skulk and Lifelink keywords are implemented. Clean card.

## Card 4: Bloodmark Mentor
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (`{1}{R}` = generic 1, red 1)
- **DSL correctness**: YES
- **Findings**: None. The continuous keyword-grant effect (red creatures you control have first strike, Layer 6 with color filter) is correctly identified as a DSL gap. Abilities left as `vec![]` per W5 policy. TODO is accurate.

## Card 5: Invisible Stalker
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (`{1}{U}` = generic 1, blue 1)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): TODO claims `CantBeBlocked` is a DSL gap, but `KeywordAbility::CantBeBlocked` exists and is fully enforced in `combat.rs` (lines 628-635, 884-887). The abilities vec should include `AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked)`. See also `whispersilk_cloak.rs` which uses `KeywordAbility::CantBeBlocked` successfully. The card is currently missing functional unblockable behavior.

## Summary
- Cards with issues: Invisible Stalker (1 HIGH -- missing CantBeBlocked keyword that exists in DSL)
- Clean cards: Mockingbird, Boromir Warden of the Tower, Vampire Cutthroat, Bloodmark Mentor
