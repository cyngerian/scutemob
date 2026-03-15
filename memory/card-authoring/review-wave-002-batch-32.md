# Card Review: Wave 2 Batch 32

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 2 MEDIUM, 1 LOW

## Card 1: Biting-Palm Ninja
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES (3/3)
- **DSL correctness**: NO
- **Findings**:
  - F1 (LOW): Redundant `AbilityDefinition::Keyword(KeywordAbility::Ninjutsu)` alongside `AbilityDefinition::Ninjutsu { cost }`. The `Ninjutsu` variant already encodes the keyword; the standalone keyword marker is superfluous and may cause the ability to be listed twice. Not a compile error but inconsistent with other Ninjutsu cards.
  - TODOs are accurate: menace counter ETB, reflexive "when you do" combat damage trigger with hand reveal and exile are genuine DSL gaps.

## Card 2: Rograkh, Son of Rohgahh
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Kobold Warrior)
- **Mana cost match**: YES ({0})
- **P/T match**: YES (0/1)
- **DSL correctness**: NO
- **Findings**:
  - F2 (MEDIUM): Missing `color_indicator: Some(vec![Color::Red])`. Rograkh has a red color indicator (it is a red card despite {0} mana cost). Without this, the engine will treat it as colorless, which breaks Commander color identity validation and protection/devotion interactions. Other DFC cards like `brutal_cathar.rs` correctly set `color_indicator`.

## Card 3: Bloodline Necromancer
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Wizard)
- **Mana cost match**: YES ({4}{B})
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**:
  - No issues. Lifelink keyword present. ETB return-from-graveyard with multi-subtype OR filter correctly documented as DSL gap. Abilities are `vec![]` except for Lifelink, which is correct per W5 policy (the unimplementable ETB is omitted, not approximated).

## Card 4: Etchings of the Chosen
- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({1}{W}{B})
- **P/T match**: N/A (non-creature, correctly absent)
- **DSL correctness**: YES
- **Findings**:
  - No issues. All three abilities (choose-a-type ETB replacement, lord effect, activated sacrifice-for-indestructible) correctly documented as DSL gaps. Empty abilities vec is correct per W5 policy.

## Card 5: Reckless One
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Avatar)
- **Mana cost match**: YES ({3}{R})
- **P/T match**: PARTIAL
- **DSL correctness**: NO
- **Findings**:
  - F3 (MEDIUM): P/T is `Some(0)/Some(0)` for a `*/*` characteristic-defining ability creature. While the CDA cannot be expressed in the DSL (correctly noted as a gap), a 0/0 creature will immediately die to SBAs upon entering the battlefield, making the card unplayable even in a game with Goblins present. This is functionally equivalent to the KI-3 pattern (card behaves incorrectly when cast). Consider using `power: None, toughness: None` if the engine treats None as 0 for layer 7 calculations, or document this as an explicit limitation in the TODO. The current state silently makes the card always die on ETB.

## Summary
- **Cards with issues**: Biting-Palm Ninja (1 LOW), Rograkh Son of Rohgahh (1 MEDIUM), Reckless One (1 MEDIUM)
- **Clean cards**: Bloodline Necromancer, Etchings of the Chosen
