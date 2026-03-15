# Card Review: Wave 2 Batch 1

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Finale of Devastation
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (X cost handled via generic: 0, X implicit)
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. Abilities correctly `vec![]` per W5 policy. TODO accurately describes three DSL gaps: search_graveyard, count_threshold conditional pump, mass keyword grant.

## Card 2: Nether Traitor
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({B}{B} = black: 2)
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. Haste and Shadow keywords correctly implemented. Triggered ability (graveyard-zone trigger with mana payment) correctly left as TODO with `vec![]`-equivalent (keywords present, trigger absent). TODO accurately describes two gaps: graveyard-zone triggers and mana-payment conditionals.

## Card 3: Flawless Maneuver
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({2}{W} = generic: 2, white: 1)
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. Abilities correctly `vec![]` per W5 policy. TODO accurately describes two DSL gaps: conditional free cast (commander check) and mass indestructible grant as spell effect.

## Card 4: Craterhoof Behemoth
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({5}{G}{G}{G} = generic: 5, green: 3)
- **P/T match**: YES (5/5)
- **DSL correctness**: YES
- **Findings**:
  - CLEAN. Haste keyword correctly implemented. ETB trigger correctly omitted with TODO describing the DSL gaps (mass trample grant + dynamic X/X pump based on creature count). No no-op placeholder.

## Card 5: Ornithopter
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature -- Thopter via types_sub)
- **Mana cost match**: YES ({0} = all defaults = 0)
- **P/T match**: YES (0/2)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): Minor style nit -- `ManaCost { ..Default::default() }` works but `ManaCost::default()` would be slightly cleaner for a zero-cost card. Not a correctness issue.

## Summary
- Cards with issues: Ornithopter (1 LOW style nit only)
- Clean cards: Finale of Devastation, Nether Traitor, Flawless Maneuver, Craterhoof Behemoth
