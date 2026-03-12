# Wave 1 Batch 7 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 3 MEDIUM, 3 LOW

## Card: Smoldering Crater
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — ETB tapped and tap-for-mana ARE implementable in current DSL (MEDIUM). Cycling {2} is a DSL gap (LOW).
- Verdict: MEDIUM
- F1 (MEDIUM): ETB tapped (`enters_tapped: true`) and `{T}: Add {R}` (ManaAbility) are both expressible in the DSL but left as TODO. These should be implemented.

## Card: Drifting Meadow
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — ETB tapped and tap-for-mana ARE implementable in current DSL (MEDIUM). Cycling {2} is a DSL gap (LOW).
- Verdict: MEDIUM
- F2 (MEDIUM): ETB tapped (`enters_tapped: true`) and `{T}: Add {W}` (ManaAbility) are both expressible in the DSL but left as TODO. These should be implemented.

## Card: Desert of the True
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land -- Desert, correctly uses `types_sub`)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — ETB tapped and tap-for-mana ARE implementable in current DSL (MEDIUM). Cycling {1}{W} is a DSL gap (LOW).
- Verdict: MEDIUM
- F3 (MEDIUM): ETB tapped (`enters_tapped: true`) and `{T}: Add {W}` (ManaAbility) are both expressible in the DSL but left as TODO. These should be implemented.

## Card: Sunken Palace
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land -- Cave, correctly uses `types_sub`)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — ETB tapped and tap-for-blue ARE implementable. The third ability (exile 7 from graveyard, add {U} with spell-copy rider) is a complex DSL gap.
- Verdict: LOW
- F4 (LOW): ETB tapped and `{T}: Add {U}` are implementable but left as TODO. However, the third ability (mana with spell-copy trigger) is a significant DSL gap, so leaving the whole card as skeleton is more defensible here. Noting as LOW rather than MEDIUM.

## Card: Flamekin Village
- card_id: ✓
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (matches Scryfall exactly)
- abilities: skeleton — The conditional ETB (reveal Elemental or enters tapped) is a DSL gap. Tap-for-red is implementable. `{R}, {T}: Target creature gains haste until end of turn` IS implementable in the DSL (activated ability with GrantKeyword effect). Left as TODO.
- Verdict: LOW
- F5 (LOW): Tap-for-mana and haste-granting activated ability are implementable but left as TODO. The conditional ETB (reveal from hand) is a DSL gap, so partial skeleton is somewhat justified. Noting as LOW.

## Summary
HIGH: 0 | MEDIUM: 3 | LOW: 2

- **Cards with MEDIUM issues**: Smoldering Crater, Drifting Meadow, Desert of the True -- all three have fully implementable ETB-tapped + mana ability that should not be left as TODO skeletons
- **Cards with LOW issues**: Sunken Palace, Flamekin Village -- have implementable portions left as TODO but also have genuine DSL gaps that partially justify skeleton status
- **Clean cards**: none

### Pattern Note
All 5 cards share the same issue: `enters_tapped: true` and basic ManaAbility are Phase 1 template patterns that were already implemented for 114 other lands. These 5 cards appear to have been generated as Phase 2 skeletons when they should have been Phase 1 templates.
