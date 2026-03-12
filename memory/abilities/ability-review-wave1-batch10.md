# Wave 1 Batch 10 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 1 LOW

## Card: Halimar Depths
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — all three abilities left as TODO
  - ETB tapped: MEDIUM — implementable via `ReplacementModification::EntersTapped`
  - ETB look at top 3 and reorder: LOW — top-of-library manipulation/reorder is a DSL gap
  - Tap for {U}: MEDIUM — implementable via `ManaAbility`
- Verdict: MEDIUM

## Card: Valakut, the Molten Pinnacle
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — all three abilities left as TODO
  - ETB tapped: MEDIUM — implementable via `ReplacementModification::EntersTapped`
  - Whenever a Mountain enters + intervening-if 5 other Mountains + deal 3 damage to any target: DSL gap — complex conditional landfall trigger with subtype counting and targeted damage
  - Tap for {R}: MEDIUM — implementable via `ManaAbility`
- Verdict: MEDIUM

## Card: Scoured Barrens
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — all three abilities left as TODO
  - ETB tapped: MEDIUM — implementable via `ReplacementModification::EntersTapped`
  - ETB gain 1 life: MEDIUM — likely implementable via ETB triggered ability with `GainLife { amount: 1, player: PlayerTarget::Controller }`
  - Tap for {W} or {B}: MEDIUM — implementable via `ManaAbility` (dual choice)
- Verdict: MEDIUM

## Card: Boros Garrison
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — all three abilities left as TODO
  - ETB tapped: MEDIUM — implementable via `ReplacementModification::EntersTapped`
  - ETB return a land you control to owner's hand: DSL gap — bounce-land trigger (return permanent to hand)
  - Tap for {R}{W}: MEDIUM — implementable via `ManaAbility` (produces both colors at once)
- Verdict: MEDIUM

## Card: Bloodfell Caves
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — all three abilities left as TODO
  - ETB tapped: MEDIUM — implementable via `ReplacementModification::EntersTapped`
  - ETB gain 1 life: MEDIUM — likely implementable via ETB triggered ability with `GainLife { amount: 1, player: PlayerTarget::Controller }`
  - Tap for {B} or {R}: MEDIUM — implementable via `ManaAbility` (dual choice)
- Verdict: MEDIUM

## Summary
HIGH: 0 | MEDIUM: 5 | LOW: 1

All 5 cards have correct card_id, name, types, and oracle_text. No data errors found.

The MEDIUM findings are all the same pattern: these are Phase 1 skeleton cards where implementable abilities (ETB tapped, basic/dual mana abilities, and simple ETB life gain) were left as TODO comments instead of being wired up. At minimum, ETB tapped and mana tap abilities should be expressible in the current DSL for all 5 cards. The gain-land ETB life gain (Scoured Barrens, Bloodfell Caves) is also likely expressible.

DSL gaps (LOW):
- Halimar Depths: top-of-library look-and-reorder (Scry-like but not Scry)
- Valakut: conditional landfall trigger with subtype counting + targeted damage
- Boros Garrison: bounce-land ETB (return permanent to hand)
