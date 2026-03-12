# Wave 1 Batch 1 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 5 LOW

## Card: Mystic Sanctuary
- card_id: OK (`mystic-sanctuary`)
- name: OK
- types/subtypes: OK (`Land -- Island`)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- DSL gaps: conditional ETB tapped (count-based, 3+ Islands), targeted ETB trigger (put card on library)
- TODO accuracy: line 14-15 truncated ("from your graveyard on top of your library" cut off) -- cosmetic, LOW
- Verdict: **LOW** (truncated TODO comment)

## Card: Ziatora's Proving Ground
- card_id: OK (`ziatoras-proving-ground`)
- name: OK
- types/subtypes: OK (`Land -- Swamp Mountain Forest`)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- DSL gaps: ETB tapped (template), cycling keyword
- TODO accuracy: accurate, describes all three abilities
- Verdict: **PASS**

## Card: Castle Vantress
- card_id: OK (`castle-vantress`)
- name: OK
- types/subtypes: OK (`Land`, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- DSL gaps: conditional ETB tapped (control an Island), mana ability, activated Scry ability
- TODO accuracy: accurate, describes all three abilities
- Verdict: **PASS**

## Card: Shifting Woodland
- card_id: OK (`shifting-woodland`)
- name: OK
- types/subtypes: OK (`Land`, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- DSL gaps: conditional ETB tapped (control a Forest), mana ability, Delirium copy activated ability
- TODO accuracy: line 14 truncated ("your graveyard until end of turn..." cut off) -- cosmetic, LOW
- Verdict: **LOW** (truncated TODO comment)

## Card: Savai Triome
- card_id: OK (`savai-triome`)
- name: OK
- types/subtypes: OK (`Land -- Mountain Plains Swamp`; def order `["Plains", "Swamp", "Mountain"]` differs from Scryfall alphabetical order but functionally equivalent since subtypes are treated as a set)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- DSL gaps: ETB tapped (template), cycling keyword
- TODO accuracy: accurate, describes all three abilities
- Verdict: **PASS**

## Summary
HIGH: 0 | MEDIUM: 0 | LOW: 2

- **Cards with issues**: Mystic Sanctuary (LOW: truncated TODO), Shifting Woodland (LOW: truncated TODO)
- **Clean cards**: Ziatora's Proving Ground, Castle Vantress, Savai Triome

All 5 cards have correct card_id, name, types, subtypes, and oracle_text matching Scryfall.
All use `abilities: vec![]` skeleton pattern as expected for Phase 2 templates with DSL gaps.
The 2 LOW findings are cosmetic (truncated TODO comments) and do not affect correctness.
