# Wave 1 Batch 8 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 2 LOW

---

## Card: Cult Conscript

- **card_id**: `cult-conscript` -- CORRECT
- **name**: "Cult Conscript" -- CORRECT
- **types/subtypes**: `creature_types(&["Warrior", "Skeleton"])` -- WRONG ORDER. Scryfall type line is "Creature - Skeleton Warrior". Should be `&["Skeleton", "Warrior"]`.
- **mana_cost**: `ManaCost { black: 1 }` -- CORRECT ({B})
- **oracle_text**: CORRECT -- matches Scryfall exactly
- **power/toughness**: 2/1 -- CORRECT
- **abilities**: Empty `vec![]` (skeleton with TODOs)
  - F1 (MEDIUM): "This creature enters tapped" IS implementable in current DSL using `AbilityDefinition::Replacement { trigger: ReplacementTrigger::WouldEnterBattlefield { filter: ObjectFilter::Any }, modification: ReplacementModification::EntersTapped, is_self: true }` (see lonely_sandbar.rs for pattern). Should be implemented, not left as TODO.
  - F2 (LOW): Activated ability "{1}{B}: Return from graveyard" is a graveyard-zone activated ability with a condition -- genuine DSL gap (return_from_graveyard + activation restriction). TODO is accurate.
  - F3 (LOW): Subtype order is `["Warrior", "Skeleton"]` but Scryfall has "Skeleton Warrior". Cosmetic, no functional impact.
- **Verdict**: MEDIUM (enters-tapped implementable but skipped; subtype order wrong)

---

## Card: Desert of the Fervent

- **card_id**: `desert-of-the-fervent` -- CORRECT
- **name**: "Desert of the Fervent" -- CORRECT
- **types/subtypes**: `types_sub(&[CardType::Land], &["Desert"])` -- CORRECT
- **mana_cost**: None -- CORRECT (land)
- **oracle_text**: CORRECT -- matches Scryfall exactly
- **abilities**: Empty `vec![]` (skeleton with TODOs)
  - F4 (MEDIUM): "This land enters tapped" IS implementable (EntersTapped replacement, see lonely_sandbar.rs).
  - F5 (MEDIUM): "{T}: Add {R}" IS implementable as `AbilityDefinition::Activated { cost: Cost::Tap, effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 1, 0, 0) }, timing_restriction: None }`.
  - F6 (LOW): "Cycling {1}{R}" IS implementable as `AbilityDefinition::Keyword(KeywordAbility::Cycling)` + `AbilityDefinition::Cycling { cost: ManaCost { generic: 1, red: 1, ..Default::default() } }`. Should be implemented.
- **Verdict**: MEDIUM (all three abilities are implementable but left as TODO)

---

## Card: Spinerock Knoll

- **card_id**: `spinerock-knoll` -- CORRECT
- **name**: "Spinerock Knoll" -- CORRECT
- **types/subtypes**: `types(&[CardType::Land])` -- CORRECT (no subtypes on Scryfall)
- **mana_cost**: None -- CORRECT (land)
- **oracle_text**: CORRECT -- matches Scryfall exactly
- **abilities**: Empty `vec![]` (skeleton with TODOs)
  - F7 (MEDIUM): "This land enters tapped" and "{T}: Add {R}" are both implementable (same patterns as above). Should not be left as TODO.
  - Hideaway 4 is implemented in the engine (KeywordAbility::Hideaway exists, AbilityDefinition for it exists). However, the Hideaway land's conditional play ability ("{R}, {T}: play exiled card if 7+ damage dealt") may be a DSL gap for the condition check. TODO is reasonable for the Hideaway + conditional play portion.
- **Verdict**: MEDIUM (enters-tapped and mana ability implementable but skipped; Hideaway conditional play is genuine DSL gap)

---

## Card: Arena of Glory

- **card_id**: `arena-of-glory` -- CORRECT
- **name**: "Arena of Glory" -- CORRECT
- **types/subtypes**: `types(&[CardType::Land])` -- CORRECT
- **mana_cost**: None -- CORRECT (land)
- **oracle_text**: CORRECT -- matches Scryfall exactly
- **abilities**: Empty `vec![]` (skeleton with TODOs)
  - F8 (MEDIUM): "{T}: Add {R}" IS implementable. Should not be left as TODO.
  - "Enters tapped unless you control a Mountain" is a conditional ETB -- existing cards (frostboil_snarl.rs, deathcap_glade.rs) note this as a DSL gap (`ReplacementModification::EntersTapped has no condition field`). TODO is accurate for this.
  - Exert + mana-tracking haste grant is a genuine DSL gap. TODO is accurate.
- **Verdict**: MEDIUM (basic mana ability implementable but skipped)

---

## Card: Minas Tirith

- **card_id**: `minas-tirith` -- CORRECT
- **name**: "Minas Tirith" -- CORRECT
- **types/subtypes**: `supertypes(&[SuperType::Legendary], &[CardType::Land])` -- CORRECT
- **mana_cost**: None -- CORRECT (land)
- **oracle_text**: CORRECT -- matches Scryfall exactly
- **abilities**: Empty `vec![]` (skeleton with TODOs)
  - F9 (MEDIUM): "{T}: Add {W}" IS implementable. Should not be left as TODO.
  - "Enters tapped unless you control a legendary creature" is a conditional ETB -- DSL gap (no condition field on EntersTapped). TODO is accurate.
  - "{1}{W}, {T}: Draw a card" with activation restriction ("if you attacked with two or more creatures this turn") is a DSL gap (no activation condition tracking). TODO is accurate.
- **Verdict**: MEDIUM (basic mana ability implementable but skipped)

---

## Summary

| Card | Verdict | Issues |
|------|---------|--------|
| Cult Conscript | MEDIUM | enters-tapped implementable; subtype order wrong |
| Desert of the Fervent | MEDIUM | all 3 abilities implementable (enters-tapped, tap-for-mana, cycling) |
| Spinerock Knoll | MEDIUM | enters-tapped + tap-for-mana implementable; Hideaway conditional is DSL gap |
| Arena of Glory | MEDIUM | tap-for-mana implementable; conditional ETB + Exert are DSL gaps |
| Minas Tirith | MEDIUM | tap-for-mana implementable; conditional ETB + activation restriction are DSL gaps |

- **Cards with issues**: Cult Conscript, Desert of the Fervent, Spinerock Knoll, Arena of Glory, Minas Tirith
- **Clean cards**: (none)

**Common pattern**: All 5 cards leave basic implementable abilities (enters-tapped replacement, tap-for-mana activated ability, cycling) as empty TODOs. This is a systematic issue with the Phase 2 skeleton generator -- it produces empty `abilities: vec![]` even for abilities that have well-established DSL patterns. Desert of the Fervent is the most egregious case where ALL three abilities are implementable.

HIGH: 0 | MEDIUM: 5 | LOW: 2
