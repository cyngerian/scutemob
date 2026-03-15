# Card Review: Wave 3 Batch 14 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 2 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Unclaimed Territory
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: YES
- **Findings**: None. First ability (colorless mana) is implemented. Second ability is correctly omitted with an accurate TODO describing the ETB creature-type choice and mana restriction as a DSL gap. Per W5 policy this is acceptable -- the implemented portion (tap for colorless) is correct and the missing portion is documented.

## Card 2: Underground River
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Second activated ability implements U/B mana choice but omits the "This land deals 1 damage to you" rider. Per W5 policy, approximate behavior that changes game outcomes is not allowed -- free painless colored mana is strictly better than the real card. The second ability should be removed entirely (leaving only the colorless tap ability) with the TODO explaining the DSL gap. The current implementation corrupts game state by giving the controller colored mana without the damage cost.

## Card 3: Urza's Saga
- **Oracle match**: YES
- **Types match**: NO
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: YES (abilities are `vec![]` with accurate TODO)
- **Findings**:
  - F2 (HIGH): Subtypes are `&["Urza's Saga"]` (one subtype string) but should be `&["Urza's", "Saga"]` (two separate subtypes). "Urza's" is a land subtype and "Saga" is an enchantment subtype per CR 205.3. This matters because the Saga mechanic keys off the "Saga" subtype, and "Urza's" is relevant for cards that reference Urza's lands.

## Card 4: Vault of the Archangel
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: YES
- **Findings**: None. Colorless mana ability is implemented correctly. The activated ability granting deathtouch+lifelink is omitted with an accurate TODO. Per W5 policy this is acceptable -- the implemented portion is correct.

## Card 5: Viridescent Bog
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, correct for land)
- **DSL correctness**: MOSTLY
- **Findings**:
  - F3 (MEDIUM): Mana pool order check -- `mana_pool(0, 0, 1, 0, 1, 0)` with signature `(white, blue, black, red, green, colorless)` gives black=1, green=1, colorless=0. This matches the oracle text "{B}{G}" so the mana production is correct. However, the cost uses `Cost::Sequence` wrapping `Cost::Mana` and `Cost::Tap` -- verify that the engine handles `Cost::Sequence` for activated abilities correctly. If `Cost::Sequence` is not supported for activated ability costs, this would silently fail. Marking MEDIUM pending engine verification.

  **Update**: On reflection, `Cost::Sequence` is used in other card defs for multi-part costs (e.g., Vault of the Archangel pattern would need it too). The implementation looks structurally correct. Downgrading concern but keeping as MEDIUM note for engine-level verification.

## Summary
- Cards with issues: Underground River (F1 HIGH -- painless mana violates W5 policy), Urza's Saga (F2 HIGH -- wrong subtypes), Viridescent Bog (F3 MEDIUM -- Cost::Sequence verification)
- Clean cards: Unclaimed Territory, Vault of the Archangel
