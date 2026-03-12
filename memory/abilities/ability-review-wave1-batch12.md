# Wave 1 Batch 12 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 1 HIGH, 3 MEDIUM, 2 LOW

---

## Card: Temple of Epiphany
- **card_id**: ✓ `temple-of-epiphany`
- **name**: ✓
- **types/subtypes**: ✓ Land, no subtypes
- **oracle_text**: ✓ Exact match with Scryfall
- **abilities**: Skeleton with TODOs only. ETB tapped is implementable via `ReplacementModification::EntersTapped`. ETB scry 1 is implementable via triggered ability (`SelfEntersBattlefield` + `Effect::Scry`). Tap for U or R is implementable via `ManaAbility`.
- **Findings**:
  - F1 (MEDIUM): All three abilities are implementable in the current DSL (EntersTapped, triggered ETB Scry, dual mana ability) but left as empty TODOs. Other lands (e.g., `lonely_sandbar.rs`, `path_of_ancestry.rs`) implement EntersTapped; `viscera_seer.rs` uses `Effect::Scry`. All sister temples (Silence, Malice, etc.) have the same gap, so this is a systematic omission across all Phase 2 temples.
- **Verdict**: MEDIUM

## Card: The World Tree
- **card_id**: ✓ `the-world-tree`
- **name**: ✓
- **types/subtypes**: ✗ Missing `Legendary` supertype. Scryfall type line is "Legendary Land". The def uses `types(&[CardType::Land])` but should use `supertypes(&[SuperType::Legendary], &[CardType::Land])`.
- **oracle_text**: ✓ Exact match with Scryfall
- **abilities**: Skeleton with TODOs. ETB tapped and {T}: Add {G} are implementable. The static ability (lands have any-color mana when 6+ lands) and the WWUUBBRRGG activated search are DSL gaps (conditional static grant of mana abilities to other permanents; search for cards by creature type).
- **Findings**:
  - F1 (HIGH): Missing `Legendary` supertype. The card is "Legendary Land" but the definition only has `types(&[CardType::Land])`. Should use `supertypes(&[SuperType::Legendary], &[CardType::Land])`. This affects Commander deck validation (legendary permanents matter for various rules interactions) and any "legendary matters" cards.
  - F2 (MEDIUM): ETB tapped and {T}: Add {G} are implementable but left as TODOs. EntersTapped replacement and single-color ManaAbility are well-established DSL patterns.
  - F3 (LOW): Static conditional mana grant and WWUUBBRRGG God search are genuine DSL gaps; TODOs are appropriate for those.
- **Verdict**: HIGH

## Card: Raffine's Tower
- **card_id**: ✓ `raffines-tower`
- **name**: ✓
- **types/subtypes**: ✓ `types_sub(&[CardType::Land], &["Plains", "Island", "Swamp"])` correctly represents the triome subtypes
- **oracle_text**: ✓ Exact match with Scryfall
- **abilities**: Skeleton with TODOs. The intrinsic mana abilities from basic land subtypes ({T}: Add {W}, {U}, or {B}) are handled by the engine's basic land subtype logic and do not need explicit ability entries. ETB tapped is implementable via `ReplacementModification::EntersTapped`. Cycling {3} is implementable (KeywordAbility::Cycling exists).
- **Findings**:
  - F1 (MEDIUM): ETB tapped is implementable but left as TODO. Other triome-style lands should have EntersTapped implemented. Cycling {3} may also be implementable depending on the Cycling DSL support (it was implemented in B14).
- **Verdict**: MEDIUM

## Card: Glistening Sphere
- **card_id**: ✓ `glistening-sphere`
- **name**: ✓
- **types/subtypes**: ✓ `types(&[CardType::Artifact])` matches Scryfall (Artifact, not Land)
- **mana_cost**: ✓ `ManaCost { generic: 3, ..Default::default() }` is correct for {3}
- **oracle_text**: ✓ Exact match with Scryfall
- **abilities**: Skeleton with TODOs. "This artifact enters tapped" is implementable via EntersTapped replacement. ETB proliferate is partially implementable (Proliferate is an implemented effect). Tap for any color mana is a DSL gap (any-color mana production). Corrupted conditional mana ability is a DSL gap (conditional activation restriction based on opponent poison counters).
- **Findings**:
  - F1 (LOW): EntersTapped and ETB Proliferate are likely implementable but given the complexity of the other abilities (any-color mana, Corrupted conditional), leaving the whole card as skeleton is reasonable. The TODO comments could be more specific about which are DSL gaps vs implementable.
- **Verdict**: LOW

## Card: Temple of the Dragon Queen
- **card_id**: ✓ `temple-of-the-dragon-queen`
- **name**: ✓
- **types/subtypes**: ✓ Land, no subtypes
- **oracle_text**: ✓ Exact match with Scryfall
- **abilities**: Skeleton with TODOs. All three abilities are genuine DSL gaps: conditional ETB tapped (reveal Dragon / control Dragon), color choice on entry, and mana production of chosen color. These require reveal-from-hand, creature-type checking, and persistent color choice — none of which are in the DSL.
- **Findings**:
  - F1 (LOW): All abilities are genuine DSL gaps. TODOs are appropriate. No implementable abilities were skipped.
- **Verdict**: PASS

---

## Summary

| Severity | Count | Cards |
|----------|-------|-------|
| HIGH | 1 | The World Tree (missing Legendary supertype) |
| MEDIUM | 3 | Temple of Epiphany (implementable abilities skipped), The World Tree (implementable ETB tapped + mana skipped), Raffine's Tower (implementable ETB tapped + cycling skipped) |
| LOW | 2 | Glistening Sphere (some implementable abilities in otherwise complex card), Temple of the Dragon Queen (accurate TODO note) |

- **Cards with issues**: The World Tree (HIGH + MEDIUM), Temple of Epiphany (MEDIUM), Raffine's Tower (MEDIUM), Glistening Sphere (LOW)
- **Clean cards**: Temple of the Dragon Queen
