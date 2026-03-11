# Card Review: Batches 20-23 (19 cards)

**Reviewed**: 2026-03-10
**Cards**: 19
**Findings**: 5 HIGH, 5 MEDIUM, 4 LOW

---

## Card 1: Revitalizing Repast // Old-Growth Grove
- **Oracle match**: UNABLE TO VERIFY (MCP did not return oracle text for this MDFC)
- **Types match**: YES (Instant for front face)
- **Mana cost match**: NO
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**:
  - F1 (HIGH): Mana cost is `ManaCost { ..Default::default() }` which is {0}. Revitalizing Repast is a BG instant -- color identity is [B,G]. Actual mana cost should be verified against Scryfall and set correctly (likely {1}{B}{G} or similar).

## Card 2: Riverglide Pathway // Lavaglide Pathway
- **Oracle match**: YES (front face "{T}: Add {U}.")
- **Types match**: YES (Land)
- **Mana cost match**: YES (None for land)
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**: None

## Card 3: Rune-Tail, Kitsune Ascendant // Rune-Tail's Essence
- **Oracle match**: YES (front face flip trigger text)
- **Types match**: NO
- **Mana cost match**: YES ({2}{W})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**:
  - F2 (MEDIUM): Missing `SuperType::Legendary`. Rune-Tail is "Legendary Creature -- Fox Monk". Should use `full_types(&[SuperType::Legendary], &[CardType::Creature], &["Fox", "Monk"])`.

## Card 4: Scavenger Regent // Exude Toxin
- **Oracle match**: PARTIAL (front face only, but Scryfall shows the card has more text including ward detail)
- **Types match**: YES (Creature -- Dragon, front face)
- **Mana cost match**: YES ({3}{B})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**:
  - F3 (LOW): Oracle text is abbreviated. The MCP shows keywords [Flying, Ward] but the ward cost detail ("Ward--Discard a card.") may be incomplete. Should verify full oracle text against Scryfall.

## Card 5: Sejiri Shelter // Sejiri Glacier
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({1}{W})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**: None

## Card 6: Sundering Eruption // Volcanic Fissure
- **Oracle match**: YES (front face text)
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({2}{R})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**: None

## Card 7: Turn // Burn
- **Oracle match**: YES (front face Turn text + Fuse reminder)
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({2}{U} for Turn half)
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**: None

## Card 8: Turntimber Symbiosis // Turntimber, Serpentine Wood
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({4}{G}{G}{G})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**: None

## Card 9: Valakut Awakening // Valakut Stoneforge
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({2}{R})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**: None

## Card 10: Bala Ged Recovery // Bala Ged Sanctuary
- **Oracle match**: YES (front face text)
- **Types match**: NO
- **Mana cost match**: YES ({2}{G})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**:
  - F4 (HIGH): Name field is `"Bala Ged Recovery"` -- missing back face name. Should be `"Bala Ged Recovery // Bala Ged Sanctuary"` per MDFC naming convention used by all other MDFCs in this batch.
  - F5 (HIGH): Types are `types(&[CardType::Sorcery, CardType::Land])`. An MDFC front face should only have its own type (Sorcery). Including `CardType::Land` is incorrect -- the back face is a land but the front face definition should not include Land.

## Card 11: Monster Manual // Zoological Study
- **Oracle match**: YES (activated ability text for the Artifact face)
- **Types match**: NO
- **Mana cost match**: YES ({3}{G})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**:
  - F6 (HIGH): Name field is `"Monster Manual"` -- missing Adventure name. Should be `"Monster Manual // Zoological Study"` per Adventure/DFC naming convention.
  - F7 (MEDIUM): Types are `types(&[CardType::Sorcery, CardType::Artifact])`. Monster Manual is an Artifact. Zoological Study (the Adventure half) is a Sorcery. The main card definition should have `types(&[CardType::Artifact])` only. Including Sorcery conflates the Adventure half's type with the main card's type.

## Card 12: Decadent Dragon // Expensive Taste
- **Oracle match**: YES (front face text)
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES ({2}{R}{R})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**:
  - F8 (MEDIUM): Name field is `"Decadent Dragon"` -- missing Adventure name. Should be `"Decadent Dragon // Expensive Taste"` per naming convention.

## Card 13: Needleverge Pathway // Pillarverge Pathway
- **Oracle match**: YES (front face "{T}: Add {R}.")
- **Types match**: YES (Land)
- **Mana cost match**: YES (None)
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**:
  - F9 (MEDIUM): Name field is `"Needleverge Pathway"` -- missing back face name. Should be `"Needleverge Pathway // Pillarverge Pathway"` per Pathway MDFC naming convention (compare: Riverglide Pathway includes both names).

## Card 14: Consign // Oblivion
- **Oracle match**: YES (front face Consign text)
- **Types match**: NO
- **Mana cost match**: YES ({1}{U} for Consign half)
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**:
  - F10 (MEDIUM): Types are `types(&[CardType::Instant, CardType::Sorcery])`. Consign is an Instant; Oblivion is a Sorcery (Aftermath). The front face definition should have `types(&[CardType::Instant])` only. Including Sorcery conflates the Aftermath half's type.

