# Card Review: Removal-Minus Batch 1

**Reviewed**: 2026-03-22
**Cards**: 4
**Findings**: 2 HIGH, 1 MEDIUM, 1 LOW

---

## Card 1: Umezawa's Jitte

- **Oracle match**: YES
- **Types match**: YES (Legendary Artifact -- Equipment, supertype present)
- **Mana cost match**: YES ({2} generic)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-2 W5): Partial implementation produces wrong game state. The card has an Equip keyword + a duplicate Activated equip ability (lines 22-29), but the two core abilities (combat damage trigger placing charge counters, modal activated ability removing a charge counter) are TODO. However, because Equip is implemented, the card is castable and equippable with no actual Jitte functionality. The Equip keyword alone is arguably acceptable (just an equipment with no abilities), but the duplicate Activated ability on lines 23-29 is redundant with the Keyword Equip and should be removed. Overall, the card should use `abilities: vec![]` until the combat damage trigger and modal activated ability can be expressed, OR keep only the Equip keyword if equip-only is considered acceptable minimal behavior.
  - F2 (MEDIUM): Duplicate Equip definition. Lines 22-29 define both `AbilityDefinition::Keyword(KeywordAbility::Equip)` AND an explicit `AbilityDefinition::Activated` with `Cost::Mana(ManaCost { generic: 2 })` and `Effect::Nothing`. The keyword already handles equip; the Activated ability is redundant and may cause the card to show two equip activations.
  - F3 (LOW): TODOs on lines 30-33 are valid -- the DSL lacks an "equipped creature deals combat damage" trigger condition and a modal activated ability with `Cost::RemoveCounter`. These are genuine gaps.

## Card 2: Tragic Slip

- **Oracle match**: YES
- **Types match**: YES (Instant, no supertypes)
- **Mana cost match**: YES ({B})
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-2 W5): Partial implementation produces wrong game state. The card always applies -1/-1, but when Morbid is active (a creature died this turn), it should apply -13/-13 instead. The partial -1/-1 implementation means the card will underperform every time Morbid is relevant -- this is a gameplay-affecting error. Per W5 policy, the card should use `abilities: vec![]` until a `Condition::CreatureDiedThisTurn` (or equivalent Morbid condition) is added to the DSL.
  - F2 (LOW / KI-19): TODO on line 14 is valid. The DSL lacks a `CreatureDiedThisTurn` condition. This is a genuine gap -- no matching `Condition` variant exists in card_definition.rs.

## Card 3: Dismember

- **Oracle match**: YES
- **Types match**: YES (Instant, no supertypes)
- **Mana cost match**: YES ({1}{B/P}{B/P} -- generic: 1, phyrexian: two Single(Black))
- **DSL correctness**: YES
- **Findings**: None. Clean card. The Phyrexian mana encoding correctly uses `PhyrexianMana::Single(ManaColor::Black)` twice. The -5/-5 continuous effect with `ModifyBoth(-5)` is correct. Targeting is correct (`TargetCreature`). Duration is `UntilEndOfTurn`. No issues found.

## Card 4: Drown in Ichor

- **Oracle match**: YES
- **Types match**: YES (Sorcery, no supertypes)
- **Mana cost match**: YES ({1}{B})
- **DSL correctness**: YES
- **Findings**: None. Clean card. The Sequence of -4/-4 continuous effect followed by Proliferate correctly matches the oracle text. Targeting is correct. The oracle text includes the reminder text for Proliferate which matches Scryfall.

---

## Summary

- **Cards with issues**: Umezawa's Jitte (1 HIGH, 1 MEDIUM, 1 LOW), Tragic Slip (1 HIGH, 1 LOW)
- **Clean cards**: Dismember, Drown in Ichor

### Issue Breakdown

| ID | Severity | Card | Pattern | Description |
|----|----------|------|---------|-------------|
| F1 | HIGH | Umezawa's Jitte | KI-2 W5 | Equip-only partial impl; core abilities (charge counters, modal activated) missing |
| F2 | MEDIUM | Umezawa's Jitte | Duplicate | Redundant Activated equip alongside Keyword Equip |
| F3 | LOW | Umezawa's Jitte | KI-19 | Valid TODOs for genuine DSL gaps |
| F4 | HIGH | Tragic Slip | KI-2 W5 | Always -1/-1; should be -13/-13 when Morbid active |
| F5 | LOW | Tragic Slip | KI-19 | Valid TODO for CreatureDiedThisTurn condition gap |
