# Card Review: Removal-Bounce Batch 1

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 2 LOW

---

## Card 1: Reprieve

- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES — `{1}{W}` = generic 1, white 1
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (LOW): TODO claims "no TargetSpell requirement for bounce-to-hand" — this is a valid DSL gap. There is no bounce/return-to-hand Effect variant in the DSL, and while `TargetRequirement::TargetSpell` exists, there is no effect to move a spell from the stack to a player's hand. The TODO is legitimate.
  - F2 (MEDIUM): The card currently implements only `DrawCards` as its spell effect, meaning it is castable and draws a card without bouncing anything. This is a **W5 policy violation (KI-2)** — the card does the wrong thing (draws a card for {1}{W} with no bounce). Should use `abilities: vec![]` until bounce-spell is expressible.

## Card 2: Cryptic Command

- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES — `{1}{U}{U}{U}` = generic 1, blue 3
- **DSL correctness**: YES (placeholder)
- **Findings**:
  - No issues. Uses `Effect::Nothing` as placeholder, which makes the card castable but does nothing. This is acceptable for a complex four-mode spell where none of the modes are fully expressible as a combination (counter + bounce + tap-all + draw requires multi-modal with per-mode targets). `Effect::Nothing` is less harmful than partial implementation since it doesn't produce wrong game state — it just does nothing.

## Card 3: Walker of Secret Ways

- **Oracle match**: YES
- **Types match**: YES — Creature, Human Ninja
- **Mana cost match**: YES — `{2}{U}` = generic 2, blue 1
- **P/T match**: YES — 1/2
- **DSL correctness**: PARTIAL
- **Findings**:
  - No issues found. Has both `Keyword(Ninjutsu)` and `Ninjutsu { cost }` definitions (satisfying KI-6). The two TODOs are valid: (1) "look at that player's hand" is hidden information reveal with no DSL support, (2) the activated ability needs subtype-filtered targeting (Ninja) + return-to-hand effect + activation condition (your turn only). While `activation_condition` exists in the DSL (PB-22 S1), the return-to-hand effect does not, so the TODO remains valid.

## Card 4: Chulane, Teller of Tales

- **Oracle match**: YES
- **Types match**: YES — Legendary Creature, Human Druid (supertype Legendary present)
- **Mana cost match**: YES — `{2}{G}{W}{U}` = generic 2, green 1, white 1, blue 1
- **P/T match**: YES — 2/4
- **DSL correctness**: YES (placeholder)
- **Findings**:
  - No issues found. Vigilance keyword is correctly implemented. Both TODOs are valid: (1) `WheneverYouCastSpell` has no spell-type filter (creature only), confirmed in card_definition.rs — it only has `during_opponent_turn: bool`, no type filter. (2) The activated ability requires return-to-hand effect which doesn't exist in the DSL.

## Card 5: Press the Enemy

- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES — `{2}{U}{U}` = generic 2, blue 2
- **DSL correctness**: YES (placeholder)
- **Findings**:
  - No issues found. Uses `Effect::Nothing` as placeholder. The TODO is valid — this card needs dual-zone targeting (spell on stack OR nonland permanent on battlefield, opponent-controlled) plus bounce-to-hand plus free-cast based on mana value comparison. None of these are expressible in the current DSL.

---

## Summary

- **Cards with issues**: Reprieve (1 MEDIUM — W5 partial implementation produces wrong game state)
- **Clean cards**: Cryptic Command, Walker of Secret Ways, Chulane Teller of Tales, Press the Enemy
- **All TODOs validated**: Every TODO in this batch claims a genuine DSL gap. No stale TODOs found.
- **Common DSL gap**: All 5 cards need a "return to hand" / bounce effect that does not exist in the DSL. This is the primary blocker for the removal-bounce archetype.

### Action Items
- **F2 (Reprieve, MEDIUM)**: Change `abilities` to `vec![]` — the current `DrawCards`-only implementation violates W5 policy by letting Reprieve draw a card without bouncing the target spell.