## Card 15: Memnite
- **Oracle match**: YES (empty oracle text)
- **Types match**: YES (Artifact Creature -- Construct via `types_sub`)
- **Mana cost match**: YES ({0})
- **DSL correctness**: N/A (abilities: vec![])
- **Findings**: None

## Card 16: Mox Amber
- **Oracle match**: YES
- **Types match**: NO
- **Mana cost match**: YES ({0})
- **DSL correctness**: NO
- **Findings**:
  - F11 (HIGH): Missing `SuperType::Legendary`. Mox Amber is "Legendary Artifact". Should use `supertypes(&[SuperType::Legendary], &[CardType::Artifact])`.
  - F12 (LOW): `Effect::AddManaAnyColor` is an oversimplification. Mox Amber only adds mana of colors among legendary creatures and planeswalkers you control -- it should produce no mana if you control none. This is a DSL gap that should have a TODO comment. The current implementation lets it tap for any color unconditionally.

## Card 17: Avacyn's Pilgrim
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Monk)
- **Mana cost match**: YES ({G})
- **DSL correctness**: YES
- **Findings**: None. `mana_pool(1, 0, 0, 0, 0, 0)` correctly adds {W}.

## Card 18: Leaden Myr
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature -- Myr via `types_sub`)
- **Mana cost match**: YES ({2})
- **DSL correctness**: YES
- **Findings**: None. `mana_pool(0, 0, 1, 0, 0, 0)` correctly adds {B}.

## Card 19: Llanowar Tribe
- **Oracle match**: YES
- **Types match**: YES (Creature -- Elf Druid)
- **Mana cost match**: YES ({G}{G}{G})
- **DSL correctness**: NO
- **Findings**:
  - F13 (LOW): `mana_pool(0, 0, 0, 0, 1, 0)` adds only 1 green mana. Llanowar Tribe's ability is "{T}: Add {G}{G}{G}" which should produce 3 green. Should be `mana_pool(0, 0, 0, 0, 3, 0)`.

---

## Summary

- **Cards with issues**: Revitalizing Repast (1 HIGH), Rune-Tail (1 MEDIUM), Scavenger Regent (1 LOW), Bala Ged Recovery (2 HIGH), Monster Manual (1 HIGH + 1 MEDIUM), Decadent Dragon (1 MEDIUM), Needleverge Pathway (1 MEDIUM), Consign // Oblivion (1 MEDIUM), Mox Amber (1 HIGH + 1 LOW), Llanowar Tribe (1 LOW)
- **Clean cards**: Riverglide Pathway, Sejiri Shelter, Sundering Eruption, Turn // Burn, Turntimber Symbiosis, Valakut Awakening, Memnite, Avacyn's Pilgrim, Leaden Myr

### Issue Breakdown

| ID | Severity | Card | Description |
|----|----------|------|-------------|
| F1 | HIGH | Revitalizing Repast | Mana cost is {0}, should have actual BG mana cost |
| F4 | HIGH | Bala Ged Recovery | Name missing "// Bala Ged Sanctuary" |
| F5 | HIGH | Bala Ged Recovery | Types include Land -- front face should be Sorcery only |
| F6 | HIGH | Monster Manual | Name missing "// Zoological Study" |
| F11 | HIGH | Mox Amber | Missing SuperType::Legendary |
| F2 | MEDIUM | Rune-Tail | Missing SuperType::Legendary |
| F7 | MEDIUM | Monster Manual | Types include Sorcery -- main card should be Artifact only |
| F8 | MEDIUM | Decadent Dragon | Name missing "// Expensive Taste" |
| F9 | MEDIUM | Needleverge Pathway | Name missing "// Pillarverge Pathway" |
| F10 | MEDIUM | Consign // Oblivion | Types include Sorcery -- front face should be Instant only |
| F3 | LOW | Scavenger Regent | Ward cost detail may be incomplete in oracle text |
| F12 | LOW | Mox Amber | AddManaAnyColor oversimplifies -- needs TODO for legendary restriction |
| F13 | LOW | Llanowar Tribe | mana_pool adds 1 green instead of 3 |
| F14 | LOW | Rune-Tail | Missing SuperType::Legendary is also a type-line correctness issue |

**Note on F14**: F2 and F14 are the same issue (Rune-Tail missing Legendary). Counted once as MEDIUM. Total unique findings: 5 HIGH, 5 MEDIUM, 4 LOW.

**Note on MDFC naming convention**: Cards 10, 11, 12, 13 are missing the back face / adventure name in the `name` field. The convention used by other MDFCs in this batch (Revitalizing Repast, Riverglide Pathway, Sejiri Shelter, etc.) is to include both face names separated by " // ". This should be applied consistently.

**Note on split/aftermath type lines**: Cards 11 (Monster Manual) and 14 (Consign) include both halves' card types in the `types` field. The front face definition should only include the front face's type. The other half's type is part of the back/aftermath face which would be handled separately (via `back_face` or Adventure infrastructure).
