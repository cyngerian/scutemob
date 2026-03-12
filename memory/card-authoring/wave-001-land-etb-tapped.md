# Wave 001: Lands — ETB Tapped (Phase 2)

**Sessions**: 1, 2, 3, 4, 5, 6, 7, 8, 9
**Cards to author**: 82 missing (out of 138 in group — 56 already done by Phase 1)
**Batch size**: 16 (Phase 2 agent sessions, author missing cards only)

---

## Author Phase

- [x] Session 1 (3 missing): Mystic Sanctuary, Ziatora's Proving Ground, Castle Vantress
- [x] Session 2 (10 missing): Shifting Woodland, Savai Triome, Mistrise Village, Spara's Headquarters, Castle Locthwain, Temple of Malady, Simic Growth Chamber, Spymaster's Vault, Forgotten Cave, Zagoth Triome
- [x] Session 3 (8 missing): Indatha Triome, Ketria Triome, Gruul Turf, Oran-Rief the Vastwood, Thousand-Faced Shadow, Raugrin Triome, Castle Embereth, Crypt of Agadeem
- [x] Session 4 (9 missing): Izzet Boilerworks, Selesnya Sanctuary, Secluded Steppe, Azorius Chancery, Creeping Tar Pit, Castle Ardenvale, Oathsworn Vampire, Mishra Claimed by Gix, Thundering Falls
- [x] Session 5 (15 missing): Smoldering Crater, Drifting Meadow, Desert of the True, Sunken Palace, Flamekin Village, Cult Conscript, Desert of the Fervent, Spinerock Knoll, Arena of Glory, Minas Tirith, Golgari Rot Farm, Temple of Silence, Orzhov Basilica, Underground Mortuary, Rakdos Carnarium
- [x] Session 6 (8 missing): Halimar Depths, Valakut the Molten Pinnacle, Scoured Barrens, Boros Garrison, Bloodfell Caves, Wind-Scarred Crag, Temple of Triumph, Jetmir's Garden
- [x] Session 7 (5 missing): Jungle Hollow, Temple of Malice, Temple of Epiphany, The World Tree, Raffine's Tower
- [x] Session 8 (8 missing): Glistening Sphere, Temple of the Dragon Queen, Temple of Deceit, Dimir Aqueduct, Skemfar Elderhall, Arixmethes Slumbering Isle, Undercity Sewers, Swiftwater Cliffs
- [x] Session 9 (16 missing): Godless Shrine, Bojuka Bog, Overgrown Tomb, Blood Crypt, Breeding Pool, Temple Garden, Stomping Ground, Hallowed Fountain, Watery Grave, Sacred Foundry, Steam Vents, Field of the Dead, Witch's Cottage, Emeria the Sky Ruin, Den of the Bugbear, Mortuary Mire

**cargo build + cargo test after all author sessions complete**: [x] (1972 tests pass, 2026-03-12)

---

## Review Phase (batches of 5, up to 4 parallel)

All 82 authored cards split into 17 review batches:

- [x] Review batch 1: Mystic Sanctuary, Ziatora's Proving Ground, Castle Vantress, Shifting Woodland, Savai Triome — 0H 0M 2L
- [x] Review batch 2: Mistrise Village, Spara's Headquarters, Castle Locthwain, Temple of Malady, Simic Growth Chamber — 0H 1M 0L
- [x] Review batch 3: Spymaster's Vault, Forgotten Cave, Zagoth Triome, Indatha Triome, Ketria Triome — 0H 2M 2L
- [x] Review batch 4: Gruul Turf, Oran-Rief the Vastwood, Thousand-Faced Shadow, Raugrin Triome, Castle Embereth — 0H 0M 2L
- [x] Review batch 5: Crypt of Agadeem, Izzet Boilerworks, Selesnya Sanctuary, Secluded Steppe, Azorius Chancery — 0H 5M 1L
- [x] Review batch 6: Creeping Tar Pit, Castle Ardenvale, Oathsworn Vampire, Mishra Claimed by Gix, Thundering Falls — 0H 4M 2L
- [x] Review batch 7: Smoldering Crater, Drifting Meadow, Desert of the True, Sunken Palace, Flamekin Village — 0H 3M 2L
- [x] Review batch 8: Cult Conscript, Desert of the Fervent, Spinerock Knoll, Arena of Glory, Minas Tirith — 0H 5M 2L
- [x] Review batch 9: Golgari Rot Farm, Temple of Silence, Orzhov Basilica, Underground Mortuary, Rakdos Carnarium — 0H 5M 0L
- [x] Review batch 10: Halimar Depths, Valakut the Molten Pinnacle, Scoured Barrens, Boros Garrison, Bloodfell Caves — 0H 5M 1L
- [x] Review batch 11: Wind-Scarred Crag, Temple of Triumph, Jetmir's Garden, Jungle Hollow, Temple of Malice — 0H 5M 0L
- [x] Review batch 12: Temple of Epiphany, The World Tree, Raffine's Tower, Glistening Sphere, Temple of the Dragon Queen — 1H 3M 2L
- [x] Review batch 13: Temple of Deceit, Dimir Aqueduct, Skemfar Elderhall, Arixmethes Slumbering Isle, Undercity Sewers — 0H 5M 5L
- [x] Review batch 14: Swiftwater Cliffs, Godless Shrine, Bojuka Bog, Overgrown Tomb, Blood Crypt — 0H 2M 0L
- [x] Review batch 15: Breeding Pool, Temple Garden, Stomping Ground, Hallowed Fountain, Watery Grave — 0H 0M 0L
- [x] Review batch 16: Sacred Foundry, Steam Vents, Field of the Dead, Witch's Cottage, Emeria the Sky Ruin — 1H 1M 3L
- [x] Review batch 17: Den of the Bugbear, Mortuary Mire — (covered in batch 16-17 run)

