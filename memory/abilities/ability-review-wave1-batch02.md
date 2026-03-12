# Wave 1 Batch 2 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 5 LOW

## Card: Mistrise Village
- card_id: OK (`mistrise-village`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK -- matches Scryfall exactly
- abilities: skeleton -- DSL gaps: conditional ETB tapped (check for Mountain or Forest), mana ability, "can't be countered" continuous effect
- TODO accuracy: OK -- all three abilities noted
- Verdict: PASS

## Card: Spara's Headquarters
- card_id: OK (`sparas-headquarters`)
- name: OK
- types/subtypes: OK (Land -- Plains Island Forest via `types_sub`)
- oracle_text: OK -- matches Scryfall exactly
- abilities: skeleton -- DSL gaps: intrinsic basic land type mana abilities, ETB tapped, Cycling
- TODO accuracy: OK -- all three abilities noted
- Verdict: PASS

Note: Subtypes are `["Plains", "Island", "Forest"]` in the def. Scryfall type line says "Forest Plains Island". The ordering difference is cosmetic and has no mechanical impact -- the engine checks set membership, not order. LOW at most.

## Card: Castle Locthwain
- card_id: OK (`castle-locthwain`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK -- matches Scryfall exactly
- abilities: skeleton -- DSL gaps: conditional ETB tapped (check for Swamp), mana ability, activated ability with "lose life equal to cards in hand" (count-based life loss)
- TODO accuracy: MEDIUM -- the second activated ability TODO comment is truncated at line 14: `// TODO: Activated -- {1}{B}{B}, {T}: Draw a card, then you lose life equal to the number of cards in` -- missing `your hand.` This is a cosmetic truncation in a comment, but TODO comments should accurately describe the full ability for future implementers.
- Verdict: MEDIUM (F1)

**F1 (MEDIUM)**: Truncated TODO comment on line 14. The comment for the activated ability is cut off mid-sentence: ends with "the number of cards in" instead of "the number of cards in your hand." Future implementers may miss the "equal to cards in hand" detail.

## Card: Temple of Malady
- card_id: OK (`temple-of-malady`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK -- matches Scryfall exactly
- abilities: skeleton -- DSL gaps: ETB tapped, scry 1 ETB trigger, dual mana ability
- TODO accuracy: OK -- the triggered ability TODO is also truncated but the oracle_text field is complete, so the full text is available. Same pattern as Castle Locthwain but less impactful since "scry 1" is already stated.
- Verdict: PASS (borderline -- truncated TODO but scry 1 is clear from context)

## Card: Simic Growth Chamber
- card_id: OK (`simic-growth-chamber`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK -- matches Scryfall exactly
- abilities: skeleton -- DSL gaps: ETB tapped, bounce land ETB trigger (return a land), dual mana ability producing two colors
- TODO accuracy: OK -- all three abilities noted accurately
- Verdict: PASS

## Summary

HIGH: 0 | MEDIUM: 1 | LOW: 5

**Cards with issues:**
- Castle Locthwain (MEDIUM: truncated TODO comment, F1)

**Clean cards:**
- Mistrise Village
- Spara's Headquarters
- Temple of Malady
- Simic Growth Chamber

**LOW notes (expected DSL gaps, not actionable):**
- All 5 cards have empty `abilities: vec![]` due to DSL gaps (conditional ETB tapped, bounce land triggers, mana abilities, cycling, "can't be countered" effects, count-based life loss). This is correct per W5 policy.
