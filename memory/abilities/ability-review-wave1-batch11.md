# Wave 1 Batch 11 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 0 LOW

---

## Card: Wind-Scarred Crag
- **card_id**: `wind-scarred-crag` -- correct
- **name**: correct
- **types/subtypes**: Land, no subtypes -- correct
- **oracle_text**: matches Scryfall exactly
- **abilities**: Empty `vec![]` with TODOs. All three abilities are implementable in the current DSL:
  1. ETB tapped -- `ReplacementModification::EntersTapped`
  2. ETB gain 1 life -- triggered ability with `SelfEntersBattlefield` + `GainLife { amount: 1, player: PlayerTarget::Controller }`
  3. Tap for R or W -- `ManaAbility` entries
- **Verdict**: MEDIUM -- all abilities are implementable but left as skeleton TODOs

**F1 (MEDIUM)**: All three abilities (ETB tapped, ETB gain life, tap for mana) are expressible in the current DSL but were left unimplemented. This card should be fully authored.

---

## Card: Temple of Triumph
- **card_id**: `temple-of-triumph` -- correct
- **name**: correct
- **types/subtypes**: Land, no subtypes -- correct
- **oracle_text**: matches Scryfall exactly
- **abilities**: Empty `vec![]` with TODOs. All three abilities are implementable:
  1. ETB tapped -- `ReplacementModification::EntersTapped`
  2. ETB scry 1 -- triggered ability with `SelfEntersBattlefield` + `Scry { amount: 1 }`
  3. Tap for R or W -- `ManaAbility` entries
- **Verdict**: MEDIUM -- all abilities are implementable but left as skeleton TODOs

**F2 (MEDIUM)**: All three abilities (ETB tapped, ETB scry 1, tap for mana) are expressible in the current DSL but were left unimplemented.

---

## Card: Jetmir's Garden
- **card_id**: `jetmirs-garden` -- correct
- **name**: correct
- **types/subtypes**: Land -- Mountain Forest Plains -- correct (uses `types_sub` with all 3 basic land subtypes)
- **oracle_text**: matches Scryfall exactly
- **abilities**: Empty `vec![]` with TODOs. Two of three abilities are implementable:
  1. Intrinsic mana abilities from basic land subtypes (R, G, W) -- these are granted by the subtypes themselves; explicit `ManaAbility` entries would also work
  2. ETB tapped -- `ReplacementModification::EntersTapped`
  3. Cycling {3} -- Cycling IS implemented as a keyword ability in the DSL (KW Cycling)
- **Verdict**: MEDIUM -- all abilities are implementable but left as skeleton TODOs

**F3 (MEDIUM)**: All three abilities (intrinsic mana from subtypes, ETB tapped, Cycling {3}) are expressible in the current DSL but were left unimplemented.

---

## Card: Jungle Hollow
- **card_id**: `jungle-hollow` -- correct
- **name**: correct
- **types/subtypes**: Land, no subtypes -- correct
- **oracle_text**: matches Scryfall exactly
- **abilities**: Empty `vec![]` with TODOs. All three abilities are implementable (same pattern as Wind-Scarred Crag):
  1. ETB tapped -- `ReplacementModification::EntersTapped`
  2. ETB gain 1 life -- triggered ability with `SelfEntersBattlefield` + `GainLife { amount: 1, player: PlayerTarget::Controller }`
  3. Tap for B or G -- `ManaAbility` entries
- **Verdict**: MEDIUM -- all abilities are implementable but left as skeleton TODOs

**F4 (MEDIUM)**: All three abilities (ETB tapped, ETB gain life, tap for mana) are expressible in the current DSL but were left unimplemented.

---

## Card: Temple of Malice
- **card_id**: `temple-of-malice` -- correct
- **name**: correct
- **types/subtypes**: Land, no subtypes -- correct
- **oracle_text**: matches Scryfall exactly
- **abilities**: Empty `vec![]` with TODOs. All three abilities are implementable (same pattern as Temple of Triumph):
  1. ETB tapped -- `ReplacementModification::EntersTapped`
  2. ETB scry 1 -- triggered ability with `SelfEntersBattlefield` + `Scry { amount: 1 }`
  3. Tap for B or R -- `ManaAbility` entries
- **Verdict**: MEDIUM -- all abilities are implementable but left as skeleton TODOs

**F5 (MEDIUM)**: All three abilities (ETB tapped, ETB scry 1, tap for mana) are expressible in the current DSL but were left unimplemented.

---

## Summary

| Severity | Count |
|----------|-------|
| HIGH     | 0     |
| MEDIUM   | 5     |
| LOW      | 0     |

- **Cards with issues**: Wind-Scarred Crag (MEDIUM), Temple of Triumph (MEDIUM), Jetmir's Garden (MEDIUM), Jungle Hollow (MEDIUM), Temple of Malice (MEDIUM)
- **Clean cards**: (none)

**Common theme**: All 5 cards are skeleton-only despite having abilities fully expressible in the current DSL. The gain-lands (Wind-Scarred Crag, Jungle Hollow) need ETB tapped + ETB gain life + dual mana. The temple-lands (Temple of Triumph, Temple of Malice) need ETB tapped + ETB scry 1 + dual mana. Jetmir's Garden needs ETB tapped + Cycling {3} (intrinsic mana comes from subtypes). All are straightforward Phase 1 template patterns that should have been caught by `bulk_generate.py`.
