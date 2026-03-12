# Wave 1 Batch 5 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 1 LOW

---

## Card: Crypt of Agadeem
- card_id: OK (`crypt-of-agadeem`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - MEDIUM: ETB tapped is implementable (`enters_tapped: true` on CardDefinition)
  - MEDIUM: `{T}: Add {B}` is implementable as a basic mana ability
  - LOW: `{2}, {T}: Add {B} for each black creature card in your graveyard` -- DSL gap (conditional mana production based on graveyard count)
- Verdict: **MEDIUM** (2 implementable abilities skipped)

## Card: Izzet Boilerworks
- card_id: OK (`izzet-boilerworks`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - MEDIUM: ETB tapped is implementable (`enters_tapped: true`)
  - MEDIUM: `{T}: Add {U}{R}` is implementable as a dual mana ability
  - Expected gap: bounce trigger is a targeted_trigger DSL gap
- Verdict: **MEDIUM** (2 implementable abilities skipped)

## Card: Selesnya Sanctuary
- card_id: OK (`selesnya-sanctuary`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - MEDIUM: ETB tapped is implementable (`enters_tapped: true`)
  - MEDIUM: `{T}: Add {G}{W}` is implementable as a dual mana ability
  - Expected gap: bounce trigger is a targeted_trigger DSL gap
- Verdict: **MEDIUM** (2 implementable abilities skipped)

## Card: Secluded Steppe
- card_id: OK (`secluded-steppe`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - MEDIUM: ETB tapped is implementable (`enters_tapped: true`)
  - MEDIUM: `{T}: Add {W}` is implementable as a basic mana ability
  - Expected gap: Cycling is a DSL gap (keyword ability not yet in card DSL)
- Verdict: **MEDIUM** (2 implementable abilities skipped)

## Card: Azorius Chancery
- card_id: OK (`azorius-chancery`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - MEDIUM: ETB tapped is implementable (`enters_tapped: true`)
  - MEDIUM: `{T}: Add {W}{U}` is implementable as a dual mana ability
  - Expected gap: bounce trigger is a targeted_trigger DSL gap
- Verdict: **MEDIUM** (2 implementable abilities skipped)

---

## Summary
HIGH: 0 | MEDIUM: 5 (all cards have implementable ETB-tapped + mana tap left as TODO) | LOW: 1 (Crypt of Agadeem conditional mana)

All 5 cards have correct card_id, name, types, and oracle_text. The consistent MEDIUM finding across all cards is that `enters_tapped: true` and basic/dual mana tap abilities are expressible in the current DSL but were left empty. This is expected for Phase 1 template skeletons but should be addressed in a bulk fix pass.
