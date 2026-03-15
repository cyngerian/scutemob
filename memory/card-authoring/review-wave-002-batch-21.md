# Card Review: Wave 2 Batch 21

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Tekuthal, Inquiry Dominus
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Phyrexian Horror)
- **Mana cost match**: YES ({2}{U}{U} = generic 2, blue 2)
- **P/T match**: YES (3/5)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Two abilities left as TODOs: (1) proliferate-doubling replacement effect, (2) activated ability with Phyrexian hybrid mana + remove-counters-from-others cost. Both are genuine DSL gaps. Flying keyword is present. `abilities: vec![]` policy is satisfied since there is a Flying keyword present; the unimplementable abilities are correctly omitted with accurate TODO comments.

## Card 2: Thrummingbird
- **Oracle match**: YES
- **Types match**: YES (Creature -- Phyrexian Bird Horror)
- **Mana cost match**: YES ({1}{U} = generic 1, blue 1)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Flying keyword and triggered proliferate-on-combat-damage ability both fully implemented. Clean card.

## Card 3: Overwhelming Stampede
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({3}{G}{G} = generic 3, green 2)
- **P/T match**: N/A (non-creature, correctly absent)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Spell effect left as TODO. The dynamic +X/+X where X is greatest power among your creatures is a genuine DSL gap (no EffectAmount variant for max-power-among-permanents). Accurate TODO comment. Empty abilities vec is correct per W5 policy since no part of the effect is independently expressible.

## Card 4: Smuggler's Surprise
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({G} = green 1)
- **P/T match**: N/A (non-creature, correctly absent)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Spree keyword is present. Three spree modes left as TODOs -- each involves complex effects (mill+selective return, put from hand to battlefield, conditional keyword grants). TODO comment accurately identifies both the Spree wiring gap and the power-filter gap. The Spree keyword ability infrastructure exists (KW 134, B7) but mode-to-effect wiring for specific cards requires per-card work beyond what the DSL automates.

## Card 5: Bloated Contaminator
- **Oracle match**: YES
- **Types match**: YES (Creature -- Phyrexian Beast)
- **Mana cost match**: YES ({2}{G} = generic 2, green 1)
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: All three abilities fully implemented: Trample keyword, Toxic(1) keyword, and triggered proliferate-on-combat-damage. Clean card.

## Summary
- Cards with issues: (none)
- Clean cards: Tekuthal Inquiry Dominus, Thrummingbird, Overwhelming Stampede, Smuggler's Surprise, Bloated Contaminator
- All 5 cards have correct oracle text, types, mana costs, and P/T values
- TODO comments are accurate where abilities cannot be expressed in the DSL
- No KI pattern violations detected
