# Wave 1 Batch 4 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 5 LOW

## Card: Gruul Turf
- card_id: gruul-turf -- correct
- name: "Gruul Turf" -- correct
- types/subtypes: Land, no subtypes -- correct (Scryfall type: "Land")
- mana_cost: None -- correct
- oracle_text: matches Scryfall exactly
- abilities: skeleton -- DSL gaps: enters-tapped, bounce-land ETB trigger, multi-mana tap ability
- Verdict: PASS

## Card: Oran-Rief, the Vastwood
- card_id: oran-rief-the-vastwood -- correct
- name: "Oran-Rief, the Vastwood" -- correct
- types/subtypes: Land, no subtypes -- correct
- mana_cost: None -- correct
- oracle_text: matches Scryfall ("This land enters tapped.\n{T}: Add {G}.\n{T}: Put a +1/+1 counter on each green creature that entered this turn.")
- abilities: skeleton -- DSL gaps: enters-tapped, mana ability, activated ability targeting green creatures that ETB'd this turn (tracking + color filter)
- Verdict: PASS

## Card: Thousand-Faced Shadow
- card_id: thousand-faced-shadow -- correct
- name: "Thousand-Faced Shadow" -- correct
- types/subtypes: Creature -- Human Ninja -- correct (uses `creature_types(&["Human", "Ninja"])`)
- mana_cost: {U} (blue: 1) -- correct
- power/toughness: 1/1 -- correct
- oracle_text: matches Scryfall exactly
- abilities: skeleton -- DSL gaps: Ninjutsu {2}{U}{U} (keyword implemented but not wired here), Flying (implementable), triggered copy-token-of-attacker ability (complex targeting + token copy)
- Note (LOW): Flying and Ninjutsu are both implemented in the engine (KW Ninjutsu done in B3, Flying is P1). These could potentially be wired up rather than left as TODO. However, the triggered ability (copy token of attacking creature) is a genuine DSL gap, so leaving all as skeleton is acceptable for Phase 2.
- Verdict: LOW (implementable keywords left as TODO)

## Card: Raugrin Triome
- card_id: raugrin-triome -- correct
- name: "Raugrin Triome" -- correct
- types/subtypes: Land -- Plains Island Mountain -- correct (uses `types_sub(&[CardType::Land], &["Plains", "Island", "Mountain"])`)
- mana_cost: None -- correct
- oracle_text: matches Scryfall exactly
- abilities: skeleton -- DSL gaps: intrinsic mana abilities from basic land subtypes, enters-tapped, Cycling {3} (keyword implemented in engine but not wired here)
- Note (LOW): Cycling is implemented in the engine. The intrinsic mana abilities from basic land subtypes (Plains/Island/Mountain) may be handled automatically by the engine's land type system. Leaving as skeleton is acceptable for Phase 2.
- Verdict: LOW (implementable keywords left as TODO)

## Card: Castle Embereth
- card_id: castle-embereth -- correct
- name: "Castle Embereth" -- correct
- types/subtypes: Land, no subtypes -- correct
- mana_cost: None -- correct
- oracle_text: matches Scryfall exactly
- abilities: skeleton -- DSL gaps: conditional ETB tapped (check if you control a Mountain), mana ability, activated pump ability ({1}{R}{R}, {T}: +1/+0 to your creatures)
- Verdict: PASS

## Summary
HIGH: 0 | MEDIUM: 0 | LOW: 2

- Cards with issues: Thousand-Faced Shadow (LOW -- implementable keywords in skeleton), Raugrin Triome (LOW -- implementable keywords in skeleton)
- Clean cards: Gruul Turf, Oran-Rief the Vastwood, Castle Embereth

All 5 cards have correct card_id, name, types, subtypes, mana cost, and oracle text matching Scryfall. The LOW findings are informational -- these Phase 2 skeletons intentionally leave all abilities as TODO even when some keywords are already supported by the engine.
