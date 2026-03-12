# Wave 1 Batch 3 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 2 MEDIUM, 2 LOW

## Card: Spymaster's Vault
- card_id: ✓ `spymasters-vault`
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (exact match)
- abilities: skeleton — DSL gaps: conditional ETB tapped ("unless you control a Swamp"), connive X (variable connive based on creatures died this turn)
- F1 (MEDIUM): The `{T}: Add {B}` mana ability is expressible in the current DSL (standard ManaAbility) but is left as TODO. Could be implemented now even though the other two abilities are genuine DSL gaps.
- Verdict: MEDIUM

## Card: Forgotten Cave
- card_id: ✓ `forgotten-cave`
- name: ✓
- types/subtypes: ✓ (Land, no subtypes)
- oracle_text: ✓ (exact match)
- abilities: skeleton — DSL gap: Cycling keyword
- F2 (MEDIUM): The ETB tapped and `{T}: Add {R}` abilities are expressible in the current DSL but are left as TODO. Only Cycling is a genuine DSL gap. The implementable abilities could be authored now.
- Verdict: MEDIUM

## Card: Zagoth Triome
- card_id: ✓ `zagoth-triome`
- name: ✓
- types/subtypes: ✓ (Land -- Swamp Forest Island; all 3 subtypes present)
- oracle_text: ✓ (exact match)
- abilities: skeleton — DSL gap: Cycling {3}
- F3 (LOW): Subtype order in def is `["Island", "Swamp", "Forest"]`, Scryfall canonical order is "Swamp Forest Island". Not mechanically significant but cosmetically inconsistent.
- Note: The mana abilities are intrinsic to the basic land subtypes and do not need explicit ability entries. ETB tapped is implementable but Cycling is a genuine DSL gap.
- Verdict: LOW

## Card: Indatha Triome
- card_id: ✓ `indatha-triome`
- name: ✓
- types/subtypes: ✓ (Land -- Plains Swamp Forest; all 3 subtypes present, order matches Scryfall)
- oracle_text: ✓ (exact match)
- abilities: skeleton — DSL gap: Cycling {3}
- Note: Mana abilities intrinsic to basic land subtypes. ETB tapped implementable but Cycling is the blocking DSL gap.
- Verdict: PASS

## Card: Ketria Triome
- card_id: ✓ `ketria-triome`
- name: ✓
- types/subtypes: ✓ (Land -- Forest Island Mountain; all 3 subtypes present)
- oracle_text: ✓ (exact match)
- abilities: skeleton — DSL gap: Cycling {3}
- F4 (LOW): Subtype order in def is `["Island", "Mountain", "Forest"]`, Scryfall canonical order is "Forest Island Mountain". Not mechanically significant but cosmetically inconsistent.
- Note: Mana abilities intrinsic to basic land subtypes. ETB tapped implementable but Cycling is the blocking DSL gap.
- Verdict: LOW

## Summary
HIGH: 0 | MEDIUM: 2 | LOW: 2

- **Cards with issues**: Spymaster's Vault (MEDIUM -- mana ability implementable now), Forgotten Cave (MEDIUM -- ETB tapped + mana ability implementable now), Zagoth Triome (LOW -- subtype order), Ketria Triome (LOW -- subtype order)
- **Clean cards**: Indatha Triome

### Notes
- All oracle texts match Scryfall exactly across all 5 cards.
- All card_id slugs, names, and primary types are correct.
- The MEDIUM findings are about implementable abilities being left as TODO alongside genuine DSL gaps. The ETB tapped and basic mana abilities could be authored now; only Cycling and conditional ETB ("unless you control a Swamp") and connive-X are true DSL gaps.
- The triomes' mana abilities are intrinsic to their basic land subtypes (Swamp/Forest/Island/Plains/Mountain) and do not require explicit ability definitions -- the engine derives them from subtypes. The ETB tapped could be implemented independently of Cycling.