---

## Fix Phase

**Total findings**: 2 HIGH, 46 MEDIUM, 22 LOW

### HIGH fixes (must fix):
1. **The World Tree** (batch 12): missing `SuperType::Legendary` — change `types(&[CardType::Land])` to `supertypes(&[SuperType::Legendary], &[CardType::Land])`
2. **Emeria, the Sky Ruin** (batch 16): same — missing `SuperType::Legendary`

### MEDIUM fixes — systematic implementable abilities (bulk fix):
The bulk-card-author was too conservative. Common patterns to implement across applicable cards:
- **ETB tapped** (unconditional): ~30+ cards — `ReplacementModification::EntersTapped`
- **Tap mana** (basic 1-color or 2-color): ~30+ cards — `ManaAbility` activated ability
- **ETB Scry 1**: temple lands (Temple of Silence, Malady, Triumph, Malice, Epiphany, Deceit) — `TriggerCondition::WhenEntersBattlefield` + `Effect::Scry(1)`
- **ETB Surveil 1**: Underground Mortuary, Undercity Sewers, Thundering Falls — ETB + `Effect::Surveil(1)`
- **ETB Gain 1 life**: gain-lands (Scoured Barrens, Bloodfell Caves, Wind-Scarred Crag, Jungle Hollow, Swiftwater Cliffs) — ETB + `Effect::GainLife`
- **Cycling** (keyword): cycling lands (Forgotten Cave, Smoldering Crater, Drifting Meadow, Desert of the True, Desert of the Fervent, Secluded Steppe) — `KeywordAbility::Cycling`

### LOW fixes (opportunistic):
- Cult Conscript: subtype order `["Warrior", "Skeleton"]` → `["Skeleton", "Warrior"]`
- Zagoth Triome: subtype order → `["Swamp", "Forest", "Island"]`
- Ketria Triome: subtype order → `["Forest", "Island", "Mountain"]`
- Castle Locthwain: fix truncated TODO comment (cosmetic)

- [ ] Apply all HIGH fixes
- [ ] Apply MEDIUM bulk fixes (ETB tapped, mana tap, ETB scry/surveil/life, cycling)
- [ ] Apply LOW metadata fixes
- [ ] cargo build + cargo test after fix pass
- [ ] Commit: `W5-cards: Phase 2 Wave 1 — land-etb-tapped (82 cards)`

---

## Status: IN PROGRESS — Review Phase (author phase complete 2026-03-12)

**Notes**:
- Sessions 1-9 = bulk-card-author agent sessions from _authoring_plan.json session IDs 1-9
- All 82 cards are skeleton-only (abilities: vec![] with TODOs) — all legitimately DSL-blocked
  (shock_etb, targeted_trigger, return_from_graveyard, count_threshold DSL gaps)
- Run up to 2 author agents in parallel
- Review wave discipline: wait for ALL author sessions before launching ANY reviewers
- Review wave discipline: 4 reviewers in parallel, wait for all before fixes
- Fix discipline: one single fix pass after ALL reviews collected — no ad-hoc fixes between review waves
