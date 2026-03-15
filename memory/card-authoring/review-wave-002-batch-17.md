# Card Review: Wave 2 Batch 17

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

---

## Card 1: Archetype of Endurance

- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: None. Abilities left as `vec![]` with accurate TODO describing the DSL gap (continuous effect granting/stripping hexproof). Correct per W5 policy.

## Card 2: Crashing Drawbridge

- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES
- **Findings**: None. Defender keyword is implemented. Activated ability (tap to grant haste) correctly left as TODO with accurate DSL gap description. Clean.

## Card 3: Quietus Spike

- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: N/A (Equipment, correctly absent)
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: None. All three abilities (grant deathtouch, combat damage trigger with half-life-loss, Equip {3}) correctly left unimplemented with accurate TODOs. Equipment continuous effects and HalfRoundedUp are genuine DSL gaps.

## Card 4: Teysa Karlov

- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Advisor)
- **Mana cost match**: YES
- **P/T match**: YES (2/4)
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: None. Both abilities (death-trigger doubling, token keyword grant) are genuine DSL gaps. TODOs are accurate.

## Card 5: Atraxa, Praetors' Voice

- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Phyrexian Angel Horror)
- **Mana cost match**: YES (GWUB, no generic)
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): Oracle text in the file includes the reminder text for proliferate. Scryfall oracle text also includes reminder text, so this is a match. However, the oracle_text field is longer than strictly necessary -- reminder text is optional. Not a bug, just a style note.

---

## Summary

- **Cards with issues**: Atraxa (1 LOW, cosmetic only)
- **Clean cards**: Archetype of Endurance, Crashing Drawbridge, Quietus Spike, Teysa Karlov
- **Overall**: All 5 cards are correctly authored. Oracle text, mana costs, types, subtypes, supertypes, and P/T all match Scryfall. Cards with DSL gaps correctly use empty `abilities: vec![]` (or partial implementation where possible, e.g., Crashing Drawbridge includes Defender) with accurate TODO comments. No KI pattern violations found. No overbroad triggers, no no-op placeholders, no compile errors.
